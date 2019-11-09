use parity_codec::{Encode, Decode};
use rstd::prelude::*;
#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};


#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq)]
pub enum ChainType {
	TEZOS,
	COSMOS,
}


#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq)]
pub enum Symbol {
	FisBond,
	XtzBond,
	AtomBond,
}
impl Default for Symbol {
	fn default() -> Symbol {
		Symbol::FisBond
	}
}

/// Stake token type.
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq)]
pub enum StakeTokenType {
	XTZ,
	ATOM
}

impl Default for StakeTokenType {
	fn default() -> StakeTokenType {
		StakeTokenType::XTZ
	}
}