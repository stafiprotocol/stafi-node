#![cfg_attr(not(feature = "std"), no_std)]

extern crate srml_support as support;
extern crate srml_system as system;
extern crate srml_balances as balances;
extern crate sr_primitives as runtime_primitives;

use support::{decl_module, decl_storage, decl_event, ensure, dispatch::Result, dispatch::Vec};
use system::ensure_signed;
use parity_codec::{Encode, Decode};
use runtime_primitives::traits::Hash;
use stafi_primitives::{Balance, BondTokenStatus, Symbol, CustomRedeemData}; 
use srml_timestamp as timestamp;


pub trait Trait: balances::Trait + timestamp::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// #[cfg_attr(feature = "std", derive(Debug))]
// #[derive(Encode, Decode, Copy, Clone, Eq, PartialEq)]
// pub enum Symbol {
// 	XtzBond,
// 	AtomBond,
// }
// impl Default for Symbol {
// 	fn default() -> Symbol {
// 		Symbol::XtzBond
// 	}
// }

#[derive(PartialEq, Eq, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive( Debug))]
pub struct BondToken<Moment, AccountId, Hash> {
	id: Hash,
	account_id: AccountId,
	symbol: Symbol,
	balance: Balance,
	capital_amount: Balance,
	rewards_amount: Balance,
	lock_amount: Balance,
	issue_time: Moment,
	stake_hash: Hash,
	status: BondTokenStatus,
}

decl_storage! {
	trait Store for Module<T: Trait> as BondToken {
		pub FreeBondToken get(free_bond_token): map (T::AccountId, T::Hash) => BondToken<T::Moment, T::AccountId, T::Hash>;
		pub BondTokenHashList get(bond_token_hash_list): map (T::AccountId, Symbol) => Vec<T::Hash>;
		pub RedeemRecords get(redeem_records): map (T::AccountId, T::Hash) => Option<CustomRedeemData<T::AccountId, T::Hash, Balance>>;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		// Custom redeem xtz
		pub fn custom_redeem(origin, bond_id: T::Hash, amount: Balance, original_account_id: Vec<u8>) -> Result {
			let sender = ensure_signed(origin)?;

			Self::lock_bond_token(sender.clone(), bond_id.clone(), amount).expect("Error locking bond token");

			<RedeemRecords<T>>::insert((sender.clone(), bond_id.clone()), CustomRedeemData {
				initiator: sender.clone(),
				bond_id: bond_id,
				amount: amount,
				original_account_id: original_account_id,
			});

			Self::deposit_event(RawEvent::RedeenBondToken(sender, bond_id));	

			Ok(())	
		}
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId, Hash = <T as system::Trait>::Hash {
		RedeenBondToken(AccountId, Hash),
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

	pub fn add_bond_token(sender: T::AccountId, symbol: Symbol, amount: Balance, stake_hash: T::Hash) -> Result {
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
			lock_amount: 0,
			account_id: sender.clone(),
			issue_time: now,
			stake_hash: stake_hash,
			status: BondTokenStatus::Normal
		};

		Self::add_bond_hash_list(sender.clone(), hash.clone(), symbol.clone());

		<FreeBondToken<T>>::insert(key, bond_token);
		Ok(())
    }

	pub fn transfer_bond_token(sender: T::AccountId, bond_id: T::Hash, to: T::AccountId, amount: Balance) -> Result {
		ensure!(amount > 0, "The amount to transfer must be greater than 0");

		let key = (sender.clone(), bond_id.clone());
		ensure!(<FreeBondToken<T>>::exists(key.clone()), "This bond token does not exist");

		let mut bond_token: BondToken<T::Moment, T::AccountId, T::Hash> = <FreeBondToken<T>>::get(key.clone());
		ensure!(bond_token.balance >= amount, "The balance of bond token is not enough");

		bond_token.balance -= amount;
		<FreeBondToken<T>>::insert(key.clone(), bond_token.clone());

		return Self::add_bond_token(to.clone(), bond_token.symbol, amount, bond_token.stake_hash);
    }

	pub fn lock_bond_token(sender: T::AccountId, bond_id: T::Hash, amount: Balance) -> Result {
		ensure!(amount > 0, "The amount to lock must be greater than 0");

		let key = (sender.clone(), bond_id.clone());
		ensure!(<FreeBondToken<T>>::exists(key.clone()), "This bond token does not exist");

		let mut bond_token: BondToken<T::Moment, T::AccountId, T::Hash> = <FreeBondToken<T>>::get(key.clone());
		ensure!(bond_token.status != BondTokenStatus::Locked, "The status of bond token has been locked");
		ensure!(bond_token.balance >= amount, "The balance of bond token is not enough");
		bond_token.balance -= amount;
		bond_token.status = BondTokenStatus::Locked;

		<FreeBondToken<T>>::insert(key, bond_token);
		Ok(())
    }

	pub fn redeem_bond_token(sender: T::AccountId, bond_id: T::Hash, amount: Balance) -> Result {
		ensure!(amount > 0, "The amount to redeem must be greater than 0");

		let key = (sender.clone(), bond_id.clone());
		ensure!(<FreeBondToken<T>>::exists(key.clone()), "This bond token does not exist");

		let mut bond_token: BondToken<T::Moment, T::AccountId, T::Hash> = <FreeBondToken<T>>::get(key.clone());
		ensure!(bond_token.balance >= amount, "The balance of bond token is not enough");
		ensure!(bond_token.status == BondTokenStatus::Locked, "The status of bond token must be locked");

		bond_token.lock_amount = 0;
		bond_token.status = BondTokenStatus::Normal;

		<FreeBondToken<T>>::insert(key, bond_token);
		Ok(())
    }

	pub fn distribute_bond_rewards(sender: T::AccountId, bond_id: T::Hash, rewards_amount: Balance) -> Result {
		ensure!(rewards_amount > 0, "The number to transfer must be greater than 0");

		let key = (sender.clone(), bond_id.clone());
		ensure!(<FreeBondToken<T>>::exists(key.clone()), "This bond token does not exist");

		let mut bond_token: BondToken<T::Moment, T::AccountId, T::Hash> = <FreeBondToken<T>>::get(key.clone());
		ensure!(bond_token.status != BondTokenStatus::Locked, "The status of bond token has been locked");

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
