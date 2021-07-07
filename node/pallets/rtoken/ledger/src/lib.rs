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
        /// symbol, old_era, new_era
        EraUpdated(RSymbol, u32, u32),
        /// EraPoolUpdated
        EraPoolUpdated(RSymbol, u32, Hash, AccountId),
        /// symbol, old_bonding_duration, new_bonding_duration
        BondingDurationUpdated(RSymbol, u32, u32),
        /// Commission has been updated.
        CommissionUpdated(Perbill, Perbill),
        /// pool added
        PoolAdded(RSymbol, Vec<u8>),
        /// pool sub account added: (symbol, pool, sub_account)
        PoolSubAccountAdded(RSymbol, Vec<u8>, Vec<u8>),
        /// bond report
        BondReported(RSymbol, Hash, AccountId),
        /// withdraw unbond
        ActiveReported(RSymbol, Hash, AccountId),
        /// withdraw reported
        WithdrawReported(RSymbol, Hash, AccountId),
        /// transfer reported
        TransferReported(RSymbol, Hash),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Got an OverFlow
        OverFlow,
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
        /// new_bonding_duration zero
        NewBondingDurationZero,
        /// new era not bigger than old
        EraSkipped,
        /// Last voter is nobody
        LastVoterNobody,
        /// snap shot not found by shotId,
        SnapShotNotFound,
        /// shot_id already processed
        ShotIdAlreadyProcessed,
        /// active already set
        ActiveAlreadySet,
        /// bond reported
        BondReported,
        /// transfer reported
        TransferReported,
        /// state not era updated
        StateNotEraUpdated,
        /// state not bond reported
        StateNotBondReported,
        /// state not active reported
        StateNotActiveReported,
        /// state not withdraw reported
        StateNotWithdrawReported,
        /// Last era not continuable
        LastEraNotContinuable,
        /// none rate error
        RateIsNone,
        /// invalid pure bond report
        InvalidPureBondReport,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RTokenLedger {
        pub ChainEras get(fn chain_eras): map hasher(blake2_128_concat) RSymbol => Option<u32>;
        pub ChainBondingDuration get(fn chain_bonding_duration): map hasher(blake2_128_concat) RSymbol => Option<u32>;

        /// commission of staking rewards
        Commission get(fn commission): Perbill = Perbill::from_percent(10);
        /// Recipient account for fees
        pub Receiver get(fn receiver): Option<T::AccountId>;

        /// Pools: maybe pubkeys
        pub Pools get(fn pools): map hasher(blake2_128_concat) RSymbol => Vec<Vec<u8>>;
        pub BondedPools get(fn bonded_pools): map hasher(blake2_128_concat) RSymbol => Vec<Vec<u8>>;
        /// Total expected active
        pub TotalExpectedActive get(fn total_expected_active): double_map hasher(blake2_128_concat) RSymbol, hasher(blake2_128_concat) u32 => Option<u128>;

        /// place bond/unbond datas for pools
        pub BondPipelines get(fn bond_pipelines): double_map hasher(blake2_128_concat) RSymbol, hasher(blake2_128_concat) Vec<u8> => Option<LinkChunk>;
        pub EraSnapShots get(fn era_snap_shots): double_map hasher(blake2_128_concat) RSymbol, hasher(blake2_128_concat) u32 => Option<Vec<T::Hash>>;
        pub Snapshots get(fn snap_shots): double_map hasher(blake2_128_concat) RSymbol, hasher(blake2_128_concat) T::Hash => Option<BondSnapshot<T::AccountId>>;
        pub CurrentEraSnapShots get(fn current_era_snap_shots): map hasher(blake2_128_concat) RSymbol => Option<Vec<T::Hash>>;

        /// pool unbond records: (symbol, pool, unlock_era) => unbonds
        pub PoolUnbonds get(fn pool_unbonds): double_map hasher(blake2_128_concat) RSymbol, hasher(blake2_128_concat) (Vec<u8>, u32) => Option<Vec<Unbonding<T::AccountId>>>;
        /// pool era unbond number limit
        pub EraUnbondLimit get(fn era_unbond_limit): map hasher(blake2_128_concat) RSymbol => u16;

        /// pool => Vec<SubAccounts>
        pub SubAccounts get(fn sub_accounts): double_map hasher(blake2_128_concat) RSymbol, hasher(blake2_128_concat) Vec<u8> => Vec<Vec<u8>>;
        /// pool => Threshold
        pub MultiThresholds get(fn multi_thresholds): double_map hasher(blake2_128_concat) RSymbol, hasher(blake2_128_concat) Vec<u8> => Option<u16>;

        /// last voter
        pub LastVoter get(fn last_voter): map hasher(blake2_128_concat) RSymbol => Option<T::AccountId>;

        /// bond at least
        pub LeastBond get(fn least_bond): map hasher(blake2_128_concat) RSymbol => Option<u128>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Update commission of staking rewards
		#[weight = 1_000_000]
		fn set_commission(origin, new_part: u32) -> DispatchResult {
            ensure_root(origin)?;

            ensure!(new_part < 1000000000, Error::<T>::OverFlow);

            let old_commission = Self::commission();
            let new_commission = Perbill::from_parts(new_part);
			Commission::put(new_commission);

			Self::deposit_event(RawEvent::CommissionUpdated(old_commission, new_commission));
			Ok(())
        }

        /// add new pool
        #[weight = 1_000_000]
        pub fn add_new_pool(origin, symbol: RSymbol, pool: Vec<u8>) -> DispatchResult {
            ensure_root(origin)?;
            let mut pools = Self::pools(symbol);
            ensure!(!pools.contains(&pool), Error::<T>::PoolAlreadyAdded);
            pools.push(pool.clone());
            <Pools>::insert(symbol, pools);

            Self::deposit_event(RawEvent::PoolAdded(symbol, pool));
            Ok(())
        }

        /// remove pool
        #[weight = 1_000_000]
        pub fn remove_pool(origin, symbol: RSymbol, pool: Vec<u8>) -> DispatchResult {
            ensure_root(origin)?;

            let mut bonded_pools = Self::bonded_pools(symbol);
            let op_bonded_index = bonded_pools.iter().position(|p| p == &pool);
            ensure!(op_bonded_index.is_some(), Error::<T>::PoolNotBonded);

            let pipe = Self::bond_pipelines(symbol, &pool).unwrap_or_default();
            ensure!(pipe.bond == 0 && pipe.unbond == 0 && pipe.active == 0, Error::<T>::ActiveAlreadySet);

            let bonded_index = op_bonded_index.unwrap();
            bonded_pools.remove(bonded_index);

            let mut pools = Self::pools(symbol);
            let op_index = pools.iter().position(|p| p == &pool).unwrap();
            pools.remove(op_index);

            BondedPools::insert(symbol, bonded_pools);
            Pools::insert(symbol, pools);

            Ok(())
        }

        /// set receiver
        #[weight = 1_000_000]
        pub fn set_receiver(origin, new_receiver: T::AccountId) -> DispatchResult {
            ensure_root(origin)?;
            <Receiver<T>>::put(new_receiver);
            Ok(())
        }

        /// set era unbond limit
        #[weight = 1_000_000]
        pub fn set_era_unbond_limit(origin, symbol: RSymbol, limit: u16) -> DispatchResult {
            ensure_root(origin)?;
            EraUnbondLimit::insert(symbol, limit);
            Ok(())
        }

        /// init bond pool
        #[weight = 1_000_000]
        pub fn set_init_bond(origin, symbol: RSymbol, pool: Vec<u8>, bond_receiver: T::AccountId, amount: u128) -> DispatchResult {
            ensure_root(origin)?;
            let pools = Self::pools(symbol);
            ensure!(pools.contains(&pool), Error::<T>::PoolNotFound);

            let mut bonded_pools = Self::bonded_pools(symbol);
            ensure!(!bonded_pools.contains(&pool), Error::<T>::RepeatInitBond);

            let rbalance = rtoken_rate::Module::<T>::token_to_rtoken(symbol, amount);
            T::RCurrency::mint(&bond_receiver, symbol, rbalance)?;

            if rtoken_rate::Rate::get(symbol).is_none() {
                rtoken_rate::Module::<T>::set_rate(symbol, 0, 0);
            }
            bonded_pools.push(pool.clone());
            <BondedPools>::insert(symbol, bonded_pools);
            <BondPipelines>::insert(symbol, &pool, LinkChunk {bond: 0, unbond: 0, active: amount});

            Ok(())
        }

        /// remove and init pool - just for test
        #[weight = 1_000_000]
        pub fn remove_pool_and_init_bond(origin, symbol: RSymbol, pool: Vec<u8>, new_pool: Vec<u8>, amount: u128) -> DispatchResult {
            ensure_root(origin)?;

            let mut bonded_pools = Self::bonded_pools(symbol);
            let op_bonded_index = bonded_pools.iter().position(|p| p == &pool);
            ensure!(op_bonded_index.is_some(), Error::<T>::PoolNotBonded);

            let bonded_index = op_bonded_index.unwrap();
            bonded_pools.remove(bonded_index);

            let mut pools = Self::pools(symbol);
            let op_index = pools.iter().position(|p| p == &pool).unwrap();
            pools.remove(op_index);

            bonded_pools.push(new_pool.clone());
            pools.push(new_pool.clone());

            BondedPools::insert(symbol, bonded_pools);
            Pools::insert(symbol, pools);

            <BondPipelines>::insert(symbol, &new_pool, LinkChunk {bond: amount, unbond: 0, active: amount});
            <BondPipelines>::remove(symbol, &pool);

            Ok(())
        }

        /// set chain bonding duration
        #[weight = 1_000_000]
        pub fn set_chain_bonding_duration(origin, symbol: RSymbol, new_bonding_duration: u32) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(new_bonding_duration > 0, Error::<T>::NewBondingDurationZero);

            let old_bonding_duration = Self::chain_bonding_duration(symbol).unwrap_or(0);
            ChainBondingDuration::insert(symbol, new_bonding_duration);

            Self::deposit_event(RawEvent::BondingDurationUpdated(symbol, old_bonding_duration, new_bonding_duration));
            Ok(())
        }

        /// add sub accounts and threshold of a pool
        #[weight = 1_000_000]
        pub fn add_sub_accounts_and_threshold(origin, symbol: RSymbol, pool: Vec<u8>, sub_accounts: Vec<Vec<u8>>, threshold: u16) -> DispatchResult {
            ensure_root(origin)?;
            let pools = Self::pools(symbol);
            ensure!(pools.contains(&pool), Error::<T>::PoolNotFound);
            ensure!(usize::from(threshold) <= sub_accounts.len(), "threshold bigger than size of sub_accounts");
            <SubAccounts>::insert(symbol, &pool, sub_accounts);
            <MultiThresholds>::insert(symbol, &pool, threshold);

            Ok(())
        }

        #[weight = 1_000_000]
        pub fn clear_current_era_snap_shots(origin, symbol: RSymbol) -> DispatchResult {
            ensure_root(origin)?;
            let empty: Vec<T::Hash> = vec![];
            <CurrentEraSnapShots<T>>::insert(symbol, empty);
            Ok(())
        }
        
        /// just for test
        #[weight = 1_000_000]
        pub fn reset_chain_era(origin, symbol: RSymbol, new_era: u32) -> DispatchResult {
            ensure_root(origin)?;
            <ChainEras>::insert(symbol, new_era);
            Ok(())
        }

        #[weight = 1_000_000]
        pub fn set_least_bond(origin, symbol: RSymbol, least: u128) -> DispatchResult {
            ensure_root(origin)?;

            <LeastBond>::insert(symbol, least);
            Ok(())
        }

        /// set chain era
        #[weight = 1_000_000]
        pub fn set_chain_era(origin, symbol: RSymbol, new_era: u32) -> DispatchResult {
            T::VoterOrigin::ensure_origin(origin)?;
            let mut era_shots = Self::current_era_snap_shots(symbol).unwrap_or(vec![]);
            ensure!(era_shots.is_empty(), Error::<T>::LastEraNotContinuable);

            // last_voter
            let op_voter = Self::last_voter(symbol);
            ensure!(op_voter.is_some(), Error::<T>::LastVoterNobody);
            let voter = op_voter.unwrap();

            let old_era = Self::chain_eras(symbol).unwrap_or(0);
            ensure!(old_era == 0 || old_era.saturating_add(1) == new_era, Error::<T>::EraSkipped);

            let pools = Self::bonded_pools(symbol);
            for pool in pools {
                let pipe = Self::bond_pipelines(symbol, &pool).unwrap_or_default();
                let snapshot = BondSnapshot {symbol, era: new_era, pool, bond: pipe.bond,
                unbond: pipe.unbond, last_voter: voter.clone(), active: pipe.active, bond_state: PoolBondState::EraUpdated};
                let shot_id = <T::Hashing as Hash>::hash_of(&snapshot);
                <Snapshots<T>>::insert(symbol, &shot_id, snapshot.clone());
                era_shots.push(shot_id.clone());
                Self::deposit_event(RawEvent::EraPoolUpdated(symbol, new_era, shot_id, voter.clone()));
            }

            <EraSnapShots<T>>::insert(symbol, new_era, &era_shots);
            <CurrentEraSnapShots<T>>::insert(symbol, era_shots);
            <ChainEras>::insert(symbol, new_era);
            Self::deposit_event(RawEvent::EraUpdated(symbol, old_era, new_era));
            Ok(())
        }

        /// bond link success
        #[weight = 1_000_000]
        pub fn bond_report(origin, symbol: RSymbol, shot_id: T::Hash) -> DispatchResult {
            Self::ensure_voter_or_admin(origin)?;
            let op_snap = Self::snap_shots(symbol, &shot_id);
            ensure!(op_snap.is_some(), Error::<T>::SnapShotNotFound);

            let mut snap = op_snap.unwrap();
            ensure!(snap.era_updated(), Error::<T>::StateNotEraUpdated);

            let mut pipe = Self::bond_pipelines(symbol, &snap.pool).unwrap_or_default();
            pipe.bond = pipe.bond.saturating_sub(snap.bond);
            pipe.unbond = pipe.unbond.saturating_sub(snap.unbond);

            <BondPipelines>::insert(symbol, &snap.pool, pipe);
            snap.update_state(PoolBondState::BondReported);
            <Snapshots<T>>::insert(symbol, &shot_id, snap.clone());
            Self::deposit_event(RawEvent::BondReported(symbol, shot_id, snap.last_voter));

            Ok(())
        }

        /// bond pure report
        #[weight = 1_000_000]
        pub fn bond_pure_report(origin, symbol: RSymbol, shot_id: T::Hash) -> DispatchResult {
            Self::ensure_voter_or_admin(origin)?;
            let mut snap = Self::snap_shots(symbol, &shot_id).ok_or(Error::<T>::SnapShotNotFound)?;
            let least_bond = Self::least_bond(symbol).unwrap_or(0);
            ensure!(snap.unbond == 0 && snap.bond <= least_bond, Error::<T>::InvalidPureBondReport);
            ensure!(snap.era_updated(), Error::<T>::StateNotEraUpdated);
            snap.update_state(PoolBondState::BondReported);
            <Snapshots<T>>::insert(symbol, &shot_id, snap.clone());
            Self::deposit_event(RawEvent::BondReported(symbol, shot_id, snap.last_voter));

            Ok(())
        }

        /// set bond active of pool
        #[weight = 1_000_000]
        pub fn active_report(origin, symbol: RSymbol, shot_id: T::Hash, active: u128) -> DispatchResult {
            Self::ensure_voter_or_admin(origin)?;

            let op_snap = Self::snap_shots(symbol, &shot_id);
            ensure!(op_snap.is_some(), Error::<T>::SnapShotNotFound);
            let mut snap = op_snap.unwrap();
            ensure!(snap.bond_reported(), Error::<T>::StateNotBondReported);
            ensure!(rtoken_rate::Rate::get(symbol).is_some(), Error::<T>::RateIsNone);

            let op_receiver = Self::receiver();
            ensure!(op_receiver.is_some(), Error::<T>::NoReceiver);
            let receiver = op_receiver.unwrap();

            let mut era_shots = Self::era_snap_shots(symbol, snap.era).unwrap_or(vec![]);
            let op_era_index = era_shots.iter().position(|shot| shot == &shot_id);
            ensure!(op_era_index.is_some(), Error::<T>::ActiveAlreadySet);
            let era_index = op_era_index.unwrap();

            let mut cur_era_shot = Self::current_era_snap_shots(symbol).unwrap_or(vec![]);
            let op_cur_era_index = cur_era_shot.iter().position(|shot| shot == &shot_id);
            ensure!(op_cur_era_index.is_some(), Error::<T>::ActiveAlreadySet);
            let cur_era_index = op_cur_era_index.unwrap();

            if active > snap.active {
                let fee = Self::commission() * (active - snap.active);
                let rfee = rtoken_rate::Module::<T>::token_to_rtoken(symbol, fee);
                T::RCurrency::mint(&receiver, symbol, rfee)?;
            }

            let mut pipe = Self::bond_pipelines(symbol, &snap.pool).unwrap_or_default();
            let expected_active = pipe.active.saturating_add(active).saturating_sub(snap.active);
            pipe.active = expected_active;
            let mut total_expected_active = Self::total_expected_active(symbol, snap.era).unwrap_or(0);
            total_expected_active = total_expected_active.saturating_add(expected_active);
            era_shots.remove(era_index);
            if era_shots.is_empty() {
                let rbalance = T::RCurrency::total_issuance(symbol);
                let rate = rtoken_rate::Module::<T>::set_rate(symbol, total_expected_active, rbalance);
                rtoken_rate::EraRate::insert(symbol, snap.era, rate);
            }
            <EraSnapShots<T>>::insert(symbol, snap.era, era_shots);
            <BondPipelines>::insert(symbol, &snap.pool, pipe);
            <TotalExpectedActive>::insert(symbol, snap.era, total_expected_active);

            if Self::pool_unbonds(symbol, (&snap.pool, snap.era)).is_some() {
                snap.update_state(PoolBondState::ActiveReported);
                Self::deposit_event(RawEvent::ActiveReported(symbol, shot_id.clone(), snap.last_voter.clone()));
            } else {
                snap.update_state(PoolBondState::WithdrawSkipped);
                cur_era_shot.remove(cur_era_index);
                <CurrentEraSnapShots<T>>::insert(symbol, cur_era_shot);
            }

            <Snapshots<T>>::insert(symbol, &shot_id, snap);
            Ok(())
        }

        /// bond and report active
        #[weight = 1_000_000]
        pub fn bond_and_report_active(origin, symbol: RSymbol, shot_id: T::Hash, active: u128, reward: u128) -> DispatchResult {
            Self::ensure_voter_or_admin(origin)?;
            let mut snap = Self::snap_shots(symbol, &shot_id).ok_or(Error::<T>::SnapShotNotFound)?;
            ensure!(snap.era_updated(), Error::<T>::StateNotEraUpdated);
            ensure!(rtoken_rate::Rate::get(symbol).is_some(), Error::<T>::RateIsNone);

            let mut pipe = Self::bond_pipelines(symbol, &snap.pool).unwrap_or_default();
            pipe.bond = pipe.bond.saturating_sub(snap.bond).saturating_add(reward);
            pipe.unbond = pipe.unbond.saturating_sub(snap.unbond);

            let receiver = Self::receiver().ok_or(Error::<T>::NoReceiver)?;
            let mut era_shots = Self::era_snap_shots(symbol, snap.era).unwrap_or(vec![]);
            let era_index = era_shots.iter().position(|shot| shot == &shot_id).ok_or(Error::<T>::ActiveAlreadySet)?;

            let mut cur_era_shot = Self::current_era_snap_shots(symbol).unwrap_or(vec![]);
            let cur_era_index = cur_era_shot.iter().position(|shot| shot == &shot_id).ok_or(Error::<T>::ActiveAlreadySet)?;

            let future_active = active.saturating_add(reward);
            if future_active > snap.active {
                let fee = Self::commission() * (future_active - snap.active);
                let rfee = rtoken_rate::Module::<T>::token_to_rtoken(symbol, fee);
                T::RCurrency::mint(&receiver, symbol, rfee)?;
            }

            let expected_active = pipe.active.saturating_add(future_active).saturating_sub(snap.active);
            pipe.active = expected_active;
            let total_expected_active = Self::total_expected_active(symbol, snap.era).unwrap_or(0).saturating_add(expected_active);
            era_shots.remove(era_index);
            if era_shots.is_empty() {
                let rbalance = T::RCurrency::total_issuance(symbol);
                let rate = rtoken_rate::Module::<T>::set_rate(symbol, total_expected_active, rbalance);
                rtoken_rate::EraRate::insert(symbol, snap.era, rate);
            }
            <EraSnapShots<T>>::insert(symbol, snap.era, era_shots);
            <BondPipelines>::insert(symbol, &snap.pool, pipe);
            <TotalExpectedActive>::insert(symbol, snap.era, total_expected_active);

            if Self::pool_unbonds(symbol, (&snap.pool, snap.era)).is_some() {
                snap.update_state(PoolBondState::ActiveReported);
                Self::deposit_event(RawEvent::ActiveReported(symbol, shot_id.clone(), snap.last_voter.clone()));
            } else {
                snap.update_state(PoolBondState::WithdrawSkipped);
                cur_era_shot.remove(cur_era_index);
                <CurrentEraSnapShots<T>>::insert(symbol, cur_era_shot);
            }

            <Snapshots<T>>::insert(symbol, &shot_id, snap);
            Ok(())
        }

        /// withdraw success
        #[weight = 1_000_000]
        pub fn withdraw_report(origin, symbol: RSymbol, shot_id: T::Hash) -> DispatchResult {
            Self::ensure_voter_or_admin(origin)?;

            let op_snap = Self::snap_shots(symbol, &shot_id);
            ensure!(op_snap.is_some(), Error::<T>::SnapShotNotFound);
            let mut snap = op_snap.unwrap();
            ensure!(snap.active_reported(), Error::<T>::StateNotActiveReported);

            snap.update_state(PoolBondState::WithdrawReported);
            <Snapshots<T>>::insert(symbol, &shot_id, snap.clone());
            Self::deposit_event(RawEvent::WithdrawReported(symbol, shot_id, snap.last_voter));

            Ok(())
        }

        /// transfer success
        #[weight = 1_000_000]
        pub fn transfer_report(origin, symbol: RSymbol, shot_id: T::Hash) -> DispatchResult {
            Self::ensure_voter_or_admin(origin)?;

            let op_snap = Self::snap_shots(symbol, &shot_id);
            ensure!(op_snap.is_some(), Error::<T>::SnapShotNotFound);
            let mut snap = op_snap.unwrap();
            ensure!(snap.withdraw_reported(), Error::<T>::StateNotWithdrawReported);

            let mut cur_era_shot = Self::current_era_snap_shots(symbol).unwrap_or(vec![]);
            let op_index = cur_era_shot.iter().position(|shot| shot == &shot_id);
            ensure!(op_index.is_some(), Error::<T>::TransferReported);
            let cur_era_index = op_index.unwrap();
            cur_era_shot.remove(cur_era_index);

            <CurrentEraSnapShots<T>>::insert(symbol, cur_era_shot);
            snap.update_state(PoolBondState::TransferReported);
            <Snapshots<T>>::insert(symbol, &shot_id, snap);

            Self::deposit_event(RawEvent::TransferReported(symbol, shot_id));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn ensure_voter_or_admin(o: T::Origin) -> DispatchResult {
        T::VoterOrigin::try_origin(o)
            .map(|_| ())
            .or_else(ensure_root)?;
        Ok(())
    }
}
