use codec::{Decode, Encode};
use node_primitives::{Balance, BlockNumber};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct ClaimInfo {
    /// rtoken amount
    pub mint_amount: u128,
    /// native token amount
    pub native_token_amount: u128,
    /// total reward fis amount
    pub total_reward: Balance,
    /// total claimed fis amount
    pub total_claimed: Balance,
    /// latest claimed block
    pub latest_claimed_block: BlockNumber,
    /// block when user mint rtoken
    pub mint_block: BlockNumber,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, Default)]
pub struct MintRewardAct<BlockNumber, Balance> {
    /// act begin block
    pub begin: BlockNumber,
    /// act end block
    pub end: BlockNumber,
    /// act cycle, cycle >= 1
    pub cycle: u32,
    /// fis/native_token
    pub reward_rate: u128,
    /// total reward fis amount
    pub total_reward: Balance,
    /// fis left amount
    pub left_amount: Balance,
    /// user limit fis amount
    pub user_limit: Balance,
    /// locked blocks
    pub locked_blocks: u32,
    /// total rtoken amount in this act
    pub total_rtoken_amount: u128,
    /// total native token amount in this act
    pub total_native_token_amount: u128,
}
