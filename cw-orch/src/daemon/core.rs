use super::{
    builder::DaemonBuilder,
    cosmos_modules,
    error::DaemonError,
    queriers::{DaemonQuerier, Node},
    sender::Wallet,
    state::DaemonState,
    tx_resp::CosmTxResponse,
};
use crate::{
    environment::{ChainUpload, TxHandler},
    prelude::{
        queriers::CosmWasm, CallAs, ContractInstance, CwOrcExecute, IndexResponse, Uploadable,
        WasmPath,
    },
    state::ChainState,
};
use cosmrs::{
    cosmwasm::{MsgExecuteContract, MsgInstantiateContract, MsgMigrateContract},
    tendermint::Time,
    AccountId, Denom,
};
use cosmwasm_std::{Addr, Coin};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::from_str;
use std::{
    fmt::Debug,
    rc::Rc,
    str::{from_utf8, FromStr},
    time::Duration,
};
use tokio::runtime::Handle;
use tonic::transport::Channel;

#[derive(Clone)]
/**
    Represents a blockchain node.
    Is constructed with the [DaemonBuilder].

    ## Usage

    ```rust
    use cw_orch::daemon::Daemon;
    use cw_orch::networks::JUNO_1;
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();
    let daemon: Daemon = Daemon::builder()
        .chain(JUNO_1)
        .handle(rt.handle())
        .build()
        .unwrap();
    ```
    ## Environment Execution

    The Daemon implements [`TxHandler`] which allows you to perform transactions on the chain.

    ## Querying

    Different Cosmos SDK modules can be queried through the daemon by calling the [`Daemon::query<Querier>`] method with a specific querier.
    See [Querier](crate::daemon::queriers) for examples.
*/
pub struct Daemon {
    pub sender: Wallet,
    pub state: Rc<DaemonState>,
}

impl Daemon {
    /// Get the daemon builder
    pub fn builder() -> DaemonBuilder {
        DaemonBuilder::default()
    }

    /// Perform a query with a given query client
    /// See [Querier](crate::daemon::queriers) for examples.
    pub fn query_client<Querier: DaemonQuerier>(&self) -> Querier {
        Querier::new(self.sender.channel())
    }

    /// Get the channel configured for this Daemon
    pub fn channel(&self) -> Channel {
        self.state().grpc_channel.clone()
    }
}

impl ChainState for Daemon {
    type Out = Rc<DaemonState>;

    fn state(&self) -> Self::Out {
        self.state.clone()
    }
}

// Execute on the real chain, returns tx response
impl Daemon {
    pub fn sender(&self) -> Addr {
        self.sender.address().unwrap()
    }

    pub async fn execute<E: Serialize>(
        &self,
        exec_msg: &E,
        coins: &[cosmwasm_std::Coin],
        contract_address: &Addr,
    ) -> Result<CosmTxResponse, DaemonError> {
        let exec_msg: MsgExecuteContract = MsgExecuteContract {
            sender: self.sender.pub_addr()?,
            contract: AccountId::from_str(contract_address.as_str())?,
            msg: serde_json::to_vec(&exec_msg)?,
            funds: parse_cw_coins(coins)?,
        };
        let result = self.sender.commit_tx(vec![exec_msg], None).await?;
        Ok(result)
    }

    pub async fn instantiate<I: Serialize + Debug>(
        &self,
        code_id: u64,
        init_msg: &I,
        label: Option<&str>,
        admin: Option<&Addr>,
        coins: &[Coin],
    ) -> Result<CosmTxResponse, DaemonError> {
        let sender = &self.sender;

        let init_msg = MsgInstantiateContract {
            code_id,
            label: Some(label.unwrap_or("instantiate_contract").to_string()),
            admin: admin.map(|a| FromStr::from_str(a.as_str()).unwrap()),
            sender: sender.pub_addr()?,
            msg: serde_json::to_vec(&init_msg)?,
            funds: parse_cw_coins(coins)?,
        };

        let result = sender.commit_tx(vec![init_msg], None).await?;

        Ok(result)
    }

    pub async fn query<Q: Serialize + Debug, T: Serialize + DeserializeOwned>(
        &self,
        query_msg: &Q,
        contract_address: &Addr,
    ) -> Result<T, DaemonError> {
        let sender = &self.sender;
        let mut client = cosmos_modules::cosmwasm::query_client::QueryClient::new(sender.channel());
        let resp = client
            .smart_contract_state(cosmos_modules::cosmwasm::QuerySmartContractStateRequest {
                address: contract_address.to_string(),
                query_data: serde_json::to_vec(&query_msg)?,
            })
            .await?;

        Ok(from_str(from_utf8(&resp.into_inner().data).unwrap())?)
    }

    pub async fn migrate<M: Serialize + Debug>(
        &self,
        migrate_msg: &M,
        new_code_id: u64,
        contract_address: &Addr,
    ) -> Result<CosmTxResponse, DaemonError> {
        let exec_msg: MsgMigrateContract = MsgMigrateContract {
            sender: self.sender.pub_addr()?,
            contract: AccountId::from_str(contract_address.as_str())?,
            msg: serde_json::to_vec(&migrate_msg)?,
            code_id: new_code_id,
        };
        let result = self.sender.commit_tx(vec![exec_msg], None).await?;
        Ok(result)
    }

    pub async fn wait_blocks(&self, amount: u64) -> Result<(), DaemonError> {
        let mut last_height = self.query_client::<Node>().block_height().await?;
        let end_height = last_height + amount;

        while last_height < end_height {
            tokio::time::sleep(Duration::from_secs(4)).await;

            // ping latest block
            last_height = self.query_client::<Node>().block_height().await?;
        }
        Ok(())
    }

    pub async fn wait_seconds(&self, secs: u64) -> Result<(), DaemonError> {
        tokio::time::sleep(Duration::from_secs(secs)).await;

        Ok(())
    }

    pub async fn next_block(&self) -> Result<(), DaemonError> {
        let mut last_height = self.query_client::<Node>().block_height().await?;
        let end_height = last_height + 1;

        while last_height < end_height {
            // wait
            tokio::time::sleep(Duration::from_secs(4));
            // ping latest block
            last_height = self.query_client::<Node>().block_height().await?;
        }
        Ok(())
    }

    pub async fn block_info(&self) -> Result<cosmwasm_std::BlockInfo, DaemonError> {
        let block = self.query_client::<Node>().latest_block().await?;
        let since_epoch = block.header.time.duration_since(Time::unix_epoch())?;
        let time = cosmwasm_std::Timestamp::from_nanos(since_epoch.as_nanos() as u64);
        Ok(cosmwasm_std::BlockInfo {
            height: block.header.height.value(),
            time,
            chain_id: block.header.chain_id.to_string(),
        })
    }

    pub async fn upload(
        &self,
        uploadable: &impl Uploadable,
    ) -> Result<CosmTxResponse, DaemonError> {
        let sender = &self.sender;
        let wasm_path = uploadable.wasm();

        log::debug!("Uploading file at {:?}", wasm_path);

        let file_contents = std::fs::read(wasm_path.path())?;
        let store_msg = cosmrs::cosmwasm::MsgStoreCode {
            sender: sender.pub_addr()?,
            wasm_byte_code: file_contents,
            instantiate_permission: None,
        };
        let result = sender.commit_tx(vec![store_msg], None).await?;

        log::info!("Uploaded: {:?}", result.txhash);

        let code_id = result.uploaded_code_id().unwrap();

        // wait for the node to return the contract information for this upload
        let wasm = CosmWasm::new(self.channel());
        while wasm.code(code_id).await.is_err() {
            tokio::time::sleep(Duration::from_secs(6)).await;
        }

        Ok(result)
    }

    /// Set the sender to use with this Daemon to be the given wallet
    pub fn set_sender(&mut self, sender: &Wallet) {
        self.sender = sender.clone();
    }
}

pub(crate) fn parse_cw_coins(
    coins: &[cosmwasm_std::Coin],
) -> Result<Vec<cosmrs::Coin>, DaemonError> {
    coins
        .iter()
        .map(|cosmwasm_std::Coin { amount, denom }| {
            Ok(cosmrs::Coin {
                amount: amount.u128(),
                denom: Denom::from_str(denom)?,
            })
        })
        .collect::<Result<Vec<_>, DaemonError>>()
}
