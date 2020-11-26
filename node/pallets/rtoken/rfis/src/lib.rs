// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{result, prelude::*};
use codec::{Codec, Encode, Decode};
use frame_support::{
    debug, Parameter, decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchResult},
    ensure,
    traits::{Currency, Get, ExistenceRequirement::{KeepAlive, AllowDeath}},
};
use frame_system::{
    self as system, ensure_signed, ensure_root, ensure_none,
    offchain::{SendTransactionTypes, SubmitTransaction},
};
use sp_runtime::{
    ModuleId, RuntimeDebug,
    traits::{
        Convert, Zero, AccountIdConversion, CheckedSub,CheckedAdd, SaturatedConversion, Saturating, StaticLookup,
    },
    transaction_validity::{
		TransactionValidityError, TransactionValidity, ValidTransaction, InvalidTransaction,
		TransactionSource, TransactionPriority, TransactionLongevity,
	},
};
use pallet_staking::{
    self as staking, MAX_NOMINATIONS, MAX_UNLOCKING_CHUNKS, Nominations,
    RewardDestination, StakingLedger, EraIndex, UnlockChunk,
};
use pallet_session as session;
use rtoken_balances::{traits::{Currency as RCurrency}};
use node_primitives::{RSymbol};

const POOL_ID_1: ModuleId = ModuleId(*b"rFISpot1");
const SYMBOL: RSymbol = RSymbol::RFIS;
pub const MAX_ONBOARD_VALIDATORS: usize = 300;
pub const RFIS_MAX_NOMINATIONS: usize = MAX_NOMINATIONS;
pub const BondingDuration: EraIndex = 1;

type BalanceOf<T> = staking::BalanceOf<T>;
type RBalanceOf<T> = <<T as Trait>::RCurrency as RCurrency<<T as frame_system::Trait>::AccountId>>::RBalance;

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
        RBalance = RBalanceOf<T>,
        <T as frame_system::Trait>::AccountId
    {
        /// liquidity stake record
        LiquidityBond(AccountId, Balance, RBalance),
        /// liquidity unbond record
        LiquidityUnBond(AccountId, Balance, RBalance),
        /// liquidity withdraw unbond
        LiquidityWithdrawUnBond(AccountId, Balance),
        /// validator onboard
        ValidatorOnboard(AccountId),
        /// validator deregistered
        ValidatorOffboard(AccountId),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Got an overflow after adding
        Overflow,
        /// pool not bonded
        PoolUnbond,
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
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RFis {
        pub Unbonding get(fn unbonding): map hasher(twox_64_concat) T::AccountId => Option<Vec<UnlockChunk<BalanceOf<T>>>>;
        pub OnboardValidators get(fn registered_validators): Vec<T::AccountId>;
        pub NominationUpdated get(fn nomination_updated): map hasher(blake2_128_concat) EraIndex => bool = false;
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

        fn offchain_worker(block: T::BlockNumber) {
            let op_current_era = staking::CurrentEra::get();
            if op_current_era.is_none() {
                debug::info!("invalid current era");
                return;
            }
            let era = op_current_era.unwrap();
            if <NominationUpdated>::get(era) {
                debug::info!("nomination already updated");
                return;
            }
            
            let mut onboards = <OnboardValidators<T>>::get();
            // if !sp_io::offchain::is_validator() {
            //     debug::info!("the node isn't a validator");
            //     return;
            // }
            
            if onboards.is_empty() {
                debug::info!("no validator onboard");
                return;
            } 
            // else {
            //     onboards = onboards.into_iter().filter(|v| staking::ErasStakers::<T>::contains_key(era, &v)).collect();
            //     if onboards.is_empty() {
            //         debug::info!("no validator onboard in era stakers");
            //         return;
            //     }
            // }
            onboards.sort_by(|a, b| staking::ErasStakers::<T>::get(era, &a).total.cmp(&staking::ErasStakers::<T>::get(era, &b).total));
            if onboards.len() > RFIS_MAX_NOMINATIONS {
                onboards.resize_with(RFIS_MAX_NOMINATIONS, Default::default);
            }

            let call = Call::submit_nomination_unsigned(onboards).into();
            if let Err(e) = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call) {
                debug::info!("failed to submit nomination unsigned: {:?}", e);
            }
        }

        /// check if new nomination was needed.
        #[weight = 100_000_000]
        fn submit_nomination_unsigned(origin, targets: Vec<T::AccountId>) -> DispatchResult {
            ensure_none(origin)?;
            ensure!(staking::EraElectionStatus::<T>::get().is_closed(), staking::Error::<T>::CallNotAllowed);
            let current_era = staking::CurrentEra::get().ok_or(staking::Error::<T>::InvalidEraToReward)?;
            let controller = Self::account_id_1();
            let ledger = staking::Ledger::<T>::get(&controller).ok_or(staking::Error::<T>::NotController)?;
			let stash = &ledger.stash;
            Self::update_nominations(current_era, targets, stash);
            <NominationUpdated>::insert(current_era, true);
            Ok(())
        }

        /// onboard as an validator which may be nominated by the pot
        #[weight = 100_000_000]
        pub fn onboard(origin) -> DispatchResult {
            let mut onboards = <OnboardValidators<T>>::get();
            ensure!(onboards.len() <= MAX_ONBOARD_VALIDATORS, Error::<T>::ValidatorLimitReached);
            let who = ensure_signed(origin)?;
            let location = onboards.binary_search(&who).err().ok_or(Error::<T>::AlreadyOnboard)?;
            let validator_id = <T as session::Trait>::ValidatorIdOf::convert(who.clone()).ok_or(Error::<T>::NoAssociatedValidatorId)?;
            session::Module::<T>::load_keys(&validator_id).ok_or(Error::<T>::NoSessionKey)?;

            onboards.insert(location, who.clone());
			<OnboardValidators<T>>::put(onboards);

            Self::deposit_event(RawEvent::ValidatorOnboard(who));
            Ok(())
        }

        /// offboard
        #[weight = 100_000_000]
        pub fn offboard(origin) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut onboards = <OnboardValidators<T>>::get();
            let location = onboards.binary_search(&who).ok().ok_or(Error::<T>::NotOnboard)?;
            onboards.remove(location);
            <OnboardValidators<T>>::put(onboards);
            
            Self::deposit_event(RawEvent::ValidatorOffboard(who));
            Ok(())
        }

        /// liquidity bond fis to get rfis
        #[weight = 100_000_000]
        pub fn liquidity_bond(origin, value: BalanceOf<T>) -> DispatchResult {
            ensure!(!value.is_zero(), Error::<T>::LiquidityBondZero);
            ensure!(staking::EraElectionStatus::<T>::get().is_closed(), staking::Error::<T>::CallNotAllowed);
            let controller = Self::account_id_1();
            let mut ledger = staking::Ledger::<T>::get(&controller).ok_or(Error::<T>::PoolUnbond)?;
            let who = ensure_signed(origin)?;

            let v = value.saturated_into::<u128>();
            let rbalance = rtoken_rate::Module::<T>::token_to_rtoken(SYMBOL, v).saturated_into::<RBalanceOf<T>>();
            
            T::Currency::transfer(&who, &controller, value.into(), KeepAlive)?;
            T::RCurrency::mint(&who, SYMBOL, rbalance)?;
            
            Self::bond_extra(&controller, &mut ledger, value.into());

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
            // better to take unbond into on_finalize.
            ensure!(unbonding.len() < MAX_UNLOCKING_CHUNKS, staking::Error::<T>::NoMoreChunks);

            let era = staking::CurrentEra::get().unwrap_or(0) + BondingDuration;
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
            // todo: should be keep alive.
            T::Currency::transfer(&controller, &who, total, AllowDeath)?;
            <Unbonding<T>>::insert(&who, new_unbonding);
            Self::deposit_event(RawEvent::LiquidityWithdrawUnBond(who, total));
            Ok(())
        }  

        /// manually bond
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
        
        // /// manually nominate
        // #[weight = 100_000_000]
		// pub fn nominate(origin, targets: Vec<<T::Lookup as StaticLookup>::Source>) {
        //     ensure_root(origin)?;
        //     ensure!(staking::EraElectionStatus::<T>::get().is_closed(), staking::Error::<T>::CallNotAllowed);

		// 	let controller = Self::account_id_1();
		// 	let ledger = staking::Ledger::<T>::get(&controller).ok_or(staking::Error::<T>::NotController)?;
		// 	let stash = &ledger.stash;
		// 	ensure!(!targets.is_empty(), staking::Error::<T>::EmptyTargets);
		// 	let targets = targets.into_iter()
		// 		.take(MAX_NOMINATIONS)
		// 		.map(|t| T::Lookup::lookup(t))
		// 		.collect::<result::Result<Vec<T::AccountId>, _>>()?;

		// 	let nominations = Nominations {
		// 		targets,
		// 		// initial nominations are considered submitted at era 0. See `Nominations` doc
		// 		submitted_in: staking::CurrentEra::get().unwrap_or(0),
		// 		suppressed: false,
		// 	};

        //     staking::Nominators::<T>::insert(stash, &nominations);
		// }
    }
}

impl<T: Trait> Module<T> {
    /// Provides an AccountId for the pallet.
    /// This is used both as an origin check and deposit/withdrawal account.
    pub fn account_id_1() -> T::AccountId {
        POOL_ID_1.into_account()
    }

    fn bond_extra(controller: &T::AccountId, ledger: &mut StakingLedger<T::AccountId, BalanceOf<T>>, max_additional: BalanceOf<T>) {
        let balance = T::Currency::free_balance(&controller);

		if let Some(extra) = balance.checked_sub(&ledger.total) {
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

    fn update_nominations(current_era: EraIndex, targets: Vec<T::AccountId>, stash: &T::AccountId) {
        let nominations = Nominations {
            targets,
            submitted_in: current_era,
            suppressed: false,
        };
        staking::Nominators::<T>::insert(stash, &nominations);
    }
}

#[allow(deprecated)]
impl<T: Trait> frame_support::unsigned::ValidateUnsigned for Module<T> {
    type Call = Call<T>;

    fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
        if let Call::submit_nomination_unsigned(targets) = call {
            if targets.len() > RFIS_MAX_NOMINATIONS {
                return InvalidTransaction::Custom(1).into();
            }

            if targets.is_empty() {
                return InvalidTransaction::Custom(0).into();
            }

            Ok(ValidTransaction {
                priority: <T as Trait>::UnsignedPriority::get(),
                requires: vec![],
                provides: vec![],
                longevity: TransactionLongevity::max_value(),
                propagate: true,
            })
        } else {
            InvalidTransaction::Call.into()
        }
    }
}

