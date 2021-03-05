// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use frame_support::{
    decl_error, decl_event, decl_storage, decl_module, dispatch::DispatchResult, ensure,
    traits::{
        EnsureOrigin,
    },
};
use frame_system::{self as system, ensure_root};
use rtoken_balances::{traits::{Currency as RCurrency}};
use node_primitives::{RSymbol};

pub mod models;
pub use models::*;

pub trait Trait: system::Trait + rtoken_rate::Trait {
    type Event: From<Event> + Into<<Self as system::Trait>::Event>;

    /// currency of rtoken
    type RCurrency: RCurrency<Self::AccountId>;

    /// Specifies the origin check provided by the voter for calls that can only be called by the votes pallet
    type VoterOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;
}

decl_event! {
    pub enum Event {
        /// symbol, era
        EraInitialized(RSymbol, u32),
        /// symbol, old_era, new_era
        EraUpdated(RSymbol, u32, u32),
        /// EraPoolUpdated
        EraPoolUpdated(RSymbol, u32, Vec<u8>, u128, u128),
        /// symbol, old_bonding_duration, new_bonding_duration
        BondingDurationUpdated(RSymbol, u32, u32),
        /// pool added
        PoolAdded(RSymbol, Vec<u8>),
        /// pool sub account added: (symbol, pool, sub_account)
        PoolSubAccountAdded(RSymbol, Vec<u8>, Vec<u8>),
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
        /// active repeat set
        ActiveRepeatSet,
        /// new era not bigger than old
        NewEraNotBiggerThanold,
        /// EraRepeatSet
        EraRepeatSet,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RTokenLedger {
        pub ChainEras get(fn chain_eras): map hasher(blake2_128_concat) RSymbol => Option<u32>;
        pub ChainBondingDuration get(fn chain_bonding_duration): map hasher(twox_64_concat) RSymbol => Option<u32>;
        /// Recipient account for fees
        pub Receiver get(fn receiver): Option<T::AccountId>;
        /// Pools: maybe pubkeys
        pub Pools get(fn pools): map hasher(blake2_128_concat) RSymbol => Vec<Vec<u8>>;
        pub BondedPools get(fn bonded_pools): map hasher(blake2_128_concat) RSymbol => Vec<Vec<u8>>;
        pub PoolWillBonded get(fn pool_will_bonded): map hasher(blake2_128_concat) (RSymbol, Vec<u8>) => Option<u128>;

        /// first place to place bond/unbond datas
        pub BondPipelines get(fn bond_pipelines): map hasher(blake2_128_concat) (RSymbol, Vec<u8>) => Option<LinkChunk>;
        pub TmpTotalBond get(fn tmp_total_bond): map hasher(blake2_128_concat) RSymbol => Option<u128>;
        pub TmpTotalUnbond get(fn tmp_total_unbond): map hasher(blake2_128_concat) RSymbol => Option<u128>;
        /// second place to place bond/unbond datas
        pub BondFaucets get(fn bond_faucets): map hasher(blake2_128_concat) (RSymbol, u32, Vec<u8>) => Option<LinkChunk>;

        /// symbol, era => pools
        pub EraBondPools get(fn era_bond_pools): map hasher(blake2_128_concat) (RSymbol, u32) => Option<Vec<Vec<u8>>>;
        pub EraPoolBonded get(fn era_pool_bonded): map hasher(blake2_128_concat) (RSymbol, u32, Vec<u8>) => Option<u128>;
        pub EraTotolBonded get(fn era_total_bonded): map hasher(blake2_128_concat) (RSymbol, u32) => Option<u128>;

        /// pool => Vec<SubAccounts>
        pub SubAccounts get(fn sub_accounts): map hasher(blake2_128_concat) Vec<u8> => Vec<Vec<u8>>;
        /// pool sub account flag
        pub PoolSubAccountFlag get(fn pool_sub_account_flag): map hasher(blake2_128_concat) (Vec<u8>, Vec<u8>) => Option<bool>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// add new pool
        #[weight = 10_000]
        pub fn add_new_pool(origin, symbol: RSymbol, pool: Vec<u8>) -> DispatchResult {
            ensure_root(origin)?;
            let mut pools = Self::pools(symbol);
            ensure!(!pools.contains(&pool), Error::<T>::PoolAlreadyAdded);
            pools.push(pool.clone());
            Pools::insert(symbol, pools);

            Self::deposit_event(Event::PoolAdded(symbol, pool));
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
            Ok(())
        }

        /// add new pool
        #[weight = 10_000]
        pub fn add_sub_account_for_pool(origin, symbol: RSymbol, pool: Vec<u8>, sub_account: Vec<u8>) -> DispatchResult {
            ensure_root(origin)?;
            let pools = Self::pools(symbol);
            ensure!(pools.contains(&pool), Error::<T>::PoolNotFound);
            let mut sub_accounts = Self::sub_accounts(&pool);
            ensure!(!sub_accounts.contains(&sub_account), Error::<T>::SubAccountAlreadyAdded);

            sub_accounts.push(sub_account.clone());
            <SubAccounts>::insert(&pool, &sub_accounts);
            <PoolSubAccountFlag>::insert((&pool, &sub_account), true);

            Self::deposit_event(Event::PoolSubAccountAdded(symbol, pool, sub_account));
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

            Self::deposit_event(Event::EraInitialized(symbol, era));
            Ok(())
        }


        /// set chain era
        #[weight = 10_000]
        pub fn set_chain_era(origin, symbol: RSymbol, new_era: u32) -> DispatchResult {
            T::VoterOrigin::ensure_origin(origin)?;
            let old_era = Self::chain_eras(symbol).unwrap_or(0);
            ensure!(old_era < new_era, Error::<T>::NewEraNotBiggerThanold);
            let mut pls = Self::era_bond_pools((symbol, new_era)).unwrap_or(vec![]);
            ensure!(pls.is_empty(), Error::<T>::EraRepeatSet);
            let pools = Self::bonded_pools(symbol);
            
            for p in pools {
                let chunk = Self::bond_pipelines((symbol, &p)).unwrap_or_default();
                pls.push(p.clone());
                <BondPipelines>::insert((symbol, &p), LinkChunk::default());
                <BondFaucets>::insert((symbol, new_era, &p), &chunk);
                Self::deposit_event(Event::EraPoolUpdated(symbol, new_era, p, chunk.bond, chunk.unbond));
            }

            <TmpTotalBond>::insert(symbol, 0);
            <TmpTotalUnbond>::insert(symbol, 0);
            <ChainEras>::insert(symbol, new_era);
            <EraBondPools>::insert((symbol, new_era), pls);
            
            Self::deposit_event(Event::EraUpdated(symbol, old_era, new_era));
            Ok(())
        }

        /// set chain era
        #[weight = 10_000]
        pub fn set_chain_bonding_duration(origin, symbol: RSymbol, new_bonding_duration: u32) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(new_bonding_duration > 0, Error::<T>::NewBondingDurationZero);

            let old_bonding_duration = Self::chain_bonding_duration(symbol).unwrap_or(0);
            ChainBondingDuration::insert(symbol, new_bonding_duration);

            Self::deposit_event(Event::BondingDurationUpdated(symbol, old_bonding_duration, new_bonding_duration));
            Ok(())
        }

        /// set bond active of pool
        #[weight = 10_000]
        pub fn set_pool_active(origin, symbol: RSymbol, era: u32, pool: Vec<u8>, active: u128) -> DispatchResult {
            T::VoterOrigin::ensure_origin(origin)?;
            let mut pls = Self::era_bond_pools((symbol, era)).unwrap_or(vec![]);
            let location = pls.binary_search(&pool).ok().ok_or(Error::<T>::PoolNotFound)?;
            ensure!(Self::era_pool_bonded((symbol, era, &pool)).is_none(), Error::<T>::ActiveRepeatSet);
            
            let mut total = Self::era_total_bonded((symbol, era)).unwrap_or(0);
            total = total.checked_add(active).ok_or(Error::<T>::OverFlow)?;
            pls.remove(location);
            if pls.is_empty() {
                let new_bond = Self::tmp_total_bond(symbol).unwrap_or(0);
                let new_unbond = Self::tmp_total_unbond(symbol).unwrap_or(0);
                total = total.checked_add(new_bond).ok_or(Error::<T>::OverFlow)?;
                total = total.checked_sub(new_unbond).ok_or(Error::<T>::Insufficient)?;
                let rbalance = T::RCurrency::total_issuance(symbol);
                let rate = rtoken_rate::Module::<T>::set_rate(symbol, total, rbalance);
                rtoken_rate::EraRate::insert(symbol, era, rate);
            }

            let pipe = Self::bond_pipelines((symbol, &pool)).unwrap_or_default();
            <PoolWillBonded>::insert((symbol, &pool), active + pipe.bond - pipe.unbond);
            <EraPoolBonded>::insert((symbol, era, &pool), active);
            <EraTotolBonded>::insert((symbol, era), total);
            <EraBondPools>::insert((symbol, era), pls);

            Ok(())
        }
        
    }
}