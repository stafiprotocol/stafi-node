use parity_codec::{Encode, Decode};
use rstd::prelude::*;

pub type TxHashType = Vec<u8>;

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, PartialEq)]
pub struct VerifiedData {
	// transaction hash
	pub tx_hash: Vec<u8>,
	
}