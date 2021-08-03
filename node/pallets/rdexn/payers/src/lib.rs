#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult, ensure
};

use frame_system::{self as system, ensure_root};
use sp_runtime::{
    traits::{StaticLookup}
};
use node_primitives::{RSymbol};

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event! {
    pub enum Event<T> where
        AccountId = <T as system::Trait>::AccountId 
    {
        /// Vote threshold has changed (new_threshold)
        PayerThresholdChanged(RSymbol, u32),
        /// Payer added to set
        PayerAdded(RSymbol, AccountId),
        /// Payer removed from set
        PayerRemoved(RSymbol, AccountId),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Payer threshold should larger than 0
        InvalidThreshold,
        /// Payer already in set
        PayerAlreadyExists,
        /// Provided accountId is not a payer
        PayerInvalid,
        /// Protected operation, must be performed by payer
        MustBePayer,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RDexnPayers {
        /// Number of votes required for a proposal to execute
        pub PayerThreshold get(fn payer_threshold): map hasher(blake2_128_concat) RSymbol => u32;

        /// Tracks current payer set
        pub Payers get(fn payers): double_map hasher(blake2_128_concat) RSymbol, hasher(blake2_128_concat) T::AccountId => bool;

        /// Number of payers in set
        pub PayerCount get(fn payer_count): map hasher(blake2_128_concat) RSymbol => u32;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Sets the vote threshold for proposals.
        #[weight = 10_000]
        pub fn set_threshold(origin, symbol: RSymbol, threshold: u32) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(threshold > 0, Error::<T>::InvalidThreshold);

            <PayerThreshold>::insert(symbol, threshold);
            Self::deposit_event(RawEvent::PayerThresholdChanged(symbol, threshold));
            Ok(())
        }

        /// Adds a new payer to the payer set.
        #[weight = 10_000]
        pub fn add_payer(origin, symbol: RSymbol, who: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            ensure_root(origin)?;
            let payer = T::Lookup::lookup(who)?;
            ensure!(!Self::is_payer(symbol, &payer), Error::<T>::PayerAlreadyExists);

            <Payers<T>>::insert(symbol, &payer, true);
            <PayerCount>::mutate(symbol, |i| {*i += 1});
    
            Self::deposit_event(RawEvent::PayerAdded(symbol, payer));
            Ok(())
        }

        /// Removes an existing payer from the set.
        #[weight = 10_000]
        pub fn remove_payer(origin, symbol: RSymbol, who: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            ensure_root(origin)?;
            let payer = T::Lookup::lookup(who)?;
            ensure!(Self::is_payer(symbol, &payer), Error::<T>::PayerInvalid);

            <Payers<T>>::remove(symbol, &payer);
            <PayerCount>::mutate(symbol, |i| {*i -= 1});

            Self::deposit_event(RawEvent::PayerRemoved(symbol, payer));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    /// Checks if who is a payer
    pub fn is_payer(symbol: RSymbol, who: &T::AccountId) -> bool {
        Self::payers(symbol, who)
    }
}