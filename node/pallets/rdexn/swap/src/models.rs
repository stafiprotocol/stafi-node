use sp_std::prelude::*;
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use node_primitives::{RSymbol};

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct TransInfo<AccountId> {
    /// account
    pub account: AccountId,
    /// receiver
    pub receiver: Vec<u8>,
    /// value
    pub value: u128,
    /// deal state
    pub is_deal: bool,
}


#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct SwapInfo<AccountId> {
    /// account id
    pub account: AccountId,
    /// rtoken value
    pub in_value: u128,
    /// native token value
    pub out_value: u128,
    /// symbol 
    pub symbol: RSymbol,
    /// rtoken rate
    pub rtoken_rate: u128,
    /// swap rate ,admin can set
    pub swap_rate: u128,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct SwapRate {
    /// lock block number
    pub lock_number: u64,
    /// swap rate ,admin can set
    pub rate: u128,
}
