#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use codec::{Decode, Encode, EncodeLike};
use frame_support::{
    Parameter, decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult, ensure,
    traits::{EnsureOrigin, Get},
    weights::{GetDispatchInfo, Pays},
};

use frame_system::{self as system, ensure_signed};
use sp_runtime::{
    RuntimeDebug, ModuleId,
    traits::{AccountIdConversion, Dispatchable}
};
use node_primitives::{RSymbol};
use rtoken_relayers as relayers;

const MODULE_ID: ModuleId = ModuleId(*b"rtk/vote");

#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum RproposalStatus {
    Initiated,
    Approved,
    Rejected,
    Expired,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct RproposalVotes<AccountId, BlockNumber> {
    pub votes_for: Vec<AccountId>,
    pub votes_against: Vec<AccountId>,
    pub status: RproposalStatus,
    pub expiry: BlockNumber,
}

impl<A: PartialEq, B: PartialOrd + Default> RproposalVotes<A, B> {
    /// derivate next status according to threshold and now
    fn derivate(&mut self, threshold: u32, total: u32) -> RproposalStatus {
        if self.votes_for.len() >= threshold as usize {
            self.status = RproposalStatus::Approved;
            RproposalStatus::Approved
        } else if total >= threshold && self.votes_against.len() as u32 + threshold > total {
            self.status = RproposalStatus::Rejected;
            RproposalStatus::Rejected
        } else {
            RproposalStatus::Initiated
        }
    }

    /// Returns true if the proposal has been rejected or approved, otherwise false.
    fn is_completed(&self) -> bool {
        self.status != RproposalStatus::Initiated
    }

    /// Returns true if `who` has voted for or against the proposal
    fn has_voted(&self, who: &A) -> bool {
        self.votes_for.contains(&who) || self.votes_against.contains(&who)
    }

    /// Return true if the expiry time has been reached
    fn is_expired(&self, now: B) -> bool {
        self.expiry <= now
    }
}

impl<AccountId, BlockNumber: Default> Default for RproposalVotes<AccountId, BlockNumber> {
    fn default() -> Self {
        Self {
            votes_for: vec![],
            votes_against: vec![],
            status: RproposalStatus::Initiated,
            expiry: BlockNumber::default(),
        }
    }
}

pub trait Trait: system::Trait + relayers::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type Proposal: Parameter + Dispatchable<Origin = Self::Origin> + EncodeLike + GetDispatchInfo;

    type ProposalLifetime: Get<Self::BlockNumber>;
}

decl_event! {
    pub enum Event<T> where
        Hash = <T as system::Trait>::Hash,
        AccountId = <T as system::Trait>::AccountId 
    {
        /// Vote submitted in favour of proposal
        VoteFor(AccountId, RSymbol, Hash),
        /// Vot submitted against proposal
        VoteAgainst(AccountId, RSymbol, Hash),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Relayer has already submitted some vote for this vote
        RelayerAlreadyVoted,
        /// No proposal with the ID was found
        ProposalDoesNotExist,
        /// Proposal has either failed or succeeded
        ProposalAlreadyCompleted,
        /// Lifetime of proposal has been exceeded
        ProposalExpired,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RTokenVotes {
        /// All known proposals.
        pub Votes get(fn votes):
            double_map hasher(blake2_128_concat) RSymbol, hasher(blake2_128_concat) (T::Hash, T::Proposal)
            => Option<RproposalVotes<T::AccountId, T::BlockNumber>>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        const ProposalLifetime: T::BlockNumber = T::ProposalLifetime::get();

        fn deposit_event() = default;

        /// Commits a vote in favour of the provided proposal.
        /// # <weight>
        /// - weight of proposed call, regardless of whether execution is performed
        /// # </weight>
        #[weight = (call.get_dispatch_info().weight + 195_000_000, call.get_dispatch_info().class, Pays::Yes)]
        pub fn acknowledge_proposal(origin, symbol: RSymbol, prop_id: T::Hash, in_favour: bool, call: Box<T::Proposal>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(relayers::Module::<T>::is_relayer(symbol, &who), relayers::Error::<T>::MustBeRelayer);

            Self::commit_vote(who, symbol, prop_id, in_favour, call.clone())?;
            Self::try_resolve_proposal(symbol, prop_id, call)
        }
    }
}

impl<T: Trait> Module<T> {
    /// Provides an AccountId for the pallet.
    /// This is used both as an origin check and deposit/withdrawal account.
    pub fn account_id() -> T::AccountId {
        MODULE_ID.into_account()
    }

    /// Commits a vote for a proposal. If the proposal doesn't exist it will be created.
    fn commit_vote(who: T::AccountId, symbol: RSymbol, prop_id: T::Hash, in_favour: bool, prop: Box<T::Proposal>) -> DispatchResult {
        let now = system::Module::<T>::block_number();
        let mut votes = <Votes<T>>::get(symbol, (prop_id, prop.clone())).unwrap_or_else(|| {
            let mut v = RproposalVotes::default();
            v.expiry = now + T::ProposalLifetime::get();
            v
        });

        // Ensure the proposal isn't complete and relayer hasn't already voted
        ensure!(!votes.is_completed(), Error::<T>::ProposalAlreadyCompleted);
        ensure!(!votes.has_voted(&who), Error::<T>::RelayerAlreadyVoted);
        if votes.is_expired(now) {
            votes.status = RproposalStatus::Expired;
            <Votes<T>>::insert(symbol, (prop_id, prop.clone()), votes.clone());
            Err(Error::<T>::ProposalExpired)?;
        }

        if in_favour {
            votes.votes_for.push(who.clone());
            Self::deposit_event(RawEvent::VoteFor(who.clone(), symbol, prop_id));
        } else {
            votes.votes_against.push(who.clone());
            Self::deposit_event(RawEvent::VoteAgainst(who.clone(), symbol, prop_id));
        }

        votes.derivate(relayers::RelayerThreshold::get(symbol), relayers::RelayerCount::get(symbol));
        <Votes<T>>::insert(symbol, (prop_id, prop.clone()), votes);

        Ok(())
    }

    /// Attempts to finalize or cancel the proposal if the vote count allows.
    fn try_resolve_proposal(symbol: RSymbol, prop_id: T::Hash, prop: Box<T::Proposal>) -> DispatchResult {
        let op_votes = <Votes<T>>::get(symbol, (prop_id, prop.clone()));
        ensure!(op_votes.is_some(), Error::<T>::ProposalDoesNotExist);

        let votes = op_votes.unwrap();
        match votes.status {
            RproposalStatus::Approved | RproposalStatus::Rejected => {
                prop.dispatch(system::RawOrigin::Signed(Self::account_id()).into())
                    .map(|_| ())
                    .map_err(|e| e.error)?;
                Ok(())
            },
            _ => Ok(()),
        }
    }
}

/// Simple ensure origin for the bridge account
pub struct EnsureVoter<T>(sp_std::marker::PhantomData<T>);
impl<T: Trait> EnsureOrigin<T::Origin> for EnsureVoter<T> {
    type Success = T::AccountId;
    fn try_origin(o: T::Origin) -> Result<Self::Success, T::Origin> {
        let voter_id = MODULE_ID.into_account();
        o.into().and_then(|o| match o {
            system::RawOrigin::Signed(who) if who == voter_id => Ok(voter_id),
            r => Err(T::Origin::from(r)),
        })
    }
}