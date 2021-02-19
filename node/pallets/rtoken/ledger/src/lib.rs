// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use frame_support::{
    decl_error, decl_event, decl_storage, decl_module, dispatch::DispatchResult, ensure,
    traits::{
        EnsureOrigin, Get,
    },
};
use frame_system::{self as system, ensure_signed};
use node_primitives::{RSymbol};

pub type ChainEra = u32;

pub trait Trait: system::Trait {
    type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;

    /// Specifies the origin check provided by the voter for calls that can only be called by the votes pallet
    type VoterOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;
}

decl_event! {
    pub enum Event {
        /// symbol, new_era
        EraUpdated(RSymbol, ChainEra, ChainEra),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// new_era not bigger than old
        NewEraNotBiggerThanOld,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Ledger {
        pub ChainEras get(fn chain_eras): map hasher(blake2_128_concat) RSymbol => Option<ChainEra>;
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

            Ok(())
        }
    }
}