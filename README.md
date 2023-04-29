<a href="https://docs.rs/cw-orch/latest" ><img alt="docs.rs" src="https://img.shields.io/docsrs/cw-orch"></a> <a href="https://crates.io/crates/cw-orch" ><img alt="Crates.io" src="https://img.shields.io/crates/d/cw-orch"></a> <a href="https://app.codecov.io/gh/AbstractSDK/cw-orchestrator" ><img alt="Codecov" src="https://img.shields.io/codecov/c/github/AbstractSDK/cw-orchestrator?token=CZZH6DJMRY"></a>

# cw-orchestrator

Multi-environment [CosmWasm](https://cosmwasm.com/) smart-contract scripting library.  Documentation is available at [orchestrator.abstract.money](https://orchestrator.abstract.money).

> [cw-orchestrator](cw-orch/README.md) is inspired by [terra-rust-api](https://github.com/PFC-Validator/terra-rust) and uses [cosmos-rust](https://github.com/cosmos/cosmos-rust) for [protocol buffer](https://developers.google.com/protocol-buffers/docs/overview) gRPC communication.

[cw-plus-orc](cw-plus-orc/README.md) uses cw-orchestrator to provide standard type-safe interfaces for interacting with [cw-plus](https://github.com/CosmWasm/cw-plus) contracts.

cw-orchestrator makes it easier to quickly deploy and iterate on your contracts. It provides a set of macros that allow you to define your contracts in a way that is more similar to how you would write them in Rust. This allows you to use the full power of Rust's type system to ensure that you are not sending invalid messages to your contracts.

## How it works

Interacting with a [CosmWasm](https://cosmwasm.com/) is possible through the contract's endpoints using the appropriate message for that endpoint (`ExecuteMsg`,`InstantiateMsg`, `QueryMsg`, `MigrateMsg`, etc.).

In order to perform actions on the contract you can define an interface to your contract, passing the contract's entry point types into the `contract` macro:

```rust
#[contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct MyContract;
```

The macro implements a set of traits for the struct. These traits contain functions that can then be used to interact with the contract and they prevent us from executing a faulty message on a contract.

As an example you can have a look at the the implementation for a CW20 token [here.](cw-plus-orc/src/contracts/cw20_base.rs)

You can then use this interface to interact with the contract:

```rust
...
let cw20_base: Cw20<Chain> = Cw20::new("my_token", chain);
// instantiate a CW20 token instance
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
// query balance
// notice that this query is generated by a macro and not defined in the object itself!
let balance = cw20_base.balance(sender.to_string())?;
```

You can find [the full cw20 implementation here](cw-orch/examples/cw20.rs). An example of how to interact with a contract in `cw-multi-test` can be found [here](cw-plus-orc/examples/cw-plus-mock.rs) while the same interaction on a real node can be found [here](cw-plus-orc/examples/cw-plus-daemon.rs).

## Advanced features

cw-orchestrator provides two additional macros that can be used to improve the scripting experience.

### ExecuteFns

The `ExecuteFns` macro can be added to the `ExecuteMsg` definition of your contract. It will generate a trait that allows you to call the variants of the message directly without the need to construct the struct yourself.

Example:

```rust
#[cw_serde]
#[derive(ExecuteFns)]
pub enum ExecuteMsg{
    /// Freeze will make a mutable contract immutable, must be called by an admin
    Freeze {},
    /// UpdateAdmins will change the admin set of the contract, must be called by an existing admin,
    /// and only works if the contract is mutable
    UpdateAdmins { admins: Vec<String> },
    /// the `payable` attribute will add a `coins` argument to the generated function
    #[payable]
    Deposit {}
}

#[contract(Empty,ExecuteMsg,Empty,Empty)]
struct Cw1

impl<Chain: CwEnv> Cw1<Chain> {
    pub fn test_macro(&self) {
        self.freeze().unwrap();
        self.update_admins(vec![]).unwrap();
        self.deposit(&[Coin::new(13,"juno")]).unwrap();
    }
}
```

> We recommend shielding the `ExecuteMsgFns` macro behind a feature flag to avoid pulling in `cw-orchestrator` by default.
> The resulting derive would look like this: `#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]`

For nested execute messages you can add an `impl_into` attribute. This expects the message to implement the `Into` trait for the provided type.

### QueryFns

The `QueryFns` derive macro works in the same way as the `ExecuteFns` macro but it also uses the `#[returns(QueryResponse)]` attribute from `cosmwasm-schema` to generate the queries with the correct response types.

# Contributing

We'd really appreciate your help! Please read our [contributing guidelines](docs/src/contributing.md) to get started.

## Documentation

The documentation is generated using [mdbook](https://rust-lang.github.io/mdBook/index.html). Edit the files in the `docs/src` folder and run

```shell
cd docs && mdbook serve --open --port 5000
```

to view the changes.

# Testing
To test the full application you can run the following command:

```shell
cargo test --jobs 1 --all-features
```

# References

Enjoy scripting your smart contracts with ease? Build your contracts with ease by using [Abstract](https://abstract.money).

# Disclaimer

This software is provided as-is without any guarantees.
