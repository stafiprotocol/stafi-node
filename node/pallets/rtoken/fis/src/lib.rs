// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use codec::{Encode, Decode};
use frame_support::{
    Parameter, decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{Currency, EnsureOrigin, Get},
};

use frame_system::{self as system, ensure_signed, ensure_root};
use sp_runtime::traits::{AccountIdConversion, CheckedAdd};
use sp_runtime::{ModuleId};
use pallet_staking::EraIndex;

const POOL_ID_1: ModuleId = ModuleId(*b"rFISpot1");

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

pub trait Trait: system::Trait + pallet_staking::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// The currency mechanism.
    type Currency: Currency<Self::AccountId>;
}

decl_event! {
    pub enum Event<T> where
        Balance = BalanceOf<T>, <T as frame_system::Trait>::AccountId
    {
        /// liquidity stake record
        LiquidityStake(AccountId, Balance),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Got an overflow after adding
		Overflow,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as FisStaking {
        pub TotalStake get(fn total_stake): BalanceOf<T>;

        pub LiquidityStakeAmount get(fn liquidity_stake_amount): map hasher(blake2_128_concat) T::AccountId => BalanceOf<T>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Generate a new account by a string
        #[weight = 195_000_000]
        pub fn liquidity_stake(origin, amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let stake = Self::liquidity_stake_amount(&who);
            let new_stake = stake.checked_add(&amount).ok_or(Error::<T>::Overflow)?;
            let total_stake = Self::total_stake();
            let new_total_stake = total_stake.checked_add(&amount).ok_or(Error::<T>::Overflow)?;
            /// TODO: finish the rest 
            

            Ok(())
        }

        #[weight = 100_000_000]
		pub fn init(origin) -> DispatchResult {
			ensure_root(origin)?;

            let stash: T::AccountId = Self::account_id_1();

            pallet_staking::Bonded::<T>::insert(&stash, &stash);

            Ok(())
		}
    }
}

impl<T: Trait> Module<T> {
    /// Provides an AccountId for the pallet.
    /// This is used both as an origin check and deposit/withdrawal account.
    pub fn account_id_1() -> T::AccountId {
        POOL_ID_1.into_account()
    }
}

