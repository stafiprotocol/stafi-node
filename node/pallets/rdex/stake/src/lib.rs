#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{Currency, ExistenceRequirement::KeepAlive},
};
use sp_std::prelude::*;

use frame_system::{self as system, ensure_root, ensure_signed};
use node_primitives::RSymbol;
use rdex_balances::traits::Currency as LpCurrency;
use sp_runtime::{
    traits::{AccountIdConversion, SaturatedConversion},
    ModuleId,
};
pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// The currency mechanism.
    type Currency: Currency<Self::AccountId>;
    /// currency of lp
    type LpCurrency: LpCurrency<Self::AccountId>;
}

pub mod models;
pub use models::*;
use sp_core::U512;

const MODULE_ID: ModuleId = ModuleId(*b"rdx/stak");
const REWARD_FACTOR: u128 = 1_000_000_000_000;
decl_event! {
    pub enum Event<T> where
        AccountId = <T as system::Trait>::AccountId
    {
        /// stake: account, symbol, lp amount
        Stake(AccountId, RSymbol, u128),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        AmountZero,
        AmountAllZero,
        PoolAlreadyExist,
        StakePoolNotExist,
        SwapPoolNotExist,
        StakeUserNotExist,
        NoGuardPool,
        EmergencySwitchOpen,
        LpBalanceNotEnough,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RDexStake {
        /// stake pools
        pub StakePools get(fn stake_pools): map hasher(blake2_128_concat) (RSymbol, u32) => Option<StakePool>;
        /// pool count
        pub PoolCount get(fn pool_count): map hasher(blake2_128_concat) RSymbol => u32;
        /// stake users
        pub StakeUsers get(fn stake_users): map hasher(blake2_128_concat) (RSymbol, u32, T::AccountId) => Option<StakeUser<T::AccountId>>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// create pool
        #[weight = 10_000]
        pub fn add_pool(origin, symbol: RSymbol, start_block: u32, lp_locked_blocks: u32, reward_per_block: u128, total_reward: u128) -> DispatchResult {
            ensure_root(origin.clone())?;

            let stake_pool = StakePool {
                symbol: symbol,
                emergency_switch: false,
                total_stake_lp: 0,
                start_block: start_block,
                reward_per_block: reward_per_block,
                total_reward: total_reward,
                left_reward: total_reward,
                lp_locked_blocks: lp_locked_blocks,
                last_reward_block: 0,
                reward_per_share: 0,
            };
            let pool_count = Self::pool_count(symbol);

            <StakePools>::insert((symbol, pool_count), stake_pool);
            <PoolCount>::insert(symbol, pool_count + 1);

            Ok(())
        }

        #[weight = 10_000]
        pub fn emergency_switch(origin, symbol: RSymbol, index: u32) -> DispatchResult {
            ensure_root(origin.clone())?;

            let mut stake_pool = Self::stake_pools((symbol, index)).ok_or(Error::<T>::StakePoolNotExist)?;
            stake_pool.emergency_switch = !stake_pool.emergency_switch;

            <StakePools>::insert((symbol, index), stake_pool);

            Ok(())
        }

        /// deposit lp
        #[weight = 10_000_000_000]
        pub fn deposit(origin, symbol: RSymbol, pool_index: u32, lp_amount: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut stake_pool = Self::stake_pools((symbol, pool_index)).ok_or(Error::<T>::StakePoolNotExist)?;
            let swap_pool = rdex_swap::SwapPools::get(symbol).ok_or(Error::<T>::SwapPoolNotExist)?;
            let mut stake_user = Self::stake_users((symbol, pool_index, &who)).unwrap_or(
                StakeUser{
                    account: who.clone(),
                    lp_amount: 0,
                    reward_debt: 0,
                    claimed_reward: 0,
                    current_reward: 0,
                    total_fis_value: 0,
                    total_rtoken_value: 0,});

            ensure!(!stake_pool.emergency_switch, Error::<T>::EmergencySwitchOpen);
            ensure!(T::LpCurrency::free_balance(&who, symbol) >= lp_amount,Error::<T>::LpBalanceNotEnough);

            stake_pool = Self::update_pool(symbol, pool_index);

            // settlement pre reward
            if stake_user.lp_amount > 0 {
                let pending_reward = stake_user.lp_amount.saturating_mul(stake_pool.reward_per_share).
                    checked_div(REWARD_FACTOR).unwrap_or(0).
                    saturating_sub(stake_user.reward_debt);
                stake_user.current_reward = stake_user.current_reward.saturating_add(pending_reward);
            }
            // deal new deposit
            if lp_amount > 0 {
                T::LpCurrency::transfer(&who, &Self::account_id(), symbol, lp_amount)?;
                stake_user.lp_amount = stake_user.lp_amount.saturating_add(lp_amount);
                stake_pool.total_stake_lp = stake_pool.total_stake_lp.saturating_add(lp_amount);
            }
            //update deposit value
            stake_user.total_fis_value = stake_user.total_fis_value.saturating_add(
                Self::cal_share_amount(swap_pool.total_unit, lp_amount, swap_pool.fis_balance));
            stake_user.total_rtoken_value = stake_user.total_rtoken_value.saturating_add(
                    Self::cal_share_amount(swap_pool.total_unit, lp_amount, swap_pool.rtoken_balance));
            //update reward debt
            stake_user.reward_debt = stake_user.lp_amount.
                saturating_mul(stake_pool.reward_per_share).
                checked_div(REWARD_FACTOR).unwrap_or(0);

            <StakeUsers<T>>::insert((symbol, pool_index, &who), stake_user);
            <StakePools>::insert((symbol, pool_index), stake_pool);

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    /// Provides an AccountId for the pallet.
    pub fn account_id() -> T::AccountId {
        MODULE_ID.into_account()
    }

    // must check emergency switch and pool exist before call this method
    pub fn update_pool(symbol: RSymbol, index: u32) -> StakePool {
        let mut stake_pool = Self::stake_pools((symbol, index)).unwrap();
        let current_block_num = system::Module::<T>::block_number().saturated_into::<u32>();
        if stake_pool.last_reward_block <= current_block_num {
            return stake_pool;
        }
        if stake_pool.total_stake_lp == 0 {
            stake_pool.last_reward_block = current_block_num;
            return stake_pool;
        }

        let reward = Self::get_pool_reward(
            stake_pool.last_reward_block,
            current_block_num,
            stake_pool.reward_per_block,
            stake_pool.left_reward,
        );
        if reward > 0 {
            stake_pool.left_reward = stake_pool.left_reward.saturating_sub(reward);
            let add_reward_per_share = reward
                .saturating_mul(REWARD_FACTOR)
                .checked_div(stake_pool.total_stake_lp)
                .unwrap_or(0);
            stake_pool.reward_per_share = stake_pool
                .reward_per_share
                .saturating_add(add_reward_per_share);
        }
        stake_pool.last_reward_block = current_block_num;

        stake_pool
    }

    pub fn get_pool_reward(from: u32, to: u32, reward_per_block: u128, left_reward: u128) -> u128 {
        let duration = to.saturating_sub(from) as u128;
        let reward = duration.saturating_mul(reward_per_block);
        if reward < left_reward {
            reward
        } else {
            left_reward
        }
    }

    pub fn cal_share_amount(pool_unit: u128, share_unit: u128, amount: u128) -> u128 {
        if pool_unit == 0 || share_unit == 0 || amount == 0 {
            return 0;
        }
        let use_pool_unit = U512::from(pool_unit);
        let use_share_unit = U512::from(share_unit);
        let use_amount = U512::from(amount);
        let share_amount = use_amount
            .saturating_mul(use_share_unit)
            .checked_div(use_pool_unit)
            .unwrap_or(U512::zero());

        share_amount.as_u128()
    }
}
