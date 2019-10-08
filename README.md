# stafi-node

[Stafi](https://stafi.io) is:
- A Decentralize Protocol to Provide the liquidity of Your Staking Assets

STAFI Protocol solves the contradiction between the token liquidity and Mainnet security by issuing ABS tokens, which provides the liquidity of your Staking Assets. ABS token increases the staking rate to a higher level (100%, theoretically) ,and it could be tradable, its security is guided by STAFI Protocol which ensure ABS token is the only collateral that can apply to redeem staking asstes from original staking blockchain ( Tezos, Cosmos, Polkadot, etc,.).

## Start
If your device is clean (such as a fresh cloud VM) you can use this script, otherwise, proceed with the *Initial Setup*.
```
./setup.sh
```
To create a keypair, install subkey with `cargo install --force --git https://github.com/paritytech/substrate subkey`. Then run the following:
```
subkey generate
```
To create an ED25519 keypair, run the following:
```
subkey -e generate
```
To create derived keypairs, use the mnemonic generated from a method above and run:
```
subkey inspect "<mnemonic>"//<derive_path>
```
For example:
```
subkey inspect "rare sweet witness roof pig hill exhibit only sentence mail south fan"//stash
```
Then proceed to the *Running* instructions or follow the instructions below for the manual setup.

### Initial Setup

```
curl https://sh.rustup.rs -sSf | sh
rustup update stable
rustup update nightly
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
To start up the Stafi node and connect to the latest testnet, run:
```
./target/release/stafi --chain=stafi --name <INSERT_NAME>
```

If you use the `--key` flag, ensure that either it is a 32-byte hex string or prefixed with `//` like so:
```
./target/release/stafi --chain=stafi --name <INSERT_NAME> --key //testkey
```

## Implemented Modules

### Stafi

* [Multisig](https://github.com/stafiprotocol/stafi-node/tree/master/node/modules/stafi-multisig)
* [Tokenbalances](https://github.com/stafiprotocol/stafi-node/tree/master/node/modules/tokenbalances)

### SRML
* [System](https://github.com/paritytech/substrate/tree/master/srml/system)
* [Aura](https://github.com/paritytech/substrate/tree/master/srml/aura)
* [Timestamp](https://github.com/paritytech/substrate/tree/master/srml/timestamp)
* [Authorship](https://github.com/paritytech/substrate/tree/master/srml/authorship)
* [Indices](https://github.com/paritytech/substrate/tree/master/srml/indices)
* [Balances](https://github.com/paritytech/substrate/tree/master/srml/balances)
* [Staking](https://github.com/paritytech/substrate/tree/master/srml/staking)
* [Session](https://github.com/paritytech/substrate/tree/master/srml/session)
* [Democracy](https://github.com/paritytech/substrate/tree/master/srml/democracy)
* [Council](https://github.com/paritytech/substrate/tree/master/srml/council)
* [Elections](https://github.com/paritytech/substrate/tree/master/srml/elections)
* [FinalityTracker](https://github.com/paritytech/substrate/tree/master/srml/finality-tracker)
* [Grandpa](https://github.com/paritytech/substrate/tree/master/srml/grandpa)
* [Treasury](https://github.com/paritytech/substrate/tree/master/srml/treasury)
* [Contracts](https://github.com/paritytech/substrate/tree/master/srml/contracts)
* [Sudo](https://github.com/paritytech/substrate/tree/master/srml/sudo)
