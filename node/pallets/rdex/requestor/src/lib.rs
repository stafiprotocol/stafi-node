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

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event! {
    pub enum Event<T> where
        AccountId = <T as system::Trait>::AccountId 
    {
        /// Vote threshold has changed (new_threshold)
        RequestorThresholdChanged(u32),
        /// Requestor added to set
        RequestorAdded(AccountId),
        /// Requestor removed from set
        RequestorRemoved(AccountId),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Requestor threshold should larger than 0
        InvalidThreshold,
        /// Requestor already in set
        RequestorAlreadyExists,
        /// Provided accountId is not a Requestor
        RequestorInvalid,
        /// Protected operation, must be performed by Requestor
        MustBeRequestor,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Requestor {
        /// Number of votes required 
        pub RequestorThreshold get(fn requestor_threshold): u32;
        /// Tracks current requestor set
        pub Requestors get(fn requestors): map hasher(twox_64_concat) T::AccountId => bool;

        /// Number of requestors in set
        pub RequestorCount get(fn requestor_count): u32;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Sets the threshold.
        #[weight = 10_000]
        pub fn set_threshold(origin, threshold: u32) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(threshold > 0, Error::<T>::InvalidThreshold);
            <RequestorThreshold>::put(threshold);
            Self::deposit_event(RawEvent::RequestorThresholdChanged(threshold));
            Ok(())
        }

        /// Adds a new requestor to the requestor set.
        #[weight = 10_000]
        pub fn add_requestor(origin, who: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            ensure_root(origin)?;
            let requestor = T::Lookup::lookup(who)?;
            ensure!(!Self::is_requestor(&requestor), Error::<T>::RequestorAlreadyExists);

            <Requestors<T>>::insert(&requestor, true);
            <RequestorCount>::mutate(|i| *i += 1);
            Self::deposit_event(RawEvent::RequestorAdded(requestor));
            Ok(())
        }

        /// Removes an existing requestor from the set.
        #[weight = 10_000]
        pub fn remove_requestor(origin, who: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            ensure_root(origin)?;
            let requestor = T::Lookup::lookup(who)?;
            ensure!(Self::is_requestor(&requestor), Error::<T>::RequestorInvalid);

            <Requestors<T>>::remove(&requestor);
            <RequestorCount>::mutate(|i| {*i -= 1});

            Self::deposit_event(RawEvent::RequestorRemoved(requestor));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    /// Checks if who is a requestor
    pub fn is_requestor(who: &T::AccountId) -> bool {
        Self::requestors(who)
    }
}