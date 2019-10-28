use parity_codec::{Encode, Decode};
use rstd::prelude::*;


#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq)]
pub enum Symbol {
	XtzBond,
	AtomBond,
}
impl Default for Symbol {
	fn default() -> Symbol {
		Symbol::XtzBond
	}
}

/// Bond token status.
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq)]
pub enum BondTokenStatus {
	/// 
	Normal,
	/// 
	Locked,
}

impl Default for BondTokenStatus {
	fn default() -> BondTokenStatus {
		BondTokenStatus::Normal
	}
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, PartialEq)]
pub struct CustomRedeemData<AccountId, Hash, Balance> {
	// creator of redeem
	pub initiator: AccountId,
	// bond token id
	pub bond_id: Hash,
	// redeem amount
	pub amount: Balance,
	// original chain account id
	pub original_account_id: Vec<u8>,
}
