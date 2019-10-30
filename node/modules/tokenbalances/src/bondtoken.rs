#![cfg_attr(not(feature = "std"), no_std)]

extern crate srml_support as support;
extern crate srml_system as system;
extern crate srml_balances as balances;
extern crate sr_primitives as runtime_primitives;

use support::{decl_module, decl_storage, decl_event, ensure, dispatch::Result, dispatch::Vec};
use system::ensure_signed;

#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};

use parity_codec::{Encode, Decode};
use runtime_primitives::traits::Hash;
use stafi_primitives::{Balance, BondTokenLockType, BondTokenLockStatus, Symbol, CustomRedeemData}; 
use srml_timestamp as timestamp;


pub trait Trait: timestamp::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub struct BondToken<Moment, AccountId, Hash> {
	id: Hash,
	account_id: AccountId,
	symbol: Symbol,
	balance: Balance,
	capital_amount: Balance,
	rewards_amount: Balance,
	issue_time: Moment,
	stake_id: Hash,
	lock_ids: Vec<Hash>,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub struct LockBondData<Moment, Hash> {
	id: Hash,
	bond_id: Hash,
	lock_type: BondTokenLockType,
	lock_amount: Balance,
	lock_time: Moment,
	status: BondTokenLockStatus
}

decl_storage! {
	trait Store for Module<T: Trait> as BondToken {
		pub FreeBondToken get(free_bond_token): map (T::AccountId, T::Hash) => BondToken<T::Moment, T::AccountId, T::Hash>;
		pub LockBondToken get(get_lock_bond_token): map T::Hash => LockBondData<T::Moment, T::Hash>;
		pub BondTokenHashList get(bond_token_hash_list): map (T::AccountId, Symbol) => Vec<T::Hash>;
		pub RedeemRecords get(redeem_records): map (T::AccountId, T::Hash) => Option<CustomRedeemData<T::AccountId, T::Hash>>;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		// Custom redeem xtz
		pub fn custom_redeem(origin, bond_id: T::Hash, amount: Balance, original_account_id: Vec<u8>) -> Result {
			let sender = ensure_signed(origin)?;

			ensure!(amount > 0, "The amount to lock must be greater than 0");

			let key = (sender.clone(), bond_id.clone());
			ensure!(<FreeBondToken<T>>::exists(key.clone()), "This bond token does not exist");

			let bond_token = <FreeBondToken<T>>::get(key.clone());
			ensure!(bond_token.balance >= amount, "The balance of bond token is not enough");

			let lock_id = Self::lock_bond_token(sender.clone(), bond_id.clone(), amount, BondTokenLockType::Redemption);

			<RedeemRecords<T>>::insert((sender.clone(), lock_id.clone()), CustomRedeemData {
				initiator: sender.clone(),
				lock_id: lock_id.clone(),
				original_account_id: original_account_id,
			});

			Self::deposit_event(RawEvent::RedeemBondToken(sender, lock_id));

			Ok(())	
		}
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId, Hash = <T as system::Trait>::Hash {
		RedeemBondToken(AccountId, Hash),
	}
);

impl<T: Trait> Module<T> {
	pub fn get_all_bond_token(sender: T::AccountId) -> Vec<BondToken<T::Moment, T::AccountId, T::Hash>> {
		let xtz_bond_list = Self::get_bond_token_list(sender.clone(), Symbol::XtzBond);
		let atom_bond_list = Self::get_bond_token_list(sender.clone(), Symbol::AtomBond);

		let mut list = Vec::with_capacity(xtz_bond_list.len() + atom_bond_list.len());
		list.extend(xtz_bond_list);
		list.extend(atom_bond_list);

		return list;
    }

	pub fn get_bond_token_list(sender: T::AccountId, symbol: Symbol) -> Vec<BondToken<T::Moment, T::AccountId, T::Hash>> {
		let bond_token_hash_list = <BondTokenHashList<T>>::get((sender.clone(), symbol));
		return bond_token_hash_list.into_iter().map(|x| <FreeBondToken<T>>::get((sender.clone(), x))).collect();
    }

	pub fn add_bond_token(sender: T::AccountId, symbol: Symbol, amount: Balance, stake_id: T::Hash) -> Result {
		ensure!(amount > 0, "The amount must be greater than 0");

		/// TODO: Check symbol
		// ensure!(symbol, "The amount must be greater than 0");

		let random_seed = <system::Module<T>>::random_seed();
		let hash = (random_seed, &sender).using_encoded(<T as system::Trait>::Hashing::hash);
		let key = (sender.clone(), hash.clone());
		let now = <timestamp::Module<T>>::get();
		let bond_token = BondToken {
			id: hash,
			symbol: symbol,
			balance: amount,
			capital_amount: amount,
			rewards_amount: 0,
			account_id: sender.clone(),
			issue_time: now,
			stake_id: stake_id,
			lock_ids: Vec::new(),
		};

		Self::add_bond_hash_list(sender.clone(), hash.clone(), symbol.clone());

		<FreeBondToken<T>>::insert(key, bond_token);
		Ok(())
    }

	pub fn transfer_bond_token(sender: T::AccountId, bond_id: T::Hash, to: T::AccountId, amount: Balance) -> Result {
		ensure!(amount > 0, "The amount to transfer must be greater than 0");

		let key = (sender.clone(), bond_id.clone());
		ensure!(<FreeBondToken<T>>::exists(key.clone()), "This bond token does not exist");

		let mut bond_token = <FreeBondToken<T>>::get(key.clone());
		ensure!(bond_token.balance >= amount, "The balance of bond token is not enough");

		bond_token.balance -= amount;
		<FreeBondToken<T>>::insert(key.clone(), bond_token.clone());

		return Self::add_bond_token(to.clone(), bond_token.symbol, amount, bond_token.stake_id);
    }

	fn lock_bond_token(sender: T::AccountId, bond_id: T::Hash, amount: Balance, lock_type: BondTokenLockType) -> T::Hash {
		let key = (sender.clone(), bond_id.clone());
		let mut bond_token = <FreeBondToken<T>>::get(key.clone());	

		let lock_id = Self::add_lock_bond_token(sender.clone(), bond_id.clone(), amount, lock_type);

		bond_token.lock_ids.push(lock_id.clone());
		bond_token.balance -= amount;

		<FreeBondToken<T>>::insert(key, bond_token);
		
		lock_id
    }

	fn add_lock_bond_token(sender: T::AccountId, bond_id: T::Hash, amount: Balance, lock_type: BondTokenLockType) -> T::Hash {
		let random_seed = <system::Module<T>>::random_seed();
		let hash = (random_seed, &sender).using_encoded(<T as system::Trait>::Hashing::hash);
		let now = <timestamp::Module<T>>::get();
		let lock_bond_token = LockBondData {
			id: hash.clone(),
			bond_id: bond_id,
			lock_type: lock_type,
			lock_amount: amount,
			lock_time: now,
			status: BondTokenLockStatus::Locked
		};

		<LockBondToken<T>>::insert(hash.clone(), lock_bond_token);

		return hash;
    }

	pub fn complete_redeem_bond_token(sender: T::AccountId, lock_id: T::Hash) -> Result {
		ensure!(<LockBondToken<T>>::exists(lock_id.clone()), "This lock bond token does not exist");
		let mut lock_bond_data = <LockBondToken<T>>::get(lock_id.clone());

		let key = (sender.clone(), lock_bond_data.bond_id.clone());
		ensure!(<FreeBondToken<T>>::exists(key.clone()), "This bond token does not exist");

		let mut bond_token = <FreeBondToken<T>>::get(key.clone());

		let mut lock_id_list: Vec<T::Hash> = Vec::new();
		for t_lock_id in bond_token.lock_ids {
			if t_lock_id != lock_id {
				lock_id_list.push(t_lock_id);
			}
		}
		bond_token.lock_ids = lock_id_list;

		<FreeBondToken<T>>::insert(key, bond_token);

		lock_bond_data.status = BondTokenLockStatus::Completed;
		<LockBondToken<T>>::insert(lock_id, lock_bond_data);

		Ok(())
    }

	pub fn distribute_bond_rewards(sender: T::AccountId, bond_id: T::Hash, rewards_amount: Balance) -> Result {
		ensure!(rewards_amount > 0, "The number to transfer must be greater than 0");

		let key = (sender.clone(), bond_id.clone());
		ensure!(<FreeBondToken<T>>::exists(key.clone()), "This bond token does not exist");

		let mut bond_token = <FreeBondToken<T>>::get(key.clone());

		match bond_token.balance.checked_add(rewards_amount) {
			Some(balance_value) => {
				match bond_token.rewards_amount.checked_add(rewards_amount) {
					Some(rewards_value) => {
						bond_token.balance = balance_value;
						bond_token.rewards_amount = rewards_value;
						<FreeBondToken<T>>::insert(key, bond_token);
					},
					None => return Err("Add rewards amount error"),
				};
			},
			None => return Err("Add balance error"),
		};

		Ok(())
    }

	pub fn add_bond_hash_list(sender: T::AccountId, bond_id: T::Hash, symbol: Symbol) {
		let key = (sender.clone(), symbol);

		let mut hash_list = <BondTokenHashList<T>>::get(key.clone());
		hash_list.push(bond_id.clone());
		<BondTokenHashList<T>>::insert(key.clone(), hash_list);
    }
}


#[cfg(test)]
mod tests {
	use super::*;
	use hex_literal::hex;
	use primitives::{H256, Blake2Hasher};
	use support::{impl_outer_origin, assert_ok, parameter_types};
	use sr_primitives::{traits::{BlakeTwo256, IdentityLookup}, testing::Header};
	use sr_primitives::weights::Weight;
	use sr_primitives::Perbill;
	use stafi_primitives::{Balance, constants::currency::*};


	#[test]
	fn test_add_bond_token() {
		let sender =
			hex!["63410a24555c6f0c5ba8f0d27f85740dca150d9a0d67e3fa8502d5d9e6a4fafe"]
				.unchecked_into();
		let random_seed = <system::Module<Test>>::random_seed();
		let hash = (random_seed, &sender).using_encoded(<Test as system::Trait>::Hashing::hash);

		let new_status = add_bond_token(sender, Symbol::XtzBond, 10,  hash.clone());

		let key = (sender.clone(), hash.clone());
		let mut bond_token = <FreeBondToken<Test>>::get(key.clone());

		assert_eq!(hash, bond_token.lock_id);
	}
}