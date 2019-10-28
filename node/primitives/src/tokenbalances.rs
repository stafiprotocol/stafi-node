use parity_codec::{Encode, Decode};


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
