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
        MonitorThresholdChanged(u32),
        /// Monitor added to set
        MonitorAdded(AccountId),
        /// Monitor removed from set
        MonitorRemoved(AccountId),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Monitor threshold should larger than 0
        InvalidThreshold,
        /// Monitor already in set
        MonitorAlreadyExists,
        /// Provided accountId is not a Monitor
        MonitorInvalid,
        /// Protected operation, must be performed by Monitor
        MustBeMonitor,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Monitor {
        /// Number of votes required 
        pub MonitorThreshold get(fn monitor_threshold): u32;
        /// Tracks current monitor set
        pub Monitors get(fn monitors): map hasher(twox_64_concat) T::AccountId => bool;

        /// Number of monitors in set
        pub MonitorCount get(fn monitor_count): u32;
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
            <MonitorThreshold>::put(threshold);
            Self::deposit_event(RawEvent::MonitorThresholdChanged(threshold));
            Ok(())
        }

        /// Adds a new monitor to the monitor set.
        #[weight = 10_000]
        pub fn add_monitor(origin, who: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            ensure_root(origin)?;
            let monitor = T::Lookup::lookup(who)?;
            ensure!(!Self::is_monitor(&monitor), Error::<T>::MonitorAlreadyExists);

            <Monitors<T>>::insert(&monitor, true);
            <MonitorCount>::mutate(|i| *i += 1);
            Self::deposit_event(RawEvent::MonitorAdded(monitor));
            Ok(())
        }

        /// Removes an existing monitor from the set.
        #[weight = 10_000]
        pub fn remove_monitor(origin, who: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            ensure_root(origin)?;
            let monitor = T::Lookup::lookup(who)?;
            ensure!(Self::is_monitor(&monitor), Error::<T>::MonitorInvalid);

            <Monitors<T>>::remove(&monitor);
            <MonitorCount>::mutate(|i| {*i -= 1});

            Self::deposit_event(RawEvent::MonitorRemoved(monitor));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    /// Checks if who is a monitor
    pub fn is_monitor(who: &T::AccountId) -> bool {
        Self::monitors(who)
    }
}