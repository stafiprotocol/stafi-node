// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{prelude::*, convert::TryFrom};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchResult},
    ensure,
    traits::Get,
};

use frame_system::{
    self as system, ensure_signed,
    offchain::{SendTransactionTypes, AppCrypto},
};
use sp_runtime::traits::Hash;
use rtoken_balances::{traits::{Currency as RCurrency}};
use node_primitives::{RSymbol, report::ReporterAppCrypto};
use sp_core::sr25519::{Public, Signature};
use rtoken_votes_bond::{self as votesbond, BondVote, BondRecord};

const SYMBOL: RSymbol = RSymbol::RDOT;

pub trait Trait: system::Trait + SendTransactionTypes<Call<Self>> + rtoken_rate::Trait +  votesbond::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// currency of rtoken
    type RCurrency: RCurrency<Self::AccountId>;

    // /// A configuration for base priority of unsigned transactions.
    // type UnsignedPriority: Get<TransactionPriority>;
}

decl_event! {
    pub enum Event<T> where
        Hash = <T as system::Trait>::Hash,
        <T as frame_system::Trait>::AccountId
    {
        /// LiquidityBond
        LiquidityBond(AccountId, Hash, Vec<u8>),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// liquidity bond Zero
        LiquidityBondZero,
        /// blockhash and txhash already bonded
        HashAlreadyBonded,
        /// Pubkey invalid
        InvalidPubkey,
        /// Signature invalid
        InvalidSignature,

    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RDot {

    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// liquidity bond dot to get rdot
        /// todo: add param "to" and check if it is in pools
        #[weight = 100_000_000]
        pub fn liquidity_bond(origin, pubkey: Vec<u8>, sig_data: Vec<u8>, blockhash: Vec<u8>, txhash: Vec<u8>, amount: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(amount > 0, Error::<T>::LiquidityBondZero);

            let public = <Public as TryFrom<_>>::try_from(&pubkey[..]).ok().ok_or(Error::<T>::InvalidPubkey)?;
            let sig = Signature::from_slice(&sig_data);
            let vrf_result = <ReporterAppCrypto as AppCrypto<_,_>>::verify(&txhash, public.into(), sig.into());
            ensure!(vrf_result, Error::<T>::InvalidSignature);

            let record = BondRecord::new(who.clone(), SYMBOL, blockhash.clone(), txhash.clone(), amount);
            let bond_id = <T::Hashing as Hash>::hash_of(&record);
            let op_vote = votesbond::BondVotes::<T>::get(&bond_id);
            let now = system::Module::<T>::block_number();
            let expiry = now + T::VoteLifetime::get();
            if op_vote.is_none() {
                let vote = BondVote::new(bond_id, expiry);                
                votesbond::BondVotes::<T>::insert(&bond_id, &vote);
                votesbond::BondRecords::<T>::insert(&bond_id, &record);
                let mut bids = votesbond::AccountBondIds::<T>::get(&who).unwrap_or(vec![]);
                ensure!(!bids.contains(&bond_id), votesbond::Error::<T>::BondIdRepeated);
                bids.push(bond_id.clone());
                votesbond::AccountBondIds::<T>::insert(&who, bids);
            } else {
                let mut vote = op_vote.unwrap();
                ensure!(!vote.is_approved(), votesbond::Error::<T>::BondAlreadyApproved);
                ensure!(!vote.is_rejected(), votesbond::Error::<T>::BondAlreadyRejected);
                ensure!(vote.is_expired(now), votesbond::Error::<T>::BondVoteAlive);
                vote.set_expiry(expiry);
            }

            Self::deposit_event(RawEvent::LiquidityBond(who, bond_id, pubkey));
            Ok(())
        }
    }
}