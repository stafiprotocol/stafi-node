use codec::{Decode, Encode};
use node_primitives::RSymbol;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct SwapPool {
    /// rToken symbol
    pub symbol: RSymbol,
    /// balance of fis
    pub fis_balance: u128,
    /// balance of rToken
    pub rtoken_balance: u128,
    /// total lp unit
    pub total_unit: u128,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct SwapLiquidityProvider<AccountId> {
    /// account
    pub account: AccountId,
    /// rToken symbol
    pub symbol: RSymbol,
    /// lp unit
    pub unit: u128,
    /// last add block number
    pub last_add_height: u32,
    /// last remove block number
    pub last_remove_height: u32,
    /// total fis add value
    pub fis_add_value: u128,
    /// total rtoken add value
    pub rtoken_add_value: u128,
}
