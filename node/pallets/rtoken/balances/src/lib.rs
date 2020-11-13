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
		/// An account was created with some free balance. \[account, free_balance\]
		Endowed(AccountId, Balance),
		/// Transfer succeeded. \[from, to, value\]
        Transfer(AccountId, AccountId, Balance),
        /// A balance was set by root. \[who, free, reserved\]
		BalanceSet(AccountId, Balance, Balance),
		/// Some balance was reserved (moved from free to reserved). \[who, value\]
		Reserved(AccountId, Balance),
		/// Some balance was unreserved (moved from reserved to free). \[who, value\]
		Unreserved(AccountId, Balance),
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

/// Rtoken Identifier
#[derive(Encode, Decode)]
pub enum RTokenIdentifier {
	/// FIS
	FIS,
}

decl_storage! {
	trait Store for Module<T: Trait> as RTokenBalances {
		/// The total units issued in the system.
        pub TotalIssuance get(fn total_issuance): T::Balance;

		/// NOTE: This is only used in the case that this module is used to store balances.
        pub Account get(fn account): map hasher(blake2_128_concat) T::AccountId => Option<AccountRData<T::Balance>>;
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
			#[compact] value: T::Balance
		) {
			let transactor = ensure_signed(origin)?;
			let dest = T::Lookup::lookup(dest)?;
            <Self as traits::Currency<_>>::transfer(&transactor, &dest, value)?;
        }

        /// Set the balances of a given account.
        #[weight = 195_000_000]
        fn set_balance(
            origin,
            who: <T::Lookup as StaticLookup>::Source,
            #[compact] new_free: T::Balance,
            #[compact] new_reserved: T::Balance
        ) {
            ensure_root(origin)?;
            let who = T::Lookup::lookup(who)?;

            let new_free = if new_free < Zero::zero() { Zero::zero() } else { new_free };
            let new_reserved = if new_reserved < Zero::zero() { Zero::zero() } else { new_reserved };

            let (free, reserved) = Self::mutate_account(&who, |account| {
                account.free = new_free;
                account.reserved = new_reserved;

                (account.free, account.reserved)
            });
            Self::deposit_event(RawEvent::BalanceSet(who, free, reserved));
		}
    }
}

impl<T: Trait> Module<T> {
    pub fn mutate_account<R>(
		who: &T::AccountId,
		f: impl FnOnce(&mut AccountRData<T::Balance>) -> R
	) -> R {
		Self::try_mutate_account(who, |a| -> Result<R, Infallible> { Ok(f(a)) })
			.expect("Error is infallible; qed")
	}

    /// Mutate an account to some new value
	/// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
	/// the caller will do this.
	fn try_mutate_account<R, E>(
		who: &T::AccountId,
		f: impl FnOnce(&mut AccountRData<T::Balance>) -> Result<R, E>
	) -> Result<R, E> {
        Account::<T>::try_mutate_exists(who, |maybe_value| {
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

	fn total_issuance() -> Self::Balance {
		<TotalIssuance<T>>::get()
	}

	// Burn funds from the total issuance
	fn burn(mut amount: Self::Balance) {
		if amount.is_zero() { return }
		<TotalIssuance<T>>::mutate(|issued| {
			*issued = issued.checked_sub(&amount).unwrap_or_else(|| {
                amount = *issued;
                Zero::zero()
			});
        });
	}

	// Create new funds into the total issuance
	fn issue(mut amount: Self::Balance) {
		if amount.is_zero() { return }
		<TotalIssuance<T>>::mutate(|issued|
			*issued = issued.checked_add(&amount).unwrap_or_else(|| {
                amount = Self::Balance::max_value() - *issued;
                Self::Balance::max_value()
			})
        );
	}

	// Ensure that an account can withdraw from their free balance given any existing withdrawal
	// restrictions like locks and vesting balance.
	// Is a no-op if amount to be withdrawn is zero.
	//
	// # <weight>
	// Despite iterating over a list of locks, they are limited by the number of
	// lock IDs, which means the number of runtime modules that intend to use and create locks.
	// # </weight>
	fn ensure_can_withdraw(
		amount: T::Balance,
		new_balance: T::Balance,
	) -> DispatchResult {
		if amount.is_zero() { return Ok(()) }
		let min_balance = Zero::zero();
		ensure!(new_balance >= min_balance, Error::<T>::LiquidityRestrictions);
		Ok(())
	}

	// Transfer some free balance from `transactor` to `dest`, respecting existence requirements.
	// Is a no-op if value to be transferred is zero or the `transactor` is the same as `dest`.
	fn transfer(
		transactor: &T::AccountId,
		dest: &T::AccountId,
		value: Self::Balance,
	) -> DispatchResult {
        if value.is_zero() || transactor == dest { return Ok(()) }
        
		Self::try_mutate_account(dest, |to_account_rdata| -> DispatchResult {
			Self::try_mutate_account(transactor, |from_account_rdata| -> DispatchResult {
				from_account_rdata.free = from_account_rdata.free.checked_sub(&value)
					.ok_or(Error::<T>::InsufficientBalance)?;

				// NOTE: total stake being stored in the same type means that this could never overflow
				// but better to be safe than sorry.
				to_account_rdata.free = to_account_rdata.free.checked_add(&value).ok_or(Error::<T>::Overflow)?;

				Self::ensure_can_withdraw(
					value,
					from_account_rdata.free,
                )?;
                
				Ok(())
			})
		})?;

		// Emit transfer event.
		Self::deposit_event(RawEvent::Transfer(transactor.clone(), dest.clone(), value));

		Ok(())
	}

	/// Deposit some `value` into the free balance of an existing target account `who`.
	///
	/// Is a no-op if the `value` to be deposited is zero.
	fn deposit_into(
		who: &T::AccountId,
		value: Self::Balance
	) -> DispatchResult {
		if value.is_zero() { return Ok(()) }

		Self::try_mutate_account(who, |account_rdata| -> DispatchResult {
			account_rdata.free = account_rdata.free.checked_add(&value).ok_or(Error::<T>::Overflow)?;
			Ok(())
		})
	}

	/// Deposit some `value` into the free balance of an existing target account `who`.
	///
	/// Is a no-op if the `value` to be deposited is zero.
	fn withdraw_from(
		who: &T::AccountId,
		value: Self::Balance
	) -> DispatchResult {
		if value.is_zero() { return Ok(()) }

		Self::try_mutate_account(who, |account_rdata| -> DispatchResult {
			account_rdata.free = account_rdata.free.checked_sub(&value).ok_or(Error::<T>::InsufficientBalance)?;

			Self::ensure_can_withdraw(
				value,
				account_rdata.free,
			)?;

			Ok(())
		})
	}
}