// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchResult}, ensure,
    traits::{Currency}
};

use frame_system::{self as system, ensure_signed};
use node_primitives::{RSymbol, ChainType};
use rdexn_payers as payers;

#[cfg(test)]
mod tests;

pub const MAX_UNLOCKING_CHUNKS: usize = 64;

pub trait Trait: system::Trait + rtoken_rate::Trait + payers::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// The currency mechanism.
    type Currency: Currency<Self::AccountId>;
}

decl_event! {
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId
    {
        /// submit signatures
        SubmitSignatures(AccountId, RSymbol, u32, Vec<u8>, Vec<u8>),
        /// signatures enough
        SignaturesEnough(RSymbol, u32, Vec<u8>),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// signature repeated
        SignatureRepeated,
        /// invalid symbol
        InvalidRSymbol,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RDexnSignatures {
        pub Signatures get(fn signatures): double_map hasher(blake2_128_concat) RSymbol, hasher(blake2_128_concat) (u32, Vec<u8>) => Option<Vec<Vec<u8>>>;
        pub AccountSignature get(fn account_signature): map hasher(blake2_128_concat) (T::AccountId, RSymbol, u32, Vec<u8>) => Option<Vec<u8>>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Submit tx signatures
        #[weight = 10_000_000]
        pub fn submit_signatures(origin, symbol: RSymbol, era: u32, proposal_id: Vec<u8>, signature: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(symbol.chain_type() != ChainType::Substrate, Error::<T>::InvalidRSymbol);
            ensure!(payers::Module::<T>::is_payer(symbol, &who), payers::Error::<T>::MustBePayer);


            ensure!(Self::account_signature((&who, symbol, era, &proposal_id)).is_none(), Error::<T>::SignatureRepeated);

            let mut signatures = Signatures::get(symbol, (era, &proposal_id)).unwrap_or(vec![]);
            ensure!(!signatures.contains(&signature), Error::<T>::SignatureRepeated);

            signatures.push(signature.clone());
            Signatures::insert(symbol, (era, &proposal_id), &signatures);

            <AccountSignature<T>>::insert((&who, symbol, era, &proposal_id), &signature);

            if signatures.len() == payers::PayerThreshold::get(symbol) as usize {
                Self::deposit_event(RawEvent::SignaturesEnough(symbol, era, proposal_id.clone()));
            }

            Self::deposit_event(RawEvent::SubmitSignatures(who.clone(), symbol, era, proposal_id, signature));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
}