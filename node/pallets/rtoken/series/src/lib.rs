// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchResult}, ensure,
    traits::{Currency, EnsureOrigin, ExistenceRequirement::{KeepAlive}}
};

use frame_system::{self as system, ensure_signed, ensure_root};
use sp_runtime::{
    Perbill,
    traits::Hash,
    SaturatedConversion
};
use rtoken_balances::{traits::{Currency as RCurrency}};
use node_primitives::{RSymbol, Balance, ChainType};
use rtoken_ledger::{self as ledger, Unbonding};
use rtoken_relayers as relayers;
use codec::{Encode};

#[cfg(test)]
mod tests;

pub mod models;
pub use models::*;

pub mod signature;
pub use signature::*;

pub const MAX_UNLOCKING_CHUNKS: usize = 64;
pub const MAX_WITHDRAWING_CHUNKS: usize = 100;

pub trait Trait: system::Trait + rtoken_rate::Trait + rtoken_ledger::Trait + relayers::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// The currency mechanism.
    type Currency: Currency<Self::AccountId>;
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
        LiquidityUnBond(AccountId, RSymbol, Vec<u8>, u128, u128, u128, Vec<u8>),
        /// liquidity withdraw unbond
        LiquidityWithdrawUnBond(AccountId, RSymbol, Vec<u8>, Vec<u8>, u128),
        /// UnbondCommission has been updated.
        UnbondCommissionUpdated(Perbill, Perbill),
        /// Set bond fees
        BondFeesSet(RSymbol, Balance),
        /// Set unbond fees
        UnbondFeesSet(RSymbol, Balance),
        /// Pool balance limit has been updated
        PoolBalanceLimitUpdated(RSymbol, u128, u128),
        /// submit signatures
        SubmitSignatures(AccountId, RSymbol, u32, Vec<u8>, OriginalTxType, Vec<u8>, Vec<u8>),
        /// signatures enough
        SignaturesEnough(RSymbol, u32, Vec<u8>, OriginalTxType, Vec<u8>),
        /// Nomination Updated for a pool
        NominationUpdated(RSymbol, Vec<u8>, Vec<Vec<u8>>),
        /// Validator Updated for a pool
        ValidatorUpdated(RSymbol, Vec<u8>, Vec<u8>, Vec<u8>),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// bond switch closed
        BondSwitchClosed,
        /// invalid proxy account
        InvalidProxyAccount,
        /// pool not found
        PoolNotFound,
        /// liquidity bond Zero
        LiquidityBondZero,
        /// txhash unavailable
        TxhashUnavailable,
        /// txhash unexecutable
        TxhashUnexecutable,
        /// bondrepeated
        BondRepeated,
        /// rSymbol invalid
        InvalidRSymbol,
        /// Pubkey invalid
        InvalidPubkey,
        /// Signature invalid
        InvalidSignature,
        /// Got an overflow after adding
        OverFlow,
        /// Pool limit reached
        PoolLimitReached,
        /// bondrecord not found
        BondNotFound,
        /// bondrecord processing
        BondProcessing,
        /// liquidity unbond Zero
        LiquidityUnbondZero,
        /// no more unbonding chunks
        NoMoreUnbondingChunks,
        /// get current era err
        NoCurrentEra,
        /// get invalid era err
        InvalidEra,
        /// era rate not updated
        EraRateNotUpdated,
        /// era rate already updated
        EraRateAlreadyUpdated,
        /// insufficient
        Insufficient,
        /// Bonding duration not set
        BondingDurationNotSet,
        /// signature repeated
        SignatureRepeated,
        /// nominations already initialized
        NominationsInitialized,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RTokenSeries {
        /// switch of bond
        BondSwitch get(fn bond_switch): bool = true;
        /// (hash, rsymbol) => record
        pub BondRecords get(fn bond_records): map hasher(blake2_128_concat) BondKey<T::Hash> => Option<BondRecord<T::AccountId>>;
        pub BondReasons get(fn bond_reasons): map hasher(blake2_128_concat) BondKey<T::Hash> => Option<BondReason>;
        pub AccountBondCount get(fn account_bond_count): map hasher(blake2_128_concat) T::AccountId => u64;
        pub AccountBondRecords get(fn account_bond_records): map hasher(blake2_128_concat) (T::AccountId, u64) => Option<BondKey<T::Hash>>;
        /// bond success histories. (symbol, blockhash, txhash) => bool
        pub BondStates get(fn bond_states): map hasher(blake2_128_concat) (RSymbol, Vec<u8>, Vec<u8>) => Option<BondState>;

        /// Proxy accounts for setting fees
        ProxyAccounts get(fn proxy_accounts): map hasher(twox_64_concat) T::AccountId => Option<u8>;
        /// fees to cover the commission happened on other chains
        pub BondFees get(fn bond_fees): map hasher(twox_64_concat) RSymbol => Balance = 1500000000000;

        /// fees to cover the commission happened on other chains
        pub UnbondFees get(fn unbond_fees): map hasher(twox_64_concat) RSymbol => Balance = 3000000000000;

        PoolBalanceLimit get(fn pool_balance_limit): map hasher(twox_64_concat) RSymbol => u128;

        /// Unbond commission
        UnbondCommission get(fn unbond_commission): Perbill = Perbill::from_parts(2000000);

        /// Account unbond records: who, symbol => [UserUnlockChunk]
        pub AccountUnbonds get(fn account_unbonds): double_map hasher(blake2_128_concat) T::AccountId, hasher(twox_64_concat) RSymbol => Option<Vec<UserUnlockChunk>>;

        pub Signatures get(fn signatures): map hasher(twox_64_concat) (RSymbol, u32, Vec<u8>, OriginalTxType, Vec<u8>) => Option<Vec<Vec<u8>>>;
        pub AccountSignature get(fn account_signature): map hasher(blake2_128_concat) (T::AccountId, RSymbol, u32, Vec<u8>, OriginalTxType, Vec<u8>) => Option<Vec<u8>>;

        pub Nominated get(fn nominated): map hasher(twox_64_concat) (RSymbol, Vec<u8>) => Option<Vec<Vec<u8>>>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// turn on/off bond switch
        #[weight = 1_000_000]
        fn toggle_bond_switch(origin) -> DispatchResult {
            ensure_root(origin)?;
            let state = Self::bond_switch();
            BondSwitch::put(!state);
			Ok(())
        }

        /// Set proxy accounts.
        #[weight = 1_000_000]
        pub fn set_proxy_accounts(origin, account: T::AccountId) -> DispatchResult {
            ensure_root(origin)?;
            <ProxyAccounts<T>>::insert(account, 0);

            Ok(())
        }

        /// Remove proxy accounts.
        #[weight = 1_000_000]
        pub fn remove_proxy_accounts(origin, account: T::AccountId) -> DispatchResult {
            ensure_root(origin)?;
            <ProxyAccounts<T>>::remove(account);

            Ok(())
        }

        /// Set fees for bond.
        #[weight = 1_000_000]
        pub fn set_bond_fees(origin, symbol: RSymbol, fees: Balance) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(<ProxyAccounts<T>>::contains_key(&who), Error::<T>::InvalidProxyAccount);

            BondFees::insert(symbol, fees);
            Self::deposit_event(RawEvent::BondFeesSet(symbol, fees));
            Ok(())
        }

        /// Set fees for unbond.
        #[weight = 1_000_000]
        pub fn set_unbond_fees(origin, symbol: RSymbol, fees: Balance) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(<ProxyAccounts<T>>::contains_key(&who), Error::<T>::InvalidProxyAccount);

            UnbondFees::insert(symbol, fees);
            Self::deposit_event(RawEvent::UnbondFeesSet(symbol, fees));
            Ok(())
        }

        /// Update pool balance limit
        #[weight = 1_000_000]
        fn set_balance_limit(origin, symbol: RSymbol, new_limit: u128) -> DispatchResult {
            ensure_root(origin)?;
            let old_limit = Self::pool_balance_limit(symbol);
            PoolBalanceLimit::insert(symbol, new_limit);

			Self::deposit_event(RawEvent::PoolBalanceLimitUpdated(symbol, old_limit, new_limit));
			Ok(())
        }

        /// set unbond commission
        #[weight = 1_000_000]
        pub fn set_unbond_commission(origin, new_part: u32) -> DispatchResult {
            ensure_root(origin)?;
            let old_commission = Self::unbond_commission();
            let new_commission = Perbill::from_parts(new_part);
            UnbondCommission::put(new_commission);

            Self::deposit_event(RawEvent::UnbondCommissionUpdated(old_commission, new_commission));
            Ok(())
        }

        /// init nominatons
        #[weight = 1_000_000]
        pub fn init_nominations(origin, symbol: RSymbol, pool: Vec<u8>, validators: Vec<Vec<u8>>) -> DispatchResult {
            ensure_root(origin)?;
            
            ensure!(Self::nominated((symbol, &pool)).is_none(), Error::<T>::NominationsInitialized);
            Nominated::insert((symbol, &pool), validators.clone());

            Ok(())
        }

        /// update nominatons
        #[weight = 1_000_000]
        pub fn update_nominations(origin, symbol: RSymbol, pool: Vec<u8>, new_validators: Vec<Vec<u8>>) -> DispatchResult {
            ensure_root(origin)?;
            
            Nominated::insert((symbol, &pool), new_validators.clone());

            Self::deposit_event(RawEvent::NominationUpdated(symbol, pool, new_validators));
            Ok(())
        }

        /// update validator
        #[weight = 1_000_000]
        pub fn update_validator(origin, symbol: RSymbol, pool: Vec<u8>, old_validator: Vec<u8>, new_validator: Vec<u8>) -> DispatchResult {
            ensure_root(origin)?;
            
            let mut validators = Self::nominated((symbol, &pool)).unwrap_or(vec![]);
            if let Ok(i) = validators.binary_search(&old_validator) {
                validators.remove(i);
            }
            validators.push(new_validator.clone());
            Nominated::insert((symbol, &pool), validators);

            Self::deposit_event(RawEvent::ValidatorUpdated(symbol, pool, old_validator, new_validator));
            Ok(())
        }

        /// liquidity bond token to get rtoken
        #[weight = 10_000_000_000]
        pub fn liquidity_bond(origin, pubkey: Vec<u8>, signature: Vec<u8>, pool: Vec<u8>, blockhash: Vec<u8>, txhash: Vec<u8>, amount: u128, symbol: RSymbol) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Self::bond_switch(), Error::<T>::BondSwitchClosed);
            ensure!(amount > 0, Error::<T>::LiquidityBondZero);
            ensure!(Self::is_txhash_available(symbol, &blockhash, &txhash), Error::<T>::TxhashUnavailable);
            ensure!(ledger::BondedPools::get(symbol).contains(&pool), ledger::Error::<T>::PoolNotBonded);
            let op_receiver = ledger::Module::<T>::receiver();
            ensure!(op_receiver.is_some(), ledger::Error::<T>::NoReceiver);

            match verify_signature(symbol, &pubkey, &signature, &who.encode()) {
                SigVerifyResult::InvalidPubkey => Err(Error::<T>::InvalidPubkey)?,
                SigVerifyResult::Fail => Err(Error::<T>::InvalidSignature)?,
                _ => (),
            }

            let limit = Self::pool_balance_limit(symbol);
            let pipe = ledger::BondPipelines::get((symbol, &pool)).unwrap_or_default();
            pipe.bond.checked_add(amount).ok_or(Error::<T>::OverFlow)?;
            let will_active = pipe.active.checked_add(amount).ok_or(Error::<T>::OverFlow)?;
            ensure!(limit == 0 || will_active <= limit, Error::<T>::PoolLimitReached);

            let record = BondRecord::new(who.clone(), symbol, pubkey.clone(), pool.clone(), blockhash.clone(), txhash.clone(), amount);
            let bond_id = <T::Hashing as Hash>::hash_of(&record);
            let bondkey = BondKey::new(symbol, bond_id);
            ensure!(Self::bond_records(&bondkey).is_none(), Error::<T>::BondRepeated);
            let old_count = Self::account_bond_count(&who);
            let new_count = old_count.checked_add(1).ok_or(Error::<T>::OverFlow)?;

            let fees = Self::bond_fees(symbol);
            if fees > 0 {
                let receiver = op_receiver.unwrap();
                T::Currency::transfer(&who, &receiver, fees.saturated_into(), KeepAlive)?;
            }

            <BondStates>::insert((symbol, &blockhash, &txhash), BondState::Dealing);
            <AccountBondCount<T>>::insert(&who, new_count);
            <AccountBondRecords<T>>::insert((&who, old_count), &bondkey);
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
            ensure!(Self::is_txhash_executable(record.symbol, &record.blockhash, &record.txhash), Error::<T>::TxhashUnexecutable);

            if reason != BondReason::Pass {
                <BondReasons<T>>::insert(&bondkey, reason);
                <BondStates>::insert((record.symbol, &record.blockhash, &record.txhash), BondState::Fail);
                return Ok(())
            }

            let mut pipe = ledger::BondPipelines::get((record.symbol, &record.pool)).unwrap_or_default();
            pipe.bond = pipe.bond.checked_add(record.amount).ok_or(Error::<T>::OverFlow)?;
            pipe.active = pipe.active.checked_add(record.amount).ok_or(Error::<T>::OverFlow)?;

            let rbalance = rtoken_rate::Module::<T>::token_to_rtoken(record.symbol, record.amount);
            <T as Trait>::RCurrency::mint(&record.bonder, record.symbol, rbalance)?;
            <BondReasons<T>>::insert(&bondkey, reason);
            <BondStates>::insert((record.symbol, &record.blockhash, &record.txhash), BondState::Success);

            ledger::BondPipelines::insert((record.symbol, &record.pool), pipe);

            Ok(())
        }

        /// liquitidy unbond to redeem token with rtoken
        #[weight = 30_000_000_000]
        pub fn liquidity_unbond(origin, symbol: RSymbol, pool: Vec<u8>, value: u128, recipient: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(value > 0, Error::<T>::LiquidityUnbondZero);
            ensure!(ledger::BondedPools::get(symbol).contains(&pool), ledger::Error::<T>::PoolNotFound);
            match verify_recipient(symbol, &recipient) {
                false => Err(Error::<T>::InvalidPubkey)?,
                _ => (),
            }

            let current_era = rtoken_ledger::ChainEras::get(symbol).ok_or(Error::<T>::NoCurrentEra)?;
            let bonding_duration = rtoken_ledger::ChainBondingDuration::get(symbol).ok_or(Error::<T>::BondingDurationNotSet)?;
            let unlock_era = current_era + bonding_duration;
            
            let op_receiver = ledger::Module::<T>::receiver();
            ensure!(op_receiver.is_some(), ledger::Error::<T>::NoReceiver);
            let receiver = op_receiver.unwrap();

            let free = <T as Trait>::RCurrency::free_balance(&who, symbol);
            free.checked_sub(value).ok_or(Error::<T>::Insufficient)?;

            let fee = Self::unbond_fee(value);
            let left_value = value - fee;
            let balance = rtoken_rate::Module::<T>::rtoken_to_token(symbol, left_value);

            let mut pipe = ledger::BondPipelines::get((symbol, &pool)).unwrap_or_default();
            pipe.unbond = pipe.unbond.checked_add(balance).ok_or(Error::<T>::OverFlow)?;
            pipe.active = pipe.active.checked_sub(balance).ok_or(Error::<T>::Insufficient)?;

            let user_unlocking = Self::account_unbonds(&who, symbol).unwrap_or(vec![]);
            let mut ac_unbonds: Vec<UserUnlockChunk> = user_unlocking.into_iter()
                .filter(|chunk| if chunk.unlock_era > current_era {
                    true
                } else {
                    false
                })
                .collect();
            ensure!(ac_unbonds.len() <= MAX_UNLOCKING_CHUNKS, Error::<T>::NoMoreUnbondingChunks);

            let mut pool_unbonds = ledger::PoolUnbonds::<T>::get((symbol, &pool, unlock_era)).unwrap_or(vec![]);
            let limit = ledger::EraUnbondLimit::get(symbol);
            ensure!(limit == 0 || pool_unbonds.len() <= usize::from(limit), Error::<T>::PoolLimitReached);

            ac_unbonds.push(UserUnlockChunk { pool: pool.clone(), unlock_era: unlock_era, value: balance, recipient: recipient.clone() });
            pool_unbonds.push(Unbonding { who: who.clone(), value: balance, recipient: recipient.clone() });

            let fees = Self::unbond_fees(symbol);
            if fees > 0 {
                T::Currency::transfer(&who, &receiver, fees.saturated_into(), KeepAlive)?;
            }
            
            <T as Trait>::RCurrency::transfer(&who, &receiver, symbol, fee)?;
            <T as Trait>::RCurrency::burn(&who, symbol, left_value)?;
            ledger::BondPipelines::insert((symbol, &pool), pipe);
            AccountUnbonds::<T>::insert(&who, symbol, &ac_unbonds);
            ledger::PoolUnbonds::<T>::insert((symbol, &pool, unlock_era), &pool_unbonds);

            Self::deposit_event(RawEvent::LiquidityUnBond(who, symbol, pool, value, left_value, balance, recipient));

            Ok(())
        }

        /// Submit tx signatures
        #[weight = 10_000_000]
        pub fn submit_signatures(origin, symbol: RSymbol, era: u32, pool: Vec<u8>, tx_type: OriginalTxType, proposal_id: Vec<u8>, signature: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(symbol.chain_type() != ChainType::Substrate, Error::<T>::InvalidRSymbol);
            ensure!(relayers::Module::<T>::is_relayer(symbol, &who), relayers::Error::<T>::MustBeRelayer);
            ensure!(ledger::BondedPools::get(symbol).contains(&pool), ledger::Error::<T>::PoolNotFound);

            let current_era = rtoken_ledger::ChainEras::get(symbol).ok_or(Error::<T>::NoCurrentEra)?;
            ensure!(era <= current_era && era >= current_era - 14, Error::<T>::InvalidEra);

            ensure!(Self::account_signature((&who, symbol, era, &pool, tx_type, &proposal_id)).is_none(), Error::<T>::SignatureRepeated);

            let mut signatures = Signatures::get((symbol, era, &pool, tx_type, &proposal_id)).unwrap_or(vec![]);
            ensure!(!signatures.contains(&signature), Error::<T>::SignatureRepeated);

            signatures.push(signature.clone());
            Signatures::insert((symbol, era, &pool, tx_type, &proposal_id), &signatures);

            <AccountSignature<T>>::insert((&who, symbol, era, &pool, tx_type, &proposal_id), &signature);

            if signatures.len() == relayers::RelayerThreshold::get(symbol) as usize {
                Self::deposit_event(RawEvent::SignaturesEnough(symbol, era, pool.clone(), tx_type, proposal_id.clone()));
            }

            Self::deposit_event(RawEvent::SubmitSignatures(who.clone(), symbol, era, pool, tx_type, proposal_id, signature));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn is_txhash_available(symbol: RSymbol, blockhash: &Vec<u8>, txhash: &Vec<u8>) -> bool {
        let op_state = Self::bond_states((symbol, &blockhash, &txhash));
        if op_state.is_none() {
            return true
        }
        let state = op_state.unwrap();
        state == BondState::Fail
    }

    fn is_txhash_executable(symbol: RSymbol, blockhash: &Vec<u8>, txhash: &Vec<u8>) -> bool {
        let op_state = Self::bond_states((symbol, &blockhash, &txhash));
        if op_state.is_none() {
            return false
        }
        let state = op_state.unwrap();
        state == BondState::Dealing
    }

    fn unbond_fee(value: u128) -> u128 {
        Self::unbond_commission() * value
    }
}