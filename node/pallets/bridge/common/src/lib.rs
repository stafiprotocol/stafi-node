// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{EnsureOrigin, Get},
};

use frame_system::{self as system, ensure_root};
use sp_core::U256;
use sp_runtime::traits::{AccountIdConversion};
use sp_runtime::{ModuleId};
use node_primitives::ChainId;

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
        /// FunglibleTransfer is for relaying fungibles (dest_id, nonce, resource_id, amount, recipient, metadata)
        FungibleTransfer(AccountId, ChainId, DepositNonce, ResourceId, U256, Vec<u8>),
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
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as ChainBridge {
        /// All whitelisted chains and their respective transaction counts
        ChainNonces get(fn chains): map hasher(opaque_blake2_256) ChainId => Option<DepositNonce>;
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
