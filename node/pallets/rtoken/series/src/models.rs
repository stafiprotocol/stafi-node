use sp_std::prelude::*;
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use node_primitives::{RSymbol, ChainId, Balance};

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

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct UserUnlockChunk {
    pub pool: Vec<u8>,
    pub unlock_era: u32,
    pub value: u128,
    pub recipient: Vec<u8>
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct BondSwap<AccountId, BlockNumber> {
    pub bonder: AccountId,
    pub swap_fee: Balance,
    pub swap_receiver: AccountId,
    pub bridger: AccountId,
    pub recipient: Vec<u8>,
    pub dest_id: ChainId,
    pub expire: BlockNumber,
    pub bond_state: BondState,
    pub refunded: bool,
}

impl<A, B: PartialOrd> BondSwap<A, B> {
    pub fn refundable(&self, now: B) -> bool {
        !self.refunded && self.bond_state == BondState::Fail && self.expire > now
    }
}
