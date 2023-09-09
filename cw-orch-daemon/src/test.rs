#![feature(prelude_import)]
//! `Daemon` and `DaemonAsync` execution environments.
//!
//! The `Daemon` type is a synchronous wrapper around the `DaemonAsync` type and can be used as a contract execution environment.
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
pub mod builder {
    use crate::{DaemonAsync, DaemonBuilder};
    use std::rc::Rc;
    use ibc_chain_registry::chain::ChainData;
    use super::{error::DaemonError, sender::Sender, state::DaemonState};
    /// The default deployment id if none is provided
    pub const DEFAULT_DEPLOYMENT: &str = "default";
    /// Create [`DaemonAsync`] through [`DaemonAsyncBuilder`]
    /// ## Example
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use cw_orch_daemon::{DaemonAsyncBuilder, networks};
    /// let daemon = DaemonAsyncBuilder::default()
    ///     .chain(networks::LOCAL_JUNO)
    ///     .deployment_id("v0.1.0")
    ///     .build()
    ///     .await.unwrap();
    /// # })
    /// ```
    pub struct DaemonAsyncBuilder {
        pub(crate) chain: Option<ChainData>,
        pub(crate) deployment_id: Option<String>,
        /// Wallet mnemonic
        pub(crate) mnemonic: Option<String>,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for DaemonAsyncBuilder {
        #[inline]
        fn clone(&self) -> DaemonAsyncBuilder {
            DaemonAsyncBuilder {
                chain: ::core::clone::Clone::clone(&self.chain),
                deployment_id: ::core::clone::Clone::clone(&self.deployment_id),
                mnemonic: ::core::clone::Clone::clone(&self.mnemonic),
            }
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for DaemonAsyncBuilder {
        #[inline]
        fn default() -> DaemonAsyncBuilder {
            DaemonAsyncBuilder {
                chain: ::core::default::Default::default(),
                deployment_id: ::core::default::Default::default(),
                mnemonic: ::core::default::Default::default(),
            }
        }
    }
    impl DaemonAsyncBuilder {
        /// Set the chain the daemon will connect to
        pub fn chain(&mut self, chain: impl Into<ChainData>) -> &mut Self {
            self.chain = Some(chain.into());
            self
        }
        /// Set the deployment id to use for the daemon interactions
        /// Defaults to `default`
        pub fn deployment_id(&mut self, deployment_id: impl Into<String>) -> &mut Self {
            self.deployment_id = Some(deployment_id.into());
            self
        }
        /// Set the mnemonic to use with this chain.
        /// Defaults to env variable depending on the environment.
        ///
        /// Variables: LOCAL_MNEMONIC, TEST_MNEMONIC and MAIN_MNEMONIC
        pub fn mnemonic(&mut self, mnemonic: impl ToString) -> &mut Self {
            self.mnemonic = Some(mnemonic.to_string());
            self
        }
        /// Build a daemon
        pub async fn build(&self) -> Result<DaemonAsync, DaemonError> {
            let chain = self
                .chain
                .clone()
                .ok_or(DaemonError::BuilderMissing("chain information".into()))?;
            let deployment_id = self
                .deployment_id
                .clone()
                .unwrap_or(DEFAULT_DEPLOYMENT.to_string());
            let state = Rc::new(DaemonState::new(chain, deployment_id).await?);
            let sender = if let Some(mnemonic) = &self.mnemonic {
                Sender::from_mnemonic(&state, mnemonic)?
            } else {
                Sender::new(&state)?
            };
            let daemon = DaemonAsync {
                state,
                sender: Rc::new(sender),
            };
            Ok(daemon)
        }
    }
    impl From<DaemonBuilder> for DaemonAsyncBuilder {
        fn from(value: DaemonBuilder) -> Self {
            DaemonAsyncBuilder {
                chain: value.chain,
                deployment_id: value.deployment_id,
                mnemonic: value.mnemonic,
            }
        }
    }
}
pub mod channel {
    use cosmrs::proto::cosmos::base::tendermint::v1beta1::{
        service_client::ServiceClient, GetNodeInfoRequest,
    };
    use ibc_chain_registry::chain::Grpc;
    use ibc_relayer_types::core::ics24_host::identifier::ChainId;
    use tonic::transport::{Channel, ClientTlsConfig};
    use super::error::DaemonError;
    /// A helper for constructing a gRPC channel
    pub struct GrpcChannel {}
    impl GrpcChannel {
        /// Connect to any of the provided gRPC endpoints
        pub async fn connect(
            grpc: &[Grpc],
            chain_id: &ChainId,
        ) -> Result<Channel, DaemonError> {
            let mut successful_connections = ::alloc::vec::Vec::new();
            for Grpc { address, .. } in grpc.iter() {
                {
                    let lvl = ::log::Level::Info;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            format_args!("Trying to connect to endpoint: {0}", address),
                            lvl,
                            &(
                                "cw_orch_daemon::channel",
                                "cw_orch_daemon::channel",
                                "cw-orch-daemon/src/channel.rs",
                            ),
                            19u32,
                            ::log::__private_api::Option::None,
                        );
                    }
                };
                let endpoint = Channel::builder(address.clone().try_into().unwrap());
                let maybe_client = ServiceClient::connect(endpoint.clone()).await;
                let mut client = if maybe_client.is_ok() {
                    maybe_client?
                } else {
                    {
                        let lvl = ::log::Level::Warn;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "Cannot connect to gRPC endpoint: {0}, {1:?}",
                                    address,
                                    maybe_client.unwrap_err(),
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::channel",
                                    "cw_orch_daemon::channel",
                                    "cw-orch-daemon/src/channel.rs",
                                ),
                                31u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    if !(address.contains("https") || address.contains("443")) {
                        continue;
                    }
                    {
                        let lvl = ::log::Level::Info;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!("Attempting to connect with TLS"),
                                lvl,
                                &(
                                    "cw_orch_daemon::channel",
                                    "cw_orch_daemon::channel",
                                    "cw-orch-daemon/src/channel.rs",
                                ),
                                43u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    let endpoint = endpoint.clone().tls_config(ClientTlsConfig::new())?;
                    let maybe_client = ServiceClient::connect(endpoint.clone()).await;
                    if maybe_client.is_err() {
                        {
                            let lvl = ::log::Level::Warn;
                            if lvl <= ::log::STATIC_MAX_LEVEL
                                && lvl <= ::log::max_level()
                            {
                                ::log::__private_api::log(
                                    format_args!(
                                        "Cannot connect to gRPC endpoint: {0}, {1:?}",
                                        address,
                                        maybe_client.unwrap_err(),
                                    ),
                                    lvl,
                                    &(
                                        "cw_orch_daemon::channel",
                                        "cw_orch_daemon::channel",
                                        "cw-orch-daemon/src/channel.rs",
                                    ),
                                    51u32,
                                    ::log::__private_api::Option::None,
                                );
                            }
                        };
                        continue;
                    }
                    maybe_client?
                };
                let node_info = client
                    .get_node_info(GetNodeInfoRequest {})
                    .await?
                    .into_inner();
                if ChainId::is_epoch_format(
                    &node_info.default_node_info.as_ref().unwrap().network,
                ) {
                    if node_info.default_node_info.as_ref().unwrap().network
                        != chain_id.as_str()
                    {
                        {
                            let lvl = ::log::Level::Error;
                            if lvl <= ::log::STATIC_MAX_LEVEL
                                && lvl <= ::log::max_level()
                            {
                                ::log::__private_api::log(
                                    format_args!(
                                        "Network mismatch: connection:{0} != config:{1}",
                                        node_info.default_node_info.as_ref().unwrap().network,
                                        chain_id.as_str(),
                                    ),
                                    lvl,
                                    &(
                                        "cw_orch_daemon::channel",
                                        "cw_orch_daemon::channel",
                                        "cw-orch-daemon/src/channel.rs",
                                    ),
                                    72u32,
                                    ::log::__private_api::Option::None,
                                );
                            }
                        };
                        continue;
                    }
                }
                successful_connections.push(endpoint.connect().await?)
            }
            if successful_connections.is_empty() {
                return Err(DaemonError::CannotConnectGRPC);
            }
            Ok(successful_connections.pop().unwrap())
        }
    }
}
pub mod core {
    use crate::{queriers::CosmWasm, DaemonState};
    use super::{
        builder::DaemonAsyncBuilder, cosmos_modules, error::DaemonError,
        queriers::{DaemonQuerier, Node},
        sender::Wallet, tx_resp::CosmTxResponse,
    };
    use cosmrs::{
        cosmwasm::{MsgExecuteContract, MsgInstantiateContract, MsgMigrateContract},
        tendermint::Time, AccountId, Denom,
    };
    use cosmwasm_std::{Addr, Coin};
    use cw_orch_core::{
        contract::interface_traits::Uploadable, environment::{ChainState, IndexResponse},
    };
    use serde::{de::DeserializeOwned, Serialize};
    use serde_json::from_str;
    use std::{
        fmt::Debug, rc::Rc, str::{from_utf8, FromStr},
        time::Duration,
    };
    use tonic::transport::Channel;
    /**
    Represents a blockchain node.
    It's constructed using [`DaemonAsyncBuilder`].

    ## Usage
    ```rust,no_run
    # tokio_test::block_on(async {
    use cw_orch_daemon::{DaemonAsync, networks};

    let daemon: DaemonAsync = DaemonAsync::builder()
        .chain(networks::JUNO_1)
        .build()
        .await.unwrap();
    # })
    ```
    ## Environment Execution

    The DaemonAsync implements async methods of [`TxHandler`](cw_orch_core::environment::TxHandler) which allows you to perform transactions on the chain.

    ## Querying

    Different Cosmos SDK modules can be queried through the daemon by calling the [`DaemonAsync::query_client<Querier>`] method with a specific querier.
    See [Querier](crate::queriers) for examples.
*/
    pub struct DaemonAsync {
        /// Sender to send transactions to the chain
        pub sender: Wallet,
        /// State of the daemon
        pub state: Rc<DaemonState>,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for DaemonAsync {
        #[inline]
        fn clone(&self) -> DaemonAsync {
            DaemonAsync {
                sender: ::core::clone::Clone::clone(&self.sender),
                state: ::core::clone::Clone::clone(&self.state),
            }
        }
    }
    impl DaemonAsync {
        /// Get the daemon builder
        pub fn builder() -> DaemonAsyncBuilder {
            DaemonAsyncBuilder::default()
        }
        /// Perform a query with a given query client.
        /// See [Querier](crate::queriers) for examples.
        pub fn query_client<Querier: DaemonQuerier>(&self) -> Querier {
            Querier::new(self.sender.channel())
        }
        /// Get the channel configured for this DaemonAsync.
        pub fn channel(&self) -> Channel {
            self.state.grpc_channel.clone()
        }
    }
    impl ChainState for DaemonAsync {
        type Out = Rc<DaemonState>;
        fn state(&self) -> Self::Out {
            self.state.clone()
        }
    }
    impl DaemonAsync {
        /// Get the sender address
        pub fn sender(&self) -> Addr {
            self.sender.address().unwrap()
        }
        /// Execute a message on a contract.
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
            let result = self
                .sender
                .commit_tx(
                    <[_]>::into_vec(#[rustc_box] ::alloc::boxed::Box::new([exec_msg])),
                    None,
                )
                .await?;
            Ok(result)
        }
        /// Instantiate a contract.
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
            let result = sender
                .commit_tx(
                    <[_]>::into_vec(#[rustc_box] ::alloc::boxed::Box::new([init_msg])),
                    None,
                )
                .await?;
            Ok(result)
        }
        /// Query a contract.
        pub async fn query<Q: Serialize + Debug, T: Serialize + DeserializeOwned>(
            &self,
            query_msg: &Q,
            contract_address: &Addr,
        ) -> Result<T, DaemonError> {
            let sender = &self.sender;
            let mut client = cosmos_modules::cosmwasm::query_client::QueryClient::new(
                sender.channel(),
            );
            let resp = client
                .smart_contract_state(cosmos_modules::cosmwasm::QuerySmartContractStateRequest {
                    address: contract_address.to_string(),
                    query_data: serde_json::to_vec(&query_msg)?,
                })
                .await?;
            Ok(from_str(from_utf8(&resp.into_inner().data).unwrap())?)
        }
        /// Migration a contract.
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
            let result = self
                .sender
                .commit_tx(
                    <[_]>::into_vec(#[rustc_box] ::alloc::boxed::Box::new([exec_msg])),
                    None,
                )
                .await?;
            Ok(result)
        }
        /// Wait for a given amount of blocks.
        pub async fn wait_blocks(&self, amount: u64) -> Result<(), DaemonError> {
            let mut last_height = self.query_client::<Node>().block_height().await?;
            let end_height = last_height + amount;
            let average_block_speed = self
                .query_client::<Node>()
                .average_block_speed(Some(0.9))
                .await?;
            let wait_time = average_block_speed * amount;
            tokio::time::sleep(Duration::from_secs(wait_time)).await;
            while last_height < end_height {
                tokio::time::sleep(Duration::from_secs(average_block_speed)).await;
                last_height = self.query_client::<Node>().block_height().await?;
            }
            Ok(())
        }
        /// Wait for a given amount of seconds.
        pub async fn wait_seconds(&self, secs: u64) -> Result<(), DaemonError> {
            tokio::time::sleep(Duration::from_secs(secs)).await;
            Ok(())
        }
        /// Wait for the next block.
        pub async fn next_block(&self) -> Result<(), DaemonError> {
            self.wait_blocks(1).await
        }
        /// Get the current block info.
        pub async fn block_info(&self) -> Result<cosmwasm_std::BlockInfo, DaemonError> {
            let block = self.query_client::<Node>().latest_block().await?;
            let since_epoch = block.header.time.duration_since(Time::unix_epoch())?;
            let time = cosmwasm_std::Timestamp::from_nanos(
                since_epoch.as_nanos() as u64,
            );
            Ok(cosmwasm_std::BlockInfo {
                height: block.header.height.value(),
                time,
                chain_id: block.header.chain_id.to_string(),
            })
        }
        /// Upload a contract to the chain.
        pub async fn upload(
            &self,
            uploadable: &impl Uploadable,
        ) -> Result<CosmTxResponse, DaemonError> {
            let sender = &self.sender;
            let wasm_path = uploadable.wasm();
            {
                let lvl = ::log::Level::Debug;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                    ::log::__private_api::log(
                        format_args!("Uploading file at {0:?}", wasm_path),
                        lvl,
                        &(
                            "cw_orch_daemon::core",
                            "cw_orch_daemon::core",
                            "cw-orch-daemon/src/core.rs",
                        ),
                        233u32,
                        ::log::__private_api::Option::None,
                    );
                }
            };
            let file_contents = std::fs::read(wasm_path.path())?;
            let store_msg = cosmrs::cosmwasm::MsgStoreCode {
                sender: sender.pub_addr()?,
                wasm_byte_code: file_contents,
                instantiate_permission: None,
            };
            let result = sender
                .commit_tx(
                    <[_]>::into_vec(#[rustc_box] ::alloc::boxed::Box::new([store_msg])),
                    None,
                )
                .await?;
            {
                let lvl = ::log::Level::Info;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                    ::log::__private_api::log(
                        format_args!("Uploaded: {0:?}", result.txhash),
                        lvl,
                        &(
                            "cw_orch_daemon::core",
                            "cw_orch_daemon::core",
                            "cw-orch-daemon/src/core.rs",
                        ),
                        244u32,
                        ::log::__private_api::Option::None,
                    );
                }
            };
            let code_id = result.uploaded_code_id().unwrap();
            let wasm = CosmWasm::new(self.channel());
            while wasm.code(code_id).await.is_err() {
                self.next_block().await?;
            }
            Ok(result)
        }
        /// Set the sender to use with this DaemonAsync to be the given wallet
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
}
pub mod error {
    #![allow(missing_docs)]
    use cw_orch_core::CwEnvError;
    use thiserror::Error;
    pub enum DaemonError {
        #[error("Reqwest HTTP(s) Error")]
        ReqwestError(#[from] ::reqwest::Error),
        #[error("JSON Conversion Error")]
        SerdeJson(#[from] ::serde_json::Error),
        #[error(transparent)]
        ParseIntError(#[from] std::num::ParseIntError),
        #[error(transparent)]
        IOErr(#[from] ::std::io::Error),
        #[error(transparent)]
        Secp256k1(#[from] ::secp256k1::Error),
        #[error(transparent)]
        VarError(#[from] ::std::env::VarError),
        #[error(transparent)]
        AnyError(#[from] ::anyhow::Error),
        #[error(transparent)]
        Status(#[from] ::tonic::Status),
        #[error(transparent)]
        TransportError(#[from] ::tonic::transport::Error),
        #[error(transparent)]
        TendermintError(#[from] ::cosmrs::tendermint::Error),
        #[error(transparent)]
        CwEnvError(#[from] ::cw_orch_core::CwEnvError),
        #[error("Bech32 Decode Error")]
        Bech32DecodeErr,
        #[error(
            "Bech32 Decode Error: Key Failed prefix {0} or length {1} Wanted:{2}/{3}"
        )]
        Bech32DecodeExpanded(String, usize, String, usize),
        #[error("Mnemonic - Wrong length, it should be 24 words")]
        WrongLength,
        #[error("Mnemonic - Bad Phrase")]
        Phrasing,
        #[error("Mnemonic - Missing Phrase")]
        MissingPhrase,
        #[error("Bad Implementation. Missing Component")]
        Implementation,
        #[error("Unable to convert into public key `{key}`")]
        Conversion { key: String, source: bitcoin::bech32::Error },
        #[error(
            "Can not augment daemon deployment after usage in more than one contract."
        )]
        SharedDaemonState,
        #[error(transparent)]
        ErrReport(#[from] ::eyre::ErrReport),
        #[error(transparent)]
        GRpcDecodeError(#[from] ::prost::DecodeError),
        #[error(transparent)]
        ED25519(#[from] ::ed25519_dalek::ed25519::Error),
        #[error(transparent)]
        DecodeError(#[from] ::base64::DecodeError),
        #[error(transparent)]
        HexError(#[from] ::hex::FromHexError),
        #[error(transparent)]
        BitCoinBip32(#[from] ::bitcoin::bip32::Error),
        #[error("83 length-missing SECP256K1 prefix")]
        ConversionSECP256k1,
        #[error("82 length-missing ED25519 prefix")]
        ConversionED25519,
        #[error("Expected Key length of 82 or 83 length was {0}")]
        ConversionLength(usize),
        #[error("Expected Key length of 40 length was {0}")]
        ConversionLengthED25519Hex(usize),
        #[error(
            "Expected ED25519 key of length 32 with a BECH32 ED25519 prefix of 5 chars - Len {0} - Hex {1}"
        )]
        ConversionPrefixED25519(usize, String),
        #[error("Can't call Transactions without some gas rules")]
        NoGasOpts,
        #[error("Can't parse `{parse}` into a coin")]
        CoinParseErrV { parse: String },
        #[error("Can't parse `{0}` into a coin")]
        CoinParseErr(String),
        #[error("TX submit returned `{0}` - {1} '{2}'")]
        TxResultError(usize, String, String),
        #[error("No price found for Gas using denom {0}")]
        GasPriceError(String),
        #[error(
            "Attempting to fetch validator set in parts, and failed Height mismatch {0} {1}"
        )]
        TendermintValidatorSet(u64, u64),
        #[error("Transaction {0} not found after {1} attempts")]
        TXNotFound(String, usize),
        #[error("unknown API error")]
        Unknown,
        #[error("Generic Error {0}")]
        StdErr(String),
        #[error("calling contract with unimplemented action")]
        NotImplemented,
        #[error("new chain detected, fill out the scaffold at {0}")]
        NewChain(String),
        #[error("new network detected, fill out the scaffold at {0}")]
        NewNetwork(String),
        #[error("Can not connect to any grpc endpoint that was provided.")]
        CannotConnectGRPC,
        #[error("tx failed: {reason} with code {code}")]
        TxFailed { code: usize, reason: String },
        #[error("The list of grpc endpoints is empty")]
        GRPCListIsEmpty,
        #[error("no wasm path provided for contract.")]
        MissingWasmPath,
        #[error("daemon builder missing {0}")]
        BuilderMissing(String),
        #[error("ibc error: {0}")]
        IbcError(String),
        #[error("insufficient fee, check gas price: {0}")]
        InsufficientFee(String),
    }
    #[allow(unused_qualifications)]
    impl std::error::Error for DaemonError {
        fn source(&self) -> std::option::Option<&(dyn std::error::Error + 'static)> {
            use thiserror::__private::AsDynError;
            #[allow(deprecated)]
            match self {
                DaemonError::ReqwestError { 0: source, .. } => {
                    std::option::Option::Some(source.as_dyn_error())
                }
                DaemonError::SerdeJson { 0: source, .. } => {
                    std::option::Option::Some(source.as_dyn_error())
                }
                DaemonError::ParseIntError { 0: transparent } => {
                    std::error::Error::source(transparent.as_dyn_error())
                }
                DaemonError::IOErr { 0: transparent } => {
                    std::error::Error::source(transparent.as_dyn_error())
                }
                DaemonError::Secp256k1 { 0: transparent } => {
                    std::error::Error::source(transparent.as_dyn_error())
                }
                DaemonError::VarError { 0: transparent } => {
                    std::error::Error::source(transparent.as_dyn_error())
                }
                DaemonError::AnyError { 0: transparent } => {
                    std::error::Error::source(transparent.as_dyn_error())
                }
                DaemonError::Status { 0: transparent } => {
                    std::error::Error::source(transparent.as_dyn_error())
                }
                DaemonError::TransportError { 0: transparent } => {
                    std::error::Error::source(transparent.as_dyn_error())
                }
                DaemonError::TendermintError { 0: transparent } => {
                    std::error::Error::source(transparent.as_dyn_error())
                }
                DaemonError::CwEnvError { 0: transparent } => {
                    std::error::Error::source(transparent.as_dyn_error())
                }
                DaemonError::Bech32DecodeErr { .. } => std::option::Option::None,
                DaemonError::Bech32DecodeExpanded { .. } => std::option::Option::None,
                DaemonError::WrongLength { .. } => std::option::Option::None,
                DaemonError::Phrasing { .. } => std::option::Option::None,
                DaemonError::MissingPhrase { .. } => std::option::Option::None,
                DaemonError::Implementation { .. } => std::option::Option::None,
                DaemonError::Conversion { source: source, .. } => {
                    std::option::Option::Some(source.as_dyn_error())
                }
                DaemonError::SharedDaemonState { .. } => std::option::Option::None,
                DaemonError::ErrReport { 0: transparent } => {
                    std::error::Error::source(transparent.as_dyn_error())
                }
                DaemonError::GRpcDecodeError { 0: transparent } => {
                    std::error::Error::source(transparent.as_dyn_error())
                }
                DaemonError::ED25519 { 0: transparent } => {
                    std::error::Error::source(transparent.as_dyn_error())
                }
                DaemonError::DecodeError { 0: transparent } => {
                    std::error::Error::source(transparent.as_dyn_error())
                }
                DaemonError::HexError { 0: transparent } => {
                    std::error::Error::source(transparent.as_dyn_error())
                }
                DaemonError::BitCoinBip32 { 0: transparent } => {
                    std::error::Error::source(transparent.as_dyn_error())
                }
                DaemonError::ConversionSECP256k1 { .. } => std::option::Option::None,
                DaemonError::ConversionED25519 { .. } => std::option::Option::None,
                DaemonError::ConversionLength { .. } => std::option::Option::None,
                DaemonError::ConversionLengthED25519Hex { .. } => {
                    std::option::Option::None
                }
                DaemonError::ConversionPrefixED25519 { .. } => std::option::Option::None,
                DaemonError::NoGasOpts { .. } => std::option::Option::None,
                DaemonError::CoinParseErrV { .. } => std::option::Option::None,
                DaemonError::CoinParseErr { .. } => std::option::Option::None,
                DaemonError::TxResultError { .. } => std::option::Option::None,
                DaemonError::GasPriceError { .. } => std::option::Option::None,
                DaemonError::TendermintValidatorSet { .. } => std::option::Option::None,
                DaemonError::TXNotFound { .. } => std::option::Option::None,
                DaemonError::Unknown { .. } => std::option::Option::None,
                DaemonError::StdErr { .. } => std::option::Option::None,
                DaemonError::NotImplemented { .. } => std::option::Option::None,
                DaemonError::NewChain { .. } => std::option::Option::None,
                DaemonError::NewNetwork { .. } => std::option::Option::None,
                DaemonError::CannotConnectGRPC { .. } => std::option::Option::None,
                DaemonError::TxFailed { .. } => std::option::Option::None,
                DaemonError::GRPCListIsEmpty { .. } => std::option::Option::None,
                DaemonError::MissingWasmPath { .. } => std::option::Option::None,
                DaemonError::BuilderMissing { .. } => std::option::Option::None,
                DaemonError::IbcError { .. } => std::option::Option::None,
                DaemonError::InsufficientFee { .. } => std::option::Option::None,
            }
        }
    }
    #[allow(unused_qualifications)]
    impl std::fmt::Display for DaemonError {
        fn fmt(&self, __formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            use thiserror::__private::AsDisplay as _;
            #[allow(unused_variables, deprecated, clippy::used_underscore_binding)]
            match self {
                DaemonError::ReqwestError(_0) => {
                    __formatter.write_fmt(format_args!("Reqwest HTTP(s) Error"))
                }
                DaemonError::SerdeJson(_0) => {
                    __formatter.write_fmt(format_args!("JSON Conversion Error"))
                }
                DaemonError::ParseIntError(_0) => std::fmt::Display::fmt(_0, __formatter),
                DaemonError::IOErr(_0) => std::fmt::Display::fmt(_0, __formatter),
                DaemonError::Secp256k1(_0) => std::fmt::Display::fmt(_0, __formatter),
                DaemonError::VarError(_0) => std::fmt::Display::fmt(_0, __formatter),
                DaemonError::AnyError(_0) => std::fmt::Display::fmt(_0, __formatter),
                DaemonError::Status(_0) => std::fmt::Display::fmt(_0, __formatter),
                DaemonError::TransportError(_0) => {
                    std::fmt::Display::fmt(_0, __formatter)
                }
                DaemonError::TendermintError(_0) => {
                    std::fmt::Display::fmt(_0, __formatter)
                }
                DaemonError::CwEnvError(_0) => std::fmt::Display::fmt(_0, __formatter),
                DaemonError::Bech32DecodeErr {} => {
                    __formatter.write_fmt(format_args!("Bech32 Decode Error"))
                }
                DaemonError::Bech32DecodeExpanded(_0, _1, _2, _3) => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "Bech32 Decode Error: Key Failed prefix {0} or length {1} Wanted:{2}/{3}",
                                _0.as_display(),
                                _1.as_display(),
                                _2.as_display(),
                                _3.as_display(),
                            ),
                        )
                }
                DaemonError::WrongLength {} => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "Mnemonic - Wrong length, it should be 24 words",
                            ),
                        )
                }
                DaemonError::Phrasing {} => {
                    __formatter.write_fmt(format_args!("Mnemonic - Bad Phrase"))
                }
                DaemonError::MissingPhrase {} => {
                    __formatter.write_fmt(format_args!("Mnemonic - Missing Phrase"))
                }
                DaemonError::Implementation {} => {
                    __formatter
                        .write_fmt(format_args!("Bad Implementation. Missing Component"))
                }
                DaemonError::Conversion { key, source } => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "Unable to convert into public key `{0}`",
                                key.as_display(),
                            ),
                        )
                }
                DaemonError::SharedDaemonState {} => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "Can not augment daemon deployment after usage in more than one contract.",
                            ),
                        )
                }
                DaemonError::ErrReport(_0) => std::fmt::Display::fmt(_0, __formatter),
                DaemonError::GRpcDecodeError(_0) => {
                    std::fmt::Display::fmt(_0, __formatter)
                }
                DaemonError::ED25519(_0) => std::fmt::Display::fmt(_0, __formatter),
                DaemonError::DecodeError(_0) => std::fmt::Display::fmt(_0, __formatter),
                DaemonError::HexError(_0) => std::fmt::Display::fmt(_0, __formatter),
                DaemonError::BitCoinBip32(_0) => std::fmt::Display::fmt(_0, __formatter),
                DaemonError::ConversionSECP256k1 {} => {
                    __formatter
                        .write_fmt(format_args!("83 length-missing SECP256K1 prefix"))
                }
                DaemonError::ConversionED25519 {} => {
                    __formatter
                        .write_fmt(format_args!("82 length-missing ED25519 prefix"))
                }
                DaemonError::ConversionLength(_0) => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "Expected Key length of 82 or 83 length was {0}",
                                _0.as_display(),
                            ),
                        )
                }
                DaemonError::ConversionLengthED25519Hex(_0) => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "Expected Key length of 40 length was {0}",
                                _0.as_display(),
                            ),
                        )
                }
                DaemonError::ConversionPrefixED25519(_0, _1) => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "Expected ED25519 key of length 32 with a BECH32 ED25519 prefix of 5 chars - Len {0} - Hex {1}",
                                _0.as_display(),
                                _1.as_display(),
                            ),
                        )
                }
                DaemonError::NoGasOpts {} => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "Can\'t call Transactions without some gas rules",
                            ),
                        )
                }
                DaemonError::CoinParseErrV { parse } => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "Can\'t parse `{0}` into a coin",
                                parse.as_display(),
                            ),
                        )
                }
                DaemonError::CoinParseErr(_0) => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "Can\'t parse `{0}` into a coin",
                                _0.as_display(),
                            ),
                        )
                }
                DaemonError::TxResultError(_0, _1, _2) => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "TX submit returned `{0}` - {1} \'{2}\'",
                                _0.as_display(),
                                _1.as_display(),
                                _2.as_display(),
                            ),
                        )
                }
                DaemonError::GasPriceError(_0) => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "No price found for Gas using denom {0}",
                                _0.as_display(),
                            ),
                        )
                }
                DaemonError::TendermintValidatorSet(_0, _1) => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "Attempting to fetch validator set in parts, and failed Height mismatch {0} {1}",
                                _0.as_display(),
                                _1.as_display(),
                            ),
                        )
                }
                DaemonError::TXNotFound(_0, _1) => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "Transaction {0} not found after {1} attempts",
                                _0.as_display(),
                                _1.as_display(),
                            ),
                        )
                }
                DaemonError::Unknown {} => {
                    __formatter.write_fmt(format_args!("unknown API error"))
                }
                DaemonError::StdErr(_0) => {
                    __formatter
                        .write_fmt(format_args!("Generic Error {0}", _0.as_display()))
                }
                DaemonError::NotImplemented {} => {
                    __formatter
                        .write_fmt(
                            format_args!("calling contract with unimplemented action"),
                        )
                }
                DaemonError::NewChain(_0) => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "new chain detected, fill out the scaffold at {0}",
                                _0.as_display(),
                            ),
                        )
                }
                DaemonError::NewNetwork(_0) => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "new network detected, fill out the scaffold at {0}",
                                _0.as_display(),
                            ),
                        )
                }
                DaemonError::CannotConnectGRPC {} => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "Can not connect to any grpc endpoint that was provided.",
                            ),
                        )
                }
                DaemonError::TxFailed { code, reason } => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "tx failed: {0} with code {1}",
                                reason.as_display(),
                                code.as_display(),
                            ),
                        )
                }
                DaemonError::GRPCListIsEmpty {} => {
                    __formatter
                        .write_fmt(format_args!("The list of grpc endpoints is empty"))
                }
                DaemonError::MissingWasmPath {} => {
                    __formatter
                        .write_fmt(format_args!("no wasm path provided for contract."))
                }
                DaemonError::BuilderMissing(_0) => {
                    __formatter
                        .write_fmt(
                            format_args!("daemon builder missing {0}", _0.as_display()),
                        )
                }
                DaemonError::IbcError(_0) => {
                    __formatter
                        .write_fmt(format_args!("ibc error: {0}", _0.as_display()))
                }
                DaemonError::InsufficientFee(_0) => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "insufficient fee, check gas price: {0}",
                                _0.as_display(),
                            ),
                        )
                }
            }
        }
    }
    #[allow(unused_qualifications)]
    impl std::convert::From<::reqwest::Error> for DaemonError {
        #[allow(deprecated)]
        fn from(source: ::reqwest::Error) -> Self {
            DaemonError::ReqwestError {
                0: source,
            }
        }
    }
    #[allow(unused_qualifications)]
    impl std::convert::From<::serde_json::Error> for DaemonError {
        #[allow(deprecated)]
        fn from(source: ::serde_json::Error) -> Self {
            DaemonError::SerdeJson {
                0: source,
            }
        }
    }
    #[allow(unused_qualifications)]
    impl std::convert::From<std::num::ParseIntError> for DaemonError {
        #[allow(deprecated)]
        fn from(source: std::num::ParseIntError) -> Self {
            DaemonError::ParseIntError {
                0: source,
            }
        }
    }
    #[allow(unused_qualifications)]
    impl std::convert::From<::std::io::Error> for DaemonError {
        #[allow(deprecated)]
        fn from(source: ::std::io::Error) -> Self {
            DaemonError::IOErr { 0: source }
        }
    }
    #[allow(unused_qualifications)]
    impl std::convert::From<::secp256k1::Error> for DaemonError {
        #[allow(deprecated)]
        fn from(source: ::secp256k1::Error) -> Self {
            DaemonError::Secp256k1 {
                0: source,
            }
        }
    }
    #[allow(unused_qualifications)]
    impl std::convert::From<::std::env::VarError> for DaemonError {
        #[allow(deprecated)]
        fn from(source: ::std::env::VarError) -> Self {
            DaemonError::VarError { 0: source }
        }
    }
    #[allow(unused_qualifications)]
    impl std::convert::From<::anyhow::Error> for DaemonError {
        #[allow(deprecated)]
        fn from(source: ::anyhow::Error) -> Self {
            DaemonError::AnyError { 0: source }
        }
    }
    #[allow(unused_qualifications)]
    impl std::convert::From<::tonic::Status> for DaemonError {
        #[allow(deprecated)]
        fn from(source: ::tonic::Status) -> Self {
            DaemonError::Status { 0: source }
        }
    }
    #[allow(unused_qualifications)]
    impl std::convert::From<::tonic::transport::Error> for DaemonError {
        #[allow(deprecated)]
        fn from(source: ::tonic::transport::Error) -> Self {
            DaemonError::TransportError {
                0: source,
            }
        }
    }
    #[allow(unused_qualifications)]
    impl std::convert::From<::cosmrs::tendermint::Error> for DaemonError {
        #[allow(deprecated)]
        fn from(source: ::cosmrs::tendermint::Error) -> Self {
            DaemonError::TendermintError {
                0: source,
            }
        }
    }
    #[allow(unused_qualifications)]
    impl std::convert::From<::cw_orch_core::CwEnvError> for DaemonError {
        #[allow(deprecated)]
        fn from(source: ::cw_orch_core::CwEnvError) -> Self {
            DaemonError::CwEnvError {
                0: source,
            }
        }
    }
    #[allow(unused_qualifications)]
    impl std::convert::From<::eyre::ErrReport> for DaemonError {
        #[allow(deprecated)]
        fn from(source: ::eyre::ErrReport) -> Self {
            DaemonError::ErrReport {
                0: source,
            }
        }
    }
    #[allow(unused_qualifications)]
    impl std::convert::From<::prost::DecodeError> for DaemonError {
        #[allow(deprecated)]
        fn from(source: ::prost::DecodeError) -> Self {
            DaemonError::GRpcDecodeError {
                0: source,
            }
        }
    }
    #[allow(unused_qualifications)]
    impl std::convert::From<::ed25519_dalek::ed25519::Error> for DaemonError {
        #[allow(deprecated)]
        fn from(source: ::ed25519_dalek::ed25519::Error) -> Self {
            DaemonError::ED25519 { 0: source }
        }
    }
    #[allow(unused_qualifications)]
    impl std::convert::From<::base64::DecodeError> for DaemonError {
        #[allow(deprecated)]
        fn from(source: ::base64::DecodeError) -> Self {
            DaemonError::DecodeError {
                0: source,
            }
        }
    }
    #[allow(unused_qualifications)]
    impl std::convert::From<::hex::FromHexError> for DaemonError {
        #[allow(deprecated)]
        fn from(source: ::hex::FromHexError) -> Self {
            DaemonError::HexError { 0: source }
        }
    }
    #[allow(unused_qualifications)]
    impl std::convert::From<::bitcoin::bip32::Error> for DaemonError {
        #[allow(deprecated)]
        fn from(source: ::bitcoin::bip32::Error) -> Self {
            DaemonError::BitCoinBip32 {
                0: source,
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for DaemonError {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                DaemonError::ReqwestError(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "ReqwestError",
                        &__self_0,
                    )
                }
                DaemonError::SerdeJson(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "SerdeJson",
                        &__self_0,
                    )
                }
                DaemonError::ParseIntError(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "ParseIntError",
                        &__self_0,
                    )
                }
                DaemonError::IOErr(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "IOErr",
                        &__self_0,
                    )
                }
                DaemonError::Secp256k1(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Secp256k1",
                        &__self_0,
                    )
                }
                DaemonError::VarError(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "VarError",
                        &__self_0,
                    )
                }
                DaemonError::AnyError(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "AnyError",
                        &__self_0,
                    )
                }
                DaemonError::Status(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Status",
                        &__self_0,
                    )
                }
                DaemonError::TransportError(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "TransportError",
                        &__self_0,
                    )
                }
                DaemonError::TendermintError(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "TendermintError",
                        &__self_0,
                    )
                }
                DaemonError::CwEnvError(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "CwEnvError",
                        &__self_0,
                    )
                }
                DaemonError::Bech32DecodeErr => {
                    ::core::fmt::Formatter::write_str(f, "Bech32DecodeErr")
                }
                DaemonError::Bech32DecodeExpanded(
                    __self_0,
                    __self_1,
                    __self_2,
                    __self_3,
                ) => {
                    ::core::fmt::Formatter::debug_tuple_field4_finish(
                        f,
                        "Bech32DecodeExpanded",
                        __self_0,
                        __self_1,
                        __self_2,
                        &__self_3,
                    )
                }
                DaemonError::WrongLength => {
                    ::core::fmt::Formatter::write_str(f, "WrongLength")
                }
                DaemonError::Phrasing => ::core::fmt::Formatter::write_str(f, "Phrasing"),
                DaemonError::MissingPhrase => {
                    ::core::fmt::Formatter::write_str(f, "MissingPhrase")
                }
                DaemonError::Implementation => {
                    ::core::fmt::Formatter::write_str(f, "Implementation")
                }
                DaemonError::Conversion { key: __self_0, source: __self_1 } => {
                    ::core::fmt::Formatter::debug_struct_field2_finish(
                        f,
                        "Conversion",
                        "key",
                        __self_0,
                        "source",
                        &__self_1,
                    )
                }
                DaemonError::SharedDaemonState => {
                    ::core::fmt::Formatter::write_str(f, "SharedDaemonState")
                }
                DaemonError::ErrReport(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "ErrReport",
                        &__self_0,
                    )
                }
                DaemonError::GRpcDecodeError(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "GRpcDecodeError",
                        &__self_0,
                    )
                }
                DaemonError::ED25519(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "ED25519",
                        &__self_0,
                    )
                }
                DaemonError::DecodeError(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "DecodeError",
                        &__self_0,
                    )
                }
                DaemonError::HexError(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "HexError",
                        &__self_0,
                    )
                }
                DaemonError::BitCoinBip32(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "BitCoinBip32",
                        &__self_0,
                    )
                }
                DaemonError::ConversionSECP256k1 => {
                    ::core::fmt::Formatter::write_str(f, "ConversionSECP256k1")
                }
                DaemonError::ConversionED25519 => {
                    ::core::fmt::Formatter::write_str(f, "ConversionED25519")
                }
                DaemonError::ConversionLength(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "ConversionLength",
                        &__self_0,
                    )
                }
                DaemonError::ConversionLengthED25519Hex(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "ConversionLengthED25519Hex",
                        &__self_0,
                    )
                }
                DaemonError::ConversionPrefixED25519(__self_0, __self_1) => {
                    ::core::fmt::Formatter::debug_tuple_field2_finish(
                        f,
                        "ConversionPrefixED25519",
                        __self_0,
                        &__self_1,
                    )
                }
                DaemonError::NoGasOpts => {
                    ::core::fmt::Formatter::write_str(f, "NoGasOpts")
                }
                DaemonError::CoinParseErrV { parse: __self_0 } => {
                    ::core::fmt::Formatter::debug_struct_field1_finish(
                        f,
                        "CoinParseErrV",
                        "parse",
                        &__self_0,
                    )
                }
                DaemonError::CoinParseErr(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "CoinParseErr",
                        &__self_0,
                    )
                }
                DaemonError::TxResultError(__self_0, __self_1, __self_2) => {
                    ::core::fmt::Formatter::debug_tuple_field3_finish(
                        f,
                        "TxResultError",
                        __self_0,
                        __self_1,
                        &__self_2,
                    )
                }
                DaemonError::GasPriceError(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "GasPriceError",
                        &__self_0,
                    )
                }
                DaemonError::TendermintValidatorSet(__self_0, __self_1) => {
                    ::core::fmt::Formatter::debug_tuple_field2_finish(
                        f,
                        "TendermintValidatorSet",
                        __self_0,
                        &__self_1,
                    )
                }
                DaemonError::TXNotFound(__self_0, __self_1) => {
                    ::core::fmt::Formatter::debug_tuple_field2_finish(
                        f,
                        "TXNotFound",
                        __self_0,
                        &__self_1,
                    )
                }
                DaemonError::Unknown => ::core::fmt::Formatter::write_str(f, "Unknown"),
                DaemonError::StdErr(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "StdErr",
                        &__self_0,
                    )
                }
                DaemonError::NotImplemented => {
                    ::core::fmt::Formatter::write_str(f, "NotImplemented")
                }
                DaemonError::NewChain(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "NewChain",
                        &__self_0,
                    )
                }
                DaemonError::NewNetwork(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "NewNetwork",
                        &__self_0,
                    )
                }
                DaemonError::CannotConnectGRPC => {
                    ::core::fmt::Formatter::write_str(f, "CannotConnectGRPC")
                }
                DaemonError::TxFailed { code: __self_0, reason: __self_1 } => {
                    ::core::fmt::Formatter::debug_struct_field2_finish(
                        f,
                        "TxFailed",
                        "code",
                        __self_0,
                        "reason",
                        &__self_1,
                    )
                }
                DaemonError::GRPCListIsEmpty => {
                    ::core::fmt::Formatter::write_str(f, "GRPCListIsEmpty")
                }
                DaemonError::MissingWasmPath => {
                    ::core::fmt::Formatter::write_str(f, "MissingWasmPath")
                }
                DaemonError::BuilderMissing(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "BuilderMissing",
                        &__self_0,
                    )
                }
                DaemonError::IbcError(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "IbcError",
                        &__self_0,
                    )
                }
                DaemonError::InsufficientFee(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "InsufficientFee",
                        &__self_0,
                    )
                }
            }
        }
    }
    impl DaemonError {
        pub fn ibc_err(msg: impl ToString) -> Self {
            Self::IbcError(msg.to_string())
        }
    }
    impl From<DaemonError> for CwEnvError {
        fn from(val: DaemonError) -> Self {
            CwEnvError::AnyError(val.into())
        }
    }
}
pub(crate) mod json_file {
    use serde_json::{from_reader, json, Value};
    use std::fs::{File, OpenOptions};
    pub fn write(
        filename: &String,
        chain_id: &String,
        network_id: &String,
        deploy_id: &String,
    ) {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(filename)
            .unwrap();
        let mut json: Value = if file.metadata().unwrap().len().eq(&0) {
            ::serde_json::Value::Object(::serde_json::Map::new())
        } else {
            from_reader(file).unwrap()
        };
        if json.get(network_id).is_none() {
            json[network_id] = ::serde_json::Value::Object(::serde_json::Map::new());
        }
        if json[network_id].get(chain_id).is_none() {
            json[network_id][chain_id] = ::serde_json::Value::Object({
                let mut object = ::serde_json::Map::new();
                let _ = object
                    .insert(
                        (deploy_id).into(),
                        ::serde_json::Value::Object(::serde_json::Map::new()),
                    );
                let _ = object
                    .insert(
                        ("code_ids").into(),
                        ::serde_json::Value::Object(::serde_json::Map::new()),
                    );
                object
            });
        }
        serde_json::to_writer_pretty(File::create(filename).unwrap(), &json).unwrap();
    }
    pub fn read(filename: &String) -> Value {
        let file = File::open(filename)
            .unwrap_or_else(|_| {
                ::core::panicking::panic_fmt(
                    format_args!("File should be present at {0}", filename),
                );
            });
        let json: serde_json::Value = from_reader(file).unwrap();
        json
    }
}
/// Proto types for different blockchains
pub mod proto {
    pub mod injective {
        #![allow(missing_docs)]
        use crate::DaemonError;
        use cosmrs::tx::SignDoc;
        use cosmrs::{proto::traits::TypeUrl, tx::Raw};
        pub const ETHEREUM_COIN_TYPE: u32 = 60;
        pub struct InjectiveEthAccount {
            #[prost(message, optional, tag = "1")]
            pub base_account: ::core::option::Option<
                super::super::cosmos_modules::auth::BaseAccount,
            >,
            #[prost(bytes, tag = "2")]
            pub code_hash: Vec<u8>,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for InjectiveEthAccount {
            #[inline]
            fn clone(&self) -> InjectiveEthAccount {
                InjectiveEthAccount {
                    base_account: ::core::clone::Clone::clone(&self.base_account),
                    code_hash: ::core::clone::Clone::clone(&self.code_hash),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for InjectiveEthAccount {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for InjectiveEthAccount {
            #[inline]
            fn eq(&self, other: &InjectiveEthAccount) -> bool {
                self.base_account == other.base_account
                    && self.code_hash == other.code_hash
            }
        }
        impl ::prost::Message for InjectiveEthAccount {
            #[allow(unused_variables)]
            fn encode_raw<B>(&self, buf: &mut B)
            where
                B: ::prost::bytes::BufMut,
            {
                if let Some(ref msg) = self.base_account {
                    ::prost::encoding::message::encode(1u32, msg, buf);
                }
                if self.code_hash != b"" as &[u8] {
                    ::prost::encoding::bytes::encode(2u32, &self.code_hash, buf);
                }
            }
            #[allow(unused_variables)]
            fn merge_field<B>(
                &mut self,
                tag: u32,
                wire_type: ::prost::encoding::WireType,
                buf: &mut B,
                ctx: ::prost::encoding::DecodeContext,
            ) -> ::core::result::Result<(), ::prost::DecodeError>
            where
                B: ::prost::bytes::Buf,
            {
                const STRUCT_NAME: &'static str = "InjectiveEthAccount";
                match tag {
                    1u32 => {
                        let mut value = &mut self.base_account;
                        ::prost::encoding::message::merge(
                                wire_type,
                                value.get_or_insert_with(::core::default::Default::default),
                                buf,
                                ctx,
                            )
                            .map_err(|mut error| {
                                error.push(STRUCT_NAME, "base_account");
                                error
                            })
                    }
                    2u32 => {
                        let mut value = &mut self.code_hash;
                        ::prost::encoding::bytes::merge(wire_type, value, buf, ctx)
                            .map_err(|mut error| {
                                error.push(STRUCT_NAME, "code_hash");
                                error
                            })
                    }
                    _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
                }
            }
            #[inline]
            fn encoded_len(&self) -> usize {
                0
                    + self
                        .base_account
                        .as_ref()
                        .map_or(
                            0,
                            |msg| ::prost::encoding::message::encoded_len(1u32, msg),
                        )
                    + if self.code_hash != b"" as &[u8] {
                        ::prost::encoding::bytes::encoded_len(2u32, &self.code_hash)
                    } else {
                        0
                    }
            }
            fn clear(&mut self) {
                self.base_account = ::core::option::Option::None;
                self.code_hash.clear();
            }
        }
        impl ::core::default::Default for InjectiveEthAccount {
            fn default() -> Self {
                InjectiveEthAccount {
                    base_account: ::core::default::Default::default(),
                    code_hash: ::core::default::Default::default(),
                }
            }
        }
        impl ::core::fmt::Debug for InjectiveEthAccount {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let mut builder = f.debug_struct("InjectiveEthAccount");
                let builder = {
                    let wrapper = &self.base_account;
                    builder.field("base_account", &wrapper)
                };
                let builder = {
                    let wrapper = {
                        fn ScalarWrapper<T>(v: T) -> T {
                            v
                        }
                        ScalarWrapper(&self.code_hash)
                    };
                    builder.field("code_hash", &wrapper)
                };
                builder.finish()
            }
        }
        pub struct InjectivePubKey {
            #[prost(bytes, tag = 1)]
            pub key: Vec<u8>,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for InjectivePubKey {
            #[inline]
            fn clone(&self) -> InjectivePubKey {
                InjectivePubKey {
                    key: ::core::clone::Clone::clone(&self.key),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for InjectivePubKey {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for InjectivePubKey {
            #[inline]
            fn eq(&self, other: &InjectivePubKey) -> bool {
                self.key == other.key
            }
        }
        impl ::prost::Message for InjectivePubKey {
            #[allow(unused_variables)]
            fn encode_raw<B>(&self, buf: &mut B)
            where
                B: ::prost::bytes::BufMut,
            {
                if self.key != b"" as &[u8] {
                    ::prost::encoding::bytes::encode(1u32, &self.key, buf);
                }
            }
            #[allow(unused_variables)]
            fn merge_field<B>(
                &mut self,
                tag: u32,
                wire_type: ::prost::encoding::WireType,
                buf: &mut B,
                ctx: ::prost::encoding::DecodeContext,
            ) -> ::core::result::Result<(), ::prost::DecodeError>
            where
                B: ::prost::bytes::Buf,
            {
                const STRUCT_NAME: &'static str = "InjectivePubKey";
                match tag {
                    1u32 => {
                        let mut value = &mut self.key;
                        ::prost::encoding::bytes::merge(wire_type, value, buf, ctx)
                            .map_err(|mut error| {
                                error.push(STRUCT_NAME, "key");
                                error
                            })
                    }
                    _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
                }
            }
            #[inline]
            fn encoded_len(&self) -> usize {
                0
                    + if self.key != b"" as &[u8] {
                        ::prost::encoding::bytes::encoded_len(1u32, &self.key)
                    } else {
                        0
                    }
            }
            fn clear(&mut self) {
                self.key.clear();
            }
        }
        impl ::core::default::Default for InjectivePubKey {
            fn default() -> Self {
                InjectivePubKey {
                    key: ::core::default::Default::default(),
                }
            }
        }
        impl ::core::fmt::Debug for InjectivePubKey {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let mut builder = f.debug_struct("InjectivePubKey");
                let builder = {
                    let wrapper = {
                        fn ScalarWrapper<T>(v: T) -> T {
                            v
                        }
                        ScalarWrapper(&self.key)
                    };
                    builder.field("key", &wrapper)
                };
                builder.finish()
            }
        }
        impl TypeUrl for InjectivePubKey {
            const TYPE_URL: &'static str = "/injective.crypto.v1beta1.ethsecp256k1.PubKey";
        }
        pub trait InjectiveSigner {
            fn sign_injective(&self, sign_doc: SignDoc) -> Result<Raw, DaemonError>;
        }
    }
}
pub mod sender {
    use crate::{networks::ChainKind, proto::injective::ETHEREUM_COIN_TYPE};
    use super::{
        cosmos_modules::{self, auth::BaseAccount},
        error::DaemonError, queriers::{DaemonQuerier, Node},
        state::DaemonState, tx_builder::TxBuilder, tx_resp::CosmTxResponse,
    };
    use crate::proto::injective::InjectiveEthAccount;
    use crate::{core::parse_cw_coins, keys::private::PrivateKey};
    use cosmrs::{
        bank::MsgSend, crypto::secp256k1::SigningKey, proto::traits::Message,
        tendermint::chain::Id,
        tx::{self, ModeInfo, Msg, Raw, SignDoc, SignMode, SignerInfo},
        AccountId,
    };
    use cosmwasm_std::Addr;
    use secp256k1::{All, Context, Secp256k1, Signing};
    use std::{convert::TryFrom, env, rc::Rc, str::FromStr};
    use cosmos_modules::vesting::PeriodicVestingAccount;
    use tonic::transport::Channel;
    /// A wallet is a sender of transactions, can be safely cloned and shared within the same thread.
    pub type Wallet = Rc<Sender<All>>;
    /// Signer of the transactions and helper for address derivation
    /// This is the main interface for simulating and signing transactions
    pub struct Sender<C: Signing + Context> {
        pub private_key: PrivateKey,
        pub secp: Secp256k1<C>,
        pub(crate) daemon_state: Rc<DaemonState>,
    }
    impl Sender<All> {
        pub fn new(daemon_state: &Rc<DaemonState>) -> Result<Sender<All>, DaemonError> {
            let kind = ChainKind::from(daemon_state.chain_data.network_type.clone());
            let mnemonic = env::var(kind.mnemonic_name())
                .unwrap_or_else(|_| {
                    {
                        ::core::panicking::panic_fmt(
                            format_args!(
                                "Wallet mnemonic environment variable {0} not set.",
                                kind.mnemonic_name(),
                            ),
                        );
                    }
                });
            Self::from_mnemonic(daemon_state, &mnemonic)
        }
        /// Construct a new Sender from a mnemonic
        pub fn from_mnemonic(
            daemon_state: &Rc<DaemonState>,
            mnemonic: &str,
        ) -> Result<Sender<All>, DaemonError> {
            let secp = Secp256k1::new();
            let p_key: PrivateKey = PrivateKey::from_words(
                &secp,
                mnemonic,
                0,
                0,
                daemon_state.chain_data.slip44,
            )?;
            let sender = Sender {
                daemon_state: daemon_state.clone(),
                private_key: p_key,
                secp,
            };
            {
                let lvl = ::log::Level::Info;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                    ::log::__private_api::log(
                        format_args!(
                            "Interacting with {0} using address: {1}",
                            daemon_state.chain_data.chain_id,
                            sender.pub_addr_str()?,
                        ),
                        lvl,
                        &(
                            "cw_orch_daemon::sender",
                            "cw_orch_daemon::sender",
                            "cw-orch-daemon/src/sender.rs",
                        ),
                        71u32,
                        ::log::__private_api::Option::None,
                    );
                }
            };
            Ok(sender)
        }
        fn cosmos_private_key(&self) -> SigningKey {
            SigningKey::from_slice(&self.private_key.raw_key()).unwrap()
        }
        pub fn channel(&self) -> Channel {
            self.daemon_state.grpc_channel.clone()
        }
        pub fn pub_addr(&self) -> Result<AccountId, DaemonError> {
            Ok(
                AccountId::new(
                    &self.daemon_state.chain_data.bech32_prefix,
                    &self.private_key.public_key(&self.secp).raw_address.unwrap(),
                )?,
            )
        }
        pub fn address(&self) -> Result<Addr, DaemonError> {
            Ok(Addr::unchecked(self.pub_addr_str()?))
        }
        pub fn pub_addr_str(&self) -> Result<String, DaemonError> {
            Ok(self.pub_addr()?.to_string())
        }
        pub async fn bank_send(
            &self,
            recipient: &str,
            coins: Vec<cosmwasm_std::Coin>,
        ) -> Result<CosmTxResponse, DaemonError> {
            let msg_send = MsgSend {
                from_address: self.pub_addr()?,
                to_address: AccountId::from_str(recipient)?,
                amount: parse_cw_coins(&coins)?,
            };
            self.commit_tx(
                    <[_]>::into_vec(#[rustc_box] ::alloc::boxed::Box::new([msg_send])),
                    Some("sending tokens"),
                )
                .await
        }
        pub async fn calculate_gas(
            &self,
            tx_body: &tx::Body,
            sequence: u64,
            account_number: u64,
        ) -> Result<u64, DaemonError> {
            let fee = TxBuilder::build_fee(
                0u8,
                &self.daemon_state.chain_data.fees.fee_tokens[0].denom,
                0,
            );
            let auth_info = SignerInfo {
                public_key: self.private_key.get_signer_public_key(&self.secp),
                mode_info: ModeInfo::single(SignMode::Direct),
                sequence,
            }
                .auth_info(fee);
            let sign_doc = SignDoc::new(
                tx_body,
                &auth_info,
                &Id::try_from(self.daemon_state.chain_data.chain_id.to_string())?,
                account_number,
            )?;
            let tx_raw = self.sign(sign_doc)?;
            Node::new(self.channel()).simulate_tx(tx_raw.to_bytes()?).await
        }
        pub async fn commit_tx<T: Msg>(
            &self,
            msgs: Vec<T>,
            memo: Option<&str>,
        ) -> Result<CosmTxResponse, DaemonError> {
            let timeout_height = Node::new(self.channel()).block_height().await? + 10u64;
            let tx_body = TxBuilder::build_body(msgs, memo, timeout_height);
            let mut tx_builder = TxBuilder::new(tx_body);
            let tx = tx_builder.build(self).await?;
            let mut tx_response = self.broadcast_tx(tx).await?;
            {
                let lvl = ::log::Level::Debug;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                    ::log::__private_api::log(
                        format_args!("tx broadcast response: {0:?}", tx_response),
                        lvl,
                        &(
                            "cw_orch_daemon::sender",
                            "cw_orch_daemon::sender",
                            "cw-orch-daemon/src/sender.rs",
                        ),
                        166u32,
                        ::log::__private_api::Option::None,
                    );
                }
            };
            if has_insufficient_fee(&tx_response.raw_log) {
                let suggested_fee = parse_suggested_fee(&tx_response.raw_log);
                let Some(new_fee) = suggested_fee else {
                    return Err(DaemonError::InsufficientFee(tx_response.raw_log));
                };
                tx_builder.fee_amount(new_fee);
                let tx = tx_builder.build(self).await?;
                tx_response = self.broadcast_tx(tx).await?;
                {
                    let lvl = ::log::Level::Debug;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            format_args!("tx broadcast response: {0:?}", tx_response),
                            lvl,
                            &(
                                "cw_orch_daemon::sender",
                                "cw_orch_daemon::sender",
                                "cw-orch-daemon/src/sender.rs",
                            ),
                            181u32,
                            ::log::__private_api::Option::None,
                        );
                    }
                };
            }
            let resp = Node::new(self.channel()).find_tx(tx_response.txhash).await?;
            if resp.code == 0 {
                Ok(resp)
            } else {
                Err(DaemonError::TxFailed {
                    code: resp.code,
                    reason: resp.raw_log,
                })
            }
        }
        pub fn sign(&self, sign_doc: SignDoc) -> Result<Raw, DaemonError> {
            let tx_raw = if self.private_key.coin_type == ETHEREUM_COIN_TYPE {
                {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "Coin Type {0} not supported without eth feature",
                            ETHEREUM_COIN_TYPE,
                        ),
                    );
                };
            } else {
                sign_doc.sign(&self.cosmos_private_key())?
            };
            Ok(tx_raw)
        }
        pub async fn base_account(&self) -> Result<BaseAccount, DaemonError> {
            let addr = self.pub_addr().unwrap().to_string();
            let mut client = cosmos_modules::auth::query_client::QueryClient::new(
                self.channel(),
            );
            let resp = client
                .account(cosmos_modules::auth::QueryAccountRequest {
                    address: addr,
                })
                .await?
                .into_inner();
            let account = resp.account.unwrap().value;
            let acc = if let Ok(acc) = BaseAccount::decode(account.as_ref()) {
                acc
            } else if let Ok(acc) = PeriodicVestingAccount::decode(account.as_ref()) {
                acc.base_vesting_account.unwrap().base_account.unwrap()
            } else if let Ok(acc) = InjectiveEthAccount::decode(account.as_ref()) {
                acc.base_account.unwrap()
            } else {
                return Err(
                    DaemonError::StdErr(
                        "Unknown account type returned from QueryAccountRequest".into(),
                    ),
                );
            };
            Ok(acc)
        }
        async fn broadcast_tx(
            &self,
            tx: Raw,
        ) -> Result<
            cosmrs::proto::cosmos::base::abci::v1beta1::TxResponse,
            DaemonError,
        > {
            let mut client = cosmos_modules::tx::service_client::ServiceClient::new(
                self.channel(),
            );
            let commit = client
                .broadcast_tx(cosmos_modules::tx::BroadcastTxRequest {
                    tx_bytes: tx.to_bytes()?,
                    mode: cosmos_modules::tx::BroadcastMode::Sync.into(),
                })
                .await?;
            let commit = commit.into_inner().tx_response.unwrap();
            Ok(commit)
        }
    }
    fn has_insufficient_fee(raw_log: &str) -> bool {
        raw_log.contains("insufficient fees")
    }
    fn parse_suggested_fee(raw_log: &str) -> Option<u128> {
        let parts: Vec<&str> = raw_log.split("required: ").collect();
        if parts.len() != 2 {
            return None;
        }
        let got_parts: Vec<&str> = parts[0].split_whitespace().collect();
        let paid_fee_with_denom = got_parts.last()?;
        let (_, denomination) = paid_fee_with_denom
            .split_at(paid_fee_with_denom.find(|c: char| !c.is_numeric())?);
        {
            ::std::io::_eprint(format_args!("denom: {0}\n", denomination));
        };
        let required_fees: Vec<&str> = parts[1].split(denomination).collect();
        {
            ::std::io::_eprint(format_args!("required fees: {0:?}\n", required_fees));
        };
        let (_, suggested_fee) = required_fees[0]
            .split_at(required_fees[0].rfind(|c: char| !c.is_numeric())?);
        {
            ::std::io::_eprint(format_args!("suggested fee: {0}\n", suggested_fee));
        };
        suggested_fee.parse::<u128>().ok().or(suggested_fee[1..].parse::<u128>().ok())
    }
}
pub mod state {
    use super::error::DaemonError;
    use crate::{channel::GrpcChannel, networks::ChainKind};
    use cosmwasm_std::Addr;
    use cw_orch_core::{
        environment::{DeployDetails, StateInterface},
        CwEnvError,
    };
    use ibc_chain_registry::chain::ChainData;
    use serde::Serialize;
    use serde_json::{json, Value};
    use std::{collections::HashMap, env, fs::File, path::Path};
    use tonic::transport::Channel;
    /// Stores the chain information and deployment state.
    /// Uses a simple JSON file to store the deployment information locally.
    pub struct DaemonState {
        /// this is passed via env var STATE_FILE
        pub json_file_path: String,
        /// Deployment identifier
        pub deployment_id: String,
        /// gRPC channel
        pub grpc_channel: Channel,
        /// Information about the chain
        pub chain_data: ChainData,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for DaemonState {
        #[inline]
        fn clone(&self) -> DaemonState {
            DaemonState {
                json_file_path: ::core::clone::Clone::clone(&self.json_file_path),
                deployment_id: ::core::clone::Clone::clone(&self.deployment_id),
                grpc_channel: ::core::clone::Clone::clone(&self.grpc_channel),
                chain_data: ::core::clone::Clone::clone(&self.chain_data),
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for DaemonState {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field4_finish(
                f,
                "DaemonState",
                "json_file_path",
                &self.json_file_path,
                "deployment_id",
                &self.deployment_id,
                "grpc_channel",
                &self.grpc_channel,
                "chain_data",
                &&self.chain_data,
            )
        }
    }
    impl DaemonState {
        /// Creates a new state from the given chain data and deployment id.
        /// Attempts to connect to any of the provided gRPC endpoints.
        pub async fn new(
            mut chain_data: ChainData,
            deployment_id: String,
        ) -> Result<DaemonState, DaemonError> {
            if chain_data.apis.grpc.is_empty() {
                return Err(DaemonError::GRPCListIsEmpty);
            }
            {
                let lvl = ::log::Level::Info;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                    ::log::__private_api::log(
                        format_args!(
                            "Found {0} gRPC endpoints",
                            chain_data.apis.grpc.len(),
                        ),
                        lvl,
                        &(
                            "cw_orch_daemon::state",
                            "cw_orch_daemon::state",
                            "cw-orch-daemon/src/state.rs",
                        ),
                        40u32,
                        ::log::__private_api::Option::None,
                    );
                }
            };
            let grpc_channel = GrpcChannel::connect(
                    &chain_data.apis.grpc,
                    &chain_data.chain_id,
                )
                .await?;
            let mut json_file_path = env::var("STATE_FILE")
                .unwrap_or("./state.json".to_string());
            if chain_data.network_type == ChainKind::Local.to_string() {
                let name = Path::new(&json_file_path)
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap();
                let folder = Path::new(&json_file_path)
                    .parent()
                    .unwrap()
                    .to_str()
                    .unwrap();
                json_file_path = {
                    let res = ::alloc::fmt::format(
                        format_args!("{0}/{1}_local.json", folder, name),
                    );
                    res
                };
            }
            let shortest_denom_token = chain_data
                .fees
                .fee_tokens
                .iter()
                .fold(
                    chain_data.fees.fee_tokens[0].clone(),
                    |acc, item| {
                        if item.denom.len() < acc.denom.len() {
                            item.clone()
                        } else {
                            acc
                        }
                    },
                );
            chain_data
                .fees
                .fee_tokens = <[_]>::into_vec(
                #[rustc_box]
                ::alloc::boxed::Box::new([shortest_denom_token]),
            );
            let state = DaemonState {
                json_file_path,
                deployment_id,
                grpc_channel,
                chain_data,
            };
            {
                let lvl = ::log::Level::Info;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                    ::log::__private_api::log(
                        format_args!(
                            "Writing daemon state JSON file: {0:#?}",
                            state.json_file_path,
                        ),
                        lvl,
                        &(
                            "cw_orch_daemon::state",
                            "cw_orch_daemon::state",
                            "cw-orch-daemon/src/state.rs",
                        ),
                        87u32,
                        ::log::__private_api::Option::None,
                    );
                }
            };
            crate::json_file::write(
                &state.json_file_path,
                &state.chain_data.chain_id.to_string(),
                &state.chain_data.chain_name,
                &state.deployment_id,
            );
            Ok(state)
        }
        /// Get the state filepath and read it as json
        fn read_state(&self) -> serde_json::Value {
            crate::json_file::read(&self.json_file_path)
        }
        /// Retrieve a stateful value using the chainId and networkId
        pub fn get(&self, key: &str) -> Value {
            let json = self.read_state();
            json[&self.chain_data.chain_name][&self.chain_data.chain_id.to_string()][key]
                .clone()
        }
        /// Set a stateful value using the chainId and networkId
        pub fn set<T: Serialize>(&self, key: &str, contract_id: &str, value: T) {
            let mut json = self.read_state();
            json[&self
                .chain_data
                .chain_name][&self
                .chain_data
                .chain_id
                .to_string()][key][contract_id] = ::serde_json::to_value(&value)
                .unwrap();
            serde_json::to_writer_pretty(
                    File::create(&self.json_file_path).unwrap(),
                    &json,
                )
                .unwrap();
        }
    }
    impl StateInterface for DaemonState {
        /// Read address for contract in deployment id from state file
        fn get_address(&self, contract_id: &str) -> Result<Addr, CwEnvError> {
            let value = self
                .get(&self.deployment_id)
                .get(contract_id)
                .ok_or_else(|| CwEnvError::AddrNotInStore(contract_id.to_owned()))?
                .clone();
            Ok(Addr::unchecked(value.as_str().unwrap()))
        }
        /// Set address for contract in deployment id in state file
        fn set_address(&mut self, contract_id: &str, address: &Addr) {
            self.set(&self.deployment_id, contract_id, address.as_str());
        }
        /// Get the locally-saved version of the contract's version on this network
        fn get_code_id(&self, contract_id: &str) -> Result<u64, CwEnvError> {
            let value = self
                .get("code_ids")
                .get(contract_id)
                .ok_or_else(|| CwEnvError::CodeIdNotInStore(contract_id.to_owned()))?
                .clone();
            Ok(value.as_u64().unwrap())
        }
        /// Set the locally-saved version of the contract's latest version on this network
        fn set_code_id(&mut self, contract_id: &str, code_id: u64) {
            self.set("code_ids", contract_id, code_id);
        }
        /// Get all addresses for deployment id from state file
        fn get_all_addresses(&self) -> Result<HashMap<String, Addr>, CwEnvError> {
            let mut store = HashMap::new();
            let addresses = self.get(&self.deployment_id);
            let value = addresses.as_object().unwrap();
            for (id, addr) in value {
                store.insert(id.clone(), Addr::unchecked(addr.as_str().unwrap()));
            }
            Ok(store)
        }
        fn get_all_code_ids(&self) -> Result<HashMap<String, u64>, CwEnvError> {
            let mut store = HashMap::new();
            let code_ids = self.get("code_ids");
            let value = code_ids.as_object().unwrap();
            for (id, code_id) in value {
                store.insert(id.clone(), code_id.as_u64().unwrap());
            }
            Ok(store)
        }
        fn deploy_details(&self) -> DeployDetails {
            DeployDetails {
                chain_id: self.chain_data.chain_id.to_string(),
                chain_name: self.chain_data.chain_name.clone(),
                deployment_id: self.deployment_id.clone(),
            }
        }
    }
}
pub mod sync {
    mod builder {
        use ibc_chain_registry::chain::ChainData;
        use crate::DaemonAsyncBuilder;
        use super::{super::error::DaemonError, core::Daemon};
        /// Create [`Daemon`] through [`DaemonBuilder`]
        /// ## Example
        /// ```no_run
        ///     use cw_orch_daemon::{networks, DaemonBuilder};
        ///
        ///     let Daemon = DaemonBuilder::default()
        ///         .chain(networks::LOCAL_JUNO)
        ///         .deployment_id("v0.1.0")
        ///         .build()
        ///         .unwrap();
        /// ```
        pub struct DaemonBuilder {
            pub(crate) chain: Option<ChainData>,
            pub(crate) handle: Option<tokio::runtime::Handle>,
            pub(crate) deployment_id: Option<String>,
            /// Wallet mnemonic
            pub(crate) mnemonic: Option<String>,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for DaemonBuilder {
            #[inline]
            fn clone(&self) -> DaemonBuilder {
                DaemonBuilder {
                    chain: ::core::clone::Clone::clone(&self.chain),
                    handle: ::core::clone::Clone::clone(&self.handle),
                    deployment_id: ::core::clone::Clone::clone(&self.deployment_id),
                    mnemonic: ::core::clone::Clone::clone(&self.mnemonic),
                }
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for DaemonBuilder {
            #[inline]
            fn default() -> DaemonBuilder {
                DaemonBuilder {
                    chain: ::core::default::Default::default(),
                    handle: ::core::default::Default::default(),
                    deployment_id: ::core::default::Default::default(),
                    mnemonic: ::core::default::Default::default(),
                }
            }
        }
        impl DaemonBuilder {
            /// Set the chain the Daemon will connect to
            pub fn chain(&mut self, chain: impl Into<ChainData>) -> &mut Self {
                self.chain = Some(chain.into());
                self
            }
            /// Set the deployment id to use for the Daemon interactions
            /// Defaults to `default`
            pub fn deployment_id(
                &mut self,
                deployment_id: impl Into<String>,
            ) -> &mut Self {
                self.deployment_id = Some(deployment_id.into());
                self
            }
            /// Set the tokio runtime handle to use for the Daemon
            ///
            /// ## Example
            /// ```no_run
            /// use cw_orch_daemon::Daemon;
            /// use tokio::runtime::Runtime;
            /// let rt = Runtime::new().unwrap();
            /// let Daemon = Daemon::builder()
            ///     .handle(rt.handle())
            ///     // ...
            ///     .build()
            ///     .unwrap();
            /// ```
            pub fn handle(&mut self, handle: &tokio::runtime::Handle) -> &mut Self {
                self.handle = Some(handle.clone());
                self
            }
            /// Set the mnemonic to use with this chain.
            pub fn mnemonic(&mut self, mnemonic: impl ToString) -> &mut Self {
                self.mnemonic = Some(mnemonic.to_string());
                self
            }
            /// Build a Daemon
            pub fn build(&self) -> Result<Daemon, DaemonError> {
                let rt_handle = self
                    .handle
                    .clone()
                    .ok_or(DaemonError::BuilderMissing("runtime handle".into()))?;
                let daemon = rt_handle
                    .block_on(DaemonAsyncBuilder::from(self.clone()).build())?;
                Ok(Daemon { rt_handle, daemon })
            }
        }
    }
    mod core {
        use std::{fmt::Debug, rc::Rc, time::Duration};
        use super::super::{sender::Wallet, DaemonAsync};
        use crate::{
            queriers::{DaemonQuerier, Node},
            CosmTxResponse, DaemonBuilder, DaemonError, DaemonState,
        };
        use cosmrs::tendermint::Time;
        use cosmwasm_std::{Addr, Coin};
        use cw_orch_core::{
            contract::{interface_traits::Uploadable, WasmPath},
            environment::{ChainState, TxHandler},
        };
        use serde::{de::DeserializeOwned, Serialize};
        use tokio::runtime::Handle;
        use tonic::transport::Channel;
        /**
    Represents a blockchain node.
    Is constructed with the [DaemonBuilder].

    ## Usage

    ```rust,no_run
    use cw_orch_daemon::{Daemon, networks};
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();
    let daemon: Daemon = Daemon::builder()
        .chain(networks::JUNO_1)
        .handle(rt.handle())
        .build()
        .unwrap();
    ```
    ## Environment Execution

    The Daemon implements [`TxHandler`] which allows you to perform transactions on the chain.

    ## Querying

    Different Cosmos SDK modules can be queried through the daemon by calling the [`Daemon.query_client<Querier>`] method with a specific querier.
    See [Querier](crate::queriers) for examples.
*/
        pub struct Daemon {
            pub daemon: DaemonAsync,
            /// Runtime handle to execute async tasks
            pub rt_handle: Handle,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Daemon {
            #[inline]
            fn clone(&self) -> Daemon {
                Daemon {
                    daemon: ::core::clone::Clone::clone(&self.daemon),
                    rt_handle: ::core::clone::Clone::clone(&self.rt_handle),
                }
            }
        }
        impl Daemon {
            /// Get the daemon builder
            pub fn builder() -> DaemonBuilder {
                DaemonBuilder::default()
            }
            /// Perform a query with a given querier
            /// See [Querier](crate::queriers) for examples.
            pub fn query_client<Querier: DaemonQuerier>(&self) -> Querier {
                self.daemon.query_client()
            }
            /// Get the channel configured for this Daemon
            pub fn channel(&self) -> Channel {
                self.daemon.state.grpc_channel.clone()
            }
            /// Get the channel configured for this Daemon
            pub fn wallet(&self) -> Wallet {
                self.daemon.sender.clone()
            }
        }
        impl ChainState for Daemon {
            type Out = Rc<DaemonState>;
            fn state(&self) -> Self::Out {
                self.daemon.state.clone()
            }
        }
        impl TxHandler for Daemon {
            type Response = CosmTxResponse;
            type Error = DaemonError;
            type ContractSource = WasmPath;
            type Sender = Wallet;
            fn sender(&self) -> Addr {
                self.daemon.sender.address().unwrap()
            }
            fn set_sender(&mut self, sender: Self::Sender) {
                self.daemon.sender = sender;
            }
            fn upload(
                &self,
                uploadable: &impl Uploadable,
            ) -> Result<Self::Response, DaemonError> {
                self.rt_handle.block_on(self.daemon.upload(uploadable))
            }
            fn execute<E: Serialize>(
                &self,
                exec_msg: &E,
                coins: &[cosmwasm_std::Coin],
                contract_address: &Addr,
            ) -> Result<Self::Response, DaemonError> {
                self.rt_handle
                    .block_on(self.daemon.execute(exec_msg, coins, contract_address))
            }
            fn instantiate<I: Serialize + Debug>(
                &self,
                code_id: u64,
                init_msg: &I,
                label: Option<&str>,
                admin: Option<&Addr>,
                coins: &[Coin],
            ) -> Result<Self::Response, DaemonError> {
                self.rt_handle
                    .block_on(
                        self.daemon.instantiate(code_id, init_msg, label, admin, coins),
                    )
            }
            fn query<Q: Serialize + Debug, T: Serialize + DeserializeOwned>(
                &self,
                query_msg: &Q,
                contract_address: &Addr,
            ) -> Result<T, DaemonError> {
                self.rt_handle.block_on(self.daemon.query(query_msg, contract_address))
            }
            fn migrate<M: Serialize + Debug>(
                &self,
                migrate_msg: &M,
                new_code_id: u64,
                contract_address: &Addr,
            ) -> Result<Self::Response, DaemonError> {
                self.rt_handle
                    .block_on(
                        self.daemon.migrate(migrate_msg, new_code_id, contract_address),
                    )
            }
            fn wait_blocks(&self, amount: u64) -> Result<(), DaemonError> {
                let mut last_height = self
                    .rt_handle
                    .block_on(self.query_client::<Node>().block_height())?;
                let end_height = last_height + amount;
                while last_height < end_height {
                    self.rt_handle.block_on(tokio::time::sleep(Duration::from_secs(4)));
                    last_height = self
                        .rt_handle
                        .block_on(self.query_client::<Node>().block_height())?;
                }
                Ok(())
            }
            fn wait_seconds(&self, secs: u64) -> Result<(), DaemonError> {
                self.rt_handle.block_on(tokio::time::sleep(Duration::from_secs(secs)));
                Ok(())
            }
            fn next_block(&self) -> Result<(), DaemonError> {
                let mut last_height = self
                    .rt_handle
                    .block_on(self.query_client::<Node>().block_height())?;
                let end_height = last_height + 1;
                while last_height < end_height {
                    self.rt_handle.block_on(tokio::time::sleep(Duration::from_secs(4)));
                    last_height = self
                        .rt_handle
                        .block_on(self.query_client::<Node>().block_height())?;
                }
                Ok(())
            }
            fn block_info(&self) -> Result<cosmwasm_std::BlockInfo, DaemonError> {
                let block = self
                    .rt_handle
                    .block_on(self.query_client::<Node>().latest_block())?;
                let since_epoch = block.header.time.duration_since(Time::unix_epoch())?;
                let time = cosmwasm_std::Timestamp::from_nanos(
                    since_epoch.as_nanos() as u64,
                );
                Ok(cosmwasm_std::BlockInfo {
                    height: block.header.height.value(),
                    time,
                    chain_id: block.header.chain_id.to_string(),
                })
            }
        }
    }
    pub use self::{builder::*, core::*};
}
pub mod tx_resp {
    use super::{
        cosmos_modules::{
            abci::{AbciMessageLog, Attribute, StringEvent, TxResponse},
            tendermint_abci::Event,
        },
        error::DaemonError,
    };
    use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
    use cosmwasm_std::{to_binary, Binary, StdError, StdResult};
    use cw_orch_core::environment::IndexResponse;
    use serde::{Deserialize, Serialize};
    const FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.f";
    const FORMAT_TZ_SUPPLIED: &str = "%Y-%m-%dT%H:%M:%S.%f%:z";
    const FORMAT_SHORT_Z: &str = "%Y-%m-%dT%H:%M:%SZ";
    const FORMAT_SHORT_Z2: &str = "%Y-%m-%dT%H:%M:%S.%fZ";
    /// The response from a transaction performed on a blockchain.
    pub struct CosmTxResponse {
        /// Height of the block in which the transaction was included.
        pub height: u64,
        /// Transaction hash.
        pub txhash: String,
        /// Transaction index within the block.
        pub codespace: String,
        /// Transaction result code
        pub code: usize,
        /// Arbitrary data that can be included in a transaction.
        pub data: String,
        /// Raw log message.
        pub raw_log: String,
        /// Logs of the transaction.
        pub logs: Vec<TxResultBlockMsg>,
        /// Transaction info.
        pub info: String,
        /// Gas limit.
        pub gas_wanted: u64,
        /// Gas used.
        pub gas_used: u64,
        /// Timestamp of the block in which the transaction was included.
        pub timestamp: DateTime<Utc>,
        /// Transaction events.
        pub events: Vec<Event>,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for CosmTxResponse {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "height",
                "txhash",
                "codespace",
                "code",
                "data",
                "raw_log",
                "logs",
                "info",
                "gas_wanted",
                "gas_used",
                "timestamp",
                "events",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.height,
                &self.txhash,
                &self.codespace,
                &self.code,
                &self.data,
                &self.raw_log,
                &self.logs,
                &self.info,
                &self.gas_wanted,
                &self.gas_used,
                &self.timestamp,
                &&self.events,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "CosmTxResponse",
                names,
                values,
            )
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for CosmTxResponse {
        #[inline]
        fn default() -> CosmTxResponse {
            CosmTxResponse {
                height: ::core::default::Default::default(),
                txhash: ::core::default::Default::default(),
                codespace: ::core::default::Default::default(),
                code: ::core::default::Default::default(),
                data: ::core::default::Default::default(),
                raw_log: ::core::default::Default::default(),
                logs: ::core::default::Default::default(),
                info: ::core::default::Default::default(),
                gas_wanted: ::core::default::Default::default(),
                gas_used: ::core::default::Default::default(),
                timestamp: ::core::default::Default::default(),
                events: ::core::default::Default::default(),
            }
        }
    }
    impl CosmTxResponse {
        /// find a attribute's value from TX logs.
        /// returns: msg_index and value
        pub fn get_attribute_from_logs(
            &self,
            event_type: &str,
            attribute_key: &str,
        ) -> Vec<(usize, String)> {
            let mut response: Vec<(usize, String)> = Default::default();
            let logs = &self.logs;
            for log_part in logs {
                let msg_index = log_part.msg_index.unwrap_or_default();
                let events = &log_part.events;
                let events_filtered = events
                    .iter()
                    .filter(|event| event.s_type == event_type)
                    .collect::<Vec<_>>();
                if let Some(event) = events_filtered.first() {
                    let attributes_filtered = event
                        .attributes
                        .iter()
                        .filter(|attr| attr.key == attribute_key)
                        .map(|f| f.value.clone())
                        .collect::<Vec<_>>();
                    if let Some(attr_key) = attributes_filtered.first() {
                        response.push((msg_index, attr_key.clone()));
                    }
                }
            }
            response
        }
        /// get the list of event types from a TX record
        pub fn get_events(&self, event_type: &str) -> Vec<TxResultBlockEvent> {
            let mut response: Vec<TxResultBlockEvent> = Default::default();
            for log_part in &self.logs {
                let events = &log_part.events;
                let events_filtered = events
                    .iter()
                    .filter(|event| event.s_type == event_type)
                    .collect::<Vec<_>>();
                for event in events_filtered {
                    response.push(event.clone());
                }
            }
            response
        }
    }
    impl From<&serde_json::Value> for TxResultBlockMsg {
        fn from(value: &serde_json::Value) -> Self {
            serde_json::from_value(value.clone()).unwrap()
        }
    }
    impl From<TxResponse> for CosmTxResponse {
        fn from(tx: TxResponse) -> Self {
            Self {
                height: tx.height as u64,
                txhash: tx.txhash,
                codespace: tx.codespace,
                code: tx.code as usize,
                data: tx.data,
                raw_log: tx.raw_log,
                logs: tx.logs.into_iter().map(TxResultBlockMsg::from).collect(),
                info: tx.info,
                gas_wanted: tx.gas_wanted as u64,
                gas_used: tx.gas_used as u64,
                timestamp: parse_timestamp(tx.timestamp).unwrap(),
                events: tx.events,
            }
        }
    }
    impl IndexResponse for CosmTxResponse {
        fn events(&self) -> Vec<cosmwasm_std::Event> {
            let mut parsed_events = ::alloc::vec::Vec::new();
            for event in &self.events {
                let mut pattr = ::alloc::vec::Vec::new();
                for attr in &event.attributes {
                    pattr
                        .push(cosmwasm_std::Attribute {
                            key: attr.key.clone(),
                            value: attr.value.clone(),
                        })
                }
                let pevent = cosmwasm_std::Event::new(event.r#type.clone())
                    .add_attributes(pattr);
                parsed_events.push(pevent);
            }
            parsed_events
        }
        fn data(&self) -> Option<Binary> {
            if self.data.is_empty() {
                None
            } else {
                Some(to_binary(self.data.as_bytes()).unwrap())
            }
        }
        fn event_attr_value(
            &self,
            event_type: &str,
            attr_key: &str,
        ) -> StdResult<String> {
            for event in &self.events {
                if event.r#type == event_type {
                    for attr in &event.attributes {
                        if attr.key == attr_key {
                            return Ok(attr.value.clone());
                        }
                    }
                }
            }
            Err(
                StdError::generic_err({
                    let res = ::alloc::fmt::format(
                        format_args!(
                            "event of type {0} does not have a value at key {1}",
                            event_type,
                            attr_key,
                        ),
                    );
                    res
                }),
            )
        }
    }
    /// The events from a single message in a transaction.
    pub struct TxResultBlockMsg {
        /// index of the message in the transaction
        pub msg_index: Option<usize>,
        /// Events from this message
        pub events: Vec<TxResultBlockEvent>,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for TxResultBlockMsg {
        #[inline]
        fn clone(&self) -> TxResultBlockMsg {
            TxResultBlockMsg {
                msg_index: ::core::clone::Clone::clone(&self.msg_index),
                events: ::core::clone::Clone::clone(&self.events),
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for TxResultBlockMsg {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "TxResultBlockMsg",
                    false as usize + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "msg_index",
                    &self.msg_index,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "events",
                    &self.events,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for TxResultBlockMsg {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "msg_index" => _serde::__private::Ok(__Field::__field0),
                            "events" => _serde::__private::Ok(__Field::__field1),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"msg_index" => _serde::__private::Ok(__Field::__field0),
                            b"events" => _serde::__private::Ok(__Field::__field1),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                }
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<TxResultBlockMsg>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = TxResultBlockMsg;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct TxResultBlockMsg",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            Option<usize>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct TxResultBlockMsg with 2 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            Vec<TxResultBlockEvent>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct TxResultBlockMsg with 2 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private::Ok(TxResultBlockMsg {
                            msg_index: __field0,
                            events: __field1,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<Option<usize>> = _serde::__private::None;
                        let mut __field1: _serde::__private::Option<
                            Vec<TxResultBlockEvent>,
                        > = _serde::__private::None;
                        while let _serde::__private::Some(__key)
                            = _serde::de::MapAccess::next_key::<__Field>(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "msg_index",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Option<usize>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("events"),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Vec<TxResultBlockEvent>,
                                        >(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("msg_index")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("events")?
                            }
                        };
                        _serde::__private::Ok(TxResultBlockMsg {
                            msg_index: __field0,
                            events: __field1,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &["msg_index", "events"];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "TxResultBlockMsg",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<TxResultBlockMsg>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::fmt::Debug for TxResultBlockMsg {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "TxResultBlockMsg",
                "msg_index",
                &self.msg_index,
                "events",
                &&self.events,
            )
        }
    }
    impl From<AbciMessageLog> for TxResultBlockMsg {
        fn from(msg: AbciMessageLog) -> Self {
            Self {
                msg_index: Some(msg.msg_index as usize),
                events: msg.events.into_iter().map(TxResultBlockEvent::from).collect(),
            }
        }
    }
    /// A single event from a transaction and its attributes.
    pub struct TxResultBlockEvent {
        #[serde(rename = "type")]
        /// Type of the event
        pub s_type: String,
        /// Attributes of the event
        pub attributes: Vec<TxResultBlockAttribute>,
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for TxResultBlockEvent {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "type" => _serde::__private::Ok(__Field::__field0),
                            "attributes" => _serde::__private::Ok(__Field::__field1),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"type" => _serde::__private::Ok(__Field::__field0),
                            b"attributes" => _serde::__private::Ok(__Field::__field1),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                }
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<TxResultBlockEvent>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = TxResultBlockEvent;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct TxResultBlockEvent",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct TxResultBlockEvent with 2 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            Vec<TxResultBlockAttribute>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct TxResultBlockEvent with 2 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private::Ok(TxResultBlockEvent {
                            s_type: __field0,
                            attributes: __field1,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field1: _serde::__private::Option<
                            Vec<TxResultBlockAttribute>,
                        > = _serde::__private::None;
                        while let _serde::__private::Some(__key)
                            = _serde::de::MapAccess::next_key::<__Field>(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("type"),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "attributes",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Vec<TxResultBlockAttribute>,
                                        >(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("type")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("attributes")?
                            }
                        };
                        _serde::__private::Ok(TxResultBlockEvent {
                            s_type: __field0,
                            attributes: __field1,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &["type", "attributes"];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "TxResultBlockEvent",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<TxResultBlockEvent>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for TxResultBlockEvent {
        #[inline]
        fn clone(&self) -> TxResultBlockEvent {
            TxResultBlockEvent {
                s_type: ::core::clone::Clone::clone(&self.s_type),
                attributes: ::core::clone::Clone::clone(&self.attributes),
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for TxResultBlockEvent {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "TxResultBlockEvent",
                    false as usize + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "type",
                    &self.s_type,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "attributes",
                    &self.attributes,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[automatically_derived]
    impl ::core::fmt::Debug for TxResultBlockEvent {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "TxResultBlockEvent",
                "s_type",
                &self.s_type,
                "attributes",
                &&self.attributes,
            )
        }
    }
    impl From<StringEvent> for TxResultBlockEvent {
        fn from(event: StringEvent) -> Self {
            Self {
                s_type: event.r#type,
                attributes: event
                    .attributes
                    .into_iter()
                    .map(TxResultBlockAttribute::from)
                    .collect(),
            }
        }
    }
    impl TxResultBlockEvent {
        /// get all key/values from the event that have the key 'key'
        pub fn get_attributes(&self, key: &str) -> Vec<TxResultBlockAttribute> {
            self.attributes.iter().filter(|attr| attr.key == key).cloned().collect()
        }
        /// return the first value of the first attribute that has the key 'key'
        pub fn get_first_attribute_value(&self, key: &str) -> Option<String> {
            self.get_attributes(key).first().map(|attr| attr.value.clone())
        }
    }
    /// A single attribute of an event.
    pub struct TxResultBlockAttribute {
        /// Key of the attribute
        pub key: String,
        /// Value of the attribute
        pub value: String,
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for TxResultBlockAttribute {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "key" => _serde::__private::Ok(__Field::__field0),
                            "value" => _serde::__private::Ok(__Field::__field1),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"key" => _serde::__private::Ok(__Field::__field0),
                            b"value" => _serde::__private::Ok(__Field::__field1),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                }
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<TxResultBlockAttribute>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = TxResultBlockAttribute;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct TxResultBlockAttribute",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct TxResultBlockAttribute with 2 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct TxResultBlockAttribute with 2 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private::Ok(TxResultBlockAttribute {
                            key: __field0,
                            value: __field1,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field1: _serde::__private::Option<String> = _serde::__private::None;
                        while let _serde::__private::Some(__key)
                            = _serde::de::MapAccess::next_key::<__Field>(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("key"),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("value"),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("key")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("value")?
                            }
                        };
                        _serde::__private::Ok(TxResultBlockAttribute {
                            key: __field0,
                            value: __field1,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &["key", "value"];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "TxResultBlockAttribute",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<TxResultBlockAttribute>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for TxResultBlockAttribute {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "TxResultBlockAttribute",
                    false as usize + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "key",
                    &self.key,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "value",
                    &self.value,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for TxResultBlockAttribute {
        #[inline]
        fn clone(&self) -> TxResultBlockAttribute {
            TxResultBlockAttribute {
                key: ::core::clone::Clone::clone(&self.key),
                value: ::core::clone::Clone::clone(&self.value),
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for TxResultBlockAttribute {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "TxResultBlockAttribute",
                "key",
                &self.key,
                "value",
                &&self.value,
            )
        }
    }
    impl From<Attribute> for TxResultBlockAttribute {
        fn from(a: Attribute) -> Self {
            Self { key: a.key, value: a.value }
        }
    }
    /// Parse a string timestamp into a `DateTime<Utc>`
    pub fn parse_timestamp(s: String) -> Result<DateTime<Utc>, DaemonError> {
        let len = s.len();
        let slice_len = if s.contains('.') { len.saturating_sub(4) } else { len };
        let sliced = &s[0..slice_len];
        match NaiveDateTime::parse_from_str(sliced, FORMAT) {
            Err(_e) => {
                match NaiveDateTime::parse_from_str(&s, FORMAT_TZ_SUPPLIED) {
                    Err(_e2) => {
                        match NaiveDateTime::parse_from_str(sliced, FORMAT_SHORT_Z) {
                            Err(_e3) => {
                                match NaiveDateTime::parse_from_str(&s, FORMAT_SHORT_Z2) {
                                    Err(_e4) => {
                                        {
                                            ::std::io::_eprint(
                                                format_args!("DateTime Fail {0} {1:#?}\n", s, _e4),
                                            );
                                        };
                                        Err(DaemonError::StdErr(_e4.to_string()))
                                    }
                                    Ok(dt) => Ok(Utc.from_utc_datetime(&dt)),
                                }
                            }
                            Ok(dt) => Ok(Utc.from_utc_datetime(&dt)),
                        }
                    }
                    Ok(dt) => Ok(Utc.from_utc_datetime(&dt)),
                }
            }
            Ok(dt) => Ok(Utc.from_utc_datetime(&dt)),
        }
    }
}
pub mod keys {
    #![allow(unused)]
    pub mod private {
        use super::public::PublicKey;
        use crate::proto::injective::{InjectivePubKey, ETHEREUM_COIN_TYPE};
        use crate::DaemonError;
        use base64::Engine;
        use bitcoin::{
            bip32::{ExtendedPrivKey, IntoDerivationPath},
            Network,
        };
        use cosmrs::tx::SignerPublicKey;
        use hkd32::mnemonic::{Phrase, Seed};
        use rand_core::OsRng;
        use secp256k1::Secp256k1;
        /// The Private key structure that is used to generate signatures and public keys
        /// WARNING: No Security Audit has been performed
        pub struct PrivateKey {
            #[allow(missing_docs)]
            pub account: u32,
            #[allow(missing_docs)]
            pub index: u32,
            #[allow(missing_docs)]
            pub coin_type: u32,
            /// The 24 words used to generate this private key
            mnemonic: Option<Phrase>,
            #[allow(dead_code)]
            /// This is used for testing
            root_private_key: ExtendedPrivKey,
            /// The private key
            private_key: ExtendedPrivKey,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for PrivateKey {
            #[inline]
            fn clone(&self) -> PrivateKey {
                PrivateKey {
                    account: ::core::clone::Clone::clone(&self.account),
                    index: ::core::clone::Clone::clone(&self.index),
                    coin_type: ::core::clone::Clone::clone(&self.coin_type),
                    mnemonic: ::core::clone::Clone::clone(&self.mnemonic),
                    root_private_key: ::core::clone::Clone::clone(
                        &self.root_private_key,
                    ),
                    private_key: ::core::clone::Clone::clone(&self.private_key),
                }
            }
        }
        impl PrivateKey {
            /// Generate a new private key
            pub fn new<C: secp256k1::Signing + secp256k1::Context>(
                secp: &Secp256k1<C>,
                coin_type: u32,
            ) -> Result<PrivateKey, DaemonError> {
                let phrase = hkd32::mnemonic::Phrase::random(
                    OsRng,
                    hkd32::mnemonic::Language::English,
                );
                PrivateKey::gen_private_key_phrase(secp, phrase, 0, 0, coin_type, "")
            }
            /// generate a new private key with a seed phrase
            pub fn new_seed<C: secp256k1::Signing + secp256k1::Context>(
                secp: &Secp256k1<C>,
                seed_phrase: &str,
                coin_type: u32,
            ) -> Result<PrivateKey, DaemonError> {
                let phrase = hkd32::mnemonic::Phrase::random(
                    OsRng,
                    hkd32::mnemonic::Language::English,
                );
                PrivateKey::gen_private_key_phrase(
                    secp,
                    phrase,
                    0,
                    0,
                    coin_type,
                    seed_phrase,
                )
            }
            /// for private key recovery. This is also used by wallet routines to re-hydrate the structure
            pub fn from_words<C: secp256k1::Signing + secp256k1::Context>(
                secp: &Secp256k1<C>,
                words: &str,
                account: u32,
                index: u32,
                coin_type: u32,
            ) -> Result<PrivateKey, DaemonError> {
                if words.split(' ').count() != 24 {
                    return Err(DaemonError::WrongLength);
                }
                match hkd32::mnemonic::Phrase::new(
                    words,
                    hkd32::mnemonic::Language::English,
                ) {
                    Ok(phrase) => {
                        PrivateKey::gen_private_key_phrase(
                            secp,
                            phrase,
                            account,
                            index,
                            coin_type,
                            "",
                        )
                    }
                    Err(_) => Err(DaemonError::Phrasing),
                }
            }
            /// for private key recovery with seed phrase
            pub fn from_words_seed<C: secp256k1::Signing + secp256k1::Context>(
                secp: &Secp256k1<C>,
                words: &str,
                seed_pass: &str,
                coin_type: u32,
            ) -> Result<PrivateKey, DaemonError> {
                match hkd32::mnemonic::Phrase::new(
                    words,
                    hkd32::mnemonic::Language::English,
                ) {
                    Ok(phrase) => {
                        PrivateKey::gen_private_key_phrase(
                            secp,
                            phrase,
                            0,
                            0,
                            coin_type,
                            seed_pass,
                        )
                    }
                    Err(_) => Err(DaemonError::Phrasing),
                }
            }
            /// generate the public key for this private key
            pub fn public_key<C: secp256k1::Signing + secp256k1::Context>(
                &self,
                secp: &Secp256k1<C>,
            ) -> PublicKey {
                if self.coin_type == ETHEREUM_COIN_TYPE {
                    {
                        ::core::panicking::panic_fmt(
                            format_args!(
                                "Coin Type {0} not supported without eth feature",
                                ETHEREUM_COIN_TYPE,
                            ),
                        );
                    };
                }
                let x = self.private_key.private_key.public_key(secp);
                PublicKey::from_bitcoin_public_key(&bitcoin::PublicKey::new(x))
            }
            pub fn get_injective_public_key<C: secp256k1::Signing + secp256k1::Context>(
                &self,
                secp: &Secp256k1<C>,
            ) -> SignerPublicKey {
                use base64::engine::general_purpose;
                use cosmrs::tx::MessageExt;
                use secp256k1::SecretKey;
                let secret_key = SecretKey::from_slice(self.raw_key().as_slice())
                    .unwrap();
                let public_key = secp256k1::PublicKey::from_secret_key(
                    secp,
                    &secret_key,
                );
                let vec_pk = public_key.serialize();
                {
                    let lvl = ::log::Level::Debug;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            format_args!(
                                "{0:?}, public key",
                                general_purpose::STANDARD.encode(vec_pk),
                            ),
                            lvl,
                            &(
                                "cw_orch_daemon::keys::private",
                                "cw_orch_daemon::keys::private",
                                "cw-orch-daemon/src/keys/private.rs",
                            ),
                            124u32,
                            ::log::__private_api::Option::None,
                        );
                    }
                };
                let inj_key = InjectivePubKey {
                    key: vec_pk.into(),
                };
                inj_key.to_any().unwrap().try_into().unwrap()
            }
            pub fn get_signer_public_key<C: secp256k1::Signing + secp256k1::Context>(
                &self,
                secp: &Secp256k1<C>,
            ) -> Option<SignerPublicKey> {
                if self.coin_type == ETHEREUM_COIN_TYPE {
                    {
                        ::core::panicking::panic_fmt(
                            format_args!(
                                "Coin Type {0} not supported without eth feature",
                                ETHEREUM_COIN_TYPE,
                            ),
                        );
                    };
                }
                Some(
                    cosmrs::crypto::secp256k1::SigningKey::from_slice(
                            self.raw_key().as_slice(),
                        )
                        .unwrap()
                        .public_key()
                        .into(),
                )
            }
            pub fn raw_key(&self) -> Vec<u8> {
                self.private_key.private_key.secret_bytes().to_vec()
            }
            fn gen_private_key_phrase<C: secp256k1::Signing + secp256k1::Context>(
                secp: &Secp256k1<C>,
                phrase: Phrase,
                account: u32,
                index: u32,
                coin_type: u32,
                seed_phrase: &str,
            ) -> Result<PrivateKey, DaemonError> {
                let seed = phrase.to_seed(seed_phrase);
                let root_private_key = ExtendedPrivKey::new_master(
                        Network::Bitcoin,
                        seed.as_bytes(),
                    )
                    .unwrap();
                let path = {
                    let res = ::alloc::fmt::format(
                        format_args!(
                            "m/44\'/{0}\'/{1}\'/0/{2}",
                            coin_type,
                            account,
                            index,
                        ),
                    );
                    res
                };
                let derivation_path = path.into_derivation_path()?;
                let private_key = root_private_key.derive_priv(secp, &derivation_path)?;
                Ok(PrivateKey {
                    account,
                    index,
                    coin_type,
                    mnemonic: Some(phrase),
                    root_private_key,
                    private_key,
                })
            }
            /// the words used to generate this private key
            pub fn words(&self) -> Option<&str> {
                self.mnemonic.as_ref().map(|phrase| phrase.phrase())
            }
            /// used for testing
            /// could potentially be used to recreate the private key instead of words
            #[allow(dead_code)]
            pub(crate) fn seed(&self, passwd: &str) -> Option<Seed> {
                self.mnemonic.as_ref().map(|phrase| phrase.to_seed(passwd))
            }
        }
    }
    pub mod public {
        use crate::DaemonError;
        use bitcoin::bech32::{decode, encode, u5, FromBase32, ToBase32, Variant};
        pub use ed25519_dalek::VerifyingKey as Ed25519;
        use ring::digest::{Context, SHA256};
        use ripemd::{Digest as _, Ripemd160};
        use serde::{Deserialize, Serialize};
        static BECH32_PUBKEY_DATA_PREFIX_SECP256K1: [u8; 5] = [
            0xeb,
            0x5a,
            0xe9,
            0x87,
            0x21,
        ];
        static BECH32_PUBKEY_DATA_PREFIX_ED25519: [u8; 5] = [
            0x16,
            0x24,
            0xde,
            0x64,
            0x20,
        ];
        /// The public key we used to generate the cosmos/tendermind/terrad addresses
        pub struct PublicKey {
            /// This is optional as we can generate non-pub keys without
            pub raw_pub_key: Option<Vec<u8>>,
            /// The raw bytes used to generate non-pub keys
            pub raw_address: Option<Vec<u8>>,
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for PublicKey {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    #[doc(hidden)]
                    enum __Field {
                        __field0,
                        __field1,
                        __ignore,
                    }
                    #[doc(hidden)]
                    struct __FieldVisitor;
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(
                                __formatter,
                                "field identifier",
                            )
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private::Ok(__Field::__field0),
                                1u64 => _serde::__private::Ok(__Field::__field1),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                "raw_pub_key" => _serde::__private::Ok(__Field::__field0),
                                "raw_address" => _serde::__private::Ok(__Field::__field1),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                b"raw_pub_key" => _serde::__private::Ok(__Field::__field0),
                                b"raw_address" => _serde::__private::Ok(__Field::__field1),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                    }
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    #[doc(hidden)]
                    struct __Visitor<'de> {
                        marker: _serde::__private::PhantomData<PublicKey>,
                        lifetime: _serde::__private::PhantomData<&'de ()>,
                    }
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = PublicKey;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(
                                __formatter,
                                "struct PublicKey",
                            )
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match _serde::de::SeqAccess::next_element::<
                                Option<Vec<u8>>,
                            >(&mut __seq)? {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct PublicKey with 2 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match _serde::de::SeqAccess::next_element::<
                                Option<Vec<u8>>,
                            >(&mut __seq)? {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct PublicKey with 2 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private::Ok(PublicKey {
                                raw_pub_key: __field0,
                                raw_address: __field1,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private::Option<
                                Option<Vec<u8>>,
                            > = _serde::__private::None;
                            let mut __field1: _serde::__private::Option<
                                Option<Vec<u8>>,
                            > = _serde::__private::None;
                            while let _serde::__private::Some(__key)
                                = _serde::de::MapAccess::next_key::<__Field>(&mut __map)? {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private::Option::is_some(&__field0) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "raw_pub_key",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private::Some(
                                            _serde::de::MapAccess::next_value::<
                                                Option<Vec<u8>>,
                                            >(&mut __map)?,
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private::Option::is_some(&__field1) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "raw_address",
                                                ),
                                            );
                                        }
                                        __field1 = _serde::__private::Some(
                                            _serde::de::MapAccess::next_value::<
                                                Option<Vec<u8>>,
                                            >(&mut __map)?,
                                        );
                                    }
                                    _ => {
                                        let _ = _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(&mut __map)?;
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private::Some(__field0) => __field0,
                                _serde::__private::None => {
                                    _serde::__private::de::missing_field("raw_pub_key")?
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private::Some(__field1) => __field1,
                                _serde::__private::None => {
                                    _serde::__private::de::missing_field("raw_address")?
                                }
                            };
                            _serde::__private::Ok(PublicKey {
                                raw_pub_key: __field0,
                                raw_address: __field1,
                            })
                        }
                    }
                    #[doc(hidden)]
                    const FIELDS: &'static [&'static str] = &[
                        "raw_pub_key",
                        "raw_address",
                    ];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "PublicKey",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private::PhantomData::<PublicKey>,
                            lifetime: _serde::__private::PhantomData,
                        },
                    )
                }
            }
        };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for PublicKey {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    let mut __serde_state = _serde::Serializer::serialize_struct(
                        __serializer,
                        "PublicKey",
                        false as usize + 1 + 1,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "raw_pub_key",
                        &self.raw_pub_key,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "raw_address",
                        &self.raw_address,
                    )?;
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        #[automatically_derived]
        impl ::core::fmt::Debug for PublicKey {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "PublicKey",
                    "raw_pub_key",
                    &self.raw_pub_key,
                    "raw_address",
                    &&self.raw_address,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for PublicKey {
            #[inline]
            fn clone(&self) -> PublicKey {
                PublicKey {
                    raw_pub_key: ::core::clone::Clone::clone(&self.raw_pub_key),
                    raw_address: ::core::clone::Clone::clone(&self.raw_address),
                }
            }
        }
        impl PublicKey {
            /// Generate a Cosmos/Tendermint/Terrad Public Key
            pub fn from_bitcoin_public_key(bpub: &bitcoin::key::PublicKey) -> PublicKey {
                let bpub_bytes = bpub.inner.serialize();
                let raw_pub_key = PublicKey::pubkey_from_public_key(&bpub_bytes);
                let raw_address = PublicKey::address_from_public_key(&bpub_bytes);
                PublicKey {
                    raw_pub_key: Some(raw_pub_key),
                    raw_address: Some(raw_address),
                }
            }
            /// Generate from secp256k1 Cosmos/Terrad Public Key
            pub fn from_public_key(bpub: &[u8]) -> PublicKey {
                let raw_pub_key = PublicKey::pubkey_from_public_key(bpub);
                let raw_address = PublicKey::address_from_public_key(bpub);
                PublicKey {
                    raw_pub_key: Some(raw_pub_key),
                    raw_address: Some(raw_address),
                }
            }
            /// Generate a Cosmos/Tendermint/Terrad Account
            pub fn from_account(
                acc_address: &str,
                prefix: &str,
            ) -> Result<PublicKey, DaemonError> {
                PublicKey::check_prefix_and_length(prefix, acc_address, 44)
                    .and_then(|vu5| {
                        let vu8 = Vec::from_base32(vu5.as_slice())
                            .map_err(|source| DaemonError::Conversion {
                                key: acc_address.into(),
                                source,
                            })?;
                        Ok(PublicKey {
                            raw_pub_key: None,
                            raw_address: Some(vu8),
                        })
                    })
            }
            /// build a public key from a tendermint public key
            pub fn from_tendermint_key(
                tendermint_public_key: &str,
            ) -> Result<PublicKey, DaemonError> {
                let len = tendermint_public_key.len();
                if len == 83 {
                    PublicKey::check_prefix_and_length(
                            "terravalconspub",
                            tendermint_public_key,
                            len,
                        )
                        .and_then(|vu5| {
                            let vu8 = Vec::from_base32(vu5.as_slice())
                                .map_err(|source| {
                                    DaemonError::Conversion {
                                        key: tendermint_public_key.into(),
                                        source,
                                    }
                                })?;
                            {
                                let lvl = ::log::Level::Debug;
                                if lvl <= ::log::STATIC_MAX_LEVEL
                                    && lvl <= ::log::max_level()
                                {
                                    ::log::__private_api::log(
                                        format_args!("{0:#?}", hex::encode(&vu8)),
                                        lvl,
                                        &(
                                            "cw_orch_daemon::keys::public",
                                            "cw_orch_daemon::keys::public",
                                            "cw-orch-daemon/src/keys/public.rs",
                                        ),
                                        74u32,
                                        ::log::__private_api::Option::None,
                                    );
                                }
                            };
                            if vu8.starts_with(&BECH32_PUBKEY_DATA_PREFIX_SECP256K1) {
                                let public_key = PublicKey::public_key_from_pubkey(&vu8)?;
                                let raw = PublicKey::address_from_public_key(&public_key);
                                Ok(PublicKey {
                                    raw_pub_key: Some(vu8),
                                    raw_address: Some(raw),
                                })
                            } else {
                                Err(DaemonError::ConversionSECP256k1)
                            }
                        })
                } else if len == 82 {
                    PublicKey::check_prefix_and_length(
                            "terravalconspub",
                            tendermint_public_key,
                            len,
                        )
                        .and_then(|vu5| {
                            let vu8 = Vec::from_base32(vu5.as_slice())
                                .map_err(|source| {
                                    DaemonError::Conversion {
                                        key: tendermint_public_key.into(),
                                        source,
                                    }
                                })?;
                            {
                                let lvl = ::log::Level::Info;
                                if lvl <= ::log::STATIC_MAX_LEVEL
                                    && lvl <= ::log::max_level()
                                {
                                    ::log::__private_api::log(
                                        format_args!("ED25519 public keys are not fully supported"),
                                        lvl,
                                        &(
                                            "cw_orch_daemon::keys::public",
                                            "cw_orch_daemon::keys::public",
                                            "cw-orch-daemon/src/keys/public.rs",
                                        ),
                                        100u32,
                                        ::log::__private_api::Option::None,
                                    );
                                }
                            };
                            if vu8.starts_with(&BECH32_PUBKEY_DATA_PREFIX_ED25519) {
                                let raw = PublicKey::address_from_public_ed25519_key(&vu8)?;
                                Ok(PublicKey {
                                    raw_pub_key: Some(vu8),
                                    raw_address: Some(raw),
                                })
                            } else {
                                Err(DaemonError::ConversionED25519)
                            }
                        })
                } else {
                    Err(DaemonError::ConversionLength(len))
                }
            }
            /// build a terravalcons address from a tendermint hex key
            /// the tendermint_hex_address should be a hex code of 40 length
            pub fn from_tendermint_address(
                tendermint_hex_address: &str,
            ) -> Result<PublicKey, DaemonError> {
                let len = tendermint_hex_address.len();
                if len == 40 {
                    let raw = hex::decode(tendermint_hex_address)?;
                    Ok(PublicKey {
                        raw_pub_key: None,
                        raw_address: Some(raw),
                    })
                } else {
                    Err(DaemonError::ConversionLengthED25519Hex(len))
                }
            }
            /// Generate a Operator address for this public key (used by the validator)
            pub fn from_operator_address(
                valoper_address: &str,
            ) -> Result<PublicKey, DaemonError> {
                PublicKey::check_prefix_and_length("terravaloper", valoper_address, 51)
                    .and_then(|vu5| {
                        let vu8 = Vec::from_base32(vu5.as_slice())
                            .map_err(|source| DaemonError::Conversion {
                                key: valoper_address.into(),
                                source,
                            })?;
                        Ok(PublicKey {
                            raw_pub_key: None,
                            raw_address: Some(vu8),
                        })
                    })
            }
            /// Generate Public key from raw address
            pub fn from_raw_address(
                raw_address: &str,
            ) -> Result<PublicKey, DaemonError> {
                let vec1 = hex::decode(raw_address)?;
                Ok(PublicKey {
                    raw_pub_key: None,
                    raw_address: Some(vec1),
                })
            }
            fn check_prefix_and_length(
                prefix: &str,
                data: &str,
                length: usize,
            ) -> Result<Vec<u5>, DaemonError> {
                let (hrp, decoded_str, _) = decode(data)
                    .map_err(|source| DaemonError::Conversion {
                        key: data.into(),
                        source,
                    })?;
                if hrp == prefix && data.len() == length {
                    Ok(decoded_str)
                } else {
                    Err(
                        DaemonError::Bech32DecodeExpanded(
                            hrp,
                            data.len(),
                            prefix.into(),
                            length,
                        ),
                    )
                }
            }
            /**
    Gets a bech32-words pubkey from a compressed bytes Secp256K1 public key.

     @param publicKey raw public key
    */
            pub fn pubkey_from_public_key(public_key: &[u8]) -> Vec<u8> {
                [BECH32_PUBKEY_DATA_PREFIX_SECP256K1.to_vec(), public_key.to_vec()]
                    .concat()
            }
            /**
    Gets a bech32-words pubkey from a compressed bytes Ed25519 public key.

    @param publicKey raw public key
    */
            pub fn pubkey_from_ed25519_public_key(public_key: &[u8]) -> Vec<u8> {
                [BECH32_PUBKEY_DATA_PREFIX_ED25519.to_vec(), public_key.to_vec()]
                    .concat()
            }
            /// Translate from a BECH32 prefixed key to a standard public key
            pub fn public_key_from_pubkey(
                pub_key: &[u8],
            ) -> Result<Vec<u8>, DaemonError> {
                if pub_key.starts_with(&BECH32_PUBKEY_DATA_PREFIX_SECP256K1) {
                    let len = BECH32_PUBKEY_DATA_PREFIX_SECP256K1.len();
                    let len2 = pub_key.len();
                    Ok(Vec::from(&pub_key[len..len2]))
                } else if pub_key.starts_with(&BECH32_PUBKEY_DATA_PREFIX_ED25519) {
                    let len = BECH32_PUBKEY_DATA_PREFIX_ED25519.len();
                    let len2 = pub_key.len();
                    let vec = &pub_key[len..len2];
                    let ed25519_pubkey = Ed25519::from_bytes(vec.try_into().unwrap())?;
                    Ok(ed25519_pubkey.to_bytes().to_vec())
                } else {
                    {
                        let lvl = ::log::Level::Info;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!("pub key does not start with BECH32 PREFIX"),
                                lvl,
                                &(
                                    "cw_orch_daemon::keys::public",
                                    "cw_orch_daemon::keys::public",
                                    "cw-orch-daemon/src/keys/public.rs",
                                ),
                                228u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    Err(DaemonError::Bech32DecodeErr)
                }
            }
            /**
    Gets a raw address from a compressed bytes public key.

    @param publicKey raw public key
    */
            pub fn address_from_public_key(public_key: &[u8]) -> Vec<u8> {
                let mut hasher = Ripemd160::new();
                let sha_result = ring::digest::digest(&SHA256, public_key);
                hasher.update(&sha_result.as_ref()[0..32]);
                let ripe_result = hasher.finalize();
                let address: Vec<u8> = ripe_result[0..20].to_vec();
                address
            }
            /**
    Gets a raw address from a  ed25519 public key.

    @param publicKey raw public key
    */
            pub fn address_from_public_ed25519_key(
                public_key: &[u8],
            ) -> Result<Vec<u8>, DaemonError> {
                if public_key.len() != (32 + 5) {
                    Err(
                        DaemonError::ConversionPrefixED25519(
                            public_key.len(),
                            hex::encode(public_key),
                        ),
                    )
                } else {
                    {
                        let lvl = ::log::Level::Debug;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "address_from_public_ed25519_key public key - {0}",
                                    hex::encode(public_key),
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::keys::public",
                                    "cw_orch_daemon::keys::public",
                                    "cw-orch-daemon/src/keys/public.rs",
                                ),
                                262u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    let mut sha_result: [u8; 32] = [0; 32];
                    let sha_result = ring::digest::digest(&SHA256, &public_key[5..]);
                    let address: Vec<u8> = sha_result.as_ref()[0..20].to_vec();
                    {
                        let lvl = ::log::Level::Debug;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "address_from_public_ed25519_key sha result - {0}",
                                    hex::encode(&address),
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::keys::public",
                                    "cw_orch_daemon::keys::public",
                                    "cw-orch-daemon/src/keys/public.rs",
                                ),
                                283u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    Ok(address)
                }
            }
            /// The main account used in most things
            pub fn account(&self, prefix: &str) -> Result<String, DaemonError> {
                match &self.raw_address {
                    Some(raw) => {
                        let data = encode(prefix, raw.to_base32(), Variant::Bech32);
                        match data {
                            Ok(acc) => Ok(acc),
                            Err(_) => Err(DaemonError::Bech32DecodeErr),
                        }
                    }
                    None => Err(DaemonError::Implementation),
                }
            }
            /// The operator address used for validators
            pub fn operator_address(&self, prefix: &str) -> Result<String, DaemonError> {
                match &self.raw_address {
                    Some(raw) => {
                        let data = encode(
                            &{
                                let res = ::alloc::fmt::format(
                                    format_args!("{0}{1}", prefix, "valoper"),
                                );
                                res
                            },
                            raw.to_base32(),
                            Variant::Bech32,
                        );
                        match data {
                            Ok(acc) => Ok(acc),
                            Err(_) => Err(DaemonError::Bech32DecodeErr),
                        }
                    }
                    None => Err(DaemonError::Implementation),
                }
            }
            /// application public key - Application keys are associated with a public key terrapub- and an address terra-
            pub fn application_public_key(
                &self,
                prefix: &str,
            ) -> Result<String, DaemonError> {
                match &self.raw_pub_key {
                    Some(raw) => {
                        let data = encode(
                            &{
                                let res = ::alloc::fmt::format(
                                    format_args!("{0}{1}", prefix, "pub"),
                                );
                                res
                            },
                            raw.to_base32(),
                            Variant::Bech32,
                        );
                        match data {
                            Ok(acc) => Ok(acc),
                            Err(_) => Err(DaemonError::Bech32DecodeErr),
                        }
                    }
                    None => {
                        {
                            let lvl = ::log::Level::Warn;
                            if lvl <= ::log::STATIC_MAX_LEVEL
                                && lvl <= ::log::max_level()
                            {
                                ::log::__private_api::log(
                                    format_args!("Missing Public Key. Can\'t continue"),
                                    lvl,
                                    &(
                                        "cw_orch_daemon::keys::public",
                                        "cw_orch_daemon::keys::public",
                                        "cw-orch-daemon/src/keys/public.rs",
                                    ),
                                    335u32,
                                    ::log::__private_api::Option::None,
                                );
                            }
                        };
                        Err(DaemonError::Implementation)
                    }
                }
            }
            /// The operator address used for validators public key.
            pub fn operator_address_public_key(
                &self,
                prefix: &str,
            ) -> Result<String, DaemonError> {
                match &self.raw_pub_key {
                    Some(raw) => {
                        let data = encode(
                            &{
                                let res = ::alloc::fmt::format(
                                    format_args!("{0}{1}", prefix, "valoperpub"),
                                );
                                res
                            },
                            raw.to_base32(),
                            Variant::Bech32,
                        );
                        match data {
                            Ok(acc) => Ok(acc),
                            Err(_) => Err(DaemonError::Bech32DecodeErr),
                        }
                    }
                    None => Err(DaemonError::Implementation),
                }
            }
            /// This is a unique key used to sign block hashes. It is associated with a public key terravalconspub.
            pub fn tendermint(&self, prefix: &str) -> Result<String, DaemonError> {
                match &self.raw_address {
                    Some(raw) => {
                        let data = encode(
                            &{
                                let res = ::alloc::fmt::format(
                                    format_args!("{0}{1}", prefix, "valcons"),
                                );
                                res
                            },
                            raw.to_base32(),
                            Variant::Bech32,
                        );
                        match data {
                            Ok(acc) => Ok(acc),
                            Err(_) => Err(DaemonError::Bech32DecodeErr),
                        }
                    }
                    None => Err(DaemonError::Implementation),
                }
            }
            /// This is a unique key used to sign block hashes. It is associated with a public key terravalconspub.
            pub fn tendermint_pubkey(
                &self,
                prefix: &str,
            ) -> Result<String, DaemonError> {
                match &self.raw_pub_key {
                    Some(raw) => {
                        let b32 = raw.to_base32();
                        let data = encode(
                            &{
                                let res = ::alloc::fmt::format(
                                    format_args!("{0}{1}", prefix, "valconspub"),
                                );
                                res
                            },
                            b32,
                            Variant::Bech32,
                        );
                        match data {
                            Ok(acc) => Ok(acc),
                            Err(_) => Err(DaemonError::Bech32DecodeErr),
                        }
                    }
                    None => Err(DaemonError::Implementation),
                }
            }
        }
    }
    pub mod signature {
        use crate::DaemonError;
        use base64::engine::{general_purpose::STANDARD, Engine};
        use ring::digest::SHA256;
        use secp256k1::{Message, Secp256k1};
        pub struct Signature {}
        impl Signature {
            pub fn verify<C: secp256k1::Verification + secp256k1::Context>(
                secp: &Secp256k1<C>,
                pub_key: &str,
                signature: &str,
                blob: &str,
            ) -> Result<(), DaemonError> {
                let public = STANDARD.decode(pub_key)?;
                let sig = STANDARD.decode(signature)?;
                let pk = secp256k1::PublicKey::from_slice(public.as_slice())?;
                let sha_result = ring::digest::digest(&SHA256, blob.as_bytes());
                let message: Message = Message::from_slice(&sha_result.as_ref()[0..32])?;
                let secp_sig = secp256k1::ecdsa::Signature::from_compact(
                    sig.as_slice(),
                )?;
                secp.verify_ecdsa(&message, &secp_sig, &pk)?;
                Ok(())
            }
        }
    }
}
pub mod live_mock {
    //! Live mock is a mock that uses a live chain to query for data.
    //! It can be used to do chain-backed unit-testing. It can't be used for state-changing operations.
    use crate::queriers::Bank;
    use crate::queriers::CosmWasm;
    use crate::queriers::DaemonQuerier;
    use crate::queriers::Staking;
    use cosmwasm_std::Addr;
    use cosmwasm_std::AllBalanceResponse;
    use cosmwasm_std::BalanceResponse;
    use cosmwasm_std::Delegation;
    use cosmwasm_std::{AllDelegationsResponse, BondedDenomResponse};
    use cosmwasm_std::BankQuery;
    use cosmwasm_std::Binary;
    use cosmwasm_std::Empty;
    use cosmwasm_std::StakingQuery;
    use ibc_chain_registry::chain::ChainData;
    use tokio::runtime::Runtime;
    use tonic::transport::Channel;
    use std::marker::PhantomData;
    use std::str::FromStr;
    use cosmwasm_std::testing::{MockApi, MockStorage};
    use cosmwasm_std::{
        from_slice, to_binary, Coin, ContractResult, OwnedDeps, Querier, QuerierResult,
        QueryRequest, SystemError, SystemResult, Uint128, WasmQuery,
    };
    use crate::channel::GrpcChannel;
    fn to_cosmwasm_coin(c: cosmrs::proto::cosmos::base::v1beta1::Coin) -> Coin {
        Coin {
            amount: Uint128::from_str(&c.amount).unwrap(),
            denom: c.denom,
        }
    }
    const QUERIER_ERROR: &str = "Only Bank balances and Wasm (raw + smart) and Some staking queries are covered for now";
    /// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
    /// this uses our CustomQuerier.
    pub fn mock_dependencies(
        chain_info: ChainData,
    ) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
        let custom_querier: WasmMockQuerier = WasmMockQuerier::new(chain_info);
        OwnedDeps {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: custom_querier,
            custom_query_type: PhantomData,
        }
    }
    /// Querier struct that fetches queries on-chain directly
    pub struct WasmMockQuerier {
        channel: Channel,
        runtime: Runtime,
    }
    impl Querier for WasmMockQuerier {
        fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
            let request: QueryRequest<Empty> = match from_slice(bin_request) {
                Ok(v) => v,
                Err(e) => {
                    return SystemResult::Err(SystemError::InvalidRequest {
                        error: {
                            let res = ::alloc::fmt::format(
                                format_args!("Parsing query request: {0}", e),
                            );
                            res
                        },
                        request: bin_request.into(),
                    });
                }
            };
            self.handle_query(&request)
        }
    }
    impl WasmMockQuerier {
        /// Function used to handle a query and customize the query behavior
        /// This implements some queries by querying an actual node for the responses
        pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
            match &request {
                QueryRequest::Wasm(x) => {
                    let querier = CosmWasm::new(self.channel.clone());
                    match x {
                        WasmQuery::Smart { contract_addr, msg } => {
                            let query_result: Result<Binary, _> = self
                                .runtime
                                .block_on(
                                    querier
                                        .contract_state(contract_addr.to_string(), msg.to_vec()),
                                )
                                .map(|query_result| query_result.into());
                            SystemResult::Ok(ContractResult::from(query_result))
                        }
                        WasmQuery::Raw { contract_addr, key } => {
                            let query_result = self
                                .runtime
                                .block_on(
                                    querier
                                        .contract_raw_state(contract_addr.to_string(), key.to_vec()),
                                )
                                .map(|query_result| query_result.data.into());
                            SystemResult::Ok(ContractResult::from(query_result))
                        }
                        _ => {
                            SystemResult::Err(SystemError::InvalidRequest {
                                error: QUERIER_ERROR.to_string(),
                                request: to_binary(&request).unwrap(),
                            })
                        }
                    }
                }
                QueryRequest::Bank(x) => {
                    let querier = Bank::new(self.channel.clone());
                    match x {
                        BankQuery::Balance { address, denom } => {
                            let query_result = self
                                .runtime
                                .block_on(querier.balance(address, Some(denom.clone())))
                                .map(|result| {
                                    to_binary(
                                            &BalanceResponse {
                                                amount: Coin {
                                                    amount: Uint128::from_str(&result[0].amount).unwrap(),
                                                    denom: result[0].denom.clone(),
                                                },
                                            },
                                        )
                                        .unwrap()
                                });
                            SystemResult::Ok(ContractResult::from(query_result))
                        }
                        BankQuery::AllBalances { address } => {
                            let query_result = self
                                .runtime
                                .block_on(querier.balance(address, None))
                                .map(|result| AllBalanceResponse {
                                    amount: result
                                        .into_iter()
                                        .map(|c| Coin {
                                            amount: Uint128::from_str(&c.amount).unwrap(),
                                            denom: c.denom,
                                        })
                                        .collect(),
                                })
                                .map(|query_result| to_binary(&query_result))
                                .unwrap();
                            SystemResult::Ok(ContractResult::from(query_result))
                        }
                        _ => {
                            SystemResult::Err(SystemError::InvalidRequest {
                                error: QUERIER_ERROR.to_string(),
                                request: to_binary(&request).unwrap(),
                            })
                        }
                    }
                }
                QueryRequest::Staking(x) => {
                    let querier = Staking::new(self.channel.clone());
                    match x {
                        StakingQuery::BondedDenom {} => {
                            let query_result = self
                                .runtime
                                .block_on(querier.params())
                                .map(|result| BondedDenomResponse {
                                    denom: result.params.unwrap().bond_denom,
                                })
                                .map(|query_result| to_binary(&query_result))
                                .unwrap();
                            SystemResult::Ok(ContractResult::from(query_result))
                        }
                        StakingQuery::AllDelegations { delegator } => {
                            let query_result = self
                                .runtime
                                .block_on(querier.delegator_delegations(delegator, None))
                                .map(|result| AllDelegationsResponse {
                                    delegations: result
                                        .delegation_responses
                                        .into_iter()
                                        .filter_map(|delegation| {
                                            delegation
                                                .delegation
                                                .map(|d| Delegation {
                                                    delegator: Addr::unchecked(d.delegator_address),
                                                    validator: d.validator_address,
                                                    amount: to_cosmwasm_coin(delegation.balance.unwrap()),
                                                })
                                        })
                                        .collect(),
                                })
                                .map(|query_result| to_binary(&query_result))
                                .unwrap();
                            SystemResult::Ok(ContractResult::from(query_result))
                        }
                        _ => ::core::panicking::panic("not yet implemented"),
                    }
                }
                _ => {
                    SystemResult::Err(SystemError::InvalidRequest {
                        error: QUERIER_ERROR.to_string(),
                        request: to_binary(&request).unwrap(),
                    })
                }
            }
        }
    }
    impl WasmMockQuerier {
        /// Creates a querier from chain information
        pub fn new(chain: ChainData) -> Self {
            let rt = Runtime::new().unwrap();
            let channel = rt
                .block_on(GrpcChannel::connect(&chain.apis.grpc, &chain.chain_id))
                .unwrap();
            WasmMockQuerier {
                channel,
                runtime: rt,
            }
        }
    }
}
pub mod queriers {
    //! # DaemonQuerier
    //!
    //! DaemonAsync queriers are gRPC query clients for the CosmosSDK modules. They can be used to query the different modules (Bank, Ibc, Authz, ...).
    //!
    //! ## Usage
    //!
    //! You will need to acquire a [gRPC channel](Channel) to a running CosmosSDK node to be able to use the queriers.
    //! Here is an example of how to acquire one using the DaemonAsync builder.
    //!
    //! ```no_run
    //! // require the querier you want to use, in this case Node
    //! use cw_orch_daemon::{queriers::Node, DaemonAsync, networks, queriers::DaemonQuerier};
    //! # tokio_test::block_on(async {
    //! // call the builder and configure it as you need
    //! let daemon = DaemonAsync::builder()
    //!     .chain(networks::LOCAL_JUNO)
    //!     .build()
    //!     .await.unwrap();
    //! // now you can use the Node querier:
    //! let node = Node::new(daemon.channel());
    //! let node_info = node.info();
    //! # })
    //! ```
    mod bank {
        use crate::{cosmos_modules, error::DaemonError};
        use cosmrs::proto::cosmos::{
            base::{query::v1beta1::PageRequest, v1beta1::Coin},
            bank::v1beta1::QueryBalanceResponse,
        };
        use tonic::transport::Channel;
        use super::DaemonQuerier;
        /// Queries for Cosmos Bank Module
        pub struct Bank {
            channel: Channel,
        }
        impl DaemonQuerier for Bank {
            fn new(channel: Channel) -> Self {
                Self { channel }
            }
        }
        impl Bank {
            /// Query the bank balance of a given address
            /// If denom is None, returns all balances
            pub async fn balance(
                &self,
                address: impl Into<String>,
                denom: Option<String>,
            ) -> Result<Vec<Coin>, DaemonError> {
                use cosmos_modules::bank::query_client::QueryClient;
                match denom {
                    Some(denom) => {
                        let resp: QueryBalanceResponse = ();
                        let mut client: QueryClient<Channel> = QueryClient::new(
                            self.channel.clone(),
                        );
                        let request = cosmos_modules::bank::QueryBalanceRequest {
                            address: address.into(),
                            denom,
                        };
                        let resp = client.balance(request).await?.into_inner();
                        let coin = resp.balance.unwrap();
                        Ok(
                            <[_]>::into_vec(
                                #[rustc_box]
                                ::alloc::boxed::Box::new([coin]),
                            ),
                        )
                    }
                    None => {
                        let mut client: QueryClient<Channel> = QueryClient::new(
                            self.channel.clone(),
                        );
                        let request = cosmos_modules::bank::QueryAllBalancesRequest {
                            address: address.into(),
                            ..Default::default()
                        };
                        let resp: QueryBalanceResponse = ();
                        let denoms_metadata: cosmos_modules::bank::QueryDenomsMetadataResponse = ();
                        let resp = client.all_balances(request).await?.into_inner();
                        let coins = resp.balances;
                        Ok(coins.into_iter().collect())
                    }
                }
            }
            /// Query spendable balance for address
            pub async fn spendable_balances(
                &self,
                address: impl Into<String>,
            ) -> Result<Vec<Coin>, DaemonError> {
                let spendable_balances: cosmos_modules::bank::QuerySpendableBalancesResponse = {
                    use crate::cosmos_modules::bank::{
                        query_client::QueryClient, QuerySpendableBalancesRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QuerySpendableBalancesRequest {
                        address: address.into(),
                        pagination: None,
                    };
                    let response = client
                        .spendable_balances(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::bank",
                                    "cw_orch_daemon::queriers::bank",
                                    "cw-orch-daemon/src/queriers/bank.rs",
                                ),
                                89u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(spendable_balances.balances)
            }
            /// Query total supply in the bank
            pub async fn total_supply(&self) -> Result<Vec<Coin>, DaemonError> {
                let total_supply: cosmos_modules::bank::QueryTotalSupplyResponse = {
                    use crate::cosmos_modules::bank::{
                        query_client::QueryClient, QueryTotalSupplyRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryTotalSupplyRequest {
                        pagination: None,
                    };
                    let response = client
                        .total_supply(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::bank",
                                    "cw_orch_daemon::queriers::bank",
                                    "cw-orch-daemon/src/queriers/bank.rs",
                                ),
                                103u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(total_supply.supply)
            }
            /// Query total supply in the bank for a denom
            pub async fn supply_of(
                &self,
                denom: impl Into<String>,
            ) -> Result<Coin, DaemonError> {
                let supply_of: cosmos_modules::bank::QuerySupplyOfResponse = {
                    use crate::cosmos_modules::bank::{
                        query_client::QueryClient, QuerySupplyOfRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QuerySupplyOfRequest {
                        denom: denom.into(),
                    };
                    let response = client.supply_of(request.clone()).await?.into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::bank",
                                    "cw_orch_daemon::queriers::bank",
                                    "cw-orch-daemon/src/queriers/bank.rs",
                                ),
                                114u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(supply_of.amount.unwrap())
            }
            /// Query params
            pub async fn params(
                &self,
            ) -> Result<cosmos_modules::bank::Params, DaemonError> {
                let params: cosmos_modules::bank::QueryParamsResponse = {
                    use crate::cosmos_modules::bank::{
                        query_client::QueryClient, QueryParamsRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryParamsRequest {};
                    let response = client.params(request.clone()).await?.into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::bank",
                                    "cw_orch_daemon::queriers::bank",
                                    "cw-orch-daemon/src/queriers/bank.rs",
                                ),
                                128u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(params.params.unwrap())
            }
            /// Query denom metadata
            pub async fn denom_metadata(
                &self,
                denom: impl Into<String>,
            ) -> Result<cosmos_modules::bank::Metadata, DaemonError> {
                let denom_metadata: cosmos_modules::bank::QueryDenomMetadataResponse = {
                    use crate::cosmos_modules::bank::{
                        query_client::QueryClient, QueryDenomMetadataRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryDenomMetadataRequest {
                        denom: denom.into(),
                    };
                    let response = client
                        .denom_metadata(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::bank",
                                    "cw_orch_daemon::queriers::bank",
                                    "cw-orch-daemon/src/queriers/bank.rs",
                                ),
                                137u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(denom_metadata.metadata.unwrap())
            }
            /// Query denoms metadata with pagination
            ///
            /// see [PageRequest] for pagination
            pub async fn denoms_metadata(
                &self,
                pagination: Option<PageRequest>,
            ) -> Result<Vec<cosmos_modules::bank::Metadata>, DaemonError> {
                let denoms_metadata: cosmos_modules::bank::QueryDenomsMetadataResponse = {
                    use crate::cosmos_modules::bank::{
                        query_client::QueryClient, QueryDenomsMetadataRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryDenomsMetadataRequest {
                        pagination: pagination,
                    };
                    let response = client
                        .denoms_metadata(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::bank",
                                    "cw_orch_daemon::queriers::bank",
                                    "cw-orch-daemon/src/queriers/bank.rs",
                                ),
                                155u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(denoms_metadata.metadatas)
            }
        }
    }
    mod cosmwasm {
        use crate::{cosmos_modules, error::DaemonError};
        use cosmrs::proto::cosmos::base::query::v1beta1::PageRequest;
        use tonic::transport::Channel;
        use super::DaemonQuerier;
        /// Querier for the CosmWasm SDK module
        pub struct CosmWasm {
            channel: Channel,
        }
        impl DaemonQuerier for CosmWasm {
            fn new(channel: Channel) -> Self {
                Self { channel }
            }
        }
        impl CosmWasm {
            /// Query code_id by hash
            pub async fn code_id_hash(
                &self,
                code_id: u64,
            ) -> Result<String, DaemonError> {
                use cosmos_modules::cosmwasm::{query_client::*, QueryCodeRequest};
                let mut client: QueryClient<Channel> = QueryClient::new(
                    self.channel.clone(),
                );
                let request = QueryCodeRequest { code_id };
                let resp = client.code(request).await?.into_inner();
                let contract_hash = resp.code_info.unwrap().data_hash;
                let on_chain_hash = base16::encode_lower(&contract_hash);
                Ok(on_chain_hash)
            }
            /// Query contract info
            pub async fn contract_info(
                &self,
                address: impl Into<String>,
            ) -> Result<cosmos_modules::cosmwasm::ContractInfo, DaemonError> {
                use cosmos_modules::cosmwasm::{
                    query_client::*, QueryContractInfoRequest,
                };
                let mut client: QueryClient<Channel> = QueryClient::new(
                    self.channel.clone(),
                );
                let request = QueryContractInfoRequest {
                    address: address.into(),
                };
                let resp = client.contract_info(request).await?.into_inner();
                let contract_info = resp.contract_info.unwrap();
                Ok(contract_info)
            }
            /// Query contract history
            pub async fn contract_history(
                &self,
                address: impl Into<String>,
                pagination: Option<PageRequest>,
            ) -> Result<
                cosmos_modules::cosmwasm::QueryContractHistoryResponse,
                DaemonError,
            > {
                use cosmos_modules::cosmwasm::{
                    query_client::*, QueryContractHistoryRequest,
                };
                let mut client: QueryClient<Channel> = QueryClient::new(
                    self.channel.clone(),
                );
                let request = QueryContractHistoryRequest {
                    address: address.into(),
                    pagination,
                };
                Ok(client.contract_history(request).await?.into_inner())
            }
            /// Query contract state
            pub async fn contract_state(
                &self,
                address: impl Into<String>,
                query_data: Vec<u8>,
            ) -> Result<Vec<u8>, DaemonError> {
                use cosmos_modules::cosmwasm::{
                    query_client::*, QuerySmartContractStateRequest,
                };
                let mut client: QueryClient<Channel> = QueryClient::new(
                    self.channel.clone(),
                );
                let request = QuerySmartContractStateRequest {
                    address: address.into(),
                    query_data,
                };
                Ok(client.smart_contract_state(request).await?.into_inner().data)
            }
            /// Query all contract state
            pub async fn all_contract_state(
                &self,
                address: impl Into<String>,
                pagination: Option<PageRequest>,
            ) -> Result<
                cosmos_modules::cosmwasm::QueryAllContractStateResponse,
                DaemonError,
            > {
                use cosmos_modules::cosmwasm::{
                    query_client::*, QueryAllContractStateRequest,
                };
                let mut client: QueryClient<Channel> = QueryClient::new(
                    self.channel.clone(),
                );
                let request = QueryAllContractStateRequest {
                    address: address.into(),
                    pagination,
                };
                Ok(client.all_contract_state(request).await?.into_inner())
            }
            /// Query code
            pub async fn code(
                &self,
                code_id: u64,
            ) -> Result<cosmos_modules::cosmwasm::CodeInfoResponse, DaemonError> {
                use cosmos_modules::cosmwasm::{query_client::*, QueryCodeRequest};
                let mut client: QueryClient<Channel> = QueryClient::new(
                    self.channel.clone(),
                );
                let request = QueryCodeRequest { code_id };
                Ok(client.code(request).await?.into_inner().code_info.unwrap())
            }
            /// Query code bytes
            pub async fn code_data(&self, code_id: u64) -> Result<Vec<u8>, DaemonError> {
                use cosmos_modules::cosmwasm::{query_client::*, QueryCodeRequest};
                let mut client: QueryClient<Channel> = QueryClient::new(
                    self.channel.clone(),
                );
                let request = QueryCodeRequest { code_id };
                Ok(client.code(request).await?.into_inner().data)
            }
            /// Query codes
            pub async fn codes(
                &self,
                pagination: Option<PageRequest>,
            ) -> Result<Vec<cosmos_modules::cosmwasm::CodeInfoResponse>, DaemonError> {
                use cosmos_modules::cosmwasm::{query_client::*, QueryCodesRequest};
                let mut client: QueryClient<Channel> = QueryClient::new(
                    self.channel.clone(),
                );
                let request = QueryCodesRequest { pagination };
                Ok(client.codes(request).await?.into_inner().code_infos)
            }
            /// Query pinned codes
            pub async fn pinned_codes(
                &self,
            ) -> Result<
                cosmos_modules::cosmwasm::QueryPinnedCodesResponse,
                DaemonError,
            > {
                use cosmos_modules::cosmwasm::{query_client::*, QueryPinnedCodesRequest};
                let mut client: QueryClient<Channel> = QueryClient::new(
                    self.channel.clone(),
                );
                let request = QueryPinnedCodesRequest {
                    pagination: None,
                };
                Ok(client.pinned_codes(request).await?.into_inner())
            }
            /// Query contracts by code
            pub async fn contract_by_codes(
                &self,
                code_id: u64,
            ) -> Result<
                cosmos_modules::cosmwasm::QueryContractsByCodeResponse,
                DaemonError,
            > {
                use cosmos_modules::cosmwasm::{
                    query_client::*, QueryContractsByCodeRequest,
                };
                let mut client: QueryClient<Channel> = QueryClient::new(
                    self.channel.clone(),
                );
                let request = QueryContractsByCodeRequest {
                    code_id,
                    pagination: None,
                };
                Ok(client.contracts_by_code(request).await?.into_inner())
            }
            /// Query raw contract state
            pub async fn contract_raw_state(
                &self,
                address: impl Into<String>,
                query_data: Vec<u8>,
            ) -> Result<
                cosmos_modules::cosmwasm::QueryRawContractStateResponse,
                DaemonError,
            > {
                use cosmos_modules::cosmwasm::{
                    query_client::*, QueryRawContractStateRequest,
                };
                let mut client: QueryClient<Channel> = QueryClient::new(
                    self.channel.clone(),
                );
                let request = QueryRawContractStateRequest {
                    address: address.into(),
                    query_data,
                };
                Ok(client.raw_contract_state(request).await?.into_inner())
            }
            /// Query params
            pub async fn params(
                &self,
            ) -> Result<cosmos_modules::cosmwasm::QueryParamsResponse, DaemonError> {
                use cosmos_modules::cosmwasm::{query_client::*, QueryParamsRequest};
                let mut client: QueryClient<Channel> = QueryClient::new(
                    self.channel.clone(),
                );
                Ok(client.params(QueryParamsRequest {}).await?.into_inner())
            }
        }
    }
    mod feegrant {
        use crate::{cosmos_modules, error::DaemonError};
        use cosmrs::proto::cosmos::base::query::v1beta1::PageRequest;
        use tonic::transport::Channel;
        use super::DaemonQuerier;
        /// Querier for the Cosmos Gov module
        pub struct Feegrant {
            channel: Channel,
        }
        impl DaemonQuerier for Feegrant {
            fn new(channel: Channel) -> Self {
                Self { channel }
            }
        }
        impl Feegrant {
            /// Query all allowances granted to the grantee address by a granter address
            pub async fn allowance(
                &self,
                granter: impl Into<String>,
                grantee: impl Into<String>,
            ) -> Result<cosmos_modules::feegrant::Grant, DaemonError> {
                let allowance: cosmos_modules::feegrant::QueryAllowanceResponse = {
                    use crate::cosmos_modules::feegrant::{
                        query_client::QueryClient, QueryAllowanceRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryAllowanceRequest {
                        granter: granter.into(),
                        grantee: grantee.into(),
                    };
                    let response = client.allowance(request.clone()).await?.into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::feegrant",
                                    "cw_orch_daemon::queriers::feegrant",
                                    "cw-orch-daemon/src/queriers/feegrant.rs",
                                ),
                                25u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(allowance.allowance.unwrap())
            }
            /// Query allowances for grantee address with a given pagination
            ///
            /// see [PageRequest] for pagination
            pub async fn allowances(
                &self,
                grantee: impl Into<String>,
                pagination: Option<PageRequest>,
            ) -> Result<Vec<cosmos_modules::feegrant::Grant>, DaemonError> {
                let allowances: cosmos_modules::feegrant::QueryAllowancesResponse = {
                    use crate::cosmos_modules::feegrant::{
                        query_client::QueryClient, QueryAllowancesRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryAllowancesRequest {
                        grantee: grantee.into(),
                        pagination: pagination,
                    };
                    let response = client
                        .allowances(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::feegrant",
                                    "cw_orch_daemon::queriers::feegrant",
                                    "cw-orch-daemon/src/queriers/feegrant.rs",
                                ),
                                45u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(allowances.allowances)
            }
        }
    }
    mod gov {
        use crate::{cosmos_modules, error::DaemonError};
        use cosmrs::proto::cosmos::base::query::v1beta1::PageRequest;
        use tonic::transport::Channel;
        use super::DaemonQuerier;
        /// Querier for the Cosmos Gov module
        pub struct Gov {
            channel: Channel,
        }
        impl DaemonQuerier for Gov {
            fn new(channel: Channel) -> Self {
                Self { channel }
            }
        }
        impl Gov {
            /// Query proposal details by proposal id
            pub async fn proposal(
                &self,
                proposal_id: u64,
            ) -> Result<cosmos_modules::gov::Proposal, DaemonError> {
                let proposal: cosmos_modules::gov::QueryProposalResponse = {
                    use crate::cosmos_modules::gov::{
                        query_client::QueryClient, QueryProposalRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryProposalRequest {
                        proposal_id: proposal_id,
                    };
                    let response = client.proposal(request.clone()).await?.into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::gov",
                                    "cw_orch_daemon::queriers::gov",
                                    "cw-orch-daemon/src/queriers/gov.rs",
                                ),
                                24u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(proposal.proposal.unwrap())
            }
            /// Query proposals based on given status
            ///
            /// see [PageRequest] for pagination
            pub async fn proposals(
                &self,
                proposal_status: GovProposalStatus,
                voter: impl Into<String>,
                depositor: impl Into<String>,
                pagination: Option<PageRequest>,
            ) -> Result<cosmos_modules::gov::QueryProposalsResponse, DaemonError> {
                let proposals: cosmos_modules::gov::QueryProposalsResponse = {
                    use crate::cosmos_modules::gov::{
                        query_client::QueryClient, QueryProposalsRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryProposalsRequest {
                        proposal_status: proposal_status as i32,
                        voter: voter.into(),
                        depositor: depositor.into(),
                        pagination: pagination,
                    };
                    let response = client.proposals(request.clone()).await?.into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::gov",
                                    "cw_orch_daemon::queriers::gov",
                                    "cw-orch-daemon/src/queriers/gov.rs",
                                ),
                                45u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(proposals)
            }
            /// Query voted information based on proposal_id for voter address
            pub async fn vote(
                &self,
                proposal_id: u64,
                voter: impl Into<String>,
            ) -> Result<cosmos_modules::gov::Vote, DaemonError> {
                let vote: cosmos_modules::gov::QueryVoteResponse = {
                    use crate::cosmos_modules::gov::{
                        query_client::QueryClient, QueryVoteRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryVoteRequest {
                        proposal_id: proposal_id,
                        voter: voter.into(),
                    };
                    let response = client.vote(request.clone()).await?.into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::gov",
                                    "cw_orch_daemon::queriers::gov",
                                    "cw-orch-daemon/src/queriers/gov.rs",
                                ),
                                65u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(vote.vote.unwrap())
            }
            /// Query votes of a given proposal
            ///
            /// see [PageRequest] for pagination
            pub async fn votes(
                &self,
                proposal_id: impl Into<u64>,
                pagination: Option<PageRequest>,
            ) -> Result<cosmos_modules::gov::QueryVotesResponse, DaemonError> {
                let votes: cosmos_modules::gov::QueryVotesResponse = {
                    use crate::cosmos_modules::gov::{
                        query_client::QueryClient, QueryVotesRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryVotesRequest {
                        proposal_id: proposal_id.into(),
                        pagination: pagination,
                    };
                    let response = client.votes(request.clone()).await?.into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::gov",
                                    "cw_orch_daemon::queriers::gov",
                                    "cw-orch-daemon/src/queriers/gov.rs",
                                ),
                                85u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(votes)
            }
            /// Query all parameters of the gov module
            pub async fn params(
                &self,
                params_type: impl Into<String>,
            ) -> Result<cosmos_modules::gov::QueryParamsResponse, DaemonError> {
                let params: cosmos_modules::gov::QueryParamsResponse = {
                    use crate::cosmos_modules::gov::{
                        query_client::QueryClient, QueryParamsRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryParamsRequest {
                        params_type: params_type.into(),
                    };
                    let response = client.params(request.clone()).await?.into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::gov",
                                    "cw_orch_daemon::queriers::gov",
                                    "cw-orch-daemon/src/queriers/gov.rs",
                                ),
                                102u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(params)
            }
            /// Query deposit information using proposal_id and depositor address
            pub async fn deposit(
                &self,
                proposal_id: u64,
                depositor: impl Into<String>,
            ) -> Result<cosmos_modules::gov::Deposit, DaemonError> {
                let deposit: cosmos_modules::gov::QueryDepositResponse = {
                    use crate::cosmos_modules::gov::{
                        query_client::QueryClient, QueryDepositRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryDepositRequest {
                        proposal_id: proposal_id,
                        depositor: depositor.into(),
                    };
                    let response = client.deposit(request.clone()).await?.into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::gov",
                                    "cw_orch_daemon::queriers::gov",
                                    "cw-orch-daemon/src/queriers/gov.rs",
                                ),
                                119u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(deposit.deposit.unwrap())
            }
            /// Query deposits of a proposal
            ///
            /// see [PageRequest] for pagination
            pub async fn deposits(
                &self,
                proposal_id: u64,
                pagination: Option<PageRequest>,
            ) -> Result<cosmos_modules::gov::QueryDepositsResponse, DaemonError> {
                let deposits: cosmos_modules::gov::QueryDepositsResponse = {
                    use crate::cosmos_modules::gov::{
                        query_client::QueryClient, QueryDepositsRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryDepositsRequest {
                        proposal_id: proposal_id,
                        pagination: pagination,
                    };
                    let response = client.deposits(request.clone()).await?.into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::gov",
                                    "cw_orch_daemon::queriers::gov",
                                    "cw-orch-daemon/src/queriers/gov.rs",
                                ),
                                139u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(deposits)
            }
            /// TallyResult queries the tally of a proposal vote.
            pub async fn tally_result(
                &mut self,
                proposal_id: u64,
            ) -> Result<cosmos_modules::gov::TallyResult, DaemonError> {
                let tally_result: cosmos_modules::gov::QueryTallyResultResponse = {
                    use crate::cosmos_modules::gov::{
                        query_client::QueryClient, QueryTallyResultRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryTallyResultRequest {
                        proposal_id: proposal_id,
                    };
                    let response = client
                        .tally_result(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::gov",
                                    "cw_orch_daemon::queriers::gov",
                                    "cw-orch-daemon/src/queriers/gov.rs",
                                ),
                                156u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(tally_result.tally.unwrap())
            }
        }
        /// Proposal status
        #[allow(missing_docs)]
        pub enum GovProposalStatus {
            Unspecified = 0,
            DepositPeriod = 1,
            VotingPeriod = 2,
            Passed = 3,
            Rejected = 4,
            Failed = 5,
        }
    }
    mod ibc {
        use super::DaemonQuerier;
        use crate::{cosmos_modules, error::DaemonError};
        use cosmos_modules::ibc_channel;
        use cosmrs::proto::ibc::{
            applications::transfer::v1::{DenomTrace, QueryDenomTraceResponse},
            core::{
                channel::v1::QueryPacketCommitmentResponse,
                client::v1::{IdentifiedClientState, QueryClientStatesResponse},
                connection::v1::{IdentifiedConnection, State},
            },
            lightclients::tendermint::v1::ClientState,
        };
        use prost::Message;
        use tonic::transport::Channel;
        /// Querier for the Cosmos IBC module
        pub struct Ibc {
            channel: Channel,
        }
        impl DaemonQuerier for Ibc {
            fn new(channel: Channel) -> Self {
                Self { channel }
            }
        }
        impl Ibc {
            /// Get the trace of a specific denom
            pub async fn denom_trace(
                &self,
                hash: String,
            ) -> Result<DenomTrace, DaemonError> {
                let denom_trace: QueryDenomTraceResponse = {
                    use crate::cosmos_modules::ibc_transfer::{
                        query_client::QueryClient, QueryDenomTraceRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryDenomTraceRequest {
                        hash: hash,
                    };
                    let response = client
                        .denom_trace(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw-orch-daemon/src/queriers/ibc.rs",
                                ),
                                32u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(denom_trace.denom_trace.unwrap())
            }
            /// Get all the IBC clients for this daemon
            pub async fn clients(
                &self,
            ) -> Result<Vec<IdentifiedClientState>, DaemonError> {
                let ibc_clients: QueryClientStatesResponse = {
                    use crate::cosmos_modules::ibc_client::{
                        query_client::QueryClient, QueryClientStatesRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryClientStatesRequest {
                        pagination: None,
                    };
                    let response = client
                        .client_states(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw-orch-daemon/src/queriers/ibc.rs",
                                ),
                                45u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(ibc_clients.client_states)
            }
            /// Get the state of a specific IBC client
            pub async fn client_state(
                &self,
                client_id: impl ToString,
            ) -> Result<
                cosmos_modules::ibc_client::QueryClientStateResponse,
                DaemonError,
            > {
                let response: cosmos_modules::ibc_client::QueryClientStateResponse = {
                    use crate::cosmos_modules::ibc_client::{
                        query_client::QueryClient, QueryClientStateRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryClientStateRequest {
                        client_id: client_id.to_string(),
                    };
                    let response = client
                        .client_state(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw-orch-daemon/src/queriers/ibc.rs",
                                ),
                                60u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(response)
            }
            /// Get the consensus state of a specific IBC client
            pub async fn consensus_states(
                &self,
                client_id: impl ToString,
            ) -> Result<
                cosmos_modules::ibc_client::QueryConsensusStatesResponse,
                DaemonError,
            > {
                let client_id = client_id.to_string();
                let response: cosmos_modules::ibc_client::QueryConsensusStatesResponse = {
                    use crate::cosmos_modules::ibc_client::{
                        query_client::QueryClient, QueryConsensusStatesRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryConsensusStatesRequest {
                        client_id: client_id,
                        pagination: None,
                    };
                    let response = client
                        .consensus_states(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw-orch-daemon/src/queriers/ibc.rs",
                                ),
                                77u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(response)
            }
            /// Get the consensus status of a specific IBC client
            pub async fn client_status(
                &self,
                client_id: impl ToString,
            ) -> Result<
                cosmos_modules::ibc_client::QueryClientStatusResponse,
                DaemonError,
            > {
                let response: cosmos_modules::ibc_client::QueryClientStatusResponse = {
                    use crate::cosmos_modules::ibc_client::{
                        query_client::QueryClient, QueryClientStatusRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryClientStatusRequest {
                        client_id: client_id.to_string(),
                    };
                    let response = client
                        .client_status(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw-orch-daemon/src/queriers/ibc.rs",
                                ),
                                95u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(response)
            }
            /// Get the ibc client parameters
            pub async fn client_params(
                &self,
            ) -> Result<
                cosmos_modules::ibc_client::QueryClientParamsResponse,
                DaemonError,
            > {
                let response: cosmos_modules::ibc_client::QueryClientParamsResponse = {
                    use crate::cosmos_modules::ibc_client::{
                        query_client::QueryClient, QueryClientParamsRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryClientParamsRequest {};
                    let response = client
                        .client_params(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw-orch-daemon/src/queriers/ibc.rs",
                                ),
                                111u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(response)
            }
            /// Query the IBC connections for a specific chain
            pub async fn connections(
                &self,
            ) -> Result<Vec<IdentifiedConnection>, DaemonError> {
                use cosmos_modules::ibc_connection::QueryConnectionsResponse;
                let ibc_connections: QueryConnectionsResponse = {
                    use crate::cosmos_modules::ibc_connection::{
                        query_client::QueryClient, QueryConnectionsRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryConnectionsRequest {
                        pagination: None,
                    };
                    let response = client
                        .connections(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw-orch-daemon/src/queriers/ibc.rs",
                                ),
                                121u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(ibc_connections.connections)
            }
            /// Search for open connections with a specific chain.
            pub async fn open_connections(
                &self,
                client_chain_id: impl ToString,
            ) -> Result<Vec<IdentifiedConnection>, DaemonError> {
                let connections = self.connections().await?;
                let mut open_connections = Vec::new();
                for connection in connections {
                    if connection.state() == State::Open {
                        open_connections.push(connection);
                    }
                }
                let mut filtered_connections = Vec::new();
                for connection in open_connections {
                    let client_state = self.connection_client(&connection.id).await?;
                    if client_state.chain_id == client_chain_id.to_string() {
                        filtered_connections.push(connection);
                    }
                }
                Ok(filtered_connections)
            }
            /// Get all the connections for this client
            pub async fn client_connections(
                &self,
                client_id: impl Into<String>,
            ) -> Result<Vec<String>, DaemonError> {
                use cosmos_modules::ibc_connection::QueryClientConnectionsResponse;
                let client_id = client_id.into();
                let ibc_client_connections: QueryClientConnectionsResponse = {
                    use crate::cosmos_modules::ibc_connection::{
                        query_client::QueryClient, QueryClientConnectionsRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryClientConnectionsRequest {
                        client_id: client_id.clone(),
                    };
                    let response = client
                        .client_connections(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw-orch-daemon/src/queriers/ibc.rs",
                                ),
                                163u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(ibc_client_connections.connection_paths)
            }
            /// Get the (tendermint) client state for a specific connection
            pub async fn connection_client(
                &self,
                connection_id: impl Into<String>,
            ) -> Result<ClientState, DaemonError> {
                use cosmos_modules::ibc_connection::QueryConnectionClientStateResponse;
                let connection_id = connection_id.into();
                let ibc_connection_client: QueryConnectionClientStateResponse = {
                    use crate::cosmos_modules::ibc_connection::{
                        query_client::QueryClient, QueryConnectionClientStateRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryConnectionClientStateRequest {
                        connection_id: connection_id.clone(),
                    };
                    let response = client
                        .connection_client_state(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw-orch-daemon/src/queriers/ibc.rs",
                                ),
                                183u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                let client_state = ibc_connection_client
                    .identified_client_state
                    .ok_or(
                        DaemonError::ibc_err({
                            let res = ::alloc::fmt::format(
                                format_args!(
                                    "error identifying client for connection {0}",
                                    connection_id,
                                ),
                            );
                            res
                        }),
                    )?;
                let client_state = ClientState::decode(
                        client_state.client_state.unwrap().value.as_slice(),
                    )
                    .map_err(|e| DaemonError::ibc_err({
                        let res = ::alloc::fmt::format(
                            format_args!("error decoding client state: {0}", e),
                        );
                        res
                    }))?;
                Ok(client_state)
            }
            /// Get the channel for a specific port and channel id
            pub async fn channel(
                &self,
                port_id: impl Into<String>,
                channel_id: impl Into<String>,
            ) -> Result<ibc_channel::Channel, DaemonError> {
                use cosmos_modules::ibc_channel::QueryChannelResponse;
                let port_id = port_id.into();
                let channel_id = channel_id.into();
                let ibc_channel: QueryChannelResponse = {
                    use crate::cosmos_modules::ibc_channel::{
                        query_client::QueryClient, QueryChannelRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryChannelRequest {
                        port_id: port_id.clone(),
                        channel_id: channel_id.clone(),
                    };
                    let response = client.channel(request.clone()).await?.into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw-orch-daemon/src/queriers/ibc.rs",
                                ),
                                218u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                ibc_channel
                    .channel
                    .ok_or(
                        DaemonError::ibc_err({
                            let res = ::alloc::fmt::format(
                                format_args!(
                                    "error fetching channel {0} on port {1}",
                                    channel_id,
                                    port_id,
                                ),
                            );
                            res
                        }),
                    )
            }
            /// Get all the channels for a specific connection
            pub async fn connection_channels(
                &self,
                connection_id: impl Into<String>,
            ) -> Result<Vec<ibc_channel::IdentifiedChannel>, DaemonError> {
                use cosmos_modules::ibc_channel::QueryConnectionChannelsResponse;
                let connection_id = connection_id.into();
                let ibc_connection_channels: QueryConnectionChannelsResponse = {
                    use crate::cosmos_modules::ibc_channel::{
                        query_client::QueryClient, QueryConnectionChannelsRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryConnectionChannelsRequest {
                        connection: connection_id.clone(),
                        pagination: None,
                    };
                    let response = client
                        .connection_channels(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw-orch-daemon/src/queriers/ibc.rs",
                                ),
                                242u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(ibc_connection_channels.channels)
            }
            /// Get the client state for a specific channel and port
            pub async fn channel_client_state(
                &self,
                port_id: impl Into<String>,
                channel_id: impl Into<String>,
            ) -> Result<IdentifiedClientState, DaemonError> {
                use cosmos_modules::ibc_channel::QueryChannelClientStateResponse;
                let port_id = port_id.into();
                let channel_id = channel_id.into();
                let ibc_channel_client_state: QueryChannelClientStateResponse = {
                    use crate::cosmos_modules::ibc_channel::{
                        query_client::QueryClient, QueryChannelClientStateRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryChannelClientStateRequest {
                        port_id: port_id.clone(),
                        channel_id: channel_id.clone(),
                    };
                    let response = client
                        .channel_client_state(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw-orch-daemon/src/queriers/ibc.rs",
                                ),
                                265u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                ibc_channel_client_state
                    .identified_client_state
                    .ok_or(
                        DaemonError::ibc_err({
                            let res = ::alloc::fmt::format(
                                format_args!(
                                    "error identifying client for channel {0} on port {1}",
                                    channel_id,
                                    port_id,
                                ),
                            );
                            res
                        }),
                    )
            }
            /// Get all the packet commitments for a specific channel and port
            pub async fn packet_commitments(
                &self,
                port_id: impl Into<String>,
                channel_id: impl Into<String>,
            ) -> Result<Vec<ibc_channel::PacketState>, DaemonError> {
                use cosmos_modules::ibc_channel::QueryPacketCommitmentsResponse;
                let port_id = port_id.into();
                let channel_id = channel_id.into();
                let ibc_packet_commitments: QueryPacketCommitmentsResponse = {
                    use crate::cosmos_modules::ibc_channel::{
                        query_client::QueryClient, QueryPacketCommitmentsRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryPacketCommitmentsRequest {
                        port_id: port_id.clone(),
                        channel_id: channel_id.clone(),
                        pagination: None,
                    };
                    let response = client
                        .packet_commitments(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw-orch-daemon/src/queriers/ibc.rs",
                                ),
                                297u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(ibc_packet_commitments.commitments)
            }
            /// Get the packet commitment for a specific channel, port and sequence
            pub async fn packet_commitment(
                &self,
                port_id: impl Into<String>,
                channel_id: impl Into<String>,
                sequence: u64,
            ) -> Result<QueryPacketCommitmentResponse, DaemonError> {
                let port_id = port_id.into();
                let channel_id = channel_id.into();
                let ibc_packet_commitment: QueryPacketCommitmentResponse = {
                    use crate::cosmos_modules::ibc_channel::{
                        query_client::QueryClient, QueryPacketCommitmentRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryPacketCommitmentRequest {
                        port_id: port_id.clone(),
                        channel_id: channel_id.clone(),
                        sequence: sequence,
                    };
                    let response = client
                        .packet_commitment(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw-orch-daemon/src/queriers/ibc.rs",
                                ),
                                320u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(ibc_packet_commitment)
            }
            /// Returns if the packet is received on the connected chain.
            pub async fn packet_receipt(
                &self,
                port_id: impl Into<String>,
                channel_id: impl Into<String>,
                sequence: u64,
            ) -> Result<bool, DaemonError> {
                let port_id = port_id.into();
                let channel_id = channel_id.into();
                let ibc_packet_receipt: ibc_channel::QueryPacketReceiptResponse = {
                    use crate::cosmos_modules::ibc_channel::{
                        query_client::QueryClient, QueryPacketReceiptRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryPacketReceiptRequest {
                        port_id: port_id.clone(),
                        channel_id: channel_id.clone(),
                        sequence: sequence,
                    };
                    let response = client
                        .packet_receipt(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw-orch-daemon/src/queriers/ibc.rs",
                                ),
                                345u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(ibc_packet_receipt.received)
            }
            /// Get all the packet acknowledgements for a specific channel, port and commitment sequences
            pub async fn packet_acknowledgements(
                &self,
                port_id: impl Into<String>,
                channel_id: impl Into<String>,
                packet_commitment_sequences: Vec<u64>,
            ) -> Result<Vec<ibc_channel::PacketState>, DaemonError> {
                use cosmos_modules::ibc_channel::QueryPacketAcknowledgementsResponse;
                let port_id = port_id.into();
                let channel_id = channel_id.into();
                let ibc_packet_acknowledgements: QueryPacketAcknowledgementsResponse = {
                    use crate::cosmos_modules::ibc_channel::{
                        query_client::QueryClient, QueryPacketAcknowledgementsRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryPacketAcknowledgementsRequest {
                        port_id: port_id.clone(),
                        channel_id: channel_id.clone(),
                        packet_commitment_sequences: packet_commitment_sequences,
                        pagination: None,
                    };
                    let response = client
                        .packet_acknowledgements(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw-orch-daemon/src/queriers/ibc.rs",
                                ),
                                372u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(ibc_packet_acknowledgements.acknowledgements)
            }
            /// Get the packet acknowledgement for a specific channel, port and sequence
            pub async fn packet_acknowledgement(
                &self,
                port_id: impl Into<String>,
                channel_id: impl Into<String>,
                sequence: u64,
            ) -> Result<Vec<u8>, DaemonError> {
                let port_id = port_id.into();
                let channel_id = channel_id.into();
                let ibc_packet_acknowledgement: ibc_channel::QueryPacketAcknowledgementResponse = {
                    use crate::cosmos_modules::ibc_channel::{
                        query_client::QueryClient, QueryPacketAcknowledgementRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryPacketAcknowledgementRequest {
                        port_id: port_id.clone(),
                        channel_id: channel_id.clone(),
                        sequence: sequence,
                    };
                    let response = client
                        .packet_acknowledgement(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw-orch-daemon/src/queriers/ibc.rs",
                                ),
                                396u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(ibc_packet_acknowledgement.acknowledgement)
            }
            /// No acknowledgement exists on receiving chain for the given packet commitment sequence on sending chain.
            /// Returns the packet sequences that have not yet been received.
            pub async fn unreceived_packets(
                &self,
                port_id: impl Into<String>,
                channel_id: impl Into<String>,
                packet_commitment_sequences: Vec<u64>,
            ) -> Result<Vec<u64>, DaemonError> {
                use cosmos_modules::ibc_channel::QueryUnreceivedPacketsResponse;
                let port_id = port_id.into();
                let channel_id = channel_id.into();
                let ibc_packet_unreceived: QueryUnreceivedPacketsResponse = {
                    use crate::cosmos_modules::ibc_channel::{
                        query_client::QueryClient, QueryUnreceivedPacketsRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryUnreceivedPacketsRequest {
                        port_id: port_id.clone(),
                        channel_id: channel_id.clone(),
                        packet_commitment_sequences: packet_commitment_sequences,
                    };
                    let response = client
                        .unreceived_packets(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw-orch-daemon/src/queriers/ibc.rs",
                                ),
                                422u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(ibc_packet_unreceived.sequences)
            }
            /// Returns the acknowledgement sequences that have not yet been received.
            /// Given a list of acknowledgement sequences from counterparty, determine if an ack on the counterparty chain has been received on the executing chain.
            /// Returns the list of acknowledgement sequences that have not yet been received.
            pub async fn unreceived_acks(
                &self,
                port_id: impl Into<String>,
                channel_id: impl Into<String>,
                packet_ack_sequences: Vec<u64>,
            ) -> Result<Vec<u64>, DaemonError> {
                let port_id = port_id.into();
                let channel_id = channel_id.into();
                let ibc_packet_unreceived: ibc_channel::QueryUnreceivedAcksResponse = {
                    use crate::cosmos_modules::ibc_channel::{
                        query_client::QueryClient, QueryUnreceivedAcksRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryUnreceivedAcksRequest {
                        port_id: port_id.clone(),
                        channel_id: channel_id.clone(),
                        packet_ack_sequences: packet_ack_sequences,
                    };
                    let response = client
                        .unreceived_acks(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw-orch-daemon/src/queriers/ibc.rs",
                                ),
                                447u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(ibc_packet_unreceived.sequences)
            }
            /// Returns the acknowledgement sequences that have not yet been received.
            /// Given a list of acknowledgement sequences from counterparty, determine if an ack on the counterparty chain has been received on the executing chain.
            /// Returns the list of acknowledgement sequences that have not yet been received.
            pub async fn next_sequence_receive(
                &self,
                port_id: impl Into<String>,
                channel_id: impl Into<String>,
            ) -> Result<u64, DaemonError> {
                let port_id = port_id.into();
                let channel_id = channel_id.into();
                let next_receive: ibc_channel::QueryNextSequenceReceiveResponse = {
                    use crate::cosmos_modules::ibc_channel::{
                        query_client::QueryClient, QueryNextSequenceReceiveRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryNextSequenceReceiveRequest {
                        port_id: port_id.clone(),
                        channel_id: channel_id.clone(),
                    };
                    let response = client
                        .next_sequence_receive(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw_orch_daemon::queriers::ibc",
                                    "cw-orch-daemon/src/queriers/ibc.rs",
                                ),
                                471u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(next_receive.next_sequence_receive)
            }
        }
    }
    mod node {
        use std::{cmp::min, time::Duration};
        use crate::{cosmos_modules, error::DaemonError, tx_resp::CosmTxResponse};
        use cosmrs::{
            proto::cosmos::{
                base::query::v1beta1::PageRequest, tx::v1beta1::SimulateResponse,
            },
            tendermint::{Block, Time},
        };
        use tonic::transport::Channel;
        use super::DaemonQuerier;
        const MAX_TX_QUERY_RETRIES: usize = 50;
        /// Querier for the Tendermint node.
        /// Supports queries for block and tx information
        pub struct Node {
            channel: Channel,
        }
        impl DaemonQuerier for Node {
            fn new(channel: Channel) -> Self {
                Self { channel }
            }
        }
        impl Node {
            /// Returns node info
            pub async fn info(
                &self,
            ) -> Result<cosmos_modules::tendermint::GetNodeInfoResponse, DaemonError> {
                let mut client = cosmos_modules::tendermint::service_client::ServiceClient::new(
                    self.channel.clone(),
                );
                let resp = client
                    .get_node_info(cosmos_modules::tendermint::GetNodeInfoRequest {
                    })
                    .await?
                    .into_inner();
                Ok(resp)
            }
            /// Queries node syncing
            pub async fn syncing(&self) -> Result<bool, DaemonError> {
                let mut client = cosmos_modules::tendermint::service_client::ServiceClient::new(
                    self.channel.clone(),
                );
                let resp = client
                    .get_syncing(cosmos_modules::tendermint::GetSyncingRequest {
                    })
                    .await?
                    .into_inner();
                Ok(resp.syncing)
            }
            /// Returns latests block information
            pub async fn latest_block(&self) -> Result<Block, DaemonError> {
                let mut client = cosmos_modules::tendermint::service_client::ServiceClient::new(
                    self.channel.clone(),
                );
                let resp = client
                    .get_latest_block(cosmos_modules::tendermint::GetLatestBlockRequest {
                    })
                    .await?
                    .into_inner();
                Ok(Block::try_from(resp.block.unwrap())?)
            }
            /// Returns block information fetched by height
            pub async fn block_by_height(
                &self,
                height: u64,
            ) -> Result<Block, DaemonError> {
                let mut client = cosmos_modules::tendermint::service_client::ServiceClient::new(
                    self.channel.clone(),
                );
                let resp = client
                    .get_block_by_height(cosmos_modules::tendermint::GetBlockByHeightRequest {
                        height: height as i64,
                    })
                    .await?
                    .into_inner();
                Ok(Block::try_from(resp.block.unwrap())?)
            }
            /// Return the average block time for the last 50 blocks or since inception
            /// This is used to estimate the time when a tx will be included in a block
            pub async fn average_block_speed(
                &self,
                multiplier: Option<f32>,
            ) -> Result<u64, DaemonError> {
                let mut latest_block = self.latest_block().await?;
                let latest_block_time = latest_block.header.time;
                let mut latest_block_height = latest_block.header.height.value();
                while latest_block_height <= 1 {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    latest_block = self.latest_block().await?;
                    latest_block_height = latest_block.header.height.value();
                }
                let avg_period = min(latest_block_height - 1, 50);
                let block_avg_period_ago = self
                    .block_by_height(latest_block_height - avg_period)
                    .await?;
                let block_avg_period_ago_time = block_avg_period_ago.header.time;
                let average_block_time = latest_block_time
                    .duration_since(block_avg_period_ago_time)?;
                let average_block_time = average_block_time.as_secs() / avg_period;
                let average_block_time = match multiplier {
                    Some(multiplier) => (average_block_time as f32 * multiplier) as u64,
                    None => average_block_time,
                };
                Ok(std::cmp::max(average_block_time, 1))
            }
            /// Returns latests validator set
            pub async fn latest_validator_set(
                &self,
                pagination: Option<PageRequest>,
            ) -> Result<
                cosmos_modules::tendermint::GetLatestValidatorSetResponse,
                DaemonError,
            > {
                let mut client = cosmos_modules::tendermint::service_client::ServiceClient::new(
                    self.channel.clone(),
                );
                let resp = client
                    .get_latest_validator_set(cosmos_modules::tendermint::GetLatestValidatorSetRequest {
                        pagination,
                    })
                    .await?
                    .into_inner();
                Ok(resp)
            }
            /// Returns latests validator set fetched by height
            pub async fn validator_set_by_height(
                &self,
                height: i64,
                pagination: Option<PageRequest>,
            ) -> Result<
                cosmos_modules::tendermint::GetValidatorSetByHeightResponse,
                DaemonError,
            > {
                let mut client = cosmos_modules::tendermint::service_client::ServiceClient::new(
                    self.channel.clone(),
                );
                let resp = client
                    .get_validator_set_by_height(cosmos_modules::tendermint::GetValidatorSetByHeightRequest {
                        height,
                        pagination,
                    })
                    .await?
                    .into_inner();
                Ok(resp)
            }
            /// Returns current block height
            pub async fn block_height(&self) -> Result<u64, DaemonError> {
                let block = self.latest_block().await?;
                Ok(block.header.height.value())
            }
            /// Returns the block timestamp (since unix epoch) in nanos
            pub async fn block_time(&self) -> Result<u128, DaemonError> {
                let block = self.latest_block().await?;
                Ok(block.header.time.duration_since(Time::unix_epoch())?.as_nanos())
            }
            /// Simulate TX
            pub async fn simulate_tx(
                &self,
                tx_bytes: Vec<u8>,
            ) -> Result<u64, DaemonError> {
                let mut client = cosmos_modules::tx::service_client::ServiceClient::new(
                    self.channel.clone(),
                );
                #[allow(deprecated)]
                let resp: SimulateResponse = client
                    .simulate(cosmos_modules::tx::SimulateRequest {
                        tx: None,
                        tx_bytes,
                    })
                    .await?
                    .into_inner();
                let gas_used = resp.gas_info.unwrap().gas_used;
                Ok(gas_used)
            }
            /// Returns all the block info
            pub async fn block_info(
                &self,
            ) -> Result<cosmwasm_std::BlockInfo, DaemonError> {
                let block = self.latest_block().await?;
                let since_epoch = block.header.time.duration_since(Time::unix_epoch())?;
                let time = cosmwasm_std::Timestamp::from_nanos(
                    since_epoch.as_nanos() as u64,
                );
                Ok(cosmwasm_std::BlockInfo {
                    height: block.header.height.value(),
                    time,
                    chain_id: block.header.chain_id.to_string(),
                })
            }
            /// Find TX by hash
            pub async fn find_tx(
                &self,
                hash: String,
            ) -> Result<CosmTxResponse, DaemonError> {
                self.find_tx_with_retries(hash, MAX_TX_QUERY_RETRIES).await
            }
            /// Find TX by hash with a given amount of retries
            pub async fn find_tx_with_retries(
                &self,
                hash: String,
                retries: usize,
            ) -> Result<CosmTxResponse, DaemonError> {
                let mut client = cosmos_modules::tx::service_client::ServiceClient::new(
                    self.channel.clone(),
                );
                let request = cosmos_modules::tx::GetTxRequest {
                    hash: hash.clone(),
                };
                let mut block_speed = self.average_block_speed(Some(0.7)).await?;
                for _ in 0..retries {
                    match client.get_tx(request.clone()).await {
                        Ok(tx) => {
                            let resp = tx.into_inner().tx_response.unwrap();
                            {
                                let lvl = ::log::Level::Debug;
                                if lvl <= ::log::STATIC_MAX_LEVEL
                                    && lvl <= ::log::max_level()
                                {
                                    ::log::__private_api::log(
                                        format_args!("TX found: {0:?}", resp),
                                        lvl,
                                        &(
                                            "cw_orch_daemon::queriers::node",
                                            "cw_orch_daemon::queriers::node",
                                            "cw-orch-daemon/src/queriers/node.rs",
                                        ),
                                        220u32,
                                        ::log::__private_api::Option::None,
                                    );
                                }
                            };
                            return Ok(resp.into());
                        }
                        Err(err) => {
                            block_speed = (block_speed as f64 * 1.6) as u64;
                            {
                                let lvl = ::log::Level::Debug;
                                if lvl <= ::log::STATIC_MAX_LEVEL
                                    && lvl <= ::log::max_level()
                                {
                                    ::log::__private_api::log(
                                        format_args!("TX not found with error: {0:?}", err),
                                        lvl,
                                        &(
                                            "cw_orch_daemon::queriers::node",
                                            "cw_orch_daemon::queriers::node",
                                            "cw-orch-daemon/src/queriers/node.rs",
                                        ),
                                        226u32,
                                        ::log::__private_api::Option::None,
                                    );
                                }
                            };
                            {
                                let lvl = ::log::Level::Debug;
                                if lvl <= ::log::STATIC_MAX_LEVEL
                                    && lvl <= ::log::max_level()
                                {
                                    ::log::__private_api::log(
                                        format_args!("Waiting {0} seconds", block_speed),
                                        lvl,
                                        &(
                                            "cw_orch_daemon::queriers::node",
                                            "cw_orch_daemon::queriers::node",
                                            "cw-orch-daemon/src/queriers/node.rs",
                                        ),
                                        227u32,
                                        ::log::__private_api::Option::None,
                                    );
                                }
                            };
                            tokio::time::sleep(Duration::from_secs(block_speed)).await;
                        }
                    }
                }
                Err(DaemonError::TXNotFound(hash, retries))
            }
        }
    }
    mod staking {
        use crate::{cosmos_modules, error::DaemonError};
        use cosmrs::proto::cosmos::base::query::v1beta1::PageRequest;
        use tonic::transport::Channel;
        use super::DaemonQuerier;
        /// Querier for the Cosmos Staking module
        pub struct Staking {
            channel: Channel,
        }
        impl DaemonQuerier for Staking {
            fn new(channel: Channel) -> Self {
                Self { channel }
            }
        }
        impl Staking {
            /// Queries validator info for given validator address
            pub async fn validator(
                &self,
                validator_addr: impl Into<String>,
            ) -> Result<cosmos_modules::staking::Validator, DaemonError> {
                let validator: cosmos_modules::staking::QueryValidatorResponse = {
                    use crate::cosmos_modules::staking::{
                        query_client::QueryClient, QueryValidatorRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryValidatorRequest {
                        validator_addr: validator_addr.into(),
                    };
                    let response = client.validator(request.clone()).await?.into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::staking",
                                    "cw_orch_daemon::queriers::staking",
                                    "cw-orch-daemon/src/queriers/staking.rs",
                                ),
                                24u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(validator.validator.unwrap())
            }
            /// Queries all validators that match the given status
            ///
            /// see [StakingBondStatus] for available statuses
            pub async fn validators(
                &self,
                status: StakingBondStatus,
            ) -> Result<Vec<cosmos_modules::staking::Validator>, DaemonError> {
                let validators: cosmos_modules::staking::QueryValidatorsResponse = {
                    use crate::cosmos_modules::staking::{
                        query_client::QueryClient, QueryValidatorsRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryValidatorsRequest {
                        status: status.to_string(),
                        pagination: None,
                    };
                    let response = client
                        .validators(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::staking",
                                    "cw_orch_daemon::queriers::staking",
                                    "cw-orch-daemon/src/queriers/staking.rs",
                                ),
                                42u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(validators.validators)
            }
            /// Query validator delegations info for given validator
            ///
            /// see [PageRequest] for pagination
            pub async fn delegations(
                &self,
                validator_addr: impl Into<String>,
                pagination: Option<PageRequest>,
            ) -> Result<Vec<cosmos_modules::staking::DelegationResponse>, DaemonError> {
                let validator_delegations: cosmos_modules::staking::QueryValidatorDelegationsResponse = {
                    use crate::cosmos_modules::staking::{
                        query_client::QueryClient, QueryValidatorDelegationsRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryValidatorDelegationsRequest {
                        validator_addr: validator_addr.into(),
                        pagination: pagination,
                    };
                    let response = client
                        .validator_delegations(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::staking",
                                    "cw_orch_daemon::queriers::staking",
                                    "cw-orch-daemon/src/queriers/staking.rs",
                                ),
                                62u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(validator_delegations.delegation_responses)
            }
            /// Query validator unbonding delegations of a validator
            pub async fn unbonding_delegations(
                &self,
                validator_addr: impl Into<String>,
            ) -> Result<Vec<cosmos_modules::staking::UnbondingDelegation>, DaemonError> {
                let validator_unbonding_delegations: cosmos_modules::staking::QueryValidatorUnbondingDelegationsResponse = {
                    use crate::cosmos_modules::staking::{
                        query_client::QueryClient,
                        QueryValidatorUnbondingDelegationsRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryValidatorUnbondingDelegationsRequest {
                        validator_addr: validator_addr.into(),
                        pagination: None,
                    };
                    let response = client
                        .validator_unbonding_delegations(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::staking",
                                    "cw_orch_daemon::queriers::staking",
                                    "cw-orch-daemon/src/queriers/staking.rs",
                                ),
                                79u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(validator_unbonding_delegations.unbonding_responses)
            }
            /// Query delegation info for given validator for a delegator
            pub async fn delegation(
                &self,
                validator_addr: impl Into<String>,
                delegator_addr: impl Into<String>,
            ) -> Result<cosmos_modules::staking::DelegationResponse, DaemonError> {
                let delegation: cosmos_modules::staking::QueryDelegationResponse = {
                    use crate::cosmos_modules::staking::{
                        query_client::QueryClient, QueryDelegationRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryDelegationRequest {
                        validator_addr: validator_addr.into(),
                        delegator_addr: delegator_addr.into(),
                    };
                    let response = client
                        .delegation(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::staking",
                                    "cw_orch_daemon::queriers::staking",
                                    "cw-orch-daemon/src/queriers/staking.rs",
                                ),
                                97u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(delegation.delegation_response.unwrap())
            }
            /// Query unbonding delegation info for given validator delegator
            pub async fn unbonding_delegation(
                &self,
                validator_addr: impl Into<String>,
                delegator_addr: impl Into<String>,
            ) -> Result<cosmos_modules::staking::UnbondingDelegation, DaemonError> {
                let unbonding_delegation: cosmos_modules::staking::QueryUnbondingDelegationResponse = {
                    use crate::cosmos_modules::staking::{
                        query_client::QueryClient, QueryUnbondingDelegationRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryUnbondingDelegationRequest {
                        validator_addr: validator_addr.into(),
                        delegator_addr: delegator_addr.into(),
                    };
                    let response = client
                        .unbonding_delegation(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::staking",
                                    "cw_orch_daemon::queriers::staking",
                                    "cw-orch-daemon/src/queriers/staking.rs",
                                ),
                                115u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(unbonding_delegation.unbond.unwrap())
            }
            /// Query all delegator delegations of a given delegator address
            ///
            /// see [PageRequest] for pagination
            pub async fn delegator_delegations(
                &self,
                delegator_addr: impl Into<String>,
                pagination: Option<PageRequest>,
            ) -> Result<
                cosmos_modules::staking::QueryDelegatorDelegationsResponse,
                DaemonError,
            > {
                let delegator_delegations: cosmos_modules::staking::QueryDelegatorDelegationsResponse = {
                    use crate::cosmos_modules::staking::{
                        query_client::QueryClient, QueryDelegatorDelegationsRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryDelegatorDelegationsRequest {
                        delegator_addr: delegator_addr.into(),
                        pagination: pagination,
                    };
                    let response = client
                        .delegator_delegations(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::staking",
                                    "cw_orch_daemon::queriers::staking",
                                    "cw-orch-daemon/src/queriers/staking.rs",
                                ),
                                135u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(delegator_delegations)
            }
            /// Queries all unbonding delegations of a given delegator address.
            ///
            /// see [PageRequest] for pagination
            pub async fn delegator_unbonding_delegations(
                &self,
                delegator_addr: impl Into<String>,
                pagination: Option<PageRequest>,
            ) -> Result<
                cosmos_modules::staking::QueryDelegatorUnbondingDelegationsResponse,
                DaemonError,
            > {
                let delegator_unbonding_delegations: cosmos_modules::staking::QueryDelegatorUnbondingDelegationsResponse = {
                    use crate::cosmos_modules::staking::{
                        query_client::QueryClient,
                        QueryDelegatorUnbondingDelegationsRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryDelegatorUnbondingDelegationsRequest {
                        delegator_addr: delegator_addr.into(),
                        pagination: pagination,
                    };
                    let response = client
                        .delegator_unbonding_delegations(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::staking",
                                    "cw_orch_daemon::queriers::staking",
                                    "cw-orch-daemon/src/queriers/staking.rs",
                                ),
                                156u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(delegator_unbonding_delegations)
            }
            /// Query redelegations of a given address
            ///
            /// see [PageRequest] for pagination
            pub async fn redelegations(
                &self,
                delegator_addr: impl Into<String>,
                src_validator_addr: impl Into<String>,
                dst_validator_addr: impl Into<String>,
                pagination: Option<PageRequest>,
            ) -> Result<
                cosmos_modules::staking::QueryRedelegationsResponse,
                DaemonError,
            > {
                let redelegations: cosmos_modules::staking::QueryRedelegationsResponse = {
                    use crate::cosmos_modules::staking::{
                        query_client::QueryClient, QueryRedelegationsRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryRedelegationsRequest {
                        delegator_addr: delegator_addr.into(),
                        src_validator_addr: src_validator_addr.into(),
                        dst_validator_addr: dst_validator_addr.into(),
                        pagination: pagination,
                    };
                    let response = client
                        .redelegations(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::staking",
                                    "cw_orch_daemon::queriers::staking",
                                    "cw-orch-daemon/src/queriers/staking.rs",
                                ),
                                178u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(redelegations)
            }
            /// Query delegator validators info for given delegator address.
            pub async fn delegator_validator(
                &self,
                validator_addr: impl Into<String>,
                delegator_addr: impl Into<String>,
            ) -> Result<
                cosmos_modules::staking::QueryDelegatorValidatorResponse,
                DaemonError,
            > {
                let delegator_validator: cosmos_modules::staking::QueryDelegatorValidatorResponse = {
                    use crate::cosmos_modules::staking::{
                        query_client::QueryClient, QueryDelegatorValidatorRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryDelegatorValidatorRequest {
                        validator_addr: validator_addr.into(),
                        delegator_addr: delegator_addr.into(),
                    };
                    let response = client
                        .delegator_validator(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::staking",
                                    "cw_orch_daemon::queriers::staking",
                                    "cw-orch-daemon/src/queriers/staking.rs",
                                ),
                                198u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(delegator_validator)
            }
            /// Query delegator validators info for given delegator address
            ///
            /// see [PageRequest] for pagination
            pub async fn delegator_validators(
                &self,
                delegator_addr: impl Into<String>,
                pagination: Option<PageRequest>,
            ) -> Result<
                cosmos_modules::staking::QueryDelegatorValidatorsResponse,
                DaemonError,
            > {
                let delegator_validators: cosmos_modules::staking::QueryDelegatorValidatorsResponse = {
                    use crate::cosmos_modules::staking::{
                        query_client::QueryClient, QueryDelegatorValidatorsRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryDelegatorValidatorsRequest {
                        delegator_addr: delegator_addr.into(),
                        pagination: pagination,
                    };
                    let response = client
                        .delegator_validators(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::staking",
                                    "cw_orch_daemon::queriers::staking",
                                    "cw-orch-daemon/src/queriers/staking.rs",
                                ),
                                218u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(delegator_validators)
            }
            /// Query historical info info for given height
            pub async fn historical_info(
                &self,
                height: i64,
            ) -> Result<
                cosmos_modules::staking::QueryHistoricalInfoResponse,
                DaemonError,
            > {
                let historical_info: cosmos_modules::staking::QueryHistoricalInfoResponse = {
                    use crate::cosmos_modules::staking::{
                        query_client::QueryClient, QueryHistoricalInfoRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryHistoricalInfoRequest {
                        height: height,
                    };
                    let response = client
                        .historical_info(request.clone())
                        .await?
                        .into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::staking",
                                    "cw_orch_daemon::queriers::staking",
                                    "cw-orch-daemon/src/queriers/staking.rs",
                                ),
                                236u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(historical_info)
            }
            /// Query the pool info
            pub async fn pool(
                &self,
            ) -> Result<cosmos_modules::staking::QueryPoolResponse, DaemonError> {
                let pool: cosmos_modules::staking::QueryPoolResponse = {
                    use crate::cosmos_modules::staking::{
                        query_client::QueryClient, QueryPoolRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryPoolRequest {};
                    let response = client.pool(request.clone()).await?.into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::staking",
                                    "cw_orch_daemon::queriers::staking",
                                    "cw-orch-daemon/src/queriers/staking.rs",
                                ),
                                248u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(pool)
            }
            /// Query staking parameters
            pub async fn params(
                &self,
            ) -> Result<cosmos_modules::staking::QueryParamsResponse, DaemonError> {
                let params: cosmos_modules::staking::QueryParamsResponse = {
                    use crate::cosmos_modules::staking::{
                        query_client::QueryClient, QueryParamsRequest,
                    };
                    let mut client = QueryClient::new(self.channel.clone());
                    #[allow(clippy::redundant_field_names)]
                    let request = QueryParamsRequest {};
                    let response = client.params(request.clone()).await?.into_inner();
                    {
                        let lvl = ::log::Level::Trace;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                format_args!(
                                    "cosmos_query: {0:?} resulted in: {1:?}",
                                    request,
                                    response,
                                ),
                                lvl,
                                &(
                                    "cw_orch_daemon::queriers::staking",
                                    "cw_orch_daemon::queriers::staking",
                                    "cw-orch-daemon/src/queriers/staking.rs",
                                ),
                                257u32,
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    response
                };
                Ok(params)
            }
        }
        /// Staking bond statuses
        pub enum StakingBondStatus {
            /// UNSPECIFIED defines an invalid validator status.
            Unspecified = 0,
            /// UNBONDED defines a validator that is not bonded.
            Unbonded = 1,
            /// UNBONDING defines a validator that is unbonding.
            Unbonding = 2,
            /// BONDED defines a validator that is bonded.
            Bonded = 3,
        }
        impl ToString for StakingBondStatus {
            /// Convert to string
            fn to_string(&self) -> String {
                match self {
                    StakingBondStatus::Unspecified => {
                        "BOND_STATUS_UNSPECIFIED".to_string()
                    }
                    StakingBondStatus::Unbonded => "BOND_STATUS_UNBONDED".to_string(),
                    StakingBondStatus::Unbonding => "BOND_STATUS_UNBONDING".to_string(),
                    StakingBondStatus::Bonded => "BOND_STATUS_BONDED".to_string(),
                }
            }
        }
    }
    pub use bank::Bank;
    pub use cosmwasm::CosmWasm;
    pub use feegrant::Feegrant;
    pub use ibc::Ibc;
    pub use node::Node;
    pub use gov::*;
    pub use staking::*;
    use tonic::transport::Channel;
    /// Constructor for a querier over a given channel
    pub trait DaemonQuerier {
        /// Construct an new querier over a given channel
        fn new(channel: Channel) -> Self;
    }
}
mod traits {
    use cw_orch_core::{
        contract::interface_traits::{CwOrchMigrate, CwOrchUpload},
        environment::TxResponse,
    };
    use crate::{queriers::CosmWasm, Daemon, DaemonError};
    /// Helper methods for conditional uploading of a contract.
    pub trait ConditionalUpload: CwOrchUpload<Daemon> {
        /// Only upload the contract if it is not uploaded yet (checksum does not match)
        fn upload_if_needed(&self) -> Result<Option<TxResponse<Daemon>>, DaemonError> {
            if self.latest_is_uploaded()? {
                Ok(None)
            } else {
                Some(self.upload()).transpose().map_err(Into::into)
            }
        }
        /// Returns whether the checksum of the WASM file matches the checksum of the latest uploaded code for this contract.
        fn latest_is_uploaded(&self) -> Result<bool, DaemonError> {
            let Some(latest_uploaded_code_id) = self.code_id().ok() else {
                return Ok(false);
            };
            let chain = self.get_chain();
            let on_chain_hash = chain
                .rt_handle
                .block_on(
                    chain
                        .query_client::<CosmWasm>()
                        .code_id_hash(latest_uploaded_code_id),
                )?;
            let local_hash = self.wasm().checksum()?;
            Ok(local_hash == on_chain_hash)
        }
        /// Returns whether the contract is running the latest uploaded code for it
        fn is_running_latest(&self) -> Result<bool, DaemonError> {
            let Some(latest_uploaded_code_id) = self.code_id().ok() else {
                return Ok(false);
            };
            let chain = self.get_chain();
            let info = chain
                .rt_handle
                .block_on(
                    chain.query_client::<CosmWasm>().contract_info(self.address()?),
                )?;
            Ok(latest_uploaded_code_id == info.code_id)
        }
    }
    impl<T> ConditionalUpload for T
    where
        T: CwOrchUpload<Daemon>,
    {}
    /// Helper methods for conditional migration of a contract.
    pub trait ConditionalMigrate: CwOrchMigrate<Daemon> + ConditionalUpload {
        /// Only migrate the contract if it is not on the latest code-id yet
        fn migrate_if_needed(
            &self,
            migrate_msg: &Self::MigrateMsg,
        ) -> Result<Option<TxResponse<Daemon>>, DaemonError> {
            if self.is_running_latest()? {
                {
                    let lvl = ::log::Level::Info;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            format_args!(
                                "{0} is already running the latest code",
                                self.id(),
                            ),
                            lvl,
                            &(
                                "cw_orch_daemon::traits",
                                "cw_orch_daemon::traits",
                                "cw-orch-daemon/src/traits.rs",
                            ),
                            61u32,
                            ::log::__private_api::Option::None,
                        );
                    }
                };
                Ok(None)
            } else {
                Some(self.migrate(migrate_msg, self.code_id()?))
                    .transpose()
                    .map_err(Into::into)
            }
        }
        /// Uploads the contract if the local contract hash is different from the latest on-chain code hash.
        /// Proceeds to migrates the contract if the contract is not running the latest code.
        fn upload_and_migrate_if_needed(
            &self,
            migrate_msg: &Self::MigrateMsg,
        ) -> Result<Option<Vec<TxResponse<Daemon>>>, DaemonError> {
            let mut txs = Vec::with_capacity(2);
            if let Some(tx) = self.upload_if_needed()? {
                txs.push(tx);
            }
            if let Some(tx) = self.migrate_if_needed(migrate_msg)? {
                txs.push(tx);
            }
            if txs.is_empty() { Ok(None) } else { Ok(Some(txs)) }
        }
    }
    impl<T> ConditionalMigrate for T
    where
        T: CwOrchMigrate<Daemon> + CwOrchUpload<Daemon>,
    {}
}
pub mod tx_builder {
    use cosmrs::tx::{ModeInfo, SignMode};
    use cosmrs::{
        proto::cosmos::auth::v1beta1::BaseAccount, tendermint::chain::Id,
        tx::{self, Body, Fee, Msg, Raw, SequenceNumber, SignDoc, SignerInfo},
        Any, Coin,
    };
    use secp256k1::All;
    use super::{sender::Sender, DaemonError};
    const GAS_BUFFER: f64 = 1.3;
    const BUFFER_THRESHOLD: u64 = 200_000;
    const SMALL_GAS_BUFFER: f64 = 1.4;
    /// Struct used to build a raw transaction and broadcast it with a sender.
    pub struct TxBuilder {
        pub(crate) body: Body,
        pub(crate) fee_amount: Option<u128>,
        pub(crate) gas_limit: Option<u64>,
        pub(crate) sequence: Option<SequenceNumber>,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for TxBuilder {
        #[inline]
        fn clone(&self) -> TxBuilder {
            TxBuilder {
                body: ::core::clone::Clone::clone(&self.body),
                fee_amount: ::core::clone::Clone::clone(&self.fee_amount),
                gas_limit: ::core::clone::Clone::clone(&self.gas_limit),
                sequence: ::core::clone::Clone::clone(&self.sequence),
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for TxBuilder {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field4_finish(
                f,
                "TxBuilder",
                "body",
                &self.body,
                "fee_amount",
                &self.fee_amount,
                "gas_limit",
                &self.gas_limit,
                "sequence",
                &&self.sequence,
            )
        }
    }
    impl TxBuilder {
        /// Create a new TxBuilder with a given body.
        pub fn new(body: Body) -> Self {
            Self {
                body,
                fee_amount: None,
                gas_limit: None,
                sequence: None,
            }
        }
        /// Set a fixed fee amount for the tx
        pub fn fee_amount(&mut self, fee_amount: u128) -> &mut Self {
            self.fee_amount = Some(fee_amount);
            self
        }
        /// Set a gas limit for the tx
        pub fn gas_limit(&mut self, gas_limit: u64) -> &mut Self {
            self.gas_limit = Some(gas_limit);
            self
        }
        /// Set a sequence number for the tx
        pub fn sequence(&mut self, sequence: u64) -> &mut Self {
            self.sequence = Some(sequence);
            self
        }
        /// Builds the body of the tx with a given memo and timeout.
        pub fn build_body<T: cosmrs::tx::Msg>(
            msgs: Vec<T>,
            memo: Option<&str>,
            timeout: u64,
        ) -> tx::Body {
            let msgs = msgs
                .into_iter()
                .map(Msg::into_any)
                .collect::<Result<Vec<Any>, _>>()
                .unwrap();
            tx::Body::new(msgs, memo.unwrap_or_default(), timeout as u32)
        }
        pub(crate) fn build_fee(
            amount: impl Into<u128>,
            denom: &str,
            gas_limit: u64,
        ) -> Fee {
            let fee = Coin::new(amount.into(), denom).unwrap();
            Fee::from_amount_and_gas(fee, gas_limit)
        }
        /// Builds the raw tx with a given body and fee and signs it.
        /// Sets the TxBuilder's gas limit to its simulated amount for later use.
        pub async fn build(&mut self, wallet: &Sender<All>) -> Result<Raw, DaemonError> {
            let BaseAccount { account_number, sequence, .. } = wallet
                .base_account()
                .await?;
            let sequence = self.sequence.unwrap_or(sequence);
            let (tx_fee, gas_limit) = if let (Some(fee), Some(gas_limit))
                = (self.fee_amount, self.gas_limit) {
                {
                    let lvl = ::log::Level::Debug;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            format_args!(
                                "Using pre-defined fee and gas limits: {0}, {1}",
                                fee,
                                gas_limit,
                            ),
                            lvl,
                            &(
                                "cw_orch_daemon::tx_builder",
                                "cw_orch_daemon::tx_builder",
                                "cw-orch-daemon/src/tx_builder.rs",
                            ),
                            91u32,
                            ::log::__private_api::Option::None,
                        );
                    }
                };
                (fee, gas_limit)
            } else {
                let sim_gas_used = wallet
                    .calculate_gas(&self.body, sequence, account_number)
                    .await?;
                {
                    let lvl = ::log::Level::Debug;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            format_args!("Simulated gas needed {0:?}", sim_gas_used),
                            lvl,
                            &(
                                "cw_orch_daemon::tx_builder",
                                "cw_orch_daemon::tx_builder",
                                "cw-orch-daemon/src/tx_builder.rs",
                            ),
                            101u32,
                            ::log::__private_api::Option::None,
                        );
                    }
                };
                let gas_expected = if sim_gas_used < BUFFER_THRESHOLD {
                    sim_gas_used as f64 * SMALL_GAS_BUFFER
                } else {
                    sim_gas_used as f64 * GAS_BUFFER
                };
                let fee_amount = gas_expected
                    * (wallet
                        .daemon_state
                        .chain_data
                        .fees
                        .fee_tokens[0]
                        .fixed_min_gas_price + 0.00001);
                {
                    let lvl = ::log::Level::Debug;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            format_args!("Calculated fee needed: {0:?}", fee_amount),
                            lvl,
                            &(
                                "cw_orch_daemon::tx_builder",
                                "cw_orch_daemon::tx_builder",
                                "cw-orch-daemon/src/tx_builder.rs",
                            ),
                            111u32,
                            ::log::__private_api::Option::None,
                        );
                    }
                };
                self.gas_limit = Some(gas_expected as u64);
                (fee_amount as u128, gas_expected as u64)
            };
            let fee = Self::build_fee(
                tx_fee,
                &wallet.daemon_state.chain_data.fees.fee_tokens[0].denom,
                gas_limit,
            );
            {
                let lvl = ::log::Level::Debug;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                    ::log::__private_api::log(
                        format_args!(
                            "submitting tx: \n fee: {0:?}\naccount_nr: {1:?}\nsequence: {2:?}",
                            fee,
                            account_number,
                            sequence,
                        ),
                        lvl,
                        &(
                            "cw_orch_daemon::tx_builder",
                            "cw_orch_daemon::tx_builder",
                            "cw-orch-daemon/src/tx_builder.rs",
                        ),
                        125u32,
                        ::log::__private_api::Option::None,
                    );
                }
            };
            let auth_info = SignerInfo {
                public_key: wallet.private_key.get_signer_public_key(&wallet.secp),
                mode_info: ModeInfo::single(SignMode::Direct),
                sequence,
            }
                .auth_info(fee);
            let sign_doc = SignDoc::new(
                &self.body,
                &auth_info,
                &Id::try_from(wallet.daemon_state.chain_data.chain_id.to_string())?,
                account_number,
            )?;
            wallet.sign(sign_doc).map_err(Into::into)
        }
    }
}
pub use self::{
    builder::*, channel::*, core::*, error::*, state::*, sync::*, traits::*, tx_resp::*,
};
pub use cw_orch_networks::chain_info::*;
pub use cw_orch_networks::networks;
pub use sender::Wallet;
pub use tx_builder::TxBuilder;
pub(crate) mod cosmos_modules {
    pub use cosmrs::proto::{
        cosmos::{
            auth::v1beta1 as auth, authz::v1beta1 as authz, bank::v1beta1 as bank,
            base::{
                abci::v1beta1 as abci, tendermint::v1beta1 as tendermint, v1beta1 as base,
            },
            crisis::v1beta1 as crisis, distribution::v1beta1 as distribution,
            evidence::v1beta1 as evidence, feegrant::v1beta1 as feegrant,
            gov::v1beta1 as gov, mint::v1beta1 as mint, params::v1beta1 as params,
            slashing::v1beta1 as slashing, staking::v1beta1 as staking,
            tx::v1beta1 as tx, vesting::v1beta1 as vesting,
        },
        cosmwasm::wasm::v1 as cosmwasm,
        ibc::{
            applications::transfer::v1 as ibc_transfer,
            core::{
                channel::v1 as ibc_channel, client::v1 as ibc_client,
                connection::v1 as ibc_connection,
            },
        },
        tendermint::abci as tendermint_abci,
    };
}
/// Re-export trait and data required to fetch daemon data from chain-registry
pub use ibc_chain_registry::{
    chain::ChainData as ChainRegistryData, fetchable::Fetchable,
};
