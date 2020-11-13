use sp_std::{prelude::*, fmt::Debug};
use codec::{FullCodec};
use sp_runtime::{
	DispatchResult, traits::{
		MaybeSerializeDeserialize, AtLeast32BitUnsigned
	},
};

pub trait Currency<AccountId> {
    type Balance: AtLeast32BitUnsigned + FullCodec + Copy + MaybeSerializeDeserialize + Debug +
    Default;

    /// Reduce the total issuance by `amount` and return the according imbalance. The imbalance will
	/// typically be used to reduce an account by the same amount with e.g. `settle`.
	///
	/// This is infallible, but doesn't guarantee that the entire `amount` is burnt, for example
	/// in the case of underflow.
    fn burn(amount: Self::Balance);

    /// Increase the total issuance by `amount` and return the according imbalance. The imbalance
	/// will typically be used to increase an account by the same amount with e.g.
	/// `resolve_into_existing` or `resolve_creating`.
	///
	/// This is infallible, but doesn't guarantee that the entire `amount` is issued, for example
	/// in the case of overflow.
	fn issue(amount: Self::Balance);

	/// Returns `Ok` iff the account is able to make a withdrawal of the given amount
	/// for the given reason. Basically, it's just a dry-run of `withdraw`.
	///
	/// `Err(...)` with the reason why not otherwise.
	fn ensure_can_withdraw(
		_amount: Self::Balance,
		new_balance: Self::Balance,
	) -> DispatchResult;
    
    /// Transfer some liquid free balance to another staker.
	///
	/// This is a very high-level function. It will ensure all appropriate fees are paid
	/// and no imbalance in the system remains.
	fn transfer(
		source: &AccountId,
		dest: &AccountId,
		value: Self::Balance,
    ) -> DispatchResult;

    /// The total amount of issuance in the system.
	fn total_issuance() -> Self::Balance;

	/// Deposit some `value` into the free balance of a target account `who`.
	///
	/// Is a no-op if the `value` to be deposited is zero.
	fn deposit_into(
		who: &AccountId,
		value: Self::Balance,
	) -> DispatchResult;

	/// Withdraw some `value` from the free balance of a target account `who`.
	///
	/// Is a no-op if the `value` to be withdrawed is zero.
	fn withdraw_from(
		who: &AccountId,
		value: Self::Balance,
	) -> DispatchResult;



}