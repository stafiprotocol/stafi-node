use sp_runtime::{DispatchResult};
use node_primitives::XSymbol;

pub trait Currency<AccountId> {
	/// The 'free' balance of a given account.
	fn free_balance(who: &AccountId, symbol: XSymbol) -> u128;
	
	/// Returns `Ok` iff the account is able to make a withdrawal of the given amount
	/// for the given reason. Basically, it's just a dry-run of `withdraw`.
	///
	/// `Err(...)` with the reason why not otherwise.
	fn ensure_can_withdraw(
		who: &AccountId,
		symbol: XSymbol,
		_amount: u128,
		new_balance: u128,
	) -> DispatchResult;
    
    /// Transfer some liquid free balance to another staker.
	///
	/// This is a very high-level function. It will ensure all appropriate fees are paid
	/// and no imbalance in the system remains.
	fn transfer(
		source: &AccountId,
		dest: &AccountId,
		symbol: XSymbol,
		value: u128,
    ) -> DispatchResult;

    /// The total amount of issuance in the system.
	fn total_issuance(symbol: XSymbol) -> u128;

	/// mint some `value` into the free balance of a target account `who`.
	///
	/// Is a no-op if the `value` to be deposited is zero.
	/// this will also change total issuance
	fn mint(
		who: &AccountId,
		symbol: XSymbol,
		value: u128,
	) -> DispatchResult;

	/// Withdraw some `value` from the free balance of a target account `who`.
	///
	/// Is a no-op if the `value` to be withdrawed is zero.
	/// this will also change total issuance 
	fn burn(
		who: &AccountId,
		symbol: XSymbol,
		value: u128,
	) -> DispatchResult;
}