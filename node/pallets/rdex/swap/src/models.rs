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

