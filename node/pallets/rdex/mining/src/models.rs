use codec::{Decode, Encode};
use node_primitives::RSymbol;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, Copy)]
pub struct StakePool {
    /// rToken symbol
    pub symbol: RSymbol,
    /// emergency switch
    pub emergency_switch: bool,
    /// total lp token staked in this pool
    pub total_stake_lp: u128,
    /// reward start block
    pub start_block: u32,
    /// reward per block
    pub reward_per_block: u128,
    /// total reward of this pool
    pub total_reward: u128,
    /// left reward of this pool
    pub left_reward: u128,
    /// lp locked blocks
    pub lp_locked_blocks: u32,
    /// last reward block
    pub last_reward_block: u32,
    /// reward per share
    pub reward_per_share: u128,
    /// guard impermanent loss
    pub guard_impermanent_loss: bool,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct StakeUser<AccountId> {
    /// account
    pub account: AccountId,
    /// How many lp the user has provided
    pub lp_amount: u128,
    /// Reward debt
    pub reward_debt: u128,
    /// The total amount minted by user reserved lp, and already claimed
    pub reserved_lp_reward: u128,
    /// total stake fis value
    pub total_fis_value: u128,
    /// total stake rtoken value
    pub total_rtoken_value: u128,
    /// last deposit height
    pub deposit_height: u32,
    /// stake pool grade index
    pub grade_index: u32,
}
