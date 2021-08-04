use sp_std::prelude::*;
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct SwapTransactionInfo<AccountId> {
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
pub struct SwapRate {
    /// lock block number
    pub lock_number: u64,
    /// swap rate ,admin can set
    pub rate: u128,
}
