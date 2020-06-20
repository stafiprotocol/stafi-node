# stafi-node

[Stafi](http://stafi.io) is:
- The First Decentralized Protocol Unlocking Liquidity of Staked Assets

The protocol of Stafi is created by Substrate and adopts Nominated Proof-of-Stake (NPoS), which complete Staking by setting up Staking Contracts in the upper layer to communicate with public chains. The Staking process is immune to Stafi’s contracts, for the latter act as the account book during Staking. Tokens staked through contracts will be written in the contracts and finally be locked-up on the original chain.

## Note

Now we are mainly testing the functions of block generation, transfer, staking, etc. And this is to prepare for the POA(Supports staking, not transfer). We will open the Staking Contracts code later when we are ready. 

## Building

### Build from Source

```bash
git clone https://github.com/stafiprotocol/stafi-node.git
cd stafi-node
./scripts/init.sh
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
