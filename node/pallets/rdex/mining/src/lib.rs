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

const MODULE_ID: ModuleId = ModuleId(*b"rdx/mine");
const REWARD_FACTOR: u128 = 1_000_000_000_000;
decl_event! {
    pub enum Event<T> where
        AccountId = <T as system::Trait>::AccountId
    {
        /// Deposit: account, symbol, pool index, grade_index, stake index, lp amount
        Deposit(AccountId, RSymbol, u32, u32, u32, u128),
        /// Withdraw: account, symbol, pool index, grade_index, stake index, lp amount,withdraw reward, guard amount
        Withdraw(AccountId, RSymbol, u32, u32, u32, u128, u128, u128),
        /// Withdraw: account, symbol, pool index, grade index, stake index, withdraw reward
        ClaimReward(AccountId, RSymbol, u32, u32, u32, u128),
        /// EmergencyEithdraw: account, symbol, pool index,grade index, stake index, lp amount
        EmergencyWithdraw(AccountId, RSymbol, u32, u32, u32, u128),
        /// AddPool: symbol, pool index, grade index, start block, lp locked block, reward per block, total reward, guard impermanent loss
        AddPool(RSymbol, u32, u32, u32, u32, u128, u128, bool),
        /// RmPool: symbol, pool index, grade index
        RmPool(RSymbol, u32, u32),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        AmountZero,
        StakePoolNotExist,
        SwapPoolNotExist,
        StakeUserNotExist,
        NoGuardAddress,
        EmergencySwitchIsOpen,
        EmergencySwitchIsClose,
        LpBalanceNotEnough,
        StakeLpNotEnough,
        CalPoolDuBlockErr,
        UnLockWillAfterEndErr,
        LpStillLocked,
        GradeIndexOverflow,
        LpBalanceNotEmpty,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RDexMining {
        /// stake pools: (symbol, pool index) => vec[]stake pool info
        pub StakePools get(fn stake_pools): map hasher(blake2_128_concat) (RSymbol, u32) => Option<Vec<StakePool>>;
        /// pool count: (symbol, pool index) => pool count
        pub PoolCount get(fn pool_count): map hasher(blake2_128_concat) RSymbol => u32;
        /// stake users: (symbol, pool index, account, stake index) => stake user info
        pub StakeUsers get(fn stake_users): map hasher(blake2_128_concat) (RSymbol, u32, T::AccountId, u32) => Option<StakeUser<T::AccountId>>;
        /// user stake count: (symbol, pool index, account) => stake count
        pub UserStakeCount get(fn user_stake_count): map hasher(blake2_128_concat) (RSymbol, u32, T::AccountId) => u32;
        /// guard address for impermanent loss
        pub GuardAddress get(fn guard_address): Option<T::AccountId>;
        /// guard blocks: (symbol, pool index) => blocks
        pub GuardLine get(fn guard_line): map hasher(blake2_128_concat) (RSymbol, u32) => u32 = 1_440_000;
        /// guard Reserve: symbol => reserve fis amount
        pub GuardReserve get(fn guard_reserve): map hasher(blake2_128_concat) RSymbol => u128;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// deposit
        #[weight = 10_000_000_000]
        pub fn deposit(origin, symbol: RSymbol, pool_index: u32, grade_index: u32, lp_amount: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut stake_pool_vec = Self::stake_pools((symbol, pool_index)).ok_or(Error::<T>::StakePoolNotExist)?;
            let mut stake_pool = *stake_pool_vec.get(grade_index as usize).ok_or(Error::<T>::GradeIndexOverflow)?;
            let swap_pool = rdex_swap::SwapPools::get(symbol).ok_or(Error::<T>::SwapPoolNotExist)?;
            let now_block = system::Module::<T>::block_number().saturated_into::<u32>();
            let user_stake_count = Self::user_stake_count((symbol, pool_index, &who));
            let pool_du_block = stake_pool.total_reward.checked_div(stake_pool.reward_per_block).ok_or(Error::<T>::CalPoolDuBlockErr)?;

            ensure!(lp_amount > 0, Error::<T>::AmountZero);
            ensure!(!stake_pool.emergency_switch, Error::<T>::EmergencySwitchIsOpen);
            ensure!(T::LpCurrency::free_balance(&who, symbol) >= lp_amount, Error::<T>::LpBalanceNotEnough);
            ensure!(now_block + stake_pool.lp_locked_blocks < stake_pool.start_block.saturating_add(pool_du_block as u32), Error::<T>::UnLockWillAfterEndErr);

            T::LpCurrency::transfer(&who, &Self::account_id(), symbol, lp_amount)?;
            stake_pool = Self::update_pool(symbol, pool_index, grade_index);
            let new_stake_user = StakeUser {
                account: who.clone(),
                lp_amount: lp_amount,
                reward_debt: lp_amount.saturating_mul(stake_pool.reward_per_share).checked_div(REWARD_FACTOR).unwrap_or(0),
                reserved_lp_reward: 0,
                total_fis_value: Self::cal_share_amount(swap_pool.total_unit, lp_amount, swap_pool.fis_balance),
                total_rtoken_value: Self::cal_share_amount(swap_pool.total_unit, lp_amount, swap_pool.rtoken_balance),
                deposit_height: now_block,
                grade_index: grade_index,
                claimed_reward: 0
            };
            let new_stake_count = user_stake_count + 1;
            stake_pool_vec[grade_index as usize] = stake_pool;

            <StakeUsers<T>>::insert((symbol, pool_index, &who, user_stake_count), new_stake_user);
            <StakePools>::insert((symbol, pool_index), stake_pool_vec);
            <UserStakeCount<T>>::insert((symbol, pool_index, &who), new_stake_count);
            Self::deposit_event(RawEvent::Deposit(who, symbol, pool_index, grade_index, user_stake_count, lp_amount));
            Ok(())
        }

        /// withdraw
        #[weight = 20_000_000_000]
        pub fn withdraw(origin, symbol: RSymbol, pool_index: u32, grade_index: u32, stake_index: u32, lp_amount: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut stake_pool_vec = Self::stake_pools((symbol, pool_index)).ok_or(Error::<T>::StakePoolNotExist)?;
            let mut stake_pool = *stake_pool_vec.get(grade_index as usize).ok_or(Error::<T>::GradeIndexOverflow)?;
            let swap_pool = rdex_swap::SwapPools::get(symbol).ok_or(Error::<T>::SwapPoolNotExist)?;
            let mut stake_user = Self::stake_users((symbol, pool_index, &who, stake_index)).ok_or(Error::<T>::StakeUserNotExist)?;
            let now_block = system::Module::<T>::block_number().saturated_into::<u32>();

            ensure!(now_block > stake_pool.lp_locked_blocks.saturating_add(stake_user.deposit_height), Error::<T>::LpStillLocked);
            ensure!(!stake_pool.emergency_switch, Error::<T>::EmergencySwitchIsOpen);
            ensure!(lp_amount > 0, Error::<T>::AmountZero);
            ensure!(stake_user.lp_amount >= lp_amount, Error::<T>::StakeLpNotEnough);
            ensure!(stake_pool.total_stake_lp >= stake_user.lp_amount, Error::<T>::LpBalanceNotEnough);

            stake_pool = Self::update_pool(symbol, pool_index, grade_index);
            stake_pool.total_stake_lp = stake_pool.total_stake_lp.saturating_sub(lp_amount);
            stake_pool_vec[grade_index as usize] = stake_pool;

            let pending_reward = stake_user.lp_amount.saturating_mul(stake_pool.reward_per_share).
                checked_div(REWARD_FACTOR).unwrap_or(0).
                saturating_sub(stake_user.reward_debt);
            let mut withdraw_reward = pending_reward;
            // recheck balance
            let reward_free_balance = T::Currency::free_balance(&Self::account_id()).saturated_into::<u128>();
            if withdraw_reward > reward_free_balance {
                withdraw_reward = reward_free_balance;
            }
            let reserved_lp_total_reward = stake_user.reserved_lp_reward.saturating_add(withdraw_reward);

            let mut guard_amount: u128 = 0;
            if stake_pool.guard_impermanent_loss {
                let guard_address = Self::guard_address().ok_or(Error::<T>::NoGuardAddress)?;
                let guard_line = Self::guard_line((symbol, pool_index));
                let guard_reserve = Self::guard_reserve(symbol);
                let total_deposit_value_in_fis = Self::cal_share_amount(swap_pool.rtoken_balance, swap_pool.fis_balance, stake_user.total_rtoken_value).
                    saturating_add(stake_user.total_fis_value);
                let total_now_value_in_fis = Self::cal_share_amount(swap_pool.total_unit, stake_user.lp_amount, swap_pool.fis_balance).saturating_mul(2);
                let total_now_value_in_fis_with_reward = total_now_value_in_fis.saturating_add(reserved_lp_total_reward);

                if  total_now_value_in_fis_with_reward < total_deposit_value_in_fis {
                    guard_amount = total_deposit_value_in_fis.saturating_sub(total_now_value_in_fis_with_reward);
                    guard_amount = Self::cal_share_amount(stake_user.lp_amount, lp_amount, guard_amount);
                    let du_block = now_block.saturating_sub(stake_user.deposit_height);
                    if du_block <= guard_line {
                        guard_amount = Self::cal_share_amount(guard_line as u128, du_block as u128, guard_amount);
                    }
                    if guard_amount > guard_reserve {
                        guard_amount = guard_reserve;
                    }
                    // recheck guard balance
                    let guard_free_balance = T::Currency::free_balance(&guard_address).saturated_into::<u128>();
                    if guard_amount > guard_free_balance {
                        guard_amount = guard_free_balance;
                    }
                    if guard_amount > 0 {
                        T::Currency::transfer(&guard_address, &who, guard_amount.saturated_into(), KeepAlive)?;
                        <GuardReserve>::insert(symbol, guard_reserve.saturating_sub(guard_amount));
                    }
                }
            }

            let reserved_lp = stake_user.lp_amount.saturating_sub(lp_amount);
            stake_user.total_fis_value = Self::cal_share_amount(stake_user.lp_amount, reserved_lp, stake_user.total_fis_value);
            stake_user.total_rtoken_value = Self::cal_share_amount(stake_user.lp_amount, reserved_lp, stake_user.total_rtoken_value);
            stake_user.reserved_lp_reward = Self::cal_share_amount(stake_user.lp_amount, reserved_lp, reserved_lp_total_reward);

            stake_user.lp_amount = reserved_lp;
            stake_user.reward_debt = stake_user.lp_amount.
                saturating_mul(stake_pool.reward_per_share).
                checked_div(REWARD_FACTOR).unwrap_or(0);
            stake_user.claimed_reward = stake_user.claimed_reward.saturating_add(withdraw_reward);

            if withdraw_reward > 0 {
                T::Currency::transfer(&Self::account_id(), &who, withdraw_reward.saturated_into(), KeepAlive)?;
            }
            T::LpCurrency::transfer(&Self::account_id(), &who, symbol, lp_amount)?;
            <StakeUsers<T>>::insert((symbol, pool_index, &who, stake_index), stake_user);
            <StakePools>::insert((symbol, pool_index), stake_pool_vec);
            Self::deposit_event(RawEvent::Withdraw(who, symbol, pool_index, grade_index, stake_index, lp_amount, withdraw_reward, guard_amount));
            Ok(())
        }

        /// claim reward
        #[weight = 10_000_000_000]
        pub fn claim_reward(origin, symbol: RSymbol, pool_index: u32, grade_index: u32, stake_index: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut stake_pool_vec = Self::stake_pools((symbol, pool_index)).ok_or(Error::<T>::StakePoolNotExist)?;
            let mut stake_pool = *stake_pool_vec.get(grade_index as usize).ok_or(Error::<T>::GradeIndexOverflow)?;
            let mut stake_user = Self::stake_users((symbol, pool_index, &who, stake_index)).ok_or(Error::<T>::StakeUserNotExist)?;

            ensure!(!stake_pool.emergency_switch, Error::<T>::EmergencySwitchIsOpen);

            stake_pool = Self::update_pool(symbol, pool_index, grade_index);
            stake_pool_vec[grade_index as usize] = stake_pool;

            let pending_reward = stake_user.lp_amount.saturating_mul(stake_pool.reward_per_share).
                checked_div(REWARD_FACTOR).unwrap_or(0).
                saturating_sub(stake_user.reward_debt);
            let mut withdraw_reward = pending_reward;
            // recheck balance
            let reward_free_balance = T::Currency::free_balance(&Self::account_id()).saturated_into::<u128>();
            if withdraw_reward > reward_free_balance {
                withdraw_reward = reward_free_balance;
            }
            stake_user.reserved_lp_reward = stake_user.reserved_lp_reward.saturating_add(withdraw_reward);
            stake_user.claimed_reward = stake_user.claimed_reward.saturating_add(withdraw_reward);

            if withdraw_reward > 0 {
                T::Currency::transfer(&Self::account_id(), &who, withdraw_reward.saturated_into(), KeepAlive)?;
            }
            <StakeUsers<T>>::insert((symbol, pool_index, &who, stake_index), stake_user);
            <StakePools>::insert((symbol, pool_index), stake_pool_vec);
            Self::deposit_event(RawEvent::ClaimReward(who, symbol, pool_index, grade_index, stake_index, withdraw_reward));
            Ok(())
        }

         /// emergency withdraw
         #[weight = 10_000_000_000]
         pub fn emergency_withdraw(origin, symbol: RSymbol, pool_index: u32, grade_index: u32, stake_index: u32) -> DispatchResult {
             let who = ensure_signed(origin)?;
             let mut stake_pool_vec = Self::stake_pools((symbol, pool_index)).ok_or(Error::<T>::StakePoolNotExist)?;
             let mut stake_pool = *stake_pool_vec.get(grade_index as usize).ok_or(Error::<T>::GradeIndexOverflow)?;
             let mut stake_user = Self::stake_users((symbol, pool_index, &who, stake_index)).ok_or(Error::<T>::StakeUserNotExist)?;

             ensure!(stake_pool.emergency_switch, Error::<T>::EmergencySwitchIsClose);
             ensure!(stake_user.lp_amount > 0, Error::<T>::AmountZero);

             let lp_amount = stake_user.lp_amount;
             stake_pool.total_stake_lp = stake_pool.total_stake_lp.saturating_sub(lp_amount);
             stake_pool_vec[grade_index as usize] = stake_pool;

             stake_user.lp_amount = 0;
             stake_user.reward_debt = 0;
             stake_user.total_fis_value = 0;
             stake_user.total_rtoken_value = 0;
             stake_user.reserved_lp_reward = 0;

             T::LpCurrency::transfer(&Self::account_id(), &who, symbol, lp_amount)?;
             <StakeUsers<T>>::insert((symbol, pool_index, &who, stake_index), stake_user);
             <StakePools>::insert((symbol, pool_index), stake_pool_vec);
             Self::deposit_event(RawEvent::EmergencyWithdraw(who, symbol, pool_index, grade_index, stake_index, lp_amount));
             Ok(())
         }

        /// create pool
        #[weight = 10_000]
        pub fn add_pool(origin, symbol: RSymbol, pool_index: u32, start_block: u32, lp_locked_blocks: u32, reward_per_block: u128, total_reward: u128, guard_impermanent_loss: bool) -> DispatchResult {
            ensure_root(origin.clone())?;
            let mut stake_pool_vec = Self::stake_pools((symbol, pool_index)).ok_or(Error::<T>::StakePoolNotExist)?;

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
                guard_impermanent_loss: guard_impermanent_loss,
            };
            stake_pool_vec.push(stake_pool);
            let grade_index = stake_pool_vec.len() as u32 - 1;
            <StakePools>::insert((symbol, pool_index), stake_pool_vec);
            Self::deposit_event(RawEvent::AddPool(symbol, pool_index, grade_index, start_block, lp_locked_blocks, reward_per_block, total_reward, guard_impermanent_loss));
            Ok(())
        }

        /// remove pool
        #[weight = 10_000]
        pub fn rm_pool(origin, symbol: RSymbol, pool_index: u32, grade_index: u32) -> DispatchResult {
            ensure_root(origin.clone())?;
            let mut stake_pool_vec = Self::stake_pools((symbol, pool_index)).ok_or(Error::<T>::StakePoolNotExist)?;
            let stake_pool = *stake_pool_vec.get(grade_index as usize).ok_or(Error::<T>::GradeIndexOverflow)?;
            ensure!(stake_pool.total_stake_lp == 0, Error::<T>::LpBalanceNotEmpty);

            stake_pool_vec.remove(grade_index as usize);
            <StakePools>::insert((symbol, pool_index), stake_pool_vec);
            Self::deposit_event(RawEvent::RmPool(symbol, pool_index, grade_index));
            Ok(())
        }

        /// create pool
        #[weight = 10_000]
        pub fn increase_pool_index(origin, symbol: RSymbol) -> DispatchResult {
            ensure_root(origin.clone())?;
            let pool_count = Self::pool_count(symbol);

            <StakePools>::insert((symbol, pool_count), Vec::<StakePool>::new());
            <PoolCount>::insert(symbol, pool_count + 1);

            Ok(())
        }

        /// emergency switch
        #[weight = 10_000]
        pub fn emergency_switch(origin, symbol: RSymbol, pool_index: u32, grade_index: u32) -> DispatchResult {
            ensure_root(origin.clone())?;

            let mut stake_pool_vec = Self::stake_pools((symbol, pool_index)).ok_or(Error::<T>::StakePoolNotExist)?;
            let mut stake_pool = *stake_pool_vec.get(grade_index as usize).ok_or(Error::<T>::GradeIndexOverflow)?;

            stake_pool.emergency_switch = !stake_pool.emergency_switch;
            stake_pool_vec[grade_index as usize] = stake_pool;

            <StakePools>::insert((symbol, pool_index), stake_pool_vec);

            Ok(())
        }

        /// set fund address
        #[weight = 100_000]
        fn set_guard_address(origin, address: T::AccountId) -> DispatchResult {
            ensure_root(origin)?;
            <GuardAddress<T>>::put(address);
            Ok(())
        }

        /// set guard line
        #[weight = 100_000]
        fn set_guard_line(origin, symbol: RSymbol, pool_index: u32, line: u32) -> DispatchResult {
            ensure_root(origin)?;
            <GuardLine>::insert((symbol, pool_index), line);
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
    pub fn update_pool(symbol: RSymbol, index: u32, grade_index: u32) -> StakePool {
        let stake_pool_vec = Self::stake_pools((symbol, index)).unwrap();
        let mut stake_pool = *stake_pool_vec.get(grade_index as usize).unwrap();
        let current_block_num = system::Module::<T>::block_number().saturated_into::<u32>();
        if current_block_num <= stake_pool.last_reward_block {
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
