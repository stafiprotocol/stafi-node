// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{result, prelude::*};
use codec::{Codec, Encode, Decode};
use frame_support::{
    debug, Parameter, decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{Currency, Get, ExistenceRequirement::{KeepAlive, AllowDeath}},
};
use frame_system::{self as system, ensure_signed, ensure_root};
use sp_runtime::traits::{
    Zero, AccountIdConversion, CheckedSub,CheckedAdd, SaturatedConversion,
    Saturating, StaticLookup,
};
use sp_runtime::{ModuleId};
use pallet_staking::{self as staking, MAX_NOMINATIONS, MAX_UNLOCKING_CHUNKS, Nominations, RewardDestination, StakingLedger, EraIndex, UnlockChunk};
use rtoken_balances::{traits::{Currency as RCurrency}};
use node_primitives::{RSymbol};

const POOL_ID_1: ModuleId = ModuleId(*b"rFISpot1");
const SYMBOL: RSymbol = RSymbol::RFIS;

type BalanceOf<T> = staking::BalanceOf<T>;
// type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type RBalanceOf<T> = <<T as Trait>::RCurrency as RCurrency<<T as frame_system::Trait>::AccountId>>::RBalance;

pub trait Trait: system::Trait + staking::Trait + rtoken_rate::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type RCurrency: RCurrency<Self::AccountId>;
}

decl_event! {
    pub enum Event<T> where
        Balance = BalanceOf<T>,
        RBalance = RBalanceOf<T>,
        <T as frame_system::Trait>::AccountId
    {
        /// liquidity stake record
        LiquidityBond(AccountId, Balance, RBalance),
        /// liquidity unbond record
        LiquidityUnBond(AccountId, Balance, RBalance),
        /// liquidity withdraw unbond
        LiquidityWithdrawUnBond(AccountId, Balance),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Got an overflow after adding
        Overflow,
        /// liquidity bond Zero
        LiquidityBondZero,
        /// liquidity unbond Zero
        LiquidityUnbondZero,
        /// insufficient balance
        InsufficientBalance,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RFis {
        pub Unbonding get(fn unbonding): map hasher(twox_64_concat) T::AccountId => Option<Vec<UnlockChunk<BalanceOf<T>>>>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        fn on_finalize() {
            let stash = Self::account_id_1();

            if let Some(active_era) = staking::ActiveEra::get() {
                let op_rate = rtoken_rate::EraRate::get(SYMBOL, active_era.index);
                if  op_rate.is_none() {
                    let era = active_era.index.saturating_sub(1);
                    if era > 0 {
                        let stashs: Vec<T::AccountId> = [stash.clone()].to_vec();
                        Self::claim_rewards(era, stashs);
                    }
                    
                    let total_balance = T::Currency::total_balance(&stash).saturated_into::<u128>();
                    let total_rbalance = T::RCurrency::total_issuance(SYMBOL).saturated_into::<u128>();
                    let rate =  rtoken_rate::Module::<T>::set_rate(SYMBOL, total_balance, total_rbalance);
                    rtoken_rate::EraRate::insert(SYMBOL, active_era.index, rate);
                }
            }
        }

        /// liquidity bond fis to get rfis
        #[weight = 100_000_000]
        pub fn liquidity_bond(origin, value: BalanceOf<T>) -> DispatchResult {
            ensure!(!value.is_zero(), Error::<T>::LiquidityBondZero);
            ensure!(staking::EraElectionStatus::<T>::get().is_closed(), staking::Error::<T>::CallNotAllowed);
            let who = ensure_signed(origin)?;

            let pot = Self::account_id_1();
            let v = value.saturated_into::<u128>();
            let rbalance = rtoken_rate::Module::<T>::token_to_rtoken(SYMBOL, v).saturated_into::<RBalanceOf<T>>();
            
            T::Currency::transfer(&who, &pot, value.into(), KeepAlive)?;
            T::RCurrency::mint(&who, SYMBOL, rbalance)?;
            
            Self::bond_extra(&pot, value.into());

            Self::deposit_event(RawEvent::LiquidityBond(who, value, rbalance));

            Ok(())
        }

        /// liquitidy unbond to redeem fis with rfis
        #[weight = 100_000_000]
        pub fn liquidity_unbond(origin, value: RBalanceOf<T>) -> DispatchResult {
            ensure!(!value.is_zero(), Error::<T>::LiquidityUnbondZero);
            ensure!(staking::EraElectionStatus::<T>::get().is_closed(), staking::Error::<T>::CallNotAllowed);
            let who = ensure_signed(origin)?;
            let free = T::RCurrency::free_balance(&who, SYMBOL);
            free.checked_sub(&value).ok_or(Error::<T>::InsufficientBalance)?;
            
            let controller = Self::account_id_1();
            let mut ledger = staking::Ledger::<T>::get(&controller).ok_or(staking::Error::<T>::NotController)?;
            ensure!(ledger.unlocking.len() < MAX_UNLOCKING_CHUNKS, staking::Error::<T>::NoMoreChunks);
            let mut unbonding = <Unbonding<T>>::get(&who).unwrap_or(vec![]);
            ensure!(unbonding.len() < MAX_UNLOCKING_CHUNKS, staking::Error::<T>::NoMoreChunks);

            let era = staking::CurrentEra::get().unwrap_or(0) + T::BondingDuration::get();
            let v = value.saturated_into::<u128>();
            let balance = rtoken_rate::Module::<T>::rtoken_to_token(SYMBOL, v).saturated_into::<BalanceOf<T>>();
            ledger.active -= balance;
            if let Some(chunk) = ledger.unlocking.iter_mut().find(|chunk| chunk.era == era) {
                chunk.value += balance;
            } else {
                ledger.unlocking.push(UnlockChunk { value: balance, era });
            }
            
            if let Some(chunk) = unbonding.iter_mut().find(|chunk| chunk.era == era) {
                chunk.value += balance;
            } else {
                unbonding.push(UnlockChunk { value: balance, era });
            }

            T::RCurrency::burn(&who, SYMBOL, value)?;
            staking::Module::<T>::update_ledger(&controller, &ledger);
            <Unbonding<T>>::insert(&who, unbonding);
            Self::deposit_event(RawEvent::LiquidityUnBond(who, balance, value));

            Ok(())
            
			// Self::deposit_event(RawEvent::Unbonded(ledger.stash, value));
        }

        /// liquitidy withdraw unbond: get undonded balance to free_balance
        #[weight = 100_000_000]
        pub fn liquidity_withdraw_unbond(origin) -> DispatchResult {
            ensure!(staking::EraElectionStatus::<T>::get().is_closed(), staking::Error::<T>::CallNotAllowed);
            let who = ensure_signed(origin)?;
            let controller = Self::account_id_1();
            let current_era = staking::CurrentEra::get().ok_or(staking::Error::<T>::InvalidEraToReward)?;
            let mut ledger = staking::Ledger::<T>::get(&controller).ok_or(staking::Error::<T>::NotController)?;
            // let (stash, old_total) = (ledger.stash.clone(), ledger.total);
            ledger = ledger.consolidate_unlocked(current_era);
            let unbonding = <Unbonding<T>>::get(&who).unwrap_or(vec![]);
            let mut total: BalanceOf<T> = Zero::zero();
            let new_unbonding: Vec<UnlockChunk<BalanceOf<T>>> = unbonding.into_iter()
                .filter(|chunk| if chunk.era > current_era {
                    total = total.saturating_add(chunk.value);
                    false
                } else {
                    true
                }).collect();
            staking::Module::<T>::update_ledger(&controller, &ledger);
            T::Currency::transfer(&controller, &who, total, AllowDeath)?;
            <Unbonding<T>>::insert(&who, new_unbonding);
            Self::deposit_event(RawEvent::LiquidityWithdrawUnBond(who, total));
            Ok(())
        }

        #[weight = 100_000_000]
        pub fn bond(origin) -> DispatchResult {
			ensure_root(origin)?;

            let stash = Self::account_id_1();

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
			// Self::deposit_event(RawEvent::Bonded(stash.clone(), value));
			let item = StakingLedger {
				stash,
				total: value,
				active: value,
				unlocking: vec![],
				claimed_rewards: vec![],
            };

            let controller: T::AccountId = Self::account_id_1();
            staking::Module::<T>::update_ledger(&controller, &item);
            Ok(())
        }
        
        #[weight = 100_000_000]
		pub fn nominate(origin, targets: Vec<<T::Lookup as StaticLookup>::Source>) {
            ensure_root(origin)?;
            ensure!(staking::EraElectionStatus::<T>::get().is_closed(), staking::Error::<T>::CallNotAllowed);

			let controller = Self::account_id_1();
			let ledger = staking::Ledger::<T>::get(&controller).ok_or(staking::Error::<T>::NotController)?;
			let stash = &ledger.stash;
			ensure!(!targets.is_empty(), staking::Error::<T>::EmptyTargets);
			let targets = targets.into_iter()
				.take(MAX_NOMINATIONS)
				.map(|t| T::Lookup::lookup(t))
				.collect::<result::Result<Vec<T::AccountId>, _>>()?;

			let nominations = Nominations {
				targets,
				// initial nominations are considered submitted at era 0. See `Nominations` doc
				submitted_in: staking::CurrentEra::get().unwrap_or(0),
				suppressed: false,
			};

            staking::Nominators::<T>::insert(stash, &nominations);
		}
    }
}

impl<T: Trait> Module<T> {
    /// Provides an AccountId for the pallet.
    /// This is used both as an origin check and deposit/withdrawal account.
    pub fn account_id_1() -> T::AccountId {
        POOL_ID_1.into_account()
    }

    fn bond_extra(stash: &T::AccountId, max_additional: BalanceOf<T>) {
		let controller = <staking::Module<T>>::bonded(&stash).unwrap();
		let mut ledger = <staking::Module<T>>::ledger(&controller).unwrap();

        let stash_balance = T::Currency::free_balance(&stash);
        // let stash_balance = <T as staking::Trait>::Currency::free_balance(&stash);

		if let Some(extra) = stash_balance.checked_sub(&ledger.total) {
			let extra = extra.min(max_additional);
			ledger.total += extra;
			ledger.active += extra;
			staking::Module::<T>::update_ledger(&controller, &ledger);
		}
    }

    fn claim_rewards(era: EraIndex, stashs: Vec<T::AccountId>) {
        for stash in &stashs {
            if let Some(nominations) = staking::Nominators::<T>::get(&stash) {
                for t in nominations.targets {
                    staking::Module::<T>::do_payout_stakers(t, era);
                    //todos deal DispatchResult of do_payout_stakers
                    // 记录下来没有成功的erahe stashs
                }
            }
        }
    }
}

