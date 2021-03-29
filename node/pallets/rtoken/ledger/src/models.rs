use sp_std::prelude::*;
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use node_primitives::{RSymbol};

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct LinkChunk {
	/// Total bond amount
	pub bond: u128,
	/// Total unbond amount
    pub unbond: u128,
    /// active
    pub active: u128,
}

impl Default for LinkChunk {
    fn default() -> Self {
        Self {
            bond: 0,
            unbond: 0,
            active: 0,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct BondSnapshot<AccountId> {
    /// rsymbol
    pub symbol: RSymbol,
    /// era
    pub era: u32,
    /// pool
    pub pool: Vec<u8>,
	/// bond amount
	pub bond: u128,
	/// unbond amount
    pub unbond: u128,
    /// active
    pub active: u128,
    /// lastVoter
    pub last_voter: AccountId,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct Unbonding<AccountId> {
    pub who: AccountId,
    pub symbol: RSymbol,
    pub pool: Vec<u8>,
    pub rvalue: u128,
    pub value: u128,
    pub current_era: u32,
    pub unlock_era: u32,
    pub recipient: Vec<u8>,
}


