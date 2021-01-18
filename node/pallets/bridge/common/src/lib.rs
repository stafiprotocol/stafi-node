// Copyright 2019-2020 Stafi Protocol.
// This file is part of Stafi.

// Stafi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stafi.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use codec::{Decode, Encode, EncodeLike};
use frame_support::{
    Parameter, decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult, ensure,
    traits::{EnsureOrigin, Get},
    weights::{GetDispatchInfo, Pays},
};

use frame_system::{self as system, ensure_signed, ensure_root};
use sp_core::U256;
use sp_runtime::{
    RuntimeDebug, ModuleId,
    traits::{AccountIdConversion, StaticLookup, Dispatchable}
};
use node_primitives::{ChainId, Balance, RSymbol};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

const DEFAULT_RELAYER_THRESHOLD: u32 = 1;
const MODULE_ID: ModuleId = ModuleId(*b"cb/bridg");

pub type DepositNonce = u64;
pub type ResourceId = [u8; 32];

/// Helper function to concatenate a chain ID and some bytes to produce a resource ID.
/// The common format is (31 bytes unique ID + 1 byte chain ID).
pub fn derive_resource_id(chain: ChainId, id: &[u8]) -> ResourceId {
    let mut r_id: ResourceId = [0; 32];
    r_id[31] = chain; // last byte is chain id
    let range = if id.len() > 31 { 31 } else { id.len() }; // Use at most 31 bytes
    for i in 0..range {
        r_id[30 - i] = id[range - 1 - i]; // Ensure left padding for eth compatibility
    }
    return r_id;
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub enum ProposalStatus {
    Active,
	Passed,
    Expired,
    Executed,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct ProposalVotes<AccountId, BlockNumber> {
    pub voted: Vec<AccountId>,
    pub status: ProposalStatus,
    pub expiry: BlockNumber,
}

impl<A: PartialEq, B: PartialOrd + Default> ProposalVotes<A, B> {
    /// derivate next status according to threshold and now
    fn derivate(&mut self, threshold: u32, now: B) -> ProposalStatus {
        if self.is_completed() {
            self.status.clone()
        } else if self.expiry <= now {
            self.status = ProposalStatus::Expired;
            ProposalStatus::Expired
        } else if self.voted.len() >= threshold as usize {
            self.status = ProposalStatus::Passed;
            ProposalStatus::Passed
        } else {
            self.status.clone()
        }
    }

    /// Returns true if the proposal has been rejected or approved, otherwise false.
    fn is_completed(&self) -> bool {
        self.status == ProposalStatus::Executed || self.status == ProposalStatus::Expired
    }

    /// Returns true if `who` has voted for or against the proposal
    fn has_voted(&self, who: &A) -> bool {
        self.voted.contains(&who)
    }

    /// Return true if the expiry time has been reached
    fn is_expired(&self, now: B) -> bool {
        self.expiry <= now
    }

    /// Set status to Executed
    fn to_be_executed(&mut self) {
        self.status = ProposalStatus::Executed;
    }
}

impl<AccountId, BlockNumber: Default> Default for ProposalVotes<AccountId, BlockNumber> {
    fn default() -> Self {
        Self {
            voted: vec![],
            status: ProposalStatus::Active,
            expiry: BlockNumber::default(),
        }
    }
}

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// Origin used to administer the pallet
    type AdminOrigin: EnsureOrigin<Self::Origin>;
    /// Proposed dispatchable call
    type Proposal: Parameter + Dispatchable<Origin = Self::Origin> + EncodeLike + GetDispatchInfo;
    /// The identifier for this chain.
    /// This must be unique and must not collide with existing IDs within a set of bridged chains.
    type ChainIdentity: Get<ChainId>;

    type ProposalLifetime: Get<Self::BlockNumber>;
}

decl_event! {
    pub enum Event<T> where
        AccountId = <T as system::Trait>::AccountId 
    {
        /// Vote threshold has changed (new_threshold)
        RelayerThresholdChanged(u32),
        /// Relayer added to set
        RelayerAdded(AccountId),
        /// Relayer removed from set
        RelayerRemoved(AccountId),
        /// Chain now available for transfers (chain_id)
        ChainWhitelisted(ChainId),
        /// Chain now unavailable
        ChainRemoved(ChainId),
        /// FunglibleTransfer is for relaying fungibles (AccountId, dest_id, nonce, resource_id, amount, recipient, metadata)
        FungibleTransfer(AccountId, ChainId, DepositNonce, ResourceId, U256, Vec<u8>),
        /// Set Chain fees
        ChainFeesSet(ChainId, Balance),
        /// Vote submitted in favour of proposal
        VoteFor(ChainId, DepositNonce, AccountId),
        /// Vot submitted against proposal
        VoteAgainst(ChainId, DepositNonce, AccountId),
        /// Voting successful for a proposal
        ProposalPassed(ChainId, DepositNonce),
        /// Voting rejected a proposal
        ProposalCancelled(ChainId, DepositNonce),
        /// Execution of call succeeded
        ProposalExecuted(ChainId, DepositNonce),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Relayer threshold not set
        ThresholdNotSet,
        /// Relayer threshold should larger than 0
        InvalidThreshold,
        /// Provided chain Id is not valid
        InvalidChainId,
        /// Interactions with this chain is not permitted
        ChainNotWhitelisted,
        /// Chain has already been enabled
        ChainAlreadyWhitelisted,
        /// Provided proxy account is not valid
        InvalidProxyAccount,
        /// Resource ID provided isn't mapped to anything
        ResourceDoesNotExist,
        /// Relayer already in set
        RelayerAlreadyExists,
        /// Provided accountId is not a relayer
        RelayerInvalid,
        /// Protected operation, must be performed by relayer
        MustBeRelayer,
        /// Relayer has already submitted some vote for this proposal
        RelayerAlreadyVoted,
        /// A proposal with these parameters has already been submitted
        ProposalAlreadyExists,
        /// No proposal with the ID was found
        ProposalDoesNotExist,
        /// Cannot complete proposal, needs more votes
        ProposalNotPassed,
        /// Proposal has either failed or succeeded
        ProposalAlreadyCompleted,
        /// Lifetime of proposal has been exceeded
        ProposalExpired,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as BridgeCommon {
        /// All whitelisted chains and their respective transaction counts
        pub ChainNonces get(fn chains): map hasher(twox_64_concat) ChainId => Option<DepositNonce>;

        /// fee to cover the commission happened on other chains such as ethereum
        pub ChainFees get(fn chain_fees): map hasher(twox_64_concat) ChainId => Option<Balance>;

        /// Proxy accounts for setting chain fees
        ProxyAccounts get(fn proxy_accounts): map hasher(twox_64_concat) T::AccountId => Option<u8>;

        /// Recipient account for fees
        FeesRecipientAccount get(fn fees_recipient_account): Option<T::AccountId>;

        /// True if the bridge is paused.
        pub IsPaused get(fn is_paused): bool = false;
        
        /// Number of votes required for a proposal to execute
        RelayerThreshold get(fn relayer_threshold): u32 = DEFAULT_RELAYER_THRESHOLD;

        /// Tracks current relayer set
        pub Relayers get(fn relayers): map hasher(blake2_128_concat) T::AccountId => bool;

        /// Number of relayers in set
        pub RelayerCount get(fn relayer_count): u32;

        /// All known proposals.
        /// The key is the hash of the call and the deposit ID, to ensure it's unique.
        pub Votes get(fn votes):
            double_map hasher(blake2_128_concat) ChainId, hasher(blake2_128_concat) (DepositNonce, T::Proposal)
            => Option<ProposalVotes<T::AccountId, T::BlockNumber>>;

        /// Utilized by the bridge software to map resource IDs to actual methods
        pub Resources get(fn resources): map hasher(blake2_128_concat) ResourceId => Option<Vec<u8>>;

        /// rId => Rsymbol
        pub ResourceRsymbol get(fn resource_rsymbol): map hasher(blake2_128_concat) ResourceId => Option<RSymbol>;
        /// Rsymbol => ResourceId
        pub RsymbolResource get(fn rsymbol_resource): map hasher(blake2_128_concat) RSymbol => Option<ResourceId>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        const ChainIdentity: ChainId = T::ChainIdentity::get();
        const BridgeAccountId: T::AccountId = MODULE_ID.into_account();
        const ProposalLifetime: T::BlockNumber = T::ProposalLifetime::get();

        fn deposit_event() = default;

        /// Sets the vote threshold for proposals.
        ///
        /// This threshold is used to determine how many votes are required
        /// before a proposal is executed.
        ///
        /// # <weight>
        /// - O(1) lookup and insert
        /// # </weight>
        #[weight = 10_000]
        pub fn set_threshold(origin, threshold: u32) -> DispatchResult {
            Self::ensure_admin(origin)?;
            ensure!(threshold > 0, Error::<T>::InvalidThreshold);
            <RelayerThreshold>::put(threshold);
            Self::deposit_event(RawEvent::RelayerThresholdChanged(threshold));
            Ok(())
        }

        /// Stores a method name on chain under an associated resource ID.
        ///
        /// # <weight>
        /// - O(1) write
        /// # </weight>
        #[weight = 195_000_000]
        pub fn add_resource(origin, id: ResourceId, method: Vec<u8>) -> DispatchResult {
            Self::ensure_admin(origin)?;
            <Resources>::insert(id, method);
            Ok(())
        }

        /// Removes a resource ID from the resource mapping.
        ///
        /// After this call, bridge transfers with the associated resource ID will
        /// be rejected.
        ///
        /// # <weight>
        /// - O(1) removal
        /// # </weight>
        #[weight = 195_000_000]
        pub fn remove_resource(origin, id: ResourceId) -> DispatchResult {
            Self::ensure_admin(origin)?;
            <Resources>::remove(id);
            Ok(())
        }

        /// Adds a new relayer to the relayer set.
        ///
        /// # <weight>
        /// - O(1) lookup and insert
        /// # </weight>
        #[weight = 10_000]
        pub fn add_relayer(origin, who: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            Self::ensure_admin(origin)?;
            let relayer = T::Lookup::lookup(who)?;
            ensure!(!Self::is_relayer(&relayer), Error::<T>::RelayerAlreadyExists);

            <Relayers<T>>::insert(&relayer, true);
            <RelayerCount>::mutate(|i| *i += 1);
    
            Self::deposit_event(RawEvent::RelayerAdded(relayer));
            Ok(())
        }

        /// Removes an existing relayer from the set.
        ///
        /// # <weight>
        /// - O(1) lookup and removal
        /// # </weight>
        #[weight = 10_000]
        pub fn remove_relayer(origin, dest: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            Self::ensure_admin(origin)?;
            let relayer = T::Lookup::lookup(dest)?;
            ensure!(Self::is_relayer(&relayer), Error::<T>::RelayerInvalid);

            <Relayers<T>>::remove(&relayer);
            <RelayerCount>::mutate(|i| *i -= 1);

            Self::deposit_event(RawEvent::RelayerRemoved(relayer));
            Ok(())
        }

        /// Map resourceId to Rsymbol
        #[weight = 10_000]
        pub fn map_resource_and_rsymbol(origin, resource_id: ResourceId, sym: RSymbol) -> DispatchResult {
            Self::ensure_admin(origin)?;

            <ResourceRsymbol>::insert(&resource_id, &sym);
            <RsymbolResource>::insert(&sym, &resource_id);

            Ok(())
        }

        /// Unmap resourceId to Rsymbol
        #[weight = 10_000]
        pub fn unmap_resource_and_rsymbol(origin, resource_id: ResourceId, sym: RSymbol) -> DispatchResult {
            Self::ensure_admin(origin)?;

            <ResourceRsymbol>::remove(&resource_id);
            <RsymbolResource>::remove(&sym);

            Ok(())
        }

        /// Commits a vote in favour of the provided proposal.
        ///
        /// If a proposal with the given nonce and source chain ID does not already exist, it will
        /// be created with an initial vote in favour from the caller.
        ///
        /// # <weight>
        /// - weight of proposed call, regardless of whether execution is performed
        /// # </weight>
        #[weight = (call.get_dispatch_info().weight + 195_000_000, call.get_dispatch_info().class, Pays::Yes)]
        pub fn acknowledge_proposal(origin, nonce: DepositNonce, src_id: ChainId, resource_id: ResourceId, call: Box<T::Proposal>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Self::is_relayer(&who), Error::<T>::MustBeRelayer);
            ensure!(Self::chain_whitelisted(src_id), Error::<T>::ChainNotWhitelisted);
            ensure!(Self::resources(resource_id).is_some(), Error::<T>::ResourceDoesNotExist);

            Self::commit_vote(who, nonce, src_id, call.clone())?;
            Self::try_resolve_proposal(nonce, src_id, call)
        }

        /// Enables a chain ID as a source or destination for a bridge transfer.
        ///
        /// # <weight>
        /// - O(1) lookup and insert
        /// # </weight>
        #[weight = 195_000_000]
        pub fn whitelist_chain(origin, id: ChainId) -> DispatchResult {
            Self::ensure_admin(origin)?;
            Self::whitelist(id)
        }

        /// Enables a chain ID as a source or destination for a bridge transfer.
        ///
        /// # <weight>
        /// - O(1) lookup and insert
        /// # </weight>
        #[weight = 195_000_000]
        pub fn remove_whitelist_chain(origin, id: ChainId) -> DispatchResult {
            Self::ensure_admin(origin)?;
            ensure!(Self::chain_whitelisted(id), Error::<T>::ChainNotWhitelisted);
            <ChainNonces>::remove(id);

            Self::deposit_event(RawEvent::ChainRemoved(id));
            Ok(())
        }

        /// Set proxy accounts.
        ///
        /// # <weight>
        /// - O(1) lookup and insert
        /// # </weight>
        #[weight = 100_000_000]
        pub fn set_proxy_accounts(origin, account: T::AccountId) -> DispatchResult {
            Self::ensure_admin(origin)?;
            <ProxyAccounts<T>>::insert(account, 0);

            Ok(())
        }

        /// Remove proxy accounts.
        ///
        /// # <weight>
        /// - O(1) lookup and insert
        /// # </weight>
        #[weight = 100_000_000]
        pub fn remove_proxy_accounts(origin, account: T::AccountId) -> DispatchResult {
            Self::ensure_admin(origin)?;
            <ProxyAccounts<T>>::remove(account);

            Ok(())
        }

        /// Set fees for a chain ID.
        ///
        /// # <weight>
        /// - O(1) lookup and insert
        /// # </weight>
        #[weight = 100_000_000]
        pub fn set_chain_fees(origin, id: ChainId, fees: Balance) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(Self::chain_whitelisted(id), Error::<T>::InvalidChainId);
            ensure!(<ProxyAccounts<T>>::contains_key(&who), Error::<T>::InvalidProxyAccount);

            <ChainFees>::insert(id, fees);

            Self::deposit_event(RawEvent::ChainFeesSet(id, fees));
            Ok(())
        }

        /// Set fees recipient account.
        ///
        /// # <weight>
        /// - O(1) lookup and insert
        /// # </weight>
        #[weight = 100_000_000]
        pub fn set_fees_recipient_account(origin, account: T::AccountId) -> DispatchResult {
            Self::ensure_admin(origin)?;

            <FeesRecipientAccount<T>>::put(account);

            Ok(())
        }

        /// Set whether to pause.
        ///
        /// # <weight>
        /// - O(1) lookup and insert
        /// # </weight>
        #[weight = 100_000_000]
        pub fn set_is_pasued(origin, is_paused: bool) -> DispatchResult {
            Self::ensure_admin(origin)?;

            <IsPaused>::put(is_paused);

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn ensure_admin(o: T::Origin) -> DispatchResult {
        T::AdminOrigin::try_origin(o)
            .map(|_| ())
            .or_else(ensure_root)?;
        Ok(())
    }

    /// Provides an AccountId for the pallet.
    /// This is used both as an origin check and deposit/withdrawal account.
    pub fn account_id() -> T::AccountId {
        MODULE_ID.into_account()
    }

    /// Checks if a chain exists as a whitelisted destination
    pub fn chain_whitelisted(id: ChainId) -> bool {
        return Self::chains(id) != None;
    }

    /// Get chain fees
    pub fn get_chain_fees(id: ChainId) -> Option<Balance> {
        return Self::chain_fees(id);
    }

    /// Provides an AccountId for the fees.
    pub fn get_fees_recipient_account() -> Option<T::AccountId> {
        return Self::fees_recipient_account();
    }

    /// Checks if the bridge function is paused.
    pub fn check_is_paused() -> bool {
        return Self::is_paused();
    }

    /// Increments the deposit nonce for the specified chain ID
    fn bump_nonce(id: ChainId) -> DepositNonce {
        let nonce = Self::chains(id).unwrap_or_default() + 1;
        <ChainNonces>::insert(id, nonce);
        nonce
    }

    /// Whitelist a chain ID for transfer
    pub fn whitelist(id: ChainId) -> DispatchResult {
        // Cannot whitelist this chain
        ensure!(id != T::ChainIdentity::get(), Error::<T>::InvalidChainId);
        // Cannot whitelist with an existing entry
        ensure!(
            !Self::chain_whitelisted(id),
            Error::<T>::ChainAlreadyWhitelisted
        );
        <ChainNonces>::insert(&id, 0);
        Self::deposit_event(RawEvent::ChainWhitelisted(id));
        Ok(())
    }

    /// Initiates a transfer of a fungible asset out of the chain. This should be called by another pallet.
    pub fn transfer_fungible(
        source: T::AccountId,
        dest_id: ChainId,
        resource_id: ResourceId,
        to: Vec<u8>,
        amount: U256,
    ) -> DispatchResult {
        ensure!(
            Self::chain_whitelisted(dest_id),
            Error::<T>::ChainNotWhitelisted
        );
        let nonce = Self::bump_nonce(dest_id);
        Self::deposit_event(RawEvent::FungibleTransfer(
            source,
            dest_id,
            nonce,
            resource_id,
            amount,
            to,
        ));
        Ok(())
    }

    /// Checks if who is a relayer
    pub fn is_relayer(who: &T::AccountId) -> bool {
        Self::relayers(who)
    }

    /// Commits a vote for a proposal. If the proposal doesn't exist it will be created.
    fn commit_vote(who: T::AccountId, nonce: DepositNonce, src_id: ChainId, prop: Box<T::Proposal>) -> DispatchResult {
        let now = system::Module::<T>::block_number();
        let mut votes = <Votes<T>>::get(src_id, (nonce, prop.clone())).unwrap_or_else(|| {
            let mut v = ProposalVotes::default();
            v.expiry = now + T::ProposalLifetime::get();
            v
        });

        // Ensure the proposal isn't complete and relayer hasn't already voted
        ensure!(!votes.is_completed(), Error::<T>::ProposalAlreadyCompleted);
        ensure!(!votes.has_voted(&who), Error::<T>::RelayerAlreadyVoted);
        if votes.is_expired(now) {
            votes.status = ProposalStatus::Expired;
            <Votes<T>>::insert(src_id, (nonce, prop.clone()), votes.clone());
        }
        ensure!(!votes.is_expired(now), Error::<T>::ProposalExpired);

        votes.voted.push(who.clone());
        votes.derivate(Self::relayer_threshold(), now);
        <Votes<T>>::insert(src_id, (nonce, prop.clone()), votes);

        Self::deposit_event(RawEvent::VoteFor(src_id, nonce, who.clone()));
        Ok(())
    }

    /// Attempts to finalize or cancel the proposal if the vote count allows.
    fn try_resolve_proposal(nonce: DepositNonce, src_id: ChainId, prop: Box<T::Proposal>) -> DispatchResult {
        let op_votes = <Votes<T>>::get(src_id, (nonce, prop.clone()));
        ensure!(op_votes.is_some(), Error::<T>::ProposalDoesNotExist);

        let mut votes = op_votes.unwrap();
        match votes.status {
            ProposalStatus::Passed => {
                Self::deposit_event(RawEvent::ProposalPassed(src_id, nonce));
                let call = prop.clone();
                call.dispatch(system::RawOrigin::Signed(Self::account_id()).into())
                    .map(|_| ())
                    .map_err(|e| e.error)?;
                votes.to_be_executed();
                <Votes<T>>::insert(src_id, (nonce, prop), votes.clone());
                Self::deposit_event(RawEvent::ProposalExecuted(src_id, nonce));
                Ok(())
            },
            ProposalStatus::Expired => {
                Self::deposit_event(RawEvent::ProposalCancelled(src_id, nonce));
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

/// Simple ensure origin for the bridge account
pub struct EnsureBridge<T>(sp_std::marker::PhantomData<T>);
impl<T: Trait> EnsureOrigin<T::Origin> for EnsureBridge<T> {
    type Success = T::AccountId;
    fn try_origin(o: T::Origin) -> Result<Self::Success, T::Origin> {
        let bridge_id = MODULE_ID.into_account();
        o.into().and_then(|o| match o {
            system::RawOrigin::Signed(who) if who == bridge_id => Ok(bridge_id),
            r => Err(T::Origin::from(r)),
        })
    }
}
