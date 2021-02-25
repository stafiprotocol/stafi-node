// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchResult}, ensure,
    traits::{EnsureOrigin}
};

use frame_system::{self as system, ensure_signed, ensure_root};
use sp_runtime::{
    Perbill,
    traits::Hash
};
use rtoken_balances::{traits::{Currency as RCurrency}};
use node_primitives::RSymbol;
use rtoken_ledger as ledger;

#[cfg(test)]
mod tests;

pub mod models;
pub use models::*;

pub mod signature;
pub use signature::*;

pub const MAX_UNLOCKING_CHUNKS: usize = 32;
pub const MAX_WITHDRAWING_CHUNKS: usize = 100;

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
        /// liquidity withdraw unbond
        LiquidityWithdrawUnBond(AccountId, RSymbol, Vec<u8>, Vec<u8>, u128),
        /// Commission has been updated.
        CommissionUpdated(Perbill, Perbill),
        /// UnbondCommission has been updated.
        UnbondCommissionUpdated(Perbill, Perbill),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// pool not found
        PoolNotFound,
        /// liquidity bond Zero
        LiquidityBondZero,
        /// txhash already bonded
        TxhashAlreadyBonded,
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
        /// era rate already updated
        EraRateAlreadyUpdated,
        /// insufficient balance
        InsufficientBalance,
        /// Can not schedule more unlock chunks.
        NoMoreChunks,
        /// Bonding duration not set
        BondingDurationNotSet,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RTokenSeries {
        /// (hash, rsymbol) => record
        pub BondRecords get(fn bond_records): map hasher(blake2_128_concat) BondKey<T::Hash> => Option<BondRecord<T::AccountId>>;
        pub BondReasons get(fn bond_reasons): map hasher(blake2_128_concat) BondKey<T::Hash> => Option<BondReason>;
        pub AccountBondCount get(fn account_bond_count): map hasher(blake2_128_concat) T::AccountId => u64;
        pub AccountBondRecords get(fn account_bond_records): map hasher(blake2_128_concat) (T::AccountId, u64) => Option<BondKey<T::Hash>>;
        /// bond success histories. (symbol, blockhash, txhash) => bool
        pub BondSuccess get(fn bond_success): map hasher(blake2_128_concat) (RSymbol, Vec<u8>, Vec<u8>) => Option<bool>;
        /// TotalBonding: (symbol, era) => [LinkChunk]
        pub TotalBonding get(fn total_bonding): map hasher(twox_64_concat) (RSymbol, u32) => Option<Vec<LinkChunk>>;
        /// Total active balance. (symbol, pool) => u128
        pub TotalBondActiveBalance get(fn total_bond_active_balance): map hasher(twox_64_concat) (RSymbol, Vec<u8>) => u128;
        /// Recipient account for fees
        Receiver get(fn receiver): Option<T::AccountId>;
        /// Unbonding: (origin, (symbol, pool)) => [BondUnlockChunk]
        pub Unbonding get(fn unbonding): double_map hasher(blake2_128_concat) T::AccountId, hasher(twox_64_concat) (RSymbol, Vec<u8>) => Option<Vec<BondUnlockChunk>>;
        /// Total unbonding: (symbol, era) => [LinkChunk]
        pub TotalUnbonding get(fn total_unbonding): map hasher(twox_64_concat) (RSymbol, u32) => Option<Vec<LinkChunk>>;

        /// Withdrawing: (symbol, unlocking_era, index) => [WithdrawChunk]
        pub TotalWithdrawing get(fn total_withdrawing): map hasher(twox_64_concat) (RSymbol, u32, u32) => Option<Vec<WithdrawChunk<T::AccountId>>>;
        /// symbol, era => count_index
        pub TotalWithdrawingChunkCount get(fn total_withdrawing_chunk_count): map hasher(twox_64_concat) (RSymbol, u32) => u32;

        /// commission of staking rewards
        Commission get(fn commission): Perbill = Perbill::from_percent(10);

        /// Unbond commission
        UnbondCommission get(fn unbond_commission): Perbill = Perbill::from_parts(2000000);
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// set receiver
        #[weight = 10_000]
        pub fn set_receiver(origin, new_receiver: T::AccountId) -> DispatchResult {
            ensure_root(origin)?;
            <Receiver<T>>::put(new_receiver);
            Ok(())
        }

        /// Update commission
		#[weight = 10_000]
		fn set_commission(origin, new_part: u32) -> DispatchResult {
            ensure_root(origin)?;
            let old_commission = Self::commission();
            let new_commission = Perbill::from_parts(new_part);
			Commission::put(new_commission);

			Self::deposit_event(RawEvent::CommissionUpdated(old_commission, new_commission));
			Ok(())
        }

        /// set unbond commission
        #[weight = 10_000]
        pub fn set_unbond_commission(origin, new_part: u32) -> DispatchResult {
            ensure_root(origin)?;
            let old_commission = Self::unbond_commission();
            let new_commission = Perbill::from_parts(new_part);
            UnbondCommission::put(new_commission);

            Self::deposit_event(RawEvent::UnbondCommissionUpdated(old_commission, new_commission));
            Ok(())
        }

        /// execute rtoken rate
        #[weight = 100_000]
        pub fn execute_rtoken_rate(origin, symbol: RSymbol, _era: u32, total_active_balance: u128, reward: u128) -> DispatchResult {
            T::VoterOrigin::ensure_origin(origin)?;

            let current_era = rtoken_ledger::ChainEras::get(symbol).ok_or(Error::<T>::NoCurrentEra)?;
            ensure!(!rtoken_rate::EraRate::get(symbol, current_era).is_some(), Error::<T>::EraRateAlreadyUpdated);

            let op_receiver = Self::receiver();
            if reward > 0 && op_receiver.is_some() {
                let fee = Self::commission() * reward;
                let rtoken_value = rtoken_rate::Module::<T>::token_to_rtoken(symbol, fee);
                let receiver = op_receiver.unwrap();
                T::RCurrency::mint(&receiver, symbol, rtoken_value)?;
            }

            let rbalance = T::RCurrency::total_issuance(symbol);
            let rate =  rtoken_rate::Module::<T>::set_rate(symbol, total_active_balance, rbalance);
            rtoken_rate::EraRate::insert(symbol, current_era, rate);

            Ok(())
        }

        /// liquidity bond token to get rtoken
        #[weight = 100_000_000]
        pub fn liquidity_bond(origin, pubkey: Vec<u8>, signature: Vec<u8>, pool: Vec<u8>, blockhash: Vec<u8>, txhash: Vec<u8>, amount: u128, symbol: RSymbol) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(amount > 0, Error::<T>::LiquidityBondZero);
            ensure!(ledger::Pools::get(symbol).contains(&pool), ledger::Error::<T>::PoolNotFound);
            ensure!(Self::bond_success((symbol, &blockhash, &txhash)).is_none(), Error::<T>::TxhashAlreadyBonded);

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
        pub fn execute_bond_record(origin, bondkey: BondKey<T::Hash>, reason: BondReason) -> DispatchResult {
            T::VoterOrigin::ensure_origin(origin)?;
            let op_record = Self::bond_records(&bondkey);
            ensure!(op_record.is_some(), Error::<T>::BondNotFound);
            let record = op_record.unwrap();

            if reason != BondReason::Pass {
                <BondReasons<T>>::insert(&bondkey, reason);
                return Ok(())
            }

            let current_era = rtoken_ledger::ChainEras::get(record.symbol).ok_or(Error::<T>::NoCurrentEra)?;

            let rbalance = rtoken_rate::Module::<T>::token_to_rtoken(record.symbol, record.amount);
            T::RCurrency::mint(&record.bonder, record.symbol, rbalance)?;
            <BondReasons<T>>::insert(&bondkey, reason);
            <BondSuccess>::insert((record.symbol, record.blockhash.clone(), record.txhash.clone()), true);

            let mut total_bonding = Self::total_bonding((record.symbol, current_era)).unwrap_or(vec![]);
            if let Some(chunk) = total_bonding.iter_mut().find(|chunk| chunk.pool == record.pool.clone()) {
                chunk.value += record.amount;
            } else {
                total_bonding.push(LinkChunk { value: record.amount, pool: record.pool.clone() });
            }
            TotalBonding::insert((record.symbol, current_era), total_bonding);
            
            let mut total_bond_active_balance = Self::total_bond_active_balance((record.symbol, record.pool.clone()));
            total_bond_active_balance += record.amount;
            TotalBondActiveBalance::insert((record.symbol, record.pool), total_bond_active_balance);

            Ok(())
        }

        /// liquitidy unbond to redeem token with rtoken
        #[weight = 1_000_000_000]
        pub fn liquidity_unbond(origin, symbol: RSymbol, pool: Vec<u8>, value: u128, recipient: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(value > 0, Error::<T>::LiquidityUnbondZero);
            ensure!(ledger::Pools::get(symbol).contains(&pool), ledger::Error::<T>::PoolNotFound);

            let op_receiver = Self::receiver();
            ensure!(op_receiver.is_some(), "No receiver to get unbond commission fee");
            
            let current_era = rtoken_ledger::ChainEras::get(symbol).ok_or(Error::<T>::NoCurrentEra)?;
            ensure!(rtoken_rate::EraRate::get(symbol, current_era).is_some(), Error::<T>::EraRateNotUpdated);

            let free = T::RCurrency::free_balance(&who, symbol);
            free.checked_sub(value).ok_or(Error::<T>::InsufficientBalance)?;

            let mut unbonding = <Unbonding<T>>::get(&who, (symbol, &pool)).unwrap_or(vec![]);
            ensure!(unbonding.len() < MAX_UNLOCKING_CHUNKS, Error::<T>::NoMoreChunks);

            let fee = Self::unbond_fee(value);
            let left_value = value - fee;
            let balance = rtoken_rate::Module::<T>::rtoken_to_token(symbol, left_value);

            let mut total_bond_active_balance = Self::total_bond_active_balance((symbol, pool.clone()));
            ensure!(total_bond_active_balance >= balance, Error::<T>::InsufficientBalance);
            
            let bonding_duration = rtoken_ledger::ChainBondingDuration::get(symbol).ok_or(Error::<T>::BondingDurationNotSet)?;
            let unlocking_era = current_era + bonding_duration + 2;
            if let Some(chunk) = unbonding.iter_mut().find(|chunk| chunk.era == unlocking_era) {
                chunk.value += balance;
            } else {
                unbonding.push(BondUnlockChunk { value: balance, era: unlocking_era });
            }

            let mut total_unbonding = Self::total_unbonding((symbol, current_era)).unwrap_or(vec![]);
            if let Some(chunk) = total_unbonding.iter_mut().find(|chunk| chunk.pool == pool) {
                chunk.value += balance;
            } else {
                total_unbonding.push(LinkChunk { value: balance, pool: pool.clone() });
            }

            let receiver = op_receiver.unwrap();
            T::RCurrency::transfer(&who, &receiver, symbol, fee)?;
            T::RCurrency::burn(&who, symbol, left_value)?;
            <Unbonding<T>>::insert(&who, (symbol, &pool), unbonding);
            TotalUnbonding::insert((symbol, current_era), total_unbonding);

            total_bond_active_balance -= balance;
            TotalBondActiveBalance::insert((symbol, pool.clone()), total_bond_active_balance);

            Self::handle_withdraw(who.clone(), symbol, unlocking_era, pool.clone(), recipient.clone(), balance);

            Self::deposit_event(RawEvent::LiquidityUnBond(who, pool, value, left_value, balance));

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn unbond_fee(value: u128) -> u128 {
        Self::unbond_commission() * value
    }

    fn handle_withdraw(who: T::AccountId, symbol: RSymbol, unlocking_era: u32, pool: Vec<u8>, recipient: Vec<u8>, value: u128) {
        let total_withdrawing_chunk_count = Self::total_withdrawing_chunk_count((symbol, unlocking_era));
        if total_withdrawing_chunk_count > 0 {
            for i in 0..total_withdrawing_chunk_count {
                let mut old_total_withdrawing = Self::total_withdrawing((symbol, unlocking_era, i)).unwrap_or(vec![]);
                if let Some(chunk) = old_total_withdrawing.iter_mut().find(|chunk| chunk.pool == pool && chunk.recipient == recipient) {
                    chunk.value += value;
                    <TotalWithdrawing<T>>::insert((symbol, unlocking_era, i), old_total_withdrawing);
                    break;
                } else {
                    if old_total_withdrawing.len() < MAX_WITHDRAWING_CHUNKS {
                        old_total_withdrawing.push(WithdrawChunk { who: who, pool: pool, recipient: recipient, value: value });
                        <TotalWithdrawing<T>>::insert((symbol, unlocking_era, i), old_total_withdrawing);
                        break;
                    } else if i == (total_withdrawing_chunk_count - 1) {
                        TotalWithdrawingChunkCount::insert((symbol, unlocking_era), total_withdrawing_chunk_count + 1);
                        let mut new_total_withdrawing: Vec<WithdrawChunk<T::AccountId>> = vec![];
                        new_total_withdrawing.push(WithdrawChunk { who: who, pool: pool, recipient: recipient, value: value });
                        <TotalWithdrawing<T>>::insert((symbol, unlocking_era, total_withdrawing_chunk_count), new_total_withdrawing);
                        break;
                    }
                }
            }
        } else {
            TotalWithdrawingChunkCount::insert((symbol, unlocking_era), total_withdrawing_chunk_count + 1);
            let mut new_total_withdrawing: Vec<WithdrawChunk<T::AccountId>> = vec![];
            new_total_withdrawing.push(WithdrawChunk { who: who, pool: pool, recipient: recipient, value: value });
            <TotalWithdrawing<T>>::insert((symbol, unlocking_era, total_withdrawing_chunk_count), new_total_withdrawing);
        }
    }
}