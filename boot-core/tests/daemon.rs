mod common;

use boot_core::{networks::LOCAL_JUNO, *};
use common::cw20::Cw20Base;
use cw20::{Cw20Coin};
use std::sync::Arc;
use tokio::runtime::Runtime;

#[test]
fn setup() -> anyhow::Result<()> {
    // create the tokio runtime
    let rt = Arc::new(Runtime::new().unwrap());

    let options = DaemonOptionsBuilder::default()
        // or provide `chain_data`
        .network(LOCAL_JUNO)
        // specify a custom deployment ID
        .deployment_id("v0.1.0")
        .build()?;

    // get sender form .env file mnemonic
    let (_sender, chain) = instantiate_daemon_env(&rt, options)?;

    let sender = chain.sender();
    // get the cw20_base contract
    let mut cw20_base = Cw20Base::new(chain.clone());
    cw20_base.upload()?;
    // instantiate an instance of it
    let cw20_init_msg = cw20_base::msg::InstantiateMsg {
        decimals: 6,
        name: "Test Token".to_string(),
        initial_balances: vec![Cw20Coin {
            address: sender.to_string(),
            amount: 1000000u128.into(),
        }],
        marketing: None,
        mint: None,
        symbol: "TEST".to_string(),
    };
    cw20_base.instantiate(&cw20_init_msg, None, None)?;

    // send some tokens
    let cw20_send_msg = cw20_base::msg::ExecuteMsg::Transfer {
        recipient: "juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y".to_string(),
        amount: 100u128.into(),
    };
    cw20_base.execute(&cw20_send_msg, None)?;

    // query the balance of the recipient
    let query_msg = cw20_base::msg::QueryMsg::Balance {
        address: "juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y".to_string(),
    };
    let _balance: cw20::BalanceResponse = cw20_base.query(&query_msg)?;

    //  // query balance after init
    // // notice that this query is generated by the macro and not defined in the object itself!
    // let balance = cw20_base.balance(sender.to_string())?;
    // assert_eq!(balance.balance.u128(), 999900u128.into());

    // // Send with the macro-generated function
    // let transfer_resp = cw20_base.transfer(100u128.into(), "recipient_addr".to_string())?;
    // assert_eq!(
    //     // index the response
    //     transfer_resp.event_attr_value("wasm", "amount")?,
    //     100.to_string()
    // );
    Ok(())
}
