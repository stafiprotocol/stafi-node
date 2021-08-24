use sp_std::prelude::*;
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use node_primitives::{RSymbol, ChainType};

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
    /// bond state
    pub bond_state: PoolBondState,
}

impl<A> BondSnapshot<A> {
    pub fn era_updated(&self) -> bool {
        self.bond_state == PoolBondState::EraUpdated
    }

    pub fn bond_reported(&self) -> bool {
        self.bond_state == PoolBondState::BondReported
    }

    pub fn active_reported(&self) -> bool {
        self.bond_state == PoolBondState::ActiveReported
    }

    pub fn withdraw_reported(&self) -> bool {
        self.bond_state == PoolBondState::WithdrawReported ||
        (self.symbol.chain_type() == ChainType::Tendermint && self.active_reported()) ||
        (self.symbol == RSymbol::RBNB && self.active_reported())
    }

    pub fn update_state(&mut self, new_state: PoolBondState) {
        self.bond_state = new_state
    }

    pub fn continuable(&self) -> bool {
        self.bond_state == PoolBondState::WithdrawSkipped || self.bond_state == PoolBondState::TransferReported
    }
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct Unbonding<AccountId> {
    pub who: AccountId,
    pub value: u128,
    pub recipient: Vec<u8>,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub enum PoolBondState {
    /// era updated
    EraUpdated,
    /// bond reported
    BondReported,
    /// active reported
    ActiveReported,
    /// withdraw skipped
    WithdrawSkipped,
    /// withdraw reported
    WithdrawReported,
    /// transfer reported
    TransferReported,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub enum BondAction {
    /// bond only
    BondOnly,
    /// unbond only
    UnbondOnly,
    /// both bond and unbond
    BothBondUnbond,
    /// either bond and unbond
    EitherBondUnbond,
}


