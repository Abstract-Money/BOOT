mod execute;
mod instantiate;
pub mod msg_type;
mod query;
mod store;

use strum::{EnumDiscriminants, EnumIter, EnumMessage};

use super::CosmosContext;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = CosmosContext)]
pub struct CwCommands {
    #[interactive_clap(subcommand)]
    action: CwAction,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = CosmosContext)]
/// Select cosmwasm action
pub enum CwAction {
    /// Store contract
    #[strum_discriminants(strum(message = "📤 Store"))]
    Store(store::StoreContractCommands),
    /// Instantiate contract
    #[strum_discriminants(strum(message = "🚀 Instantiate"))]
    Instantiate(instantiate::InstantiateContractCommands),
    /// Execute contract method
    #[strum_discriminants(strum(message = "⚡ Execute"))]
    Execute(execute::ExecuteContractCommands),
    /// Query contract
    #[strum_discriminants(strum(message = "🔍 Query"))]
    Query(query::QueryCommands),
}
