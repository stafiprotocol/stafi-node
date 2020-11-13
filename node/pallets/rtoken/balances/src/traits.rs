use sp_std::{prelude::*, fmt::Debug};
use codec::{FullCodec, Encode, Decode};
use sp_runtime::{
	RuntimeDebug, DispatchResult, traits::{
		MaybeSerializeDeserialize, AtLeast32BitUnsigned
	},
};

/// Rtoken Identifier
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum RTokenIdentifier {
	/// FIS
	FIS,
}

pub trait Currency<AccountId> {
    type Balance: AtLeast32BitUnsigned + FullCodec + Copy + MaybeSerializeDeserialize + Debug +
    Default;
    
    /// Transfer some liquid free balance to another staker.
	///
	/// This is a very high-level function. It will ensure all appropriate fees are paid
	/// and no imbalance in the system remains.
	fn transfer(
		source: &AccountId,
		dest: &AccountId,
		symbol: RTokenIdentifier,
		value: Self::Balance,
    ) -> DispatchResult;

    /// The total amount of issuance in the system.
	fn total_issuance(symbol: RTokenIdentifier) -> Self::Balance;

	/// mint some `value` into the free balance of a target account `who`.
	///
	/// Is a no-op if the `value` to be deposited is zero.
	/// this will also change total issuance
	fn mint(
		who: &AccountId,
		symbol: RTokenIdentifier,
		value: Self::Balance,
	) -> DispatchResult;

	/// Withdraw some `value` from the free balance of a target account `who`.
	///
	/// Is a no-op if the `value` to be withdrawed is zero.
	/// this will also change total issuance 
	fn burn(
		who: &AccountId,
		symbol: RTokenIdentifier,
		value: Self::Balance,
	) -> DispatchResult;



}