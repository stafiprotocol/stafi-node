use codec::{Decode, Encode};
use node_primitives::{Balance, BlockNumber};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct ClaimInfo {
    /// token amount
    pub mint_amount: u128,
    /// total reward
    pub total_reward: Balance,
    /// total claimed
    pub total_claimed: Balance,
    /// latest claimed block
    pub latest_claimed_block: BlockNumber,
    /// block when user mint rtoken
    pub mint_block: BlockNumber,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, Default)]
pub struct MintRewardAct<BlockNumber, Balance> {
    pub begin: BlockNumber,
    pub end: BlockNumber,
    pub cycle: u32,
    pub reward_rate: u128,
    pub total_reward: Balance,
    pub left_amount: Balance,
    pub user_limit: Balance,
    pub locked_blocks: u32,
}
