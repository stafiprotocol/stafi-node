#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult, ensure,
    traits::{Get},
};

use frame_system::{self as system, ensure_signed, ensure_root};
use sp_runtime::{
    traits::{StaticLookup}
};
use rtoken_balances::{traits::{Currency as RCurrency}};
use node_primitives::{RSymbol};

pub mod models;
pub use models::*;

const DEFAULT_RELAYER_THRESHOLD: u32 = 1;



pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// currency of rtoken
    type RCurrency: RCurrency<Self::AccountId>;

    type VoteLifetime: Get<Self::BlockNumber>;
}

decl_event! {
    pub enum Event<T> where
        Hash = <T as system::Trait>::Hash,
        AccountId = <T as system::Trait>::AccountId 
    {
        /// Vote threshold has changed (new_threshold)
        RelayerThresholdChanged(u32),
        /// Relayer added to set
        RelayerAdded(AccountId),
        /// Relayer removed from set
        RelayerRemoved(AccountId),
        /// Vote submitted in favour of proposal
        VoteFor(AccountId, Hash),
        /// Vot submitted against proposal
        VoteAgainst(AccountId, Hash),
        /// Bond Approved
        BondApproved(Hash),
        /// Bond Rejected
        BondRejected(Hash),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Relayer threshold not set
        ThresholdNotSet,
        /// Relayer threshold should larger than 0
        InvalidThreshold,
        /// Relayer already in set
        RelayerAlreadyExists,
        /// Provided accountId is not a relayer
        RelayerInvalid,
        /// Protected operation, must be performed by relayer
        MustBeRelayer,
        /// Relayer has already submitted some vote for this vote
        RelayerAlreadyVoted,
        /// A vote with these parameters has already been submitted
        BondVoteAlreadyExists,
        /// No vote with the ID was found
        BondVoteNotFound,
        /// No record with the ID was found
        BondRecordNotFound,
        /// Bond id repeated
        BondIdRepeated,
        /// Bond Already Approved
        BondAlreadyApproved,
        /// Bond Already Rejected
        BondAlreadyRejected,
        /// Cannot complete vote, needs more votes
        BondVoteNotPassed,
        /// BondVote has either failed or succeeded
        BondVoteAlreadyCompleted,
        /// Lifetime of vote has been exceeded
        BondVoteExpired,
        /// Bond Voting processing
        BondVoteAlive,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as BridgeCommon {
        /// Number of votes required for a proposal to execute
        RelayerThreshold get(fn relayer_threshold): u32 = DEFAULT_RELAYER_THRESHOLD;

        /// Tracks current relayer set
        pub Relayers get(fn relayers): map hasher(blake2_128_concat) T::AccountId => bool;

        /// Number of relayers in set
        pub RelayerCount get(fn relayer_count): u32;

        pub BondVotes get(fn bond_votes): map hasher(blake2_128_concat) T::Hash => Option<BondVote<T::Hash, T::AccountId, T::BlockNumber>>;

        pub BondRecords get(fn bond_records): map hasher(blake2_128_concat) T::Hash => Option<BondRecord<T::AccountId, RSymbol>>;

        pub AccountBondIds get(fn account_bond_ids): map hasher(twox_64_concat) T::AccountId => Option<Vec<T::Hash>>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        const VoteLifetime: T::BlockNumber = T::VoteLifetime::get();

        fn deposit_event() = default;

        /// Sets the vote threshold for proposals.
        #[weight = 10_000]
        pub fn set_threshold(origin, threshold: u32) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(threshold > 0, Error::<T>::InvalidThreshold);
            <RelayerThreshold>::put(threshold);
            Self::deposit_event(RawEvent::RelayerThresholdChanged(threshold));
            Ok(())
        }

        /// Adds a new relayer to the relayer set.
        ///
        /// # <weight>
        /// - O(1) lookup and insert
        /// # </weight>
        #[weight = 10_000]
        pub fn add_relayer(origin, who: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            ensure_root(origin)?;
            let relayer = T::Lookup::lookup(who)?;
            ensure!(!Self::is_relayer(&relayer), Error::<T>::RelayerAlreadyExists);

            <Relayers<T>>::insert(&relayer, true);
            <RelayerCount>::mutate(|i| *i += 1);
    
            Self::deposit_event(RawEvent::RelayerAdded(relayer));
            Ok(())
        }

        /// Removes an existing relayer from the set.
        #[weight = 10_000]
        pub fn remove_relayer(origin, dest: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            ensure_root(origin)?;
            let relayer = T::Lookup::lookup(dest)?;
            ensure!(Self::is_relayer(&relayer), Error::<T>::RelayerInvalid);

            <Relayers<T>>::remove(&relayer);
            <RelayerCount>::mutate(|i| *i -= 1);

            Self::deposit_event(RawEvent::RelayerRemoved(relayer));
            Ok(())
        }

        /// Commits a vote in favour of the bond.
        #[weight = 100_000]
        pub fn votebond(origin, bond_id: T::Hash, in_favour: bool, reason: OpposeReason) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Self::is_relayer(&who), Error::<T>::MustBeRelayer);
            let op_record = <BondRecords<T>>::get(&bond_id);
            ensure!(op_record.is_some(), Error::<T>::BondRecordNotFound);

            let op_vote = <BondVotes<T>>::get(&bond_id);
            ensure!(op_vote.is_some(), Error::<T>::BondVoteNotFound);

            let mut vote = op_vote.unwrap();

            let now = system::Module::<T>::block_number();

            // Ensure the vote isn't complete and relayer hasn't already voted
            ensure!(!vote.is_completed(), Error::<T>::BondVoteAlreadyCompleted);
            ensure!(!vote.has_voted(&who), Error::<T>::RelayerAlreadyVoted);
            ensure!(!vote.is_expired(now), Error::<T>::BondVoteExpired);

            if in_favour {
                vote.votes_for.push(who.clone());
                Self::deposit_event(RawEvent::VoteFor(who, bond_id));
            } else {
                vote.votes_against.push(who.clone());
                Self::deposit_event(RawEvent::VoteAgainst(who, bond_id));
            }
            
            vote.derivate(<RelayerThreshold>::get(), <RelayerCount>::get());
            if vote.is_approved() {
                let r = op_record.unwrap();
                T::RCurrency::mint(&r.bonder, r.symbol, r.amount)?;
            }

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    /// Checks if who is a relayer
    pub fn is_relayer(who: &T::AccountId) -> bool {
        Self::relayers(who)
    }
}