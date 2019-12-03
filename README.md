# stafi-node

[Stafi](https://stafi.io) is:
- A Decentralize Protocol to Provide the liquidity of Your Staking Assets

STAFI Protocol solves the contradiction between the token liquidity and Mainnet security by issuing ABS tokens, which provides the liquidity of your Staking Assets. ABS token increases the staking rate to a higher level (100%, theoretically) ,and it could be tradable, its security is guided by STAFI Protocol which ensure ABS token is the only collateral that can apply to redeem staking asstes from original staking blockchain ( Tezos, Cosmos, Polkadot, etc,.).


### Initial Setup

```
curl https://sh.rustup.rs -sSf | sh
rustup update stable
rustup update nightly
rustup default nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
cargo install --git https://github.com/alexcrichton/wasm-gc
```

You will also need to install the following packages:

Linux:
```
sudo apt install cmake pkg-config libssl-dev git clang libclang-dev
```

Mac:
```
brew install cmake pkg-config openssl git llvm
```

### Building

```
cargo build --release
```

### Running

Ensure you have a fresh start if updating from another version:
```
./scripts/purge-chain.sh
```
To start up the Stafi node, run:
```
./target/release/stafi --dev
```

## Implemented Modules

### Stafi

* [Stafi-crypto](https://github.com/stafiprotocol/stafi-node/tree/master/node/modules/stafi-crypto)
* [Stafi-externalrpc](https://github.com/stafiprotocol/stafi-node/tree/master/node/modules/stafi-externalrpc)
* [Stafi-multisig](https://github.com/stafiprotocol/stafi-node/tree/master/node/modules/stafi-multisig)
* [Stafi-offchain-worker](https://github.com/stafiprotocol/stafi-node/tree/master/node/modules/stafi-offchain-worker)
* [Stafi-staking](https://github.com/stafiprotocol/stafi-node/tree/master/node/modules/stafi-staking)
* [Tokenbalances](https://github.com/stafiprotocol/stafi-node/tree/master/node/modules/tokenbalances)
