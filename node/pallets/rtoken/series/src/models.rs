use sp_std::prelude::*;
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use node_primitives::{RSymbol};

/// Rtoken Identifier
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum BondReason {
    /// Pass
    Pass,
	/// blockhash
	BlockhashUnmatch,
	/// txhash
    TxhashUnmatch,
    /// from not match
    PubkeyUnmatch,
    /// to not match
    PoolUnmatch,
    /// amount not match
    AmountUnmatch,
}

/// Rtoken Identifier
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum BondState {
    /// dealing
    Dealing,
	/// fail
	Fail,
	/// Success
    Success,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct BondRecord<AccountId> {
    pub bonder: AccountId,
    pub symbol: RSymbol,
    pub pubkey: Vec<u8>,
    pub pool: Vec<u8>,
    pub blockhash: Vec<u8>,
    pub txhash: Vec<u8>,
    pub amount: u128,
}

impl<A: PartialEq> BondRecord<A> {
    pub fn new(bonder: A, symbol: RSymbol, pubkey: Vec<u8>, pool: Vec<u8>, blockhash: Vec<u8>, txhash: Vec<u8>, amount: u128) -> Self {
        Self {
            bonder: bonder,
            symbol: symbol,
            pubkey: pubkey,
            pool: pool,
            blockhash: blockhash,
            txhash: txhash,
            amount: amount,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct BondKey<Hash> {
    pub symbol: RSymbol,
    pub bond_id: Hash,
}

impl<A: PartialEq> BondKey<A> {
    pub fn new(symbol: RSymbol, bond_id: A) -> Self {
        Self {
            symbol: symbol,
            bond_id: bond_id,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct BondUnlockChunk {
	/// Amount of funds to be unlocked.
	pub value: u128,
	/// Era number at which point it'll be unlocked.
	pub era: u32,
}

/// Original tx type
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum OriginalTxType {
    /// transfer
    Transfer,
    /// bond
    Bond,
	/// unbond
	Unbond,
    /// withdraw unbond
    WithdrawUnbond,
	/// claim rewards
    ClaimRewards,
}
