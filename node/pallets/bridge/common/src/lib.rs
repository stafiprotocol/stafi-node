// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{EnsureOrigin, Get},
};

use frame_system::{self as system, ensure_signed, ensure_root};
use sp_core::U256;
use sp_runtime::traits::{AccountIdConversion};
use sp_runtime::{ModuleId};
use node_primitives::{ChainId, Balance};

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

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// Origin used to administer the pallet
    type AdminOrigin: EnsureOrigin<Self::Origin>;
    /// The identifier for this chain.
    /// This must be unique and must not collide with existing IDs within a set of bridged chains.
    type ChainIdentity: Get<ChainId>;
}

decl_event! {
    pub enum Event<T> where
        AccountId = <T as system::Trait>::AccountId 
    {
        /// Chain now available for transfers (chain_id)
        ChainWhitelisted(ChainId),
        /// FunglibleTransfer is for relaying fungibles (AccountId, dest_id, nonce, resource_id, amount, recipient, metadata)
        FungibleTransfer(AccountId, ChainId, DepositNonce, ResourceId, U256, Vec<u8>),
        /// Set Chain fees
        ChainFeesSet(ChainId, Balance),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Provided chain Id is not valid
        InvalidChainId,
        /// Interactions with this chain is not permitted
        ChainNotWhitelisted,
        /// Chain has already been enabled
        ChainAlreadyWhitelisted,
        /// Provided proxy account is not valid
        InvalidProxyAccount,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as BridgeCommon {
        /// All whitelisted chains and their respective transaction counts
        pub ChainNonces get(fn chains): map hasher(twox_64_concat) ChainId => Option<DepositNonce>;

        /// All whitelisted chains and their respective transaction fees
        pub ChainFees get(fn chain_fees): map hasher(twox_64_concat) ChainId => Option<Balance>;

        /// Proxy accounts for setting chain fees
        ProxyAccounts get(fn proxy_accounts): map hasher(twox_64_concat) T::AccountId => Option<u8>;

        /// Recipient account for fees
        FeesRecipientAccount get(fn fees_recipient_account): Option<T::AccountId>;

        /// True if the bridge is paused.
		pub IsPaused get(fn is_paused): bool = false;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        const ChainIdentity: ChainId = T::ChainIdentity::get();
        const BridgeAccountId: T::AccountId = MODULE_ID.into_account();

        fn deposit_event() = default;

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
}
