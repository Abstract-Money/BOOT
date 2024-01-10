use crate::{queriers::CosmWasm, Daemon, DaemonError};
use cosmwasm_std::ContractInfoResponse;
use cw_orch_core::{
    contract::interface_traits::ContractInstance,
    environment::{
        queriers::bank::{BankQuerier, BankQuerierGetter},
        TxHandler,
    },
};

impl cw_orch_core::environment::WasmCodeQuerier for Daemon {
    /// Returns the checksum of provided code_id
    fn contract_hash(&self, code_id: u64) -> Result<String, DaemonError> {
        let on_chain_hash = self
            .rt_handle
            .block_on(self.query_client::<CosmWasm>().code_id_hash(code_id))?;
        Ok(on_chain_hash)
    }

    /// Returns the code_info structure of the provided contract
    fn contract_info<T: ContractInstance<Self>>(
        &self,
        contract: &T,
    ) -> Result<ContractInfoResponse, DaemonError> {
        let info = self.rt_handle.block_on(
            self.query_client::<CosmWasm>()
                .contract_info(contract.address()?),
        )?;

        let mut contract_info = ContractInfoResponse::default();
        contract_info.code_id = info.code_id;
        contract_info.creator = info.creator;
        contract_info.admin = Some(info.admin);

        Ok(contract_info)
    }
}

impl cw_orch_core::environment::BankQuerier for Daemon {
    fn balance(
        &self,
        address: impl Into<String>,
        denom: Option<String>,
    ) -> Result<Vec<cosmwasm_std::Coin>, <Self as TxHandler>::Error> {
        self.bank_querier().balance(address, denom)
    }

    fn supply_of(
        &self,
        denom: impl Into<String>,
    ) -> Result<cosmwasm_std::Coin, <Self as TxHandler>::Error> {
        self.bank_querier().supply_of(denom)
    }
}
