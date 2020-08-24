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
use frame_system::{self as system, ensure_root, ensure_none};
use sp_runtime::{
	traits::{
		Zero, Saturating, CheckedSub
	},
    transaction_validity::{
		TransactionLongevity, TransactionValidity, ValidTransaction, InvalidTransaction,
		TransactionSource,
	},
};
use codec::{Encode};
use node_primitives::ValidityError;

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

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
        Claims get(fn claims): map hasher(identity) T::AccountId => Option<BalanceOf<T>>;
        Claimed get(fn claimed): map hasher(identity) T::AccountId => Option<BalanceOf<T>>;
        Total get(fn total): BalanceOf<T>;
	}
}

// The pallet's events
decl_event!(
	pub enum Event<T> where
        Balance = BalanceOf<T>,
        AccountId = <T as system::Trait>::AccountId
    {
        /// Someone claimed some tokens.
		Claimed(AccountId, Balance),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
        /// Address has no new claim.
		InvalidClaimValue,
        /// Address has no claim.
		DestHasNoClaim,
		/// There's not enough in the pot to pay out some unvested amount. Generally implies a logic
		/// error.
		PotUnderflow,
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

        /// Make a claim to collect your FIS.
		///
		/// The dispatch origin for this call must be _None_.
		///
        /// Total Complexity: O(1)
		/// ----------------------------
		/// DB Weight:
		/// - Read: Claims, Claimed, Total, Balance Lock, Account
		/// - Write: Account, Balance Lock, Total, Claimed
		/// </weight>
		#[weight = T::DbWeight::get().reads_writes(5, 4) + 50_000_000]
		pub fn claim(origin, dest: T::AccountId) -> dispatch::DispatchResult {
			ensure_none(origin)?;

            let balance_claim = <Claims<T>>::get(&dest).ok_or(Error::<T>::DestHasNoClaim)?;

            let balance_claimed = match <Claimed<T>>::get(&dest) {
                None => Zero::zero(),
                Some(claimed) => claimed
            };

            let balance_due = balance_claim.saturating_sub(balance_claimed);

            if balance_due.is_zero() {
                return Err(Error::<T>::DestHasNoClaim)?;
            }

            let new_total = Self::total().checked_sub(&balance_due).ok_or(Error::<T>::PotUnderflow)?;

            // We first need to deposit the balance to ensure that the account exists.
		    T::Currency::deposit_creating(&dest, balance_due);

            <Total<T>>::put(new_total);
            <Claimed<T>>::insert(&dest, balance_claim);

			Self::deposit_event(RawEvent::Claimed(dest, balance_due));
			Ok(())
		}

        /// Mint a new claim to collect FIS.
		///
		/// The dispatch origin for this call must be _Root_.
		///
		/// Parameters:
		/// - `who`: The address allowed to collect this claim.
		/// - `value`: The number of FIS that will be claimed.
        /// 
		/// <weight>
		/// The weight of this call is invariant over the input parameters.
		/// - One storage mutate to increase the total claims available.
		/// - One storage write to add a new claim.
		///
		/// Total Complexity: O(1)
		/// ---------------------
		/// DB Weight:
		/// - Reads: Claims, Total
		/// - Writes: Total, Claims
		/// - Maybe Write: Vesting, Statement
		/// </weight>
		#[weight = T::DbWeight::get().reads_writes(2, 2) + 10_000_000]
		pub fn mint_claim(origin, who: T::AccountId, value: BalanceOf<T>) -> dispatch::DispatchResult {
			ensure_root(origin)?;

            let old_balance = match <Claims<T>>::get(&who) {
                None => Zero::zero(),
                Some(old) => old
            };

            let balance_due = value.saturating_sub(old_balance);

            if balance_due.is_zero() {
                return Err(Error::<T>::InvalidClaimValue)?;
            }    

            <Total<T>>::mutate(|t| *t += balance_due);
            <Claims<T>>::insert(who, value);

            Ok(())
		}
	}
}

impl<T: Trait> sp_runtime::traits::ValidateUnsigned for Module<T> {
	type Call = Call<T>;

	fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
		const PRIORITY: u64 = 100;

		let dest = match call {
			Call::claim(account) => {
				account
			}
			_ => return Err(InvalidTransaction::Call.into()),
		};

		let e = InvalidTransaction::Custom(ValidityError::DestHasNoClaim.into());
		ensure!(<Claims<T>>::contains_key(&dest), e);

		let balance_claims = match <Claims<T>>::get(&dest) {
			None => Zero::zero(),
			Some(claims) => claims
		};

		Ok(ValidTransaction {
			priority: PRIORITY,
			requires: vec![],
			provides: vec![("claims", dest, balance_claims).encode()],
			longevity: TransactionLongevity::max_value(),
			propagate: true,
		})
	}
}