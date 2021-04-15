// Copyright 2019-2020 Stafi Protocol.
// This file is part of Stafi.

// Stafi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stafi.  If not, see <http://www.gnu.org/licenses/>.

//! Module to process claims from Staking drop.


#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{prelude::*};
use frame_support::{
	decl_event, decl_storage, decl_module, decl_error, dispatch, ensure,
	traits::{Currency, Get}
};
use frame_system::{self as system, ensure_root, ensure_signed};
use sp_runtime::{
	ModuleId,
	traits::{
		CheckedSub
	},
};
use codec::{Encode};

const MODULE_ID: ModuleId = ModuleId(*b"xsym/clm");

/// Configuration trait.
pub trait Trait: system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// The currency trait.
	type Currency: Currency<Self::AccountId>;
}

// This pallet's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as Claim {
		pub Claims get(fn claims): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) XSymbol => Option<u128>;
		pub Claimed get(fn claimed): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) XSymbol => Option<u128>;
		pub Total get(fn total): map hasher(blake2_128_concat) XSymbol => u128;
		/// Proxy accounts for setting fees
        ProxyAccounts get(fn proxy_accounts): map hasher(blake2_128_concat) T::AccountId => Option<u8>;
	}
}

// The pallet's events
decl_event!(
	pub enum Event<T> where
        AccountId = <T as system::Trait>::AccountId
    {
        /// Someone claimed some XSymbol tokens.
		Claimed(AccountId, XSymbol, u128),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
        /// Address has no claim.
		HasNoClaim,
		/// There's not enough in the pot to pay out some unvested amount. Generally implies a logic
		/// error.
		PotUnderflow,
		/// zero value
		ValueZero
		/// invalid proxy account
		InvalidProxyAccount,
		/// Got an overflow after adding
        OverFlow,
	}
}

// The pallet's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing errors
		type Error = Error<T>;

		// Initializing events
		fn deposit_event() = default;

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

        /// Make a claim
		#[weight = T::DbWeight::get().reads_writes(5, 4) + 50_000_000]
		pub fn claim(origin, XSymbol: symbol) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;

            let balance_claim = <Claims<T>>::get(&who, symbol).ok_or(Error::<T>::HasNoClaim)?;
			let balance_claimed = <Claimed<T>>::get(&who, symbol).unwrap_or(0);
			let balance_due = balance_claim.checked_sub(balance_claimed).ok_or(Error::<T>::PotUnderflow)?;

			ensure!(balance_due > 0, Error::<T>::HasNoClaim);

            let new_total = Self::total(symbol).checked_sub(&balance_due).ok_or(Error::<T>::PotUnderflow)?;

			/// Todo: Transfer to user

            Total::insert(symbol, new_total);
            <Claimed<T>>::insert(&who, symbol, balance_claim);

			Self::deposit_event(RawEvent::Claimed(who, symbol, balance_due));
			Ok(())
		}

        /// Mint a new claim.
		#[weight = T::DbWeight::get().reads_writes(2, 2) + 10_000_000]
		pub fn mint_claim(origin, dest: T::AccountId, XSymbol: symbol, value: u128) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(value > 0, Error::<T>::ValueZero);
            ensure!(<ProxyAccounts<T>>::contains_key(&who), Error::<T>::InvalidProxyAccount);

            let old_balance = <Claims<T>>::get(&dest, symbol).unwrap_or(0);
			let balance_due = value.checked_add(old_balance).ok_or(Error::<T>::OverFlow)?;
			let new_total = Self::total(symbol).checked_add(value).ok_or(Error::<T>::OverFlow)?;

			/// Todo: Check new_total

            Total::insert(symbol, new_total);
            <Claims<T>>::insert(dest, symbol, balance_due);

            Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
    /// Provides an AccountId for the pallet
    /// This is used to claim XSymbol token
    pub fn account_id() -> T::AccountId {
        MODULE_ID.into_account()
    }
}