#![cfg_attr(not(feature = "std"), no_std)]
use sp_std::prelude::*;
use sp_std::{cmp, result, mem, fmt::Debug, convert::Infallible};
use codec::{Codec, Encode, Decode};
use frame_support::{
	StorageValue, Parameter, decl_event, decl_storage, decl_module, decl_error, ensure,
};
use sp_runtime::{
	RuntimeDebug, DispatchResult, DispatchError,
	traits::{
		Zero, AtLeast32BitUnsigned, StaticLookup, Member, CheckedAdd, CheckedSub,
		MaybeSerializeDeserialize, Saturating, Bounded,
	},
};
use frame_system::{self as system, ensure_signed, ensure_root};
pub mod traits;

pub type RTokenIdentifier = traits::RTokenIdentifier;

pub trait Trait: system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	/// The balance of an account.
	type Balance: Parameter + Member + AtLeast32BitUnsigned + Codec + Default + Copy +
        MaybeSerializeDeserialize + Debug;
}

decl_event!(
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
		<T as Trait>::Balance
	{
		/// Transfer succeeded. \[from, to, symbol, value\]
        Transfer(AccountId, AccountId, RTokenIdentifier, Balance),
        /// A balance was set by root. \[who, free, reserved\]
		BalanceSet(AccountId, Balance, RTokenIdentifier, Balance),
		/// Some balance was reserved (moved from free to reserved). \[who, value\]
		Reserved(AccountId, RTokenIdentifier, Balance),
		/// Some balance was unreserved (moved from reserved to free). \[who, value\]
		Unreserved(AccountId, RTokenIdentifier, Balance),
		/// Some balance was deposited
		Minted(AccountId, RTokenIdentifier, Balance),
		/// Some balance was withdraswed
		Burned(AccountId, RTokenIdentifier, Balance),
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
pub struct AccountRData<Balance> {
	/// Non-reserved part of the balance.
	pub free: Balance,
	/// Balance which is reserved and may not be used at all.
	pub reserved: Balance,
}

decl_storage! {
	trait Store for Module<T: Trait> as RTokenBalances {
		/// The total units issued in the system.
        pub TotalIssuance get(fn total_issuance): map hasher(blake2_128_concat) RTokenIdentifier => T::Balance;

		/// NOTE: This is only used in the case that this module is used to store balances.
		pub Account get(fn account):
			double_map hasher(blake2_128_concat) RTokenIdentifier, hasher(blake2_128_concat) T::AccountId => Option<AccountRData<T::Balance>>;
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
			symbol: RTokenIdentifier,
			#[compact] value: T::Balance
		) {
			let transactor = ensure_signed(origin)?;
			let dest = T::Lookup::lookup(dest)?;
            <Self as traits::Currency<_>>::transfer(&transactor, &dest, symbol, value)?;
        }

        /// Set the balances of a given account.
        #[weight = 195_000_000]
        fn set_balance(
            origin,
			who: <T::Lookup as StaticLookup>::Source,
			symbol: RTokenIdentifier,
            #[compact] value: T::Balance,
        ) {
            ensure_root(origin)?;
            let who = T::Lookup::lookup(who)?;
			<Self as traits::Currency<_>>::mint(&who, symbol, value)?;
		}
    }
}

impl<T: Trait> Module<T> {
    pub fn mutate_account<R>(
		who: &T::AccountId,
		symbol: RTokenIdentifier,
		f: impl FnOnce(&mut AccountRData<T::Balance>) -> R
	) -> R {
		Self::try_mutate_account(who, symbol, |a| -> Result<R, Infallible> { Ok(f(a)) })
			.expect("Error is infallible; qed")
	}

    /// Mutate an account to some new value
	/// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
	/// the caller will do this.
	fn try_mutate_account<R, E>(
		who: &T::AccountId,
		symbol: RTokenIdentifier,
		f: impl FnOnce(&mut AccountRData<T::Balance>) -> Result<R, E>
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

impl<T: Trait> traits::Currency<T::AccountId> for Module<T> where
	T::Balance: MaybeSerializeDeserialize + Debug
{
	type Balance = T::Balance;

	fn total_issuance(symbol: RTokenIdentifier) -> Self::Balance {
		<TotalIssuance<T>>::get(symbol)
	}

	// Transfer some free balance from `transactor` to `dest`, respecting existence requirements.
	// Is a no-op if value to be transferred is zero or the `transactor` is the same as `dest`.
	fn transfer(
		transactor: &T::AccountId,
		dest: &T::AccountId,
		symbol: RTokenIdentifier,
		value: Self::Balance,
	) -> DispatchResult {
        if value.is_zero() || transactor == dest { return Ok(()) }
        
		Self::try_mutate_account(dest, symbol, |to_account_rdata| -> DispatchResult {
			Self::try_mutate_account(transactor, symbol, |from_account_rdata| -> DispatchResult {
				from_account_rdata.free = from_account_rdata.free.checked_sub(&value)
					.ok_or(Error::<T>::InsufficientBalance)?;

				to_account_rdata.free = to_account_rdata.free.checked_add(&value).ok_or(Error::<T>::Overflow)?;
                
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
		symbol: RTokenIdentifier,
		value: Self::Balance
	) -> DispatchResult {
		if value.is_zero() { return Ok(()) }

		Self::try_mutate_account(who, symbol, |account_rdata| -> DispatchResult {
			account_rdata.free = account_rdata.free.checked_add(&value).ok_or(Error::<T>::Overflow)?;
			Ok(())
		})?;

		<TotalIssuance<T>>::mutate(symbol, |issued|
			*issued = issued.checked_add(&value).unwrap_or_else(|| {
                Self::Balance::max_value()
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
		symbol: RTokenIdentifier,
		value: Self::Balance
	) -> DispatchResult {
		if value.is_zero() { return Ok(()) }
		
		Self::try_mutate_account(who, symbol, |account_rdata| -> DispatchResult {
			account_rdata.free = account_rdata.free.checked_sub(&value).ok_or(Error::<T>::InsufficientBalance)?;

			Ok(())
		})?;

		<TotalIssuance<T>>::mutate(symbol, |issued| {
			*issued = issued.checked_sub(&value).unwrap_or_else(|| {
                Zero::zero()
			});
        });

		// withdraw from event.
		Self::deposit_event(RawEvent::Burned(who.clone(), symbol.clone(), value));
		Ok(())
	}
}