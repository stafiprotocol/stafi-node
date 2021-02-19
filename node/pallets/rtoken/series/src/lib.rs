// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchResult}, ensure,
    traits::{EnsureOrigin, Get}
};

use frame_system::{self as system, ensure_signed, ensure_root};
use sp_runtime::traits::Hash;
use rtoken_balances::{traits::{Currency as RCurrency}};
use node_primitives::RSymbol;

#[cfg(test)]
mod tests;

pub mod models;
pub use models::*;

pub mod signature;
pub use signature::*;

pub trait Trait: system::Trait + rtoken_rate::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// Specifies the origin check provided by the voter for calls that can only be called by the votes pallet
    type VoterOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;

    /// currency of rtoken
    type RCurrency: RCurrency<Self::AccountId>;
}

decl_event! {
    pub enum Event<T> where
        Hash = <T as system::Trait>::Hash,
        <T as frame_system::Trait>::AccountId
    {
        /// LiquidityBond
        LiquidityBond(AccountId, RSymbol, Hash),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// pool already added
        PoolAlreadyAdded,
        /// pool not found
        PoolNotFound,
        /// liquidity bond Zero
        LiquidityBondZero,
        /// blockhash and txhash already bonded
        HashAlreadyBonded,
        /// bondrepeated
        BondRepeated,
        /// Pubkey invalid
        InvalidPubkey,
        /// Signature invalid
        InvalidSignature,
        /// Got an overflow after adding
        OverFlow,
        /// bondrecord not found
        BondNotFound,
        /// bondrecord processing
        BondProcessing,
    }
}



decl_storage! {
    trait Store for Module<T: Trait> as RTokenSeries {
        /// Pools of rsymbol
        pub Pools get(fn pools): map hasher(blake2_128_concat) RSymbol => Vec<Vec<u8>>;
        /// (hash, rsymbol) => record
        pub BondRecords get(fn bond_records): map hasher(blake2_128_concat) BondKey<T::Hash> => Option<BondRecord<T::AccountId>>;
        pub BondReasons get(fn bond_reasons): map hasher(blake2_128_concat) BondKey<T::Hash> => Option<BondReason>;
        pub AccountBondCount get(fn account_bond_count): map hasher(twox_64_concat) T::AccountId => u64;
        pub AccountBondRecords get(fn account_bond_records): map hasher(twox_64_concat) (T::AccountId, u64) => Option<BondKey<T::Hash>>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// add new pool
        #[weight = 10_000]
        pub fn add_new_pool(origin, symbol: RSymbol, pool: Vec<u8>) -> DispatchResult {
            ensure_root(origin)?;
            let mut pools = Self::pools(symbol);
            ensure!(!pools.contains(&pool), Error::<T>::PoolAlreadyAdded);
            pools.push(pool);
            Pools::insert(symbol, pools);

            Ok(())
        }

        /// liquidity bond token to get rtoken
        #[weight = 100_000_000]
        pub fn liquidity_bond(origin, pubkey: Vec<u8>, signature: Vec<u8>, pool: Vec<u8>, blockhash: Vec<u8>, txhash: Vec<u8>, amount: u128, symbol: RSymbol) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(amount > 0, Error::<T>::LiquidityBondZero);
            ensure!(Self::pools(symbol).contains(&pool), Error::<T>::PoolNotFound);

            match verify_signature(symbol, &pubkey, &signature, &txhash) {
                SigVerifyResult::InvalidPubkey => Err(Error::<T>::InvalidPubkey)?,
                SigVerifyResult::Fail => Err(Error::<T>::InvalidSignature)?,
                _ => (),
            }

            let record = BondRecord::new(who.clone(), symbol, pubkey.clone(), pool.clone(), blockhash.clone(), txhash.clone(), amount);
            let bond_id = <T::Hashing as Hash>::hash_of(&record);
            let bondkey = BondKey::new(symbol, bond_id);
            ensure!(Self::bond_records(&bondkey).is_none(), Error::<T>::BondRepeated);
            let old_count = Self::account_bond_count(&who);
            let new_count = old_count.checked_add(1).ok_or(Error::<T>::OverFlow)?;
            

            <AccountBondCount<T>>::insert(&who, new_count);
            <AccountBondRecords<T>>::insert((&who, new_count), &bondkey);
            <BondRecords<T>>::insert(&bondkey, &record);

            Self::deposit_event(RawEvent::LiquidityBond(who, symbol, bond_id));
            Ok(())
        }

        /// execute bond record
        #[weight = 100_000]
        pub fn execute_bond_record(origin, symbol: RSymbol, bond_id: T::Hash, reason: BondReason) -> DispatchResult {
            T::VoterOrigin::ensure_origin(origin)?;
            let bondkey = BondKey::new(symbol, bond_id);
            let op_record = Self::bond_records(&bondkey);
            ensure!(op_record.is_some(), Error::<T>::BondNotFound);
            let record = op_record.unwrap();

            if reason != BondReason::Pass {
                <BondReasons<T>>::insert(&bondkey, reason);
                return Ok(())
            }

            let rbalance = rtoken_rate::Module::<T>::token_to_rtoken(symbol, record.amount);
            T::RCurrency::mint(&record.bonder, symbol, rbalance)?;
            <BondReasons<T>>::insert(&bondkey, reason);
            // Self::bond_extra(&controller, &mut ledger, value);

            Ok(())
        }
    }
}