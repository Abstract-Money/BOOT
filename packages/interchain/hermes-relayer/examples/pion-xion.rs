use cosmrs::Any;
use cw_orch::prelude::*;
use cw_orch::{
    daemon::networks::{PION_1, XION_TESTNET_1},
    tokio::runtime::Runtime,
};
use cw_orch_interchain::prelude::*;
use cw_orch_traits::Stargate;
use hermes_relayer::core::HermesRelayer;
use old_ibc_relayer_types::core::ics24_host::identifier::PortId;
use old_ibc_relayer_types::tx_msg::Msg;
use old_ibc_relayer_types::{
    applications::transfer::msgs::transfer::MsgTransfer,
    core::ics04_channel::timeout::TimeoutHeight, timestamp::Timestamp,
};

pub fn main() -> cw_orch::anyhow::Result<()> {
    dotenv::dotenv()?;
    pretty_env_logger::init();
    let rt = Runtime::new()?;

    let relayer = HermesRelayer::new(
        rt.handle(),
        vec![
            (
                PION_1,
                None,
                true,
                "https://rpc-falcron.pion-1.ntrn.tech/".to_string(),
            ),
            (
                XION_TESTNET_1,
                None,
                false,
                "https://xion-testnet-rpc.polkachu.com".to_string(),
            ),
        ],
        vec![(
            (
                XION_TESTNET_1.chain_id.to_string(),
                PION_1.chain_id.to_string(),
            ),
            "connection-63".to_string(),
        )]
        .into_iter()
        .collect(),
    )?;

    let channel = relayer.create_channel(
        "xion-testnet-1",
        "pion-1",
        &PortId::transfer(),
        &PortId::transfer(),
        "ics20-1",
        None,
    )?;

    let xion = relayer.chain("xion-testnet-1")?;
    let pion = relayer.chain("pion-1")?;

    let msg = MsgTransfer {
        source_port: PortId::transfer(),
        source_channel: channel
            .interchain_channel
            .get_chain("xion-testnet-1")?
            .channel
            .unwrap(),
        token: ibc_proto::cosmos::base::v1beta1::Coin {
            denom: "uxion".to_string(),
            amount: "1987".to_string(),
        },

        sender: xion.sender().to_string().parse().unwrap(),
        receiver: pion.sender().to_string().parse().unwrap(),
        timeout_height: TimeoutHeight::Never,
        timeout_timestamp: Timestamp::from_nanoseconds(1_800_000_000_000_000_000)?,
        memo: None,
    };
    let response = xion.commit_any::<u64>(
        vec![Any {
            type_url: msg.type_url(),
            value: msg.to_any().value,
        }],
        None,
    )?;

    relayer.check_ibc("xion-testnet-1", response)?;

    Ok(())
}
