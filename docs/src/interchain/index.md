# Interchain Capabilities

Because of its asynchronous and decentralized nature, the **I**nter-**B**lockchain **C**ommunication protocol makes developing and debugging applications more difficult than simple blockchain transactions.
Cw-orch simplifies those tasks by providing developers tools, full testing environments and standard interfaces to interact with applications that leverage IBC capabilities.

Here are a few examples of what cw-orchestrator allows:

## Interchain transaction waiting

To simplify the developer experience and provide simple syntax, cw-orchestrator allows developers to await the whole packet IBC lifecycle (*Receive*, *Acknowledge* or *Timeout*) from transactions that create IBC packets.
This is mostly useful when interacting with existing contracts or modules in a script to simplify the execution pipeline.

```rust,ignore
// We order an ICA controller to burn some funds on juno from akash
// Upon submitting this transaction successfully, a packet will be sent from akash to juno to trigger the ICA action
let transaction_response = controller.send_msgs("channel-16", vec![
        CosmosMsg::Bank(cosmwasm_std::BankMsg::Burn {
            amount: vec![cosmwasm_std::coin(100u128, "ujuno")],
        })
    ])?;

// This function won't return before the packet is relayed successfully or timeouts. 
// In case of a timeout, it will error
interchain.await_and_check_packets(
    "akash",
    transaction_response
)?;

// You can safely continue with the rest of your application, the packet has been successfully relayed
```

This namely removes the need for pausing the program and resuming manually or with a timer.
This also allows to automatically get extra information about the relayed packet and assert the IBC-cycle ended successfully.

## Interchain packet following

Using some simple tools, one can also follow the execution of IBC packets through their whole lifetime (*Receive*, *Acknowledge* or *Timeout*). This is a little more advanced, because you need to know and get more details about the packet manually.

This is mostly useful for packet analysis of certain channels, ports or connections.

```rust,ignore
# use cw_orch_interchain::InterchainEnv;
# fn main() -> anyhow::Result<()>{
    # let interchain = cw_orch_interchain::MockInterchainEnv::new(vec![]);    
    # #[allow(unused)]
    let packet_lifetime = interchain.await_single_packet(
        "juno",
        "transfer".parse()?,
        "channel-16".parse()?,
        "akash",
        3u64.into()
    )?;
    # Ok(())
# }
```

## Interchain application testing

Cw-orch allows developers to test their IBC applications and smart-contracts using a common interface. As we know that setting an IBC testing environment is heavy on resources and can be time-consuming, we provide 3 testing environments that will help them streamline their development process:

### [Rust-only](./integrations/mock.md)

The `MockInterchainEnv` object allows developers to test their application without leaving Rust and without compromising on test speed.

Built on top of cw-multi-test, this environment replicates the actual on-chain IBC module (channel creation as well as packet relaying). This allows you to test any IBC application that leverages Smart-Contract or Bank-module IBC packets. It is really powerful and **doesn't** rely on **ANY** external tools to work. No node setup, no relayer setup, no cluster setup, everything runs inside your crate. Visit the dedicated [Mock Interchain Env](./integrations/mock.md) page for more details and code snippets.

### [Cosmos SDK Node Testing](./integrations/daemon.md#for-testing)

The `Starship` object allows developers to test their application against the actual binaries of running chains. If you want to run your application with specific logic, custom messages or modules, this is the preferred way. This is the IBC version of the local chains that you can run locally. It can also spin up relayers and set up connections between your local chains automatically.

Visit the dedicated [Starship](./integrations/daemon.md#for-testing) page for more details and code snippets.

### [Cosmos SDK Node Scripting](./integrations/daemon.md#for-scripting)

The `DaemonInterchain` object allows developers to script, deploy and manage their application on running chains with attention to IBC functionalities. This enhances the developer experience with more tooling, more useful logging. This is the all-in-one toolbox cor the cosmwasm IBC developer.

Visit the dedicated [Daemon Interchain](./integrations/daemon.md#for-scripting) page for more details and code snippets.
