use parity_codec::{Encode, Decode};
use rstd::prelude::*;


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

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, PartialEq)]
pub struct XtzTransferData<AccountId, Hash> {
	// identifier id
	pub id: Hash,
	// creator of stake
	pub initiator: AccountId,
	// transaction hash
	pub tx_hash: Vec<u8>,
	// block hash
	pub block_hash: Vec<u8>,
}