// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use frame_support::{
    decl_error, decl_event, decl_storage, decl_module, dispatch::DispatchResult, ensure,
    traits::{
        EnsureOrigin,
    },
};
use frame_system::{self as system, ensure_root};
use node_primitives::{RSymbol};

pub type ChainEra = u32;

pub trait Trait: system::Trait + rtoken_rate::Trait {
    type Event: From<Event> + Into<<Self as system::Trait>::Event>;

    /// Specifies the origin check provided by the voter for calls that can only be called by the votes pallet
    type VoterOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;
}

decl_event! {
    pub enum Event {
        /// symbol, era
        EraInitialized(RSymbol, ChainEra),
        /// symbol, old_era, new_era
        EraUpdated(RSymbol, ChainEra, ChainEra),
        /// symbol, old_bonding_duration, new_bonding_duration
        BondingDurationUpdated(RSymbol, u32, u32),
        /// pool added
        PoolAdded(RSymbol, Vec<u8>),
        /// pool sub account added: (symbol, pool, sub_account)
        PoolSubAccountAdded(RSymbol, Vec<u8>, Vec<u8>),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// pool already added
        PoolAlreadyAdded,
        /// pool not found
        PoolNotFound,
        /// sub account already added
        SubAccountAlreadyAdded,
        /// era zero
        EraZero,
        /// era already initialized
        EraAlreadyInitialized,
        /// new_era not bigger than old
        NewEraNotBiggerThanOld,
        /// new_bonding_duration zero
        NewBondingDurationZero
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RTokenLedger {
        pub ChainEras get(fn chain_eras): map hasher(blake2_128_concat) RSymbol => Option<ChainEra>;
        pub ChainBondingDuration get(fn chain_bonding_duration): map hasher(twox_64_concat) RSymbol => Option<u32>;
        /// Pools
        pub Pools get(fn pools): map hasher(blake2_128_concat) RSymbol => Vec<Vec<u8>>;
        /// pool => Vec<SubAccounts>
        pub SubAccounts get(fn sub_accounts): map hasher(blake2_128_concat) Vec<u8> => Vec<Vec<u8>>;
        /// pool sub account flag
        pub PoolSubAccountFlag get(fn pool_sub_account_flag): map hasher(blake2_128_concat) (Vec<u8>, Vec<u8>) => Option<bool>;
        /// pool bonded
        pub PoolBonded get(fn pool_bonded): map hasher(blake2_128_concat) (RSymbol, Vec<u8>) => Option<bool>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// add new pool
        #[weight = 10_000]
        pub fn add_new_pool(origin, symbol: RSymbol, pool: Vec<u8>) -> DispatchResult {
            ensure_root(origin)?;
            let mut pools = Self::pools(symbol);
            ensure!(!pools.contains(&pool), Error::<T>::PoolAlreadyAdded);
            pools.push(pool.clone());
            Pools::insert(symbol, pools);

            Self::deposit_event(Event::PoolAdded(symbol, pool));
            Ok(())
        }

        /// add new pool
        #[weight = 10_000]
        pub fn add_sub_account_for_pool(origin, symbol: RSymbol, pool: Vec<u8>, sub_account: Vec<u8>) -> DispatchResult {
            ensure_root(origin)?;
            let pools = Self::pools(symbol);
            ensure!(pools.contains(&pool), Error::<T>::PoolNotFound);
            let mut sub_accounts = Self::sub_accounts(&pool);
            ensure!(!sub_accounts.contains(&sub_account), Error::<T>::SubAccountAlreadyAdded);

            sub_accounts.push(sub_account.clone());
            <SubAccounts>::insert(&pool, &sub_accounts);
            <PoolSubAccountFlag>::insert((&pool, &sub_account), true);

            Self::deposit_event(Event::PoolSubAccountAdded(symbol, pool, sub_account));
            Ok(())
        }

        /// Initialize chain era
        #[weight = 10_000]
        pub fn initialize_chain_era(origin, symbol: RSymbol, era: u32) -> DispatchResult {
            ensure_root(origin)?;

            ensure!(era > 0, Error::<T>::EraZero);
            ensure!(!Self::chain_eras(symbol).is_some(), Error::<T>::EraAlreadyInitialized);
            <ChainEras>::insert(symbol, era);

            let rate = rtoken_rate::Module::<T>::set_rate(symbol, 0, 0);
            rtoken_rate::EraRate::insert(symbol, era, rate);

            Self::deposit_event(Event::EraInitialized(symbol, era));
            Ok(())
        }


        /// set chain era
        #[weight = 10_000]
        pub fn set_chain_era(origin, symbol: RSymbol, new_era: u32) -> DispatchResult {
            T::VoterOrigin::ensure_origin(origin)?;
            let old_era = Self::chain_eras(symbol).unwrap_or(0);
            ensure!(new_era > old_era, Error::<T>::NewEraNotBiggerThanOld);
            <ChainEras>::insert(symbol, new_era);

            Self::deposit_event(Event::EraUpdated(symbol, old_era, new_era));
            Ok(())
        }

        /// set chain era
        #[weight = 10_000]
        pub fn set_chain_bonding_duration(origin, symbol: RSymbol, new_bonding_duration: u32) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(new_bonding_duration > 0, Error::<T>::NewBondingDurationZero);

            let old_bonding_duration = Self::chain_bonding_duration(symbol).unwrap_or(0);
            ChainBondingDuration::insert(symbol, new_bonding_duration);

            Self::deposit_event(Event::BondingDurationUpdated(symbol, old_bonding_duration, new_bonding_duration));
            Ok(())
        }
    }
}