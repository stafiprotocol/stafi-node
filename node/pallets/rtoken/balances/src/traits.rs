use sp_std::{prelude::*, fmt::Debug};
use codec::{FullCodec, Encode, Decode};
use sp_runtime::{
	RuntimeDebug, DispatchResult, traits::{
		MaybeSerializeDeserialize, AtLeast32BitUnsigned
	},
};
use node_primitives::RSymbol;

pub trait Currency<AccountId> {
    type RBalance: AtLeast32BitUnsigned + FullCodec + Copy + MaybeSerializeDeserialize + Debug +
	Default;

	/// The 'free' balance of a given account.
	fn free_balance(who: &AccountId, symbol: RSymbol) -> Self::RBalance;
	
	/// Returns `Ok` iff the account is able to make a withdrawal of the given amount
	/// for the given reason. Basically, it's just a dry-run of `withdraw`.
	///
	/// `Err(...)` with the reason why not otherwise.
	fn ensure_can_withdraw(
		who: &AccountId,
		symbol: RSymbol,
		_amount: Self::RBalance,
		new_balance: Self::RBalance,
	) -> DispatchResult;
    
    /// Transfer some liquid free balance to another staker.
	///
	/// This is a very high-level function. It will ensure all appropriate fees are paid
	/// and no imbalance in the system remains.
	fn transfer(
		source: &AccountId,
		dest: &AccountId,
		symbol: RSymbol,
		value: Self::RBalance,
    ) -> DispatchResult;

    /// The total amount of issuance in the system.
	fn total_issuance(symbol: RSymbol) -> Self::RBalance;

	/// mint some `value` into the free balance of a target account `who`.
	///
	/// Is a no-op if the `value` to be deposited is zero.
	/// this will also change total issuance
	fn mint(
		who: &AccountId,
		symbol: RSymbol,
		value: Self::RBalance,
	) -> DispatchResult;

	/// Withdraw some `value` from the free balance of a target account `who`.
	///
	/// Is a no-op if the `value` to be withdrawed is zero.
	/// this will also change total issuance 
	fn burn(
		who: &AccountId,
		symbol: RSymbol,
		value: Self::RBalance,
	) -> DispatchResult;
}