// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchResult},
    ensure,
    traits::{Currency, Get, ExistenceRequirement::{AllowDeath, KeepAlive}},
};
use frame_system::{
    self as system, ensure_signed, ensure_root, ensure_none,
    offchain::{SendTransactionTypes, SubmitTransaction},
};
use sp_runtime::{
    ModuleId, Perbill,
    traits::{
        Convert, Zero, AccountIdConversion, CheckedAdd, CheckedSub, SaturatedConversion, Saturating, StaticLookup
    },
    transaction_validity::{
		TransactionValidity, ValidTransaction, InvalidTransaction,
		TransactionSource, TransactionPriority,
	},
};
use pallet_staking::{
    self as staking, MAX_NOMINATIONS, Nominations,
    RewardDestination, StakingLedger, EraIndex, UnlockChunk,
};
use pallet_session as session;
use rtoken_balances::{traits::{Currency as RCurrency}};
use node_primitives::{RSymbol};

const SYMBOL: RSymbol = RSymbol::RFIS;
const MAX_ONBOARD_VALIDATORS: usize = 300;
const DEFAULT_LONGEVITY: u64 = 600;

pub(crate) const LOG_TARGET: &'static str = "rfis";

// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		frame_support::debug::$level!(
			target: crate::LOG_TARGET,
			$patter $(, $values)*
		)
	};
}

pub type BalanceOf<T> = staking::BalanceOf<T>;

pub trait Trait: system::Trait + staking::Trait + SendTransactionTypes<Call<Self>> + session::Trait + rtoken_rate::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// currency of rtoken
    type RCurrency: RCurrency<Self::AccountId>;

    /// A configuration for base priority of unsigned transactions.
    type UnsignedPriority: Get<TransactionPriority>;
}

decl_event! {
    pub enum Event<T> where
        Balance = BalanceOf<T>,
        <T as frame_system::Trait>::AccountId
    {   
        /// NewPool
        NewPool(Vec<u8>, AccountId),
        /// Commission has been updated.
        CommissionUpdated(Perbill, Perbill),
        /// max validator commission updated
        MaxValidatorCommissionUpdated(Perbill, Perbill),
        /// Pool balance limit has been updated
        PoolBalanceLimitUpdated(Balance, Balance),
        /// liquidity stake record
        LiquidityBond(AccountId, AccountId, Balance, u128),
        /// liquidity unbond record
        LiquidityUnBond(AccountId, AccountId, u128, u128, Balance),
        /// liquidity withdraw unbond
        LiquidityWithdrawUnBond(AccountId, AccountId, Balance),
        /// validator onboard
        ValidatorOnboard(AccountId, AccountId),
        /// validator deregistered
        ValidatorOffboard(AccountId, AccountId),
        /// total bonded before payout
        TotalBondedBeforePayout(EraIndex, Balance),
        /// total bonded after payout
        TotalBondedAfterPayout(EraIndex, Balance),
        /// Nomination Toggle
        NominateSwitchToggle(bool),
        /// MinNominationNumSet
        MinNominationNumSet(u8),
        /// MaxNominationNumSet
        MaxNominationNumSet(u8),
        /// Nomination Updated for a pool
        NominationUpdated(EraIndex, Vec<AccountId>, AccountId),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// length not eight
        ModuleIDLengthNotEight,
        /// module id repeated
        ModuleIDRepeated,
        /// pool not found
        PoolNotFound,
        /// Got an overflow after adding
        Overflow,
        /// pool not bonded
        PoolUnbond,
        /// Pool limit reached
        PoolLimitReached,
        /// liquidity bond Zero
        LiquidityBondZero,
        /// liquidity unbond Zero
        LiquidityUnbondZero,
        /// insufficient balance
        InsufficientBalance,
        /// register validator limit reached
        ValidatorLimitReached,
        /// no associated validatorId
        NoAssociatedValidatorId,
        /// already onboard
        AlreadyOnboard,
        /// not onboard
        NotOnboard,
        /// no session key
        NoSessionKey,
        /// get curreent era err
        NoCurrentEra,
        /// not current era
        NotCurrentEra,
        /// Total_bonded_before not submitted
        TotalBondedBeforeNotSubmitted,
        /// Has No Unbonding
        HasNoUnbonding,
        /// Nominate Switch Closed
        NominateSwitchClosed,
        /// Pool Already Unlocked
        PoolAlreadyUnlocked,
        /// era rate not updated
        EraRateNotUpdated,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RFis {
        Pools get(fn pools): Vec<T::AccountId>;
        pub OnboardValidators get(fn onboard_validators): Vec<T::AccountId>;
        PoolBalanceLimit get(fn pool_balance_limit): BalanceOf<T>;
        /// Recipient account for fees
        Receiver get(fn receiver): Option<T::AccountId>;
        /// commission of staking fis rewards
        Commission get(fn commission): Perbill = Perbill::from_percent(10);
        /// max validator commission
        MaxValidatorCommission get(fn max_validator_commission): Perbill = Perbill::from_percent(10);
        /// switch of nomination
        NominateSwitch get(fn nominate_switch): bool = false;
        /// min nomination
        MinNominationNum get(fn min_nomination_num): u8 = 3;
        /// max nomination
        MaxNominationNum get(fn max_nomination_num): u8 = 10;

        TotalBondedBeforePayout get(fn total_bonded_before_payout): map hasher(blake2_128_concat) EraIndex => Option<BalanceOf<T>>;
        TotalBondedAfterPayout get(fn total_bonded_after_payout): map hasher(blake2_128_concat) EraIndex => Option<BalanceOf<T>>;

        /// Paidouts: (era, validator) => bool
        Paidouts get(fn paidouts):
            double_map hasher(blake2_128_concat) EraIndex, hasher(twox_64_concat) T::AccountId => Option<bool>;
        /// Unbonding: (origin, pool) => [UnlockChunks]
        pub Unbonding get(fn unbonding): double_map hasher(twox_64_concat) T::AccountId, hasher(twox_64_concat) T::AccountId => Option<Vec<UnlockChunk<BalanceOf<T>>>>;
        /// Nominated: (era, pool) => targets
        Nominated get(fn nominated):
            double_map hasher(blake2_128_concat) EraIndex, hasher(twox_64_concat) T::AccountId => Option<Vec<T::AccountId>>;
        /// unlocked_pools
        UnlockedPools get(fn unlocked_pools): map hasher(blake2_128_concat) EraIndex => Option<Vec<T::AccountId>>;

        /// Unbond commission
        UnbondCommission get(fn unbond_commission): Perbill = Perbill::from_parts(2000000);
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// set up rate
        fn on_finalize() {
            let op_active = staking::ActiveEra::get();
            if op_active.is_none() {
                debug::info!("active era is none");
                return;
            }
            let era = op_active.unwrap().index;

            if rtoken_rate::EraRate::get(SYMBOL, era).is_some() {
                return;
            }

            let op_before = Self::total_bonded_before_payout(era);
            let op_after = Self::total_bonded_after_payout(era);
            if op_before.is_none() || op_after.is_none() {
                debug::info!("bonded data is none");
                return;
            }
            let before = op_before.unwrap();
            let after = op_after.unwrap();
            let op_receiver = Self::receiver();
            if after > before && op_receiver.is_some() {
                let fee = (Self::commission() * (after - before)).saturated_into::<u128>();
                let rfis = rtoken_rate::Module::<T>::token_to_rtoken(SYMBOL, fee);
                let receiver = op_receiver.unwrap();
                if let Err(e) = T::RCurrency::mint(&receiver, SYMBOL, rfis) {
                    debug::error!("rfis commission err: {:?}", e);
                }
            }

            let balance = after.saturated_into::<u128>();
            let rbalance = T::RCurrency::total_issuance(SYMBOL);
            let rate =  rtoken_rate::Module::<T>::set_rate(SYMBOL, balance, rbalance);
            rtoken_rate::EraRate::insert(SYMBOL, era, rate);
        }

        /// add new pool
        #[weight = 100_000_000]
        pub fn add_new_pool(origin, module_id: Vec<u8>) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(module_id.len() == 8, Error::<T>::ModuleIDLengthNotEight);
            let mut r = [0u8; 8];
            r.copy_from_slice(&module_id);
            let pool = ModuleId(r).into_account();
            let mut pools = Self::pools();
            let location = pools.binary_search(&pool).err().ok_or(Error::<T>::ModuleIDRepeated)?;
            pools.insert(location, pool.clone());
            <Pools<T>>::put(pools);
            
            Self::deposit_event(RawEvent::NewPool(module_id, pool));
            Ok(())
        }

        /// bond for a pool
        #[weight = 100_000_000]
        pub fn bond_for_pool(origin, pool: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            ensure_root(origin)?;
            let stash = T::Lookup::lookup(pool)?;
            let controller = stash.clone();
            let pools = Self::pools();
            pools.binary_search(&stash).ok().ok_or(Error::<T>::PoolNotFound)?;

            if staking::Bonded::<T>::contains_key(&stash) {
                Err(staking::Error::<T>::AlreadyBonded)?
            }

            if staking::Ledger::<T>::contains_key(&stash) {
                Err(staking::Error::<T>::AlreadyPaired)?
            }

            staking::Bonded::<T>::insert(&stash, &stash);
            staking::Payee::<T>::insert(&stash, RewardDestination::Staked);

            system::Module::<T>::inc_ref(&stash);

            let value = Zero::zero();
			let item = StakingLedger {
				stash,
				total: value,
				active: value,
				unlocking: vec![],
				claimed_rewards: vec![],
            };

            staking::Module::<T>::update_ledger(&controller, &item);
            Ok(())
        }

        /// turn on/off nominate switch
        #[weight = 10_000]
        fn toggle_nominate_switch(origin) -> DispatchResult {
            ensure_root(origin)?;
            let state = Self::nominate_switch();
            NominateSwitch::put(!state);
            Self::deposit_event(RawEvent::NominateSwitchToggle(!state));
			Ok(())
        }

        /// set MinNominationNum
        #[weight = 10_000]
        fn set_min_nomination_num(origin, new_num: u8) -> DispatchResult {
            ensure_root(origin)?;
            let max = Self::max_nomination_num();
            ensure!(new_num > 0 && new_num <= max, "min_nomination_num should in (0, max_nomination_num]");
            MinNominationNum::put(new_num);
            Self::deposit_event(RawEvent::MinNominationNumSet(new_num));
            Ok(())
        }

        /// set MaxNominationNum
        #[weight = 10_000]
        fn set_max_nomination_num(origin, new_num: u8) -> DispatchResult {
            ensure_root(origin)?;
            let min = Self::min_nomination_num();
            ensure!(new_num >= min && usize::from(new_num) <= MAX_NOMINATIONS, "max_nomination_num should in [min_nomination_num, MAX_NOMINATIONS]");
            MaxNominationNum::put(new_num);
            Self::deposit_event(RawEvent::MaxNominationNumSet(new_num));
            Ok(())
        }

        /// Update commission
		#[weight = 10_000]
		fn set_commission(origin, new_part: u32) -> DispatchResult {
            ensure_root(origin)?;
            let old_commission = Self::commission();
            let new_commission = Perbill::from_parts(new_part);
			Commission::put(new_commission);

			Self::deposit_event(RawEvent::CommissionUpdated(old_commission, new_commission));
			Ok(())
        }

        /// set max validator commission
		#[weight = 10_000]
		fn set_max_validator_commission(origin, new_part: u32) -> DispatchResult {
            ensure_root(origin)?;
            let old_commission = Self::max_validator_commission();
            let new_commission = Perbill::from_parts(new_part);
			MaxValidatorCommission::put(new_commission);

			Self::deposit_event(RawEvent::MaxValidatorCommissionUpdated(old_commission, new_commission));
			Ok(())
        }

        /// Update pool balance limit
        #[weight = 10_000]
        fn set_balance_limit(origin, new_limit: BalanceOf<T>) -> DispatchResult {
            ensure_root(origin)?;
            let old_limit = Self::pool_balance_limit();
            <PoolBalanceLimit<T>>::put(new_limit);

			Self::deposit_event(RawEvent::PoolBalanceLimitUpdated(old_limit, new_limit));
			Ok(())
        }

        /// set commission
        #[weight = 10_000]
        pub fn set_receiver(origin, new_receiver: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            ensure_root(origin)?;
            let dest = T::Lookup::lookup(new_receiver)?;
            <Receiver<T>>::put(dest);
            Ok(())
        }

        /// set unbond commission
        #[weight = 10_000]
        pub fn set_unbond_commission(origin, new_part: u32) -> DispatchResult {
            ensure_root(origin)?;
            let new_commission = Perbill::from_parts(new_part);
            UnbondCommission::put(new_commission);

            Ok(())
        }

        fn offchain_worker(block: T::BlockNumber) {
            if !sp_io::offchain::is_validator() {
                return;
            }

            let op_active = staking::ActiveEra::get();
            if op_active.is_none() {
                debug::info!("active era is none");
                return;
            }
            let era = op_active.unwrap().index;

            let unlocked_pools = Self::unlocked_pools(&era).unwrap_or(vec![]);
            if unlocked_pools.len() < Self::bonded_pools().len() {
                for p in Self::bonded_pools() {
                    if !unlocked_pools.contains(&p) && staking::Ledger::<T>::get(&p).unwrap().unlocking.iter().any(|chunk| chunk.era <= era) {
                        let call = Call::submit_unlocks(era, p).into();
                        if let Err(e) = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call) {
                            debug::info!("failed to submit unlocks: {:?}", e);
                        } else {
                            // wait for next block
                            return;
                        }
                    }
                }
            }

            if Self::total_bonded_before_payout(&era).is_none() {
                let before = Self::total_bonded();
                let call = Call::submit_total_bonded_before(era, before).into();
                if let Err(e) = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call) {
                    debug::info!("failed to submit total bonded before: {:?}", e);
                }
                return;
            }

            let last_era = era.saturating_sub(1);
            // ensure one block won't deal more than one pool
            let mut wait_next_block = false;
            if Self::total_bonded_after_payout(era).is_none() {
                for p in Self::bonded_pools() {
                    let op_nominations = staking::Nominators::<T>::get(&p);
                    if op_nominations.is_none() {
                        debug::info!("{:?} has none nominations", &p);
                        continue;
                    }
                    let nominations = op_nominations.unwrap();
                    if nominations.targets.is_empty() {
                        debug::info!("nominations targets of {:?} is empty", &p);
                        continue;
                    }
    
                    for t in nominations.targets {
                        if Self::paidouts(last_era, &t).is_none() && Self::is_validator(last_era, &t) {
                            wait_next_block = true;
                            let call = Call::submit_paidouts(last_era, p.clone(), t).into();
                            if let Err(e) = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call) {
                                debug::info!("failed to submit paidouts: {:?}", e);
                            }
                        }
                    }
                    if wait_next_block {
                        return;
                    }
                }
            }

            if Self::total_bonded_after_payout(era).is_none() {
                let after = Self::total_bonded();
                let call = Call::submit_total_bonded_after(era, after).into();
                if let Err(e) = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call) {
                    debug::info!("failed to submit total bonded after: {:?}", e);
                }
                return;
            }

            if !Self::nominate_switch() {
                return;
            }

            let mut pools: Vec<T::AccountId> = Self::bonded_pools().into_iter().filter(|p| Self::nominated(&era, &p).is_none()).collect();
            if pools.is_empty() {
                return;
            }

            pools.sort_by(|a, b| Self::bonded_of(&b).cmp(&Self::bonded_of(&a)));
            let mut validators = Self::nominatable_validators(era);
            let min = Self::min_nomination_num();
            let max = Self::max_nomination_num();
            for p in pools {
                if validators.len() < min.into() {
                    let call = Call::submit_nomination(era, p, vec![]).into();
                    if let Err(e) = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call) {
                        debug::error!("failed to submit nomination: {:?}", e);
                    }
                    continue
                }

                let mut targets: Vec<T::AccountId> = vec![];
                if let Some(nominated) = staking::Nominators::<T>::get(&p) {
                    for t in nominated.targets {
                        if let Ok(i) = validators.binary_search(&t) {
                            targets.push(t.clone());
                            validators.remove(i);
                        }
                    }
                }

                while targets.len() < max.into() && !validators.is_empty() {
                    let t = validators.pop().unwrap();
                    targets.push(t.clone());
                }

                let call = Call::submit_nomination(era, p, targets).into();
                if let Err(e) = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call) {
                    debug::error!("failed to submit nomination: {:?}", e);
                }
            }
        }

        /// unlock
        #[weight = 10_000]
        pub fn submit_unlocks(origin, era: EraIndex, pool: T::AccountId) -> DispatchResult {
            ensure_none(origin)?;
            let current_era = staking::CurrentEra::get().ok_or(Error::<T>::NoCurrentEra)?;
            ensure!(era == current_era, Error::<T>::NotCurrentEra);
            let mut unlocked_pools = Self::unlocked_pools(&era).unwrap_or(vec![]);
            ensure!(!unlocked_pools.contains(&pool), Error::<T>::PoolAlreadyUnlocked);
            let mut ledger = staking::Ledger::<T>::get(&pool).ok_or(staking::Error::<T>::NotController)?;
            ledger = ledger.consolidate_unlocked(era);
            staking::Module::<T>::update_ledger(&pool, &ledger);
            unlocked_pools.push(pool);
            <UnlockedPools<T>>::insert(era, unlocked_pools);

            Ok(())
        }

        /// total bonded before
        #[weight = 10_000]
        pub fn submit_total_bonded_before(origin, era: EraIndex, total: BalanceOf<T>) -> DispatchResult {
            ensure_none(origin)?;
            let current_era = staking::CurrentEra::get().ok_or(Error::<T>::NoCurrentEra)?;
            ensure!(era == current_era, Error::<T>::NotCurrentEra);
            <TotalBondedBeforePayout<T>>::insert(era, total);

            Self::deposit_event(RawEvent::TotalBondedBeforePayout(era, total));
            Ok(())
        }

        /// pay out result
        #[weight = 10_000]
        pub fn submit_paidouts(origin, era: EraIndex, _pool: T::AccountId, validator: T::AccountId) -> DispatchResult {
            ensure_none(origin)?;
            let result = staking::Module::<T>::do_payout_stakers(validator.clone(), era).is_ok();
            <Paidouts<T>>::insert(era, validator, result);
            Ok(())
        }

        /// total bonded after
        #[weight = 10_000]
        pub fn submit_total_bonded_after(origin, era: EraIndex, total: BalanceOf<T>) -> DispatchResult {
            ensure_none(origin)?;
            let current_era = staking::CurrentEra::get().ok_or(Error::<T>::NoCurrentEra)?;
            ensure!(era == current_era, Error::<T>::NotCurrentEra);
            ensure!(Self::total_bonded_before_payout(era).is_some(), Error::<T>::TotalBondedBeforeNotSubmitted);
            <TotalBondedAfterPayout<T>>::insert(era, total);

            Self::deposit_event(RawEvent::TotalBondedAfterPayout(era, total));
            Ok(())
        }

        /// submit new nomination
        #[weight = 100_000_000]
        fn submit_nomination(origin, era: EraIndex, pool: T::AccountId, targets: Vec<T::AccountId>) -> DispatchResult {
            ensure_none(origin)?;
            ensure!(staking::EraElectionStatus::<T>::get().is_closed(), staking::Error::<T>::CallNotAllowed);
            let current_era = staking::CurrentEra::get().ok_or(Error::<T>::NoCurrentEra)?;
            ensure!(era == current_era, Error::<T>::NotCurrentEra);
            let ledger = staking::Ledger::<T>::get(&pool).ok_or(staking::Error::<T>::NotController)?;
			let stash = &ledger.stash;
            Self::update_nominations(current_era, &targets, stash);
            <Nominated<T>>::insert(era, pool, targets.to_vec());

            Self::deposit_event(RawEvent::NominationUpdated(current_era, targets, stash.clone()));
            Ok(())
        }

        /// onboard as an validator which may be nominated by the pot
        #[weight = 100_000_000]
        pub fn onboard(origin) -> DispatchResult {
            let controller = ensure_signed(origin)?;
            let ledger = staking::Ledger::<T>::get(&controller).ok_or(staking::Error::<T>::NotController)?;
            let stash = &ledger.stash;
            let validator_id = <T as session::Trait>::ValidatorIdOf::convert(controller.clone()).ok_or(Error::<T>::NoAssociatedValidatorId)?;
            session::Module::<T>::load_keys(&validator_id).ok_or(Error::<T>::NoSessionKey)?;
            let mut onboards = <OnboardValidators<T>>::get();
            ensure!(onboards.len() <= MAX_ONBOARD_VALIDATORS, Error::<T>::ValidatorLimitReached);
            let location = onboards.binary_search(&stash).err().ok_or(Error::<T>::AlreadyOnboard)?;
            onboards.insert(location, stash.clone());
			<OnboardValidators<T>>::put(onboards);

            Self::deposit_event(RawEvent::ValidatorOnboard(controller, stash.clone()));
            Ok(())
        }

        /// offboard
        #[weight = 100_000_000]
        pub fn offboard(origin) -> DispatchResult {
            let controller = ensure_signed(origin)?;
            let ledger = staking::Ledger::<T>::get(&controller).ok_or(staking::Error::<T>::NotController)?;
            let stash = &ledger.stash;
            let mut onboards = <OnboardValidators<T>>::get();
            let location = onboards.binary_search(&stash).ok().ok_or(Error::<T>::NotOnboard)?;
            onboards.remove(location);
            <OnboardValidators<T>>::put(onboards);
            
            Self::deposit_event(RawEvent::ValidatorOffboard(controller, stash.clone()));
            Ok(())
        }

        /// liquidity bond fis to get rfis
        #[weight = 100_000_000]
        pub fn liquidity_bond(origin, pool: <T::Lookup as StaticLookup>::Source, value: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Self::nominate_switch(), Error::<T>::NominateSwitchClosed);
            ensure!(!value.is_zero(), Error::<T>::LiquidityBondZero);
            ensure!(staking::EraElectionStatus::<T>::get().is_closed(), staking::Error::<T>::CallNotAllowed);
            let controller = T::Lookup::lookup(pool)?;
            ensure!(Self::is_in_pools(&controller), Error::<T>::PoolNotFound);
            let mut ledger = staking::Ledger::<T>::get(&controller).ok_or(Error::<T>::PoolUnbond)?;
            let active_era_info = staking::ActiveEra::get().ok_or(Error::<T>::NoCurrentEra)?;
            ensure!(rtoken_rate::EraRate::get(SYMBOL, active_era_info.index).is_some(), Error::<T>::EraRateNotUpdated);

            let limit = Self::pool_balance_limit();
            let bonded = Self::bonded_of(&controller).checked_add(&value).ok_or(Error::<T>::Overflow)?;
            ensure!(limit.is_zero() || bonded <= limit, Error::<T>::PoolLimitReached);

            let v = value.saturated_into::<u128>();
            let rbalance = rtoken_rate::Module::<T>::token_to_rtoken(SYMBOL, v);
            
            T::Currency::transfer(&who, &controller, value, AllowDeath)?;
            T::RCurrency::mint(&who, SYMBOL, rbalance)?;
            
            Self::bond_extra(&controller, &mut ledger, value);

            Self::deposit_event(RawEvent::LiquidityBond(who, controller, value, rbalance));

            Ok(())
        }

        /// liquitidy unbond to redeem fis with rfis
        #[weight = 100_000_000]
        pub fn liquidity_unbond(origin, pool: <T::Lookup as StaticLookup>::Source, value: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!value.is_zero(), Error::<T>::LiquidityUnbondZero);
            let op_receiver = Self::receiver();
            ensure!(op_receiver.is_some(), "No receiver to get unbond commission fee");
            ensure!(staking::EraElectionStatus::<T>::get().is_closed(), staking::Error::<T>::CallNotAllowed);
            let controller = T::Lookup::lookup(pool)?;
            ensure!(Self::is_in_pools(&controller), Error::<T>::PoolNotFound);
            let mut ledger = staking::Ledger::<T>::get(&controller).ok_or(staking::Error::<T>::NotController)?;
            let active_era_info = staking::ActiveEra::get().ok_or(Error::<T>::NoCurrentEra)?;
            ensure!(rtoken_rate::EraRate::get(SYMBOL, active_era_info.index).is_some(), Error::<T>::EraRateNotUpdated);

            let free = T::RCurrency::free_balance(&who, SYMBOL);
            free.checked_sub(value).ok_or(Error::<T>::InsufficientBalance)?;

            let max_chunks = usize::from(T::BondingDuration::get() as u16).saturating_add(2);
            ensure!(ledger.unlocking.len() < max_chunks, staking::Error::<T>::NoMoreChunks);
            let mut unbonding = <Unbonding<T>>::get(&who, &controller).unwrap_or(vec![]);
            ensure!(unbonding.len() < max_chunks, staking::Error::<T>::NoMoreChunks);

            let era = active_era_info.index + T::BondingDuration::get();
            let fee = Self::unbond_fee(value);
            let left_value = value - fee;
            let balance = rtoken_rate::Module::<T>::rtoken_to_token(SYMBOL, left_value).saturated_into::<BalanceOf<T>>();
            ensure!(ledger.active >= balance, Error::<T>::InsufficientBalance);
            ledger.active -= balance;
            if let Some(chunk) = ledger.unlocking.iter_mut().find(|chunk| chunk.era == era) {
                chunk.value.checked_add(&balance).ok_or(Error::<T>::Overflow)?;
            } else {
                ledger.unlocking.push(UnlockChunk { value: balance, era });
            }
            
            if let Some(chunk) = unbonding.iter_mut().find(|chunk| chunk.era == era) {
                chunk.value += balance;
            } else {
                unbonding.push(UnlockChunk { value: balance, era });
            }

            let receiver = op_receiver.unwrap();
            T::RCurrency::transfer(&who, &receiver, SYMBOL, fee)?;
            T::RCurrency::burn(&who, SYMBOL, left_value)?;
            staking::Module::<T>::update_ledger(&controller, &ledger);
            <Unbonding<T>>::insert(&who, &controller, unbonding);
            Self::deposit_event(RawEvent::LiquidityUnBond(who, controller, value, left_value, balance));

            Ok(())
        }

        /// liquitidy withdraw unbond: get undonded balance to free_balance
        #[weight = 100_000_000]
        pub fn liquidity_withdraw_unbond(origin, pool: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(staking::EraElectionStatus::<T>::get().is_closed(), staking::Error::<T>::CallNotAllowed);
            let controller = T::Lookup::lookup(pool)?;
            ensure!(Self::is_in_pools(&controller), Error::<T>::PoolNotFound);
            let current_era = staking::CurrentEra::get().ok_or(Error::<T>::NoCurrentEra)?;
            let unbonding = <Unbonding<T>>::get(&who, &controller).unwrap_or(vec![]);
            ensure!(!unbonding.is_empty(), Error::<T>::HasNoUnbonding);
            let mut total: BalanceOf<T> = Zero::zero();
            let new_unbonding: Vec<UnlockChunk<BalanceOf<T>>> = unbonding.into_iter()
                .filter(|chunk| if chunk.era > current_era {
                    true
                } else {
                    total = total.saturating_add(chunk.value);
                    false
                }).collect();
            
            T::Currency::transfer(&controller, &who, total, KeepAlive)?;
            if new_unbonding.is_empty() {
                <Unbonding<T>>::remove(&who, &controller);
            } else {
                <Unbonding<T>>::insert(&who, &controller, new_unbonding);
            }
            Self::deposit_event(RawEvent::LiquidityWithdrawUnBond(who, controller, total));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn is_in_pools(pool: &T::AccountId) -> bool {
        let pools = Self::pools();
        pools.contains(&pool)
    }

    fn bonded_of(pool: &T::AccountId) -> BalanceOf<T> {
        let op_ledger = staking::Ledger::<T>::get(&pool);
        if op_ledger.is_none() {
            return Zero::zero()
        }
        op_ledger.unwrap().active
    }

    fn bond_extra(controller: &T::AccountId, ledger: &mut StakingLedger<T::AccountId, BalanceOf<T>>, max_additional: BalanceOf<T>) {
        let balance = <T as staking::Trait>::Currency::free_balance(&controller);

		if let Some(extra) = balance.checked_sub(&ledger.total) {
			let extra = extra.min(max_additional);
			ledger.total += extra;
			ledger.active += extra;
			staking::Module::<T>::update_ledger(&controller, &ledger);
		}
    }

    fn total_bonded() -> BalanceOf<T> {
        Self::bonded_pools().into_iter().fold(Zero::zero(), |acc, p| acc + Self::bonded_of(&p))
    }

    fn bonded_pools() -> Vec<T::AccountId> {
        Self::pools().into_iter().filter(|p| staking::Ledger::<T>::get(&p).is_some()).collect()
    }

    fn nominatable_validators(current_era: EraIndex) -> Vec<T::AccountId> {
        let mut onboards = Self::onboard_validators();
        let max_commission = Self::max_validator_commission();
        onboards = onboards.into_iter()
            .filter(|v| staking::Validators::<T>::contains_key(&v) && staking::Validators::<T>::get(&v).commission <= max_commission)
            .collect();

        onboards.sort_by(|a, b| Self::validator_stake(current_era, &b).cmp(&Self::validator_stake(current_era, &a)));
        onboards
    }

    fn validator_stake(era: EraIndex, v: &T::AccountId) -> BalanceOf<T> {
        staking::ErasStakers::<T>::get(&era, &v).total
    }

    fn update_nominations(current_era: EraIndex, targets: &Vec<T::AccountId>, stash: &T::AccountId) {
        let nominations = Nominations {
            targets: targets.to_vec(),
            submitted_in: current_era,
            suppressed: false,
        };
        staking::Nominators::<T>::insert(stash, &nominations);
    }

    fn is_validator(era: EraIndex, t: &T::AccountId) -> bool {
        staking::ErasRewardPoints::<T>::get(&era).individual.contains_key(&t)
    }
    
    fn unbond_fee(value: u128) -> u128 {
        Self::unbond_commission() * value
    }
}

#[allow(deprecated)]
impl<T: Trait> frame_support::unsigned::ValidateUnsigned for Module<T> {
    type Call = Call<T>;

    fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
        match call {
            Call::submit_unlocks(era, pool) => {
                if !Self::is_in_pools(pool) {
                    log!(debug, "rejecting submit_unlocks as the pool is not in pools");
                    return InvalidTransaction::Call.into();
                }

                ValidTransaction::with_tag_prefix("RfisOffchain_Unlocks")
				.priority(<T as Trait>::UnsignedPriority::get())
				.and_provides(era)
				.longevity(DEFAULT_LONGEVITY)
				.propagate(true)
				.build()
            },
            Call::submit_total_bonded_before(era, _) => {
                if Self::total_bonded_before_payout(era).is_some() {
                    log!(debug, "rejecting total_bonded_before_payout because total_bonded_before is already submitted.");
					return InvalidTransaction::Call.into();
                }

                ValidTransaction::with_tag_prefix("RfisOffchain_BondedBefore")
				.priority(<T as Trait>::UnsignedPriority::get())
				.and_provides(era)
				.longevity(DEFAULT_LONGEVITY)
				.propagate(true)
				.build()
            },
            Call::submit_paidouts(era, pool, validator) => {
                if !Self::is_in_pools(pool) {
                    log!(debug, "rejecting submit_paidouts as the pool is not in pools");
					return InvalidTransaction::Call.into();
                }

                if !Self::is_validator(*era, &validator) {
                    log!(debug, "rejecting submit_paidouts as target is not an elected validator");
					return InvalidTransaction::Call.into();
                }

                ValidTransaction::with_tag_prefix("RfisOffchain_Paidouts")
				.priority(<T as Trait>::UnsignedPriority::get())
				.and_provides(era)
				.longevity(DEFAULT_LONGEVITY)
				.propagate(true)
				.build()
            },
            Call::submit_total_bonded_after(era, _) => {
                if Self::total_bonded_after_payout(era).is_some() {
                    log!(debug, "rejecting total_bonded_before_payout because total_bonded_after is already submitted.");
					return InvalidTransaction::Call.into();
                }

                ValidTransaction::with_tag_prefix("RfisOffchain_BondedAfter")
				.priority(<T as Trait>::UnsignedPriority::get())
				.and_provides(era)
				.longevity(DEFAULT_LONGEVITY)
				.propagate(true)
				.build()
            },

            Call::submit_nomination(era, pool, targets) => {
                if !Self::is_in_pools(pool) {
                    log!(debug, "rejecting submit_nomination as the pool is not in pools");
					return InvalidTransaction::Call.into();
                }

                let l = targets.len();
                if l != 0 && (l < Self::min_nomination_num().into() || l > Self::max_nomination_num().into()) {
                    log!(debug, "rejecting submit_nomination because len of target: ");
					return InvalidTransaction::Call.into();
                }

                ValidTransaction::with_tag_prefix("RfisOffchain_Nomination")
				.priority(<T as Trait>::UnsignedPriority::get())
				.and_provides(era)
				.longevity(DEFAULT_LONGEVITY)
				.propagate(true)
				.build()
            },

            _ => {
                return InvalidTransaction::Call.into();
            }
        }
    }
}

