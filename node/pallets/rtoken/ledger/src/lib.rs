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

pub trait Trait: system::Trait {
    type Event: From<Event> + Into<<Self as system::Trait>::Event>;

    /// Specifies the origin check provided by the voter for calls that can only be called by the votes pallet
    type VoterOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;
}

decl_event! {
    pub enum Event {
        /// symbol, old_era, new_era
        EraUpdated(RSymbol, ChainEra, ChainEra),
        /// symbol, old_bonding_duration, new_bonding_duration
        BondingDurationUpdated(RSymbol, u32, u32),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// new_era not bigger than old
        NewEraNotBiggerThanOld,
        /// new_bonding_duration zero
        NewBondingDurationZero
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RTokenLedger {
        pub ChainEras get(fn chain_eras): map hasher(twox_64_concat) RSymbol => Option<ChainEra>;
        pub ChainBondingDuration get(fn chain_bonding_duration): map hasher(twox_64_concat) RSymbol => Option<u32>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

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