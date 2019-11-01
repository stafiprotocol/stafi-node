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
pub struct HostData {
	pub host: Vec<u8>,
	pub weight: u8,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, PartialEq)]
pub enum VerifyStatus {
	UnVerified = 0,
	Verified = 1,
	Confirmed = 2,
	Rollback = 3,
	NotFoundTx = 4,
	//BadRequest = 5,
	NotFoundBlock = 5,
	NotResponse = 6,
	Error = 99,
}

impl VerifyStatus {
	pub fn create(num: i8) -> Self {
		match num {
			0 => VerifyStatus::UnVerified,
			1 => VerifyStatus::Verified,
			2 => VerifyStatus::Confirmed,
			3 => VerifyStatus::Rollback,
			4 => VerifyStatus::NotFoundTx,
			5 => VerifyStatus::NotFoundBlock,
			6 => VerifyStatus::NotResponse,
			99=> VerifyStatus::Error,
			_ => VerifyStatus::Error,
		}
	}
}