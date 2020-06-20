# stafi-node

[Stafi](http://stafi.io) is:
- The First Decentralized Protocol Unlocking Liquidity of Staked Assets

The protocol of Stafi is created by Substrate and adopts Nominated Proof-of-Stake (NPoS), which complete Staking by setting up Staking Contracts in the upper layer to communicate with public chains. The Staking process is immune to Stafi’s contracts, for the latter act as the account book during Staking. Tokens staked through contracts will be written in the contracts and finally be locked-up on the original chain.

## Note

Now we are mainly testing the functions of block generation, transfer, staking, etc. And this is to prepare for the POA(Supports staking, not transfer). We will open the Staking Contracts code later when we are ready. 

## Building

### Build from Source

Download the source:

```bash
git clone https://github.com/stafiprotocol/stafi-node.git
cd stafi-node
```

Install system dependencies(We recommend ubuntu or macos):

```bash
./scripts/init.sh
```

Build Stafi:

```bash
cargo build --release
```


### Upgrade

```bash
cd stafi-node
git pull
cargo build --release
```

## Running

### Stafi Testnet

```bash
./target/release/stafi --chain=testnet
```

You can see your node on [telemetry] (set a custom name with `--name "my custom name"`).

[telemetry]: https://telemetry.polkadot.io/#list/Stafi%20Testnet%20v0.1.0


### Using Docker
Coming


## Connect to node
You can check node status on [Stafi-apps].

[Stafi-apps]: http://apps.stafi.io/


## Compile error

1. failed to run custom build command for `wabt-sys`
```bash
/home/stafi/.cargo/registry/src/github.com-1ecc6299db9ec823/wabt-sys-0.7.2/wabt/src/option-parser.cc:60:20: error: MAKE_PROJECT_VERSIONwas not declared in this scope
     printf("%s\n", CMAKE_PROJECT_VERSION);
                    ^~~~~~~~~~~~~~~~~~~~~
make[2]: *** [CMakeFiles/wabt.dir/src/option-parser.cc.o] Error 1
make[1]: *** [CMakeFiles/wabt.dir/all] Error 2
make: *** [all] Error 2
```

This may be a problem with your cmake version. You can try this to fix it.

```bash
wget https://github.com/Kitware/CMake/releases/download/v3.17.3/cmake-3.17.3-Linux-x86_64.tar.gz
tar -xzvf cmake-3.17.3-Linux-x86_64.tar.gz

sudo mv cmake-3.17.3-Linux-x86_64 /opt/cmake-3.17.3

sudo ln -sf /opt/cmake-3.17.3/bin/*  /usr/bin/

cmake --version
```

More download versions on [Cmake](https://cmake.org/download/)