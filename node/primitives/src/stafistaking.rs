use parity_codec::{Encode, Decode};


/// Stake token type.
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq)]
pub enum StakeTokenType {
	/// stafi token
	FIS,
	/// tezos token
	XTZ,
	/// cosmos token
	ATOM
}

impl Default for StakeTokenType {
	fn default() -> StakeTokenType {
		StakeTokenType::FIS
	}
}