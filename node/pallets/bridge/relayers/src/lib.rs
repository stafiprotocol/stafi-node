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
use node_primitives::{ChainId};

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event! {
    pub enum Event<T> where
        AccountId = <T as system::Trait>::AccountId
    {
        /// Vote threshold has changed (new_threshold)
        RelayerThresholdChanged(ChainId, u32),
        /// Relayer added to set
        RelayerAdded(ChainId, AccountId),
        /// Relayer removed from set
        RelayerRemoved(ChainId, AccountId),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Relayer threshold should larger than 0
        InvalidThreshold,
        /// Relayer already in set
        RelayerAlreadyExists,
        /// Provided accountId is not a relayer
        RelayerInvalid,
        /// Protected operation, must be performed by relayer
        MustBeRelayer,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as BridgeRelayers {
        /// Number of votes required for a proposal to execute
        pub RelayerThreshold get(fn relayer_threshold): map hasher(blake2_128_concat) ChainId => u32;

        /// Tracks current relayer set
        pub Relayers get(fn relayers): double_map hasher(blake2_128_concat) ChainId, hasher(twox_64_concat) T::AccountId => bool;

        /// Number of relayers in set
        pub RelayerCount get(fn relayer_count): map hasher(blake2_128_concat) ChainId => u32;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Sets the vote threshold for proposals.
        #[weight = 10_000]
        pub fn set_threshold(origin, chain_id: ChainId, threshold: u32) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(threshold > 0, Error::<T>::InvalidThreshold);

            <RelayerThreshold>::insert(chain_id, threshold);
            Self::deposit_event(RawEvent::RelayerThresholdChanged(chain_id, threshold));
            Ok(())
        }

        /// Adds a new relayer to the relayer set.
        #[weight = 10_000]
        pub fn add_relayer(origin, chain_id: ChainId, who: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            ensure_root(origin)?;
            let relayer = T::Lookup::lookup(who)?;
            ensure!(!Self::is_relayer(chain_id, &relayer), Error::<T>::RelayerAlreadyExists);

            <Relayers<T>>::insert(chain_id, &relayer, true);
            <RelayerCount>::mutate(chain_id, |i| {*i += 1});

            Self::deposit_event(RawEvent::RelayerAdded(chain_id, relayer));
            Ok(())
        }

        /// Removes an existing relayer from the set.
        #[weight = 10_000]
        pub fn remove_relayer(origin, chain_id: ChainId, who: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            ensure_root(origin)?;
            let relayer = T::Lookup::lookup(who)?;
            ensure!(Self::is_relayer(chain_id, &relayer), Error::<T>::RelayerInvalid);

            <Relayers<T>>::remove(chain_id, &relayer);
            <RelayerCount>::mutate(chain_id, |i| {*i -= 1});

            Self::deposit_event(RawEvent::RelayerRemoved(chain_id, relayer));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    /// Checks if who is a relayer
    pub fn is_relayer(chain_id: ChainId, who: &T::AccountId) -> bool {
        Self::relayers(chain_id, who)
    }
}