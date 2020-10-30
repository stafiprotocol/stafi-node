// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use codec::{Encode, Decode};
use frame_support::{
    Parameter, decl_error, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{EnsureOrigin, Get},
};

use frame_system::{self as system, ensure_signed, ensure_root};
use sp_core::U256;
use sp_runtime::traits::{AccountIdConversion};
use sp_runtime::{ModuleId};
use pallet_staking::EraIndex;

const MODULE_ID: ModuleId = ModuleId(*b"rFIS/pot");

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// The currency mechanism.
    type Currency: Currency<Self::AccountId>;
}

decl_event! {
    pub enum Event<T> where
        AccountId = <T as system::Trait>::AccountId
    {
        /// liquidity stake record
        LiquidityStake(AccountId, BalanceOf<T>),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Got an overflow after adding
		Overflow,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RTokenFis {
        pub TotalStake: get(fn total_stake): BalanceOf<T>;

        pub LiquidityStake: get(liquidity_stake) map hasher(blake2_128_concat) T:AccountId => BalanceOf<T>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Generate a new account by a string
        #[weight = 195_000_000]
        pub fn liquidity_stake(origin, amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let stake = Self::liquidity_stake(&who);
            let new_stake = stake.checked_add(&amount).ok_or(Error::<T>::Overflow)?;
            let total_stake = Self::total_stake();
            let new_total_stake = total_stake.checked_add(&amount).ok_or(Error::<T>::Overflow)?;
            /// TODO: finish the rest
        }
    }
}

