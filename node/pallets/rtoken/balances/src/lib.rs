#![cfg_attr(not(feature = "std"), no_std)]
use sp_std::prelude::*;
use sp_std::{convert::Infallible};
use codec::{Encode, Decode};
use frame_support::{
	decl_event, decl_storage, decl_module, decl_error, ensure,
};
use sp_runtime::{
	RuntimeDebug, DispatchResult,
	traits::{
		Zero, StaticLookup,
	},
};
use frame_system::{self as system, ensure_signed};
use node_primitives::RSymbol;

pub mod traits;

pub trait Trait: system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_event!(
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
	{
		/// Transfer succeeded. \[from, to, symbol, value\]
        Transfer(AccountId, AccountId, RSymbol, u128),
		/// Some balance was deposited
		Minted(AccountId, RSymbol, u128),
		/// Some balance was burned
		Burned(AccountId, RSymbol, u128),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Account liquidity restrictions prevent withdrawal
		LiquidityRestrictions,
		/// Got an overflow after adding
		Overflow,
		/// Balance too low to send value
		InsufficientBalance,
	}
}

/// All balance information for an account.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct AccountRData {
	/// Non-reserved part of the balance.
	pub free: u128,
}

decl_storage! {
	trait Store for Module<T: Trait> as RBalances {
		/// The total units issued in the system.
        pub TotalIssuance get(fn total_issuance): map hasher(blake2_128_concat) RSymbol => u128;

		/// NOTE: This is only used in the case that this module is used to store balances.
		pub Account get(fn account):
			double_map hasher(blake2_128_concat) RSymbol, hasher(blake2_128_concat) T::AccountId => Option<AccountRData>;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		/// Transfer some liquid free balance to another account.
		#[weight = 195_000_000]
		pub fn transfer(
			origin,
			dest: <T::Lookup as StaticLookup>::Source,
			symbol: RSymbol,
			value: u128
		) {
			let transactor = ensure_signed(origin)?;
			let dest = T::Lookup::lookup(dest)?;
            <Self as traits::Currency<_>>::transfer(&transactor, &dest, symbol, value)?;
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn mutate_account<R>(
		who: &T::AccountId,
		symbol: RSymbol,
		f: impl FnOnce(&mut AccountRData) -> R
	) -> R {
		Self::try_mutate_account(who, symbol, |a| -> Result<R, Infallible> { Ok(f(a)) })
			.expect("Error is infallible; qed")
	}

    /// Mutate an account to some new value
	/// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
	/// the caller will do this.
	fn try_mutate_account<R, E>(
		who: &T::AccountId,
		symbol: RSymbol,
		f: impl FnOnce(&mut AccountRData) -> Result<R, E>
	) -> Result<R, E> {
        Account::<T>::try_mutate_exists(symbol, who, |maybe_value| {
            let mut maybe_data = maybe_value.take().unwrap_or_default();
			f(&mut maybe_data).map(|result| {
                *maybe_value = Some(maybe_data);
				result
			})
		}).map(|result| {
            result
        })
	}
}

impl<T: Trait> traits::Currency<T::AccountId> for Module<T>
{
	fn free_balance(who: &T::AccountId, symbol: RSymbol) -> u128 {
		if let Some(rdata) = <Account<T>>::get(symbol, &who) {
			rdata.free
		} else {
			0
		}
	}

	fn total_issuance(symbol: RSymbol) -> u128 {
		<TotalIssuance>::get(symbol)
	}

	// Ensure that an account can withdraw from their free balance given any existing withdrawal
	// restrictions like locks and vesting balance.
	// Is a no-op if amount to be withdrawn is zero.
	fn ensure_can_withdraw(
		_who: &T::AccountId,
		_symbol: RSymbol,
		amount: u128,
		new_balance: u128,
	) -> DispatchResult {
		if amount.is_zero() { return Ok(()) }
		// let min_balance = <Account<T>>::get(symbol, &who).unwrap_or_default().frozen;
		ensure!(new_balance >= Zero::zero(), Error::<T>::LiquidityRestrictions);
		Ok(())
	}

	// Transfer some free balance from `transactor` to `dest`, respecting existence requirements.
	// Is a no-op if value to be transferred is zero or the `transactor` is the same as `dest`.
	fn transfer(
		transactor: &T::AccountId,
		dest: &T::AccountId,
		symbol: RSymbol,
		value: u128,
	) -> DispatchResult {
        if value.is_zero() || transactor == dest { return Ok(()) }
        
		Self::try_mutate_account(dest, symbol, |to_account_rdata| -> DispatchResult {
			Self::try_mutate_account(transactor, symbol, |from_account_rdata| -> DispatchResult {
				from_account_rdata.free = from_account_rdata.free.checked_sub(value)
					.ok_or(Error::<T>::InsufficientBalance)?;

				to_account_rdata.free = to_account_rdata.free.checked_add(value).ok_or(Error::<T>::Overflow)?;

				Self::ensure_can_withdraw(transactor, symbol, value, from_account_rdata.free)?;
                
				Ok(())
			})
		})?;

		// Emit transfer event.
		Self::deposit_event(RawEvent::Transfer(transactor.clone(), dest.clone(), symbol.clone(), value));

		Ok(())
	}

	/// Deposit some `value` into the free balance of an existing target account `who`.
	///
	/// Is a no-op if the `value` to be deposited is zero.
	fn mint(
		who: &T::AccountId,
		symbol: RSymbol,
		value: u128
	) -> DispatchResult {
		if value.is_zero() { return Ok(()) }

		Self::try_mutate_account(who, symbol, |account_rdata| -> DispatchResult {
			account_rdata.free = account_rdata.free.checked_add(value).ok_or(Error::<T>::Overflow)?;
			Ok(())
		})?;

		<TotalIssuance>::mutate(symbol, |issued|
			*issued = issued.checked_add(value).unwrap_or_else(|| {
                u128::max_value()
			})
        );

		// deposit into event.
		Self::deposit_event(RawEvent::Minted(who.clone(), symbol.clone(), value));
		Ok(())
	}

	/// Deposit some `value` into the free balance of an existing target account `who`.
	///
	/// Is a no-op if the `value` to be deposited is zero.
	fn burn(
		who: &T::AccountId,
		symbol: RSymbol,
		value: u128
	) -> DispatchResult {
		if value.is_zero() { return Ok(()) }
		
		Self::try_mutate_account(who, symbol, |account_rdata| -> DispatchResult {
			account_rdata.free = account_rdata.free.checked_sub(value).ok_or(Error::<T>::InsufficientBalance)?;
			Self::ensure_can_withdraw(who, symbol, value, account_rdata.free)?;

			Ok(())
		})?;

		<TotalIssuance>::mutate(symbol, |issued| {
			*issued = issued.checked_sub(value).unwrap_or_else(|| {
                0
			});
        });

		// withdraw from event.
		Self::deposit_event(RawEvent::Burned(who.clone(), symbol.clone(), value));
		Ok(())
	}
}