// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchResult}, ensure,
    traits::{EnsureOrigin, Get}
};

use frame_system::{self as system, ensure_signed, ensure_root};
use sp_runtime::{
    Perbill,
    traits::Hash
};
use rtoken_balances::{traits::{Currency as RCurrency}};
use node_primitives::RSymbol;

#[cfg(test)]
mod tests;

pub mod models;
pub use models::*;

pub mod signature;
pub use signature::*;

pub const MAX_UNLOCKING_CHUNKS: usize = 16;

pub trait Trait: system::Trait + rtoken_rate::Trait + rtoken_ledger::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

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
        /// liquidity unbond record
        LiquidityUnBond(AccountId, Vec<u8>, u128, u128, u128),
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
        /// liquidity unbond Zero
        LiquidityUnbondZero,
        /// get current era err
        NoCurrentEra,
        /// era rate not updated
        EraRateNotUpdated,
        /// insufficient balance
        InsufficientBalance,
        /// Can not schedule more unlock chunks.
        NoMoreChunks,
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
        /// Recipient account for fees
        Receiver get(fn receiver): Option<T::AccountId>;
        /// Unbonding: (origin, pool) => [UnlockChunks]
        pub Unbonding get(fn unbonding): double_map hasher(twox_64_concat) T::AccountId, hasher(twox_64_concat) (RSymbol, Vec<u8>) => Option<Vec<BondUnlockChunk>>;
        pub TotalUnbonding get(fn total_unbonding): map hasher(twox_64_concat) (RSymbol, u32) => u128;

        /// Unbond commission
        UnbondCommission get(fn unbond_commission): Perbill = Perbill::from_parts(2000000);
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

        /// set receiver
        #[weight = 10_000]
        pub fn set_receiver(origin, new_receiver: T::AccountId) -> DispatchResult {
            ensure_root(origin)?;
            <Receiver<T>>::put(new_receiver);
            Ok(())
        }

        /// set unbond commission
        #[weight = 10_000]
        pub fn set_unbond_commission(origin, new_part: u32) -> DispatchResult {
            ensure_root(origin)?;
            let new_commission = Perbill::from_parts(new_part);
            UnbondCommission::put(new_commission);

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

        /// liquitidy unbond to redeem token with rtoken
        #[weight = 1_000_000_000]
        pub fn liquidity_unbond(origin, pool: Vec<u8>, value: u128, symbol: RSymbol) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(value > 0, Error::<T>::LiquidityUnbondZero);
            let op_receiver = Self::receiver();
            ensure!(op_receiver.is_some(), "No receiver to get unbond commission fee");
            ensure!(Self::pools(symbol).contains(&pool), Error::<T>::PoolNotFound);

            let current_era = rtoken_ledger::ChainEras::get(symbol).ok_or(Error::<T>::NoCurrentEra)?;
            ensure!(rtoken_rate::EraRate::get(symbol, current_era).is_some(), Error::<T>::EraRateNotUpdated);

            let free = T::RCurrency::free_balance(&who, symbol);
            free.checked_sub(value).ok_or(Error::<T>::InsufficientBalance)?;

            let mut unbonding = <Unbonding<T>>::get(&who, (symbol, &pool)).unwrap_or(vec![]);
            ensure!(unbonding.len() < MAX_UNLOCKING_CHUNKS, Error::<T>::NoMoreChunks);

            let fee = Self::unbond_fee(value);
            let left_value = value - fee;
            let balance = rtoken_rate::Module::<T>::rtoken_to_token(symbol, left_value);

            // TODO
            // ensure!(pool_active >= balance, Error::<T>::InsufficientBalance);

            let old_total_unbonding_value = Self::total_unbonding((symbol, current_era));
            let new_total_unbonding_value = old_total_unbonding_value.checked_add(balance).ok_or(Error::<T>::OverFlow)?;
            
            let era = Self::unlocking_era(symbol, current_era);
            if let Some(chunk) = unbonding.iter_mut().find(|chunk| chunk.era == era) {
                chunk.value += balance;
            } else {
                unbonding.push(BondUnlockChunk { value: balance, era });
            }

            let receiver = op_receiver.unwrap();
            T::RCurrency::transfer(&who, &receiver, symbol, fee)?;
            T::RCurrency::burn(&who, symbol, left_value)?;
            <Unbonding<T>>::insert(&who, (symbol, &pool), unbonding);
            TotalUnbonding::insert((symbol, current_era), new_total_unbonding_value);

            Self::deposit_event(RawEvent::LiquidityUnBond(who, pool, value, left_value, balance));

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn unbond_fee(value: u128) -> u128 {
        Self::unbond_commission() * value
    }

    fn unlocking_era(symbol: RSymbol, current_era: u32) -> u32 {
        match symbol {
            RSymbol::RDOT => current_era + 30,
            _ => current_era + 58,
        }
    }
}