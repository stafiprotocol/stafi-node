use parity_codec::{Encode, Decode};
use rstd::prelude::*;

#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};

/// Bond token lock type.
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq)]
pub enum BondTokenLockType {
	/// Redemption
	Redemption,
}

impl Default for BondTokenLockType {
	fn default() -> BondTokenLockType {
		BondTokenLockType::Redemption
	}
}

/// Bond token lock type.
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq)]
pub enum BondTokenLockStatus {
	/// locked
	Locked,
	/// completed
	Completed,
}

impl Default for BondTokenLockStatus {
	fn default() -> BondTokenLockStatus {
		BondTokenLockStatus::Locked
	}
}

#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq)]
pub struct CustomRedeemData<AccountId, Hash> {
	// creator of redeem
	pub initiator: AccountId,
	// lock bond token id
	pub lock_id: Hash,
	// original chain account id
	pub original_account_id: Vec<u8>,
}
