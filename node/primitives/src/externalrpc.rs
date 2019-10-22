use parity_codec::{Encode, Decode};
use rstd::prelude::*;

pub type TxHashType = Vec<u8>;
pub type BabeIdType = Vec<u8>;

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, PartialEq)]
pub struct VerifiedData {
	// transaction hash
	pub tx_hash: TxHashType,
	// time
	pub timestamp: u64,
	// status
	pub status: i8,
	pub babe_id: BabeIdType,
	pub babe_num: u8,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, PartialEq)]
pub enum VerifyStatus {
	UnVerified = 0,
	Verified = 1,
	Confirmed = 2,
	Rollback = 3,
	NotFound = 4,
	BadRequest = 5,
	Error = 99,
}

impl VerifyStatus {
	pub fn create(num: i8) -> Self {
		match num {
			0 => VerifyStatus::UnVerified,
			1 => VerifyStatus::Verified,
			2 => VerifyStatus::Confirmed,
			3 => VerifyStatus::Rollback,
			4 => VerifyStatus::NotFound,
			5 => VerifyStatus::BadRequest,
			99=> VerifyStatus::Error,
			_ => VerifyStatus::Error,
		}
	}
}