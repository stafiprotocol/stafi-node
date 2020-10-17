// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use bridge_common::{self as bridge, ResourceId};
use frame_support::traits::{Currency, ExistenceRequirement::AllowDeath, Get};
use frame_support::{decl_error, decl_module, dispatch::DispatchResult, ensure};
use frame_system::{self as system, ensure_signed};
use sp_core::U256;
use sp_arithmetic::traits::SaturatedConversion;
use node_primitives::ChainId;

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

pub trait Trait: system::Trait + bridge::Trait {
    /// The currency mechanism.
    type Currency: Currency<Self::AccountId>;

    // Ids can be defined by the runtime and passed in, perhaps from blake2b_128 hashes.
    type NativeTokenId: Get<ResourceId>;
}

decl_error! {
    pub enum Error for Module<T: Trait>{
        InvalidTransfer,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        const NativeTokenId: ResourceId = T::NativeTokenId::get();

        /// Transfers some amount of the native token to some recipient on a (whitelisted) destination chain.
        #[weight = 1_000_000_000]
        pub fn transfer_native(origin, amount: BalanceOf<T>, recipient: Vec<u8>, dest_id: ChainId) -> DispatchResult {
            let source = ensure_signed(origin)?;
            ensure!(<bridge::Module<T>>::chain_whitelisted(dest_id), Error::<T>::InvalidTransfer);
            let bridge_id = <bridge::Module<T>>::account_id();
            T::Currency::transfer(&source, &bridge_id, amount.into(), AllowDeath)?;

            let resource_id = T::NativeTokenId::get();
            <bridge::Module<T>>::transfer_fungible(source, dest_id, resource_id, recipient, U256::from(amount.saturated_into()))
        }
    }
}
