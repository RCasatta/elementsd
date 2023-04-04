[![MIT license](https://img.shields.io/github/license/RCasatta/elementsd)](https://github.com/RCasatta/elementsd/blob/master/LICENSE)
[![Crates](https://img.shields.io/crates/v/elementsd.svg)](https://crates.io/crates/elementsd)

# ElementsD

Utility to run a liquidregtest elementsd process, useful in integration testing environment.


```rust
use elementsd::bitcoincore_rpc::RpcApi;
let exe = elementsd::exe_path().expect("elementsd executable must be provided in ELEMENTSD_EXE, or with a feature like '0_21_0', or be in PATH");
let elementsd = elementsd::ElementsD::new(exe).unwrap();
let info = elementsd
    .client()
    .call::<bitcoind::bitcoincore_rpc::jsonrpc::serde_json::Value>("getblockchaininfo", &[])
    .unwrap();
assert_eq!(info.get("chain").unwrap(), "liquidregtest");
```

## Validate pegin

You can also start elementsd with validate pegin capability by connecting an instance of `bitcoind`.
See test [`test_elementsd_with_validatepegin`](https://github.com/RCasatta/elementsd/blob/8e60bc64d09890e18defd860f04b710d08a6f536/src/lib.rs#L162)


See the similar [BitcoinD](https://github.com/RCasatta/bitcoind) for details

## Doc

To build docs:

```sh
RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --features download,doc --open
```

## MSRV

- 1.57.0 with one of the auto download features
- 1.41.1 without features

MSRV 1.41.1 also requires following deps
```sh
cargo update -p which --precise 4.3.0
cargo update -p serde --precise 1.0.152
cargo update -p tempfile --precise 3.3.0
```