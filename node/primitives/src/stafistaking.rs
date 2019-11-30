use parity_codec::{Encode, Decode};
use rstd::prelude::*;
#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};
use crate::chain::StakeTokenType;
use sr_primitives::RuntimeDebug;

#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq)]
pub enum XtzStakeStage {
	// Init - Transfer token to multi sig address
	Init,
	// Successful transfer
	Completed,
}

impl Default for XtzStakeStage {
	fn default() -> XtzStakeStage {
		XtzStakeStage::Init
	}
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Default, RuntimeDebug)]
pub struct XtzStakeData<AccountId, Hash, Balance> {
	// stake identifier id
	pub id: Hash,
	// creator of stake
	pub initiator: AccountId,
	// Stage of stake
	pub stage: XtzStakeStage,
	// multi sig address
	pub multi_sig_address: Vec<u8>,
	// Token data of stake
	pub stake_amount: Balance,
	// transaction hash
	pub tx_hash: Vec<u8>,
	// block hash
	pub block_hash: Vec<u8>,
	// xtz account
	pub stake_account: Vec<u8>,
	// xtz sig
	pub sig: Vec<u8>,
}

#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq)]
pub enum AtomStakeStage {
	// Init
	Init,
	// Successful transfer
	TransferSuccess,
	// Active staking stage
	Staking,
	// Completed staking stage
	Completed,
}

impl Default for AtomStakeStage {
	fn default() -> AtomStakeStage {
		AtomStakeStage::Init
	}
}

#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Default)]
pub struct AtomStakeData<AccountId, Hash, Balance> {
	// stake identifier id
	pub id: Hash,
	// creator of stake
	pub initiator: AccountId,
	// Stage of stake
	pub stage: AtomStakeStage,
	// multi sig address
	pub multi_sig_address: Vec<u8>,
	// Token data of stake
	pub stake_amount: Balance,
	// transaction hash
	pub tx_hash: Vec<u8>,
	// block hash
	pub block_hash: Vec<u8>,
	// atom account
	pub stake_account: Vec<u8>,
	// atom sig
	pub sig: Vec<u8>,
}


#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, PartialEq, Default)]
pub struct StakeDropAct<BlockNumber, Balance> {
	pub begin: BlockNumber,
	pub end: BlockNumber,
	pub current_cycle: u32,
	pub token_type: StakeTokenType,
	pub issue_amount: Balance,
}