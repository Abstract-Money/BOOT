use cw_orch_core::environment::{EnvironmentInfo, EnvironmentQuerier};

use crate::{
    senders::{querier_trait::QuerierTrait, sender_trait::SenderTrait},
    DaemonBase,
};

impl<SenderGen: SenderTrait, QuerierGen: QuerierTrait> EnvironmentQuerier
    for DaemonBase<SenderGen, QuerierGen>
{
    fn env_info(&self) -> EnvironmentInfo {
        let state = &self.state()?;
        EnvironmentInfo {
            chain_id: state.chain_data.chain_id.to_string(),
            chain_name: state.chain_data.network_info.chain_name.to_string(),
            deployment_id: state.deployment_id.clone(),
        }
    }
}
