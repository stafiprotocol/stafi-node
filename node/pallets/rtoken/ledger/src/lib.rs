// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use frame_support::{
    decl_error, decl_event, decl_storage, decl_module, dispatch::DispatchResult, ensure,
    traits::{
        EnsureOrigin,
    },
};
use sp_runtime::{
    Perbill,
    traits::Hash,
};
use frame_system::{self as system, ensure_root};
use rtoken_balances::{traits::{Currency as RCurrency}};
use node_primitives::{RSymbol};

pub mod models;
pub use models::*;

pub trait Trait: system::Trait + rtoken_rate::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// currency of rtoken
    type RCurrency: RCurrency<Self::AccountId>;

    /// Specifies the origin check provided by the voter for calls that can only be called by the votes pallet
    type VoterOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;
}

decl_event! {
    pub enum Event<T> where
        Hash = <T as system::Trait>::Hash,
        AccountId = <T as system::Trait>::AccountId
    {
        /// symbol, era
        EraInitialized(RSymbol, u32),
        /// symbol, old_era, new_era
        EraUpdated(RSymbol, u32, u32),
        /// EraPoolUpdated
        EraPoolUpdated(Hash, AccountId),
        /// symbol, old_bonding_duration, new_bonding_duration
        BondingDurationUpdated(RSymbol, u32, u32),
        /// pool added
        PoolAdded(RSymbol, Vec<u8>),
        /// pool sub account added: (symbol, pool, sub_account)
        PoolSubAccountAdded(RSymbol, Vec<u8>, Vec<u8>),
        /// bond report
        BondReport(Hash, Vec<u8>, AccountId),
        /// Commission has been updated.
        CommissionUpdated(Perbill, Perbill),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// pool already added
        PoolAlreadyAdded,
        /// pool not found
        PoolNotFound,
        /// pool not bonded
        PoolNotBonded,
        /// RepeatInitBond
        RepeatInitBond,
        /// No receiver
        NoReceiver,
        /// sub account already added
        SubAccountAlreadyAdded,
        /// era zero
        EraZero,
        /// era already initialized
        EraAlreadyInitialized,
        /// new_bonding_duration zero
        NewBondingDurationZero,
        /// OverFlow
        OverFlow,
        /// Insufficient
        Insufficient,
        /// new era not bigger than old
        NewEraNotBiggerThanOld,
        /// EraRepeatSet
        EraRepeatSet,
        /// Last voter is nobody
        LastVoterNobody,
        /// shot_id not found for BondSnapshot,
        BondShotIdNotFound,
        /// shot_id already processed
        ShotIdAlreadyProcessed,
        /// active already set
        ActiveAlreadySet,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RTokenLedger {
        pub ChainEras get(fn chain_eras): map hasher(blake2_128_concat) RSymbol => Option<u32>;
        pub ChainBondingDuration get(fn chain_bonding_duration): map hasher(twox_64_concat) RSymbol => Option<u32>;

        /// commission of staking rewards
        Commission get(fn commission): Perbill = Perbill::from_percent(10);
        /// Recipient account for fees
        pub Receiver get(fn receiver): Option<T::AccountId>;

        /// Pools: maybe pubkeys
        pub Pools get(fn pools): map hasher(blake2_128_concat) RSymbol => Vec<Vec<u8>>;
        pub BondedPools get(fn bonded_pools): map hasher(blake2_128_concat) RSymbol => Vec<Vec<u8>>;

        /// place bond/unbond datas for pools
        pub BondPipelines get(fn bond_pipelines): map hasher(blake2_128_concat) (RSymbol, Vec<u8>) => Option<LinkChunk>;
        pub EraSnapShots get(fn era_snap_shots): map hasher(blake2_128_concat) (RSymbol, u32) => Option<Vec<T::Hash>>;
        pub Snapshots get(fn snap_shots): map hasher(blake2_128_concat) T::Hash => Option<BondSnapshot<T::AccountId>>;

        /// pool => Vec<SubAccounts>
        pub SubAccounts get(fn sub_accounts): map hasher(blake2_128_concat) (RSymbol, Vec<u8>) => Vec<Vec<u8>>;
        /// pool => Threshold
        pub MultiThresholds get(fn multi_thresholds): map hasher(blake2_128_concat) (RSymbol, Vec<u8>) => Option<u16>;

        /// last voter
        pub LastVoter get(fn last_voter): map hasher(blake2_128_concat) RSymbol => Option<T::AccountId>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

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

        /// add new pool
        #[weight = 10_000]
        pub fn add_new_pool(origin, symbol: RSymbol, pool: Vec<u8>) -> DispatchResult {
            ensure_root(origin)?;
            let mut pools = Self::pools(symbol);
            ensure!(!pools.contains(&pool), Error::<T>::PoolAlreadyAdded);
            pools.push(pool.clone());
            <Pools>::insert(symbol, pools);

            Self::deposit_event(RawEvent::PoolAdded(symbol, pool));
            Ok(())
        }

        /// set receiver
        #[weight = 10_000]
        pub fn set_receiver(origin, new_receiver: T::AccountId) -> DispatchResult {
            ensure_root(origin)?;
            <Receiver<T>>::put(new_receiver);
            Ok(())
        }

        /// add new pool
        #[weight = 10_000]
        pub fn set_init_bond(origin, symbol: RSymbol, pool: Vec<u8>, amount: u128) -> DispatchResult {
            ensure_root(origin)?;
            let pools = Self::pools(symbol);
            ensure!(pools.contains(&pool), Error::<T>::PoolNotFound);

            let mut bonded_pools = Self::bonded_pools(symbol);
            ensure!(!bonded_pools.contains(&pool), Error::<T>::RepeatInitBond);
            
            let op_receiver = Self::receiver();
            ensure!(op_receiver.is_some(), Error::<T>::NoReceiver);
            let receiver = op_receiver.unwrap();

            let rbalance = rtoken_rate::Module::<T>::token_to_rtoken(symbol, amount);
            T::RCurrency::mint(&receiver, symbol, rbalance)?;

            bonded_pools.push(pool.clone());
            <BondedPools>::insert(symbol, bonded_pools);
            <BondPipelines>::insert((symbol, &pool), LinkChunk {bond: 0, unbond: 0, active: amount});

            Ok(())
        }

        /// set chain era
        #[weight = 10_000]
        pub fn set_chain_bonding_duration(origin, symbol: RSymbol, new_bonding_duration: u32) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(new_bonding_duration > 0, Error::<T>::NewBondingDurationZero);

            let old_bonding_duration = Self::chain_bonding_duration(symbol).unwrap_or(0);
            ChainBondingDuration::insert(symbol, new_bonding_duration);

            Self::deposit_event(RawEvent::BondingDurationUpdated(symbol, old_bonding_duration, new_bonding_duration));
            Ok(())
        }

        /// add new pool
        #[weight = 10_000]
        pub fn add_sub_accounts_and_threshold(origin, symbol: RSymbol, pool: Vec<u8>, sub_accounts: Vec<Vec<u8>>, threshold: u16) -> DispatchResult {
            ensure_root(origin)?;
            let pools = Self::pools(symbol);
            ensure!(pools.contains(&pool), Error::<T>::PoolNotFound);
            <SubAccounts>::insert((symbol, &pool), sub_accounts);
            <MultiThresholds>::insert((symbol, &pool), threshold);

            Ok(())
        }

        /// Initialize chain era
        /// may not be needed. need more think
        #[weight = 10_000]
        pub fn initialize_chain_era(origin, symbol: RSymbol, era: u32) -> DispatchResult {
            ensure_root(origin)?;

            ensure!(era > 0, Error::<T>::EraZero);
            ensure!(!Self::chain_eras(symbol).is_some(), Error::<T>::EraAlreadyInitialized);
            <ChainEras>::insert(symbol, era);

            let rate = rtoken_rate::Module::<T>::set_rate(symbol, 0, 0);
            rtoken_rate::EraRate::insert(symbol, era, rate);

            Self::deposit_event(RawEvent::EraInitialized(symbol, era));
            Ok(())
        }

        /// set chain era
        #[weight = 10_000]
        pub fn init_last_voter(origin) -> DispatchResult {
            T::VoterOrigin::ensure_origin(origin)?;
            Ok(())
        }

        /// set chain era
        #[weight = 10_000]
        pub fn set_chain_era(origin, symbol: RSymbol, new_era: u32) -> DispatchResult {
            T::VoterOrigin::ensure_origin(origin)?;
            // last_voter
            let op_voter = Self::last_voter(symbol);
            ensure!(op_voter.is_some(), Error::<T>::LastVoterNobody);
            let voter = op_voter.unwrap();

            let old_era = Self::chain_eras(symbol).unwrap_or(0);
            ensure!(old_era < new_era, Error::<T>::NewEraNotBiggerThanOld);
            let pools = Self::bonded_pools(symbol);
            let mut era_shots = Self::era_snap_shots((symbol, new_era)).unwrap_or(vec![]);
            
            for pool in pools {
                let chunk = Self::bond_pipelines((symbol, &pool)).unwrap_or_default();
                let snapshot = BondSnapshot {symbol, era: new_era, pool, bond: chunk.bond, unbond: chunk.unbond, last_voter: voter.clone(), active: chunk.active};
                let shot_id = <T::Hashing as Hash>::hash_of(&snapshot);
                <Snapshots<T>>::insert(&shot_id, snapshot.clone());
                era_shots.push(shot_id.clone());
                Self::deposit_event(RawEvent::EraPoolUpdated(shot_id, voter.clone()));
            }

            <EraSnapShots<T>>::insert((symbol, new_era), era_shots);
            <ChainEras>::insert(symbol, new_era);
            Self::deposit_event(RawEvent::EraUpdated(symbol, old_era, new_era));
            Ok(())
        }

        /// bond link success
        #[weight = 10_000]
        pub fn bond_report(origin, shot_id: T::Hash) -> DispatchResult {
            T::VoterOrigin::ensure_origin(origin)?;
            ensure!(Self::snap_shots(&shot_id).is_some(), Error::<T>::BondShotIdNotFound);
            let snap = Self::snap_shots(&shot_id).unwrap();

            let op_voter = Self::last_voter(snap.symbol);
            ensure!(op_voter.is_some(), Error::<T>::LastVoterNobody);
            let voter = op_voter.unwrap();
            
            let mut chunk = Self::bond_pipelines((snap.symbol, &snap.pool)).unwrap_or_default();
            chunk.bond = chunk.bond.saturating_sub(snap.bond);
            chunk.unbond = chunk.unbond.saturating_sub(snap.unbond);

            <BondPipelines>::insert((snap.symbol, &snap.pool), chunk);
            Self::deposit_event(RawEvent::BondReport(shot_id, snap.pool, voter));

            Ok(())
        }

        /// set bond active of pool
        #[weight = 10_000]
        pub fn active_report(origin, shot_id: T::Hash, active: u128) -> DispatchResult {
            T::VoterOrigin::ensure_origin(origin)?;

            let op_receiver = Self::receiver();
            ensure!(op_receiver.is_some(), Error::<T>::NoReceiver);
            let receiver = op_receiver.unwrap();

            ensure!(Self::snap_shots(&shot_id).is_some(), Error::<T>::BondShotIdNotFound);
            let snap = Self::snap_shots(&shot_id).unwrap();
            let symbol = snap.symbol;
            
            let mut era_shots = Self::era_snap_shots((symbol, snap.era)).unwrap_or(vec![]);
            let location = era_shots.binary_search(&shot_id).ok().ok_or(Error::<T>::ActiveAlreadySet)?;
            
            let rbalance = T::RCurrency::total_issuance(symbol);
            let before = rtoken_rate::Module::<T>::rtoken_to_token(symbol, rbalance);
            let after = before.saturating_add(active).saturating_sub(snap.active);
            if after > before {
                let fee = Self::commission() * (after - before);
                let rfee = rtoken_rate::Module::<T>::token_to_rtoken(symbol, fee);
                T::RCurrency::mint(&receiver, symbol, rfee)?;
            }

            let mut rate = rtoken_rate::RATEBASE;
            if after != before {
                let rbalance = T::RCurrency::total_issuance(symbol);
                rate = rtoken_rate::Module::<T>::set_rate(symbol, after, rbalance);
            }

            era_shots.remove(location);
            if era_shots.is_empty() {
                rtoken_rate::EraRate::insert(symbol, snap.era, rate);
            }

            Ok(())
        }    
    }
}