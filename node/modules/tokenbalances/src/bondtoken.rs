#![allow(unused_imports)]

extern crate sr_primitives as runtime_primitives;
extern crate randomness_collective_flip as random;

use support::{decl_module, decl_storage, decl_event, ensure, dispatch::Result, dispatch::Vec, traits::Randomness};
use system::ensure_signed;

use sr_std::{
	convert::{TryInto},
};

use num_traits::float::FloatCore;

#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};

use parity_codec::{Encode, Decode};
use runtime_primitives::traits::Hash;
use node_primitives::{BlockNumber, Balance, BondTokenLockType, BondTokenLockStatus, Symbol, CustomRedeemData}; 


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
	last_reward_block_num: BlockNumber,
	issue_time: Moment,
	stake_id: Hash,
	stake_address: Vec<u8>,
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

#[derive(PartialEq, Eq, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub struct BondReward {
	block_num: BlockNumber,
	total_reward: Balance,
	total_balance: Balance
}

decl_storage! {
	trait Store for Module<T: Trait> as BondToken {
		pub BondTokens get(bond_token): map T::Hash => BondToken<T::Moment, T::AccountId, T::Hash>;

		pub TotalBondTokenBalance get(total_bond_token_balance): map Symbol => Balance;

		pub SymbolBondTokensArray get(symbol_bond_by_index): map (Symbol, u64) => T::Hash;
        pub SymbolBondTokensCount get(symbol_bond_count): map Symbol => u64;

		pub OwnedBondTokensArray get(bond_of_owner_by_index): map (T::AccountId, u64) => T::Hash;
        pub OwnedBondTokensCount get(owned_bond_count): map T::AccountId => u64;

		pub LockBondToken get(get_lock_bond_token): map T::Hash => LockBondData<T::Moment, T::Hash>;
		pub RedeemRecords get(redeem_records): map (T::AccountId, T::Hash) => Option<CustomRedeemData<T::AccountId, T::Hash>>;

		pub BondRewardsArray get(bond_reward_by_index): map u64 => BondReward;
        pub BondRewardsCount get(bond_rewards_count): u64;
		pub BondRewardsIndex: map BlockNumber => u64;

		CreateNonce: u64;
		LockNonce: u64;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		fn on_finalize(n: T::BlockNumber) {
			if let Some(value) = n.try_into().ok() {
				if value % 30 == 0 {
					let total_rewards: Balance = 14321;
					if total_rewards > 0 {
						let symbol = Symbol::XTZ;
						let total_balance = Self::total_bond_token_balance(symbol);

						let count = Self::symbol_bond_count(symbol);
						for i in 0..count {
							let bond_id = <SymbolBondTokensArray<T>>::get((symbol, i));
							let bond_token = <BondTokens<T>>::get(bond_id.clone());
							if bond_token.balance == 0 {
								continue;
							}
							let rewards_amount = ((total_rewards as f64) * (bond_token.balance as f64 / total_balance as f64)).round() as Balance;
							Self::distribute_bond_rewards(bond_id.clone(), rewards_amount).ok();
						}
					}
				}
			}
		}

		pub fn transfer_bond_token(origin, bond_id: T::Hash, to: T::AccountId, amount: Balance) -> Result {
			let sender = ensure_signed(origin)?;
			
			ensure!(amount > 0, "The amount to transfer must be greater than 0");

			ensure!(<BondTokens<T>>::exists(&bond_id), "This bond token does not exist");

			let mut bond_token = <BondTokens<T>>::get(&bond_id);
			ensure!(bond_token.account_id == sender, "You do not own this bond token");
			ensure!(bond_token.balance >= amount, "The balance of bond token is not enough");

			Self::mint(to.clone(), bond_token.clone().symbol, amount, bond_token.clone().stake_id, bond_token.clone().stake_address)?;

			bond_token.balance -= amount;
			<BondTokens<T>>::insert(bond_id, bond_token.clone());

			Ok(())
		}

		// Custom redeem
		pub fn custom_redeem_bond(origin, bond_id: T::Hash, amount: Balance, original_account_id: Vec<u8>) -> Result {
			let sender = ensure_signed(origin)?;

			ensure!(amount > 0, "The amount to lock must be greater than 0");

			ensure!(<BondTokens<T>>::exists(&bond_id), "This bond token does not exist");

			let bond_token = <BondTokens<T>>::get(&bond_id);
			ensure!(bond_token.account_id == sender, "You do not own this bond token");
			ensure!(bond_token.balance >= amount, "The balance of bond token is not enough");

			let total_balance = Self::total_bond_token_balance(bond_token.symbol);
			let new_total_balance = total_balance.checked_sub(amount).ok_or("Underflow substracting total balance")?;
			
			let lock_id = Self::lock_bond_token(sender.clone(), bond_id, amount, BondTokenLockType::Redemption);

			<RedeemRecords<T>>::insert((sender.clone(), lock_id.clone()), CustomRedeemData {
				initiator: sender.clone(),
				lock_id: lock_id.clone(),
				original_account_id: original_account_id,
			});

			<TotalBondTokenBalance>::insert(bond_token.symbol, new_total_balance);

			Self::deposit_event(RawEvent::RedeemBondToken(sender, lock_id));

			Ok(())	
		}

		// Custom redeem
		pub fn custom_redeem_batch(origin, stake_address: Vec<u8>, amount: Balance, original_account_id: Vec<u8>) -> Result {
			let sender = ensure_signed(origin)?;

			ensure!(amount > 0, "The amount to lock must be greater than 0");

			// ensure!(<BondTokens<T>>::exists(bond_id.clone()), "This bond token does not exist");

			// let bond_token = <BondTokens<T>>::get(bond_id.clone());
			// ensure!(bond_token.balance >= amount, "The balance of bond token is not enough");

			// let lock_id = Self::lock_bond_token(sender.clone(), bond_id.clone(), amount, BondTokenLockType::Redemption);

			// <RedeemRecords<T>>::insert((sender.clone(), lock_id.clone()), CustomRedeemData {
			// 	initiator: sender.clone(),
			// 	lock_id: lock_id.clone(),
			// 	original_account_id: original_account_id,
			// });

			// Self::deposit_event(RawEvent::RedeemBondToken(sender, lock_id));

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

	pub fn create_bond_token(sender: T::AccountId, symbol: Symbol, amount: Balance, stake_id: T::Hash, stake_address: Vec<u8>) -> Result {
		ensure!(amount > 0, "The amount must be greater than 0");

		let total_balance = Self::total_bond_token_balance(symbol);
		let new_total_balance = total_balance.checked_add(amount).ok_or("Overflow adding total balance")?;
		
		Self::mint(sender, symbol, amount, stake_id, stake_address)?;

		<TotalBondTokenBalance>::insert(symbol, new_total_balance);

		Ok(())
    }

	fn mint(sender: T::AccountId, symbol: Symbol, amount: Balance, stake_id: T::Hash, stake_address: Vec<u8>) -> Result {
		let owned_bond_count = Self::owned_bond_count(&sender);
        let new_owned_bond_count = owned_bond_count.checked_add(1).ok_or("Overflow adding a new owned bond token")?;

		let symbol_bond_count = Self::symbol_bond_count(symbol);
        let new_symbol_bond_count = symbol_bond_count.checked_add(1).ok_or("Overflow adding a new symbol bond token")?;

		let nonce = <CreateNonce>::get();
		let random_seed = <random::Module<T>>::random_seed();
		let bond_id = (random_seed, &sender, nonce).using_encoded(<T as system::Trait>::Hashing::hash);

		ensure!(!<BondTokens<T>>::exists(&bond_id), "Bond token already exists");

		let now = <timestamp::Module<T>>::get();
		let block_num: BlockNumber = <system::Module<T>>::block_number().try_into().ok().unwrap() as BlockNumber;
		let bond_token = BondToken {
			id: bond_id,
			account_id: sender.clone(),
			symbol: symbol,
			balance: amount,
			capital_amount: amount,
			rewards_amount: 0,
			last_reward_block_num: block_num,
			issue_time: now,
			stake_id: stake_id,
			stake_address: stake_address,
			lock_ids: Vec::new(),
		};
		<BondTokens<T>>::insert(bond_id, bond_token);

		<OwnedBondTokensArray<T>>::insert((sender.clone(), owned_bond_count), &bond_id);
        <OwnedBondTokensCount<T>>::insert(&sender, new_owned_bond_count);

		<SymbolBondTokensArray<T>>::insert((symbol, symbol_bond_count), &bond_id);
        <SymbolBondTokensCount>::insert(&symbol, new_symbol_bond_count);

		<CreateNonce>::mutate(|n| *n += 1);

		Ok(())
    }

	fn lock_bond_token(sender: T::AccountId, bond_id: T::Hash, amount: Balance, lock_type: BondTokenLockType) -> T::Hash {
		let nonce = <LockNonce>::get();
		let random_seed = <random::Module<T>>::random_seed();
		let lock_id = (random_seed, &sender, nonce).using_encoded(<T as system::Trait>::Hashing::hash);
		let now = <timestamp::Module<T>>::get();
		let lock_bond_token = LockBondData {
			id: lock_id,
			bond_id: bond_id,
			lock_type: lock_type,
			lock_amount: amount,
			lock_time: now,
			status: BondTokenLockStatus::Locked
		};
		<LockBondToken<T>>::insert(lock_id, lock_bond_token);

		let mut bond_token = <BondTokens<T>>::get(bond_id);	
		bond_token.lock_ids.push(lock_id);
		bond_token.balance -= amount;

		<BondTokens<T>>::insert(bond_id, bond_token);

		<LockNonce>::mutate(|n| *n += 1);
		
		lock_id
    }

	pub fn complete_redeem_bond_token(lock_id: T::Hash) -> Result {
		ensure!(<LockBondToken<T>>::exists(&lock_id), "This lock bond token does not exist");
		let mut lock_bond_data = <LockBondToken<T>>::get(&lock_id);

		let key = lock_bond_data.bond_id;
		ensure!(<BondTokens<T>>::exists(key), "This bond token does not exist");

		let mut bond_token = <BondTokens<T>>::get(key);

		let mut lock_id_list: Vec<T::Hash> = Vec::new();
		for t_lock_id in bond_token.lock_ids {
			if t_lock_id != lock_id {
				lock_id_list.push(t_lock_id);
			}
		}
		bond_token.lock_ids = lock_id_list;

		<BondTokens<T>>::insert(key, bond_token);

		lock_bond_data.status = BondTokenLockStatus::Completed;
		<LockBondToken<T>>::insert(lock_id, lock_bond_data);

		Ok(())
    }

	pub fn distribute_bond_rewards(bond_id: T::Hash, rewards_amount: Balance) -> Result {
		ensure!(rewards_amount > 0, "The number to transfer must be greater than 0");

		let key = bond_id;
		ensure!(<BondTokens<T>>::exists(key), "This bond token does not exist");

		let mut bond_token = <BondTokens<T>>::get(key);

		let balance_value = bond_token.balance.checked_add(rewards_amount).ok_or("Overflow adding balance of bond")?;

		let rewards_value = bond_token.rewards_amount.checked_add(rewards_amount).ok_or("Overflow adding rewards of bond")?;

		let total_balance = Self::total_bond_token_balance(bond_token.symbol);
		let new_total_balance = total_balance.checked_add(rewards_amount).ok_or("Overflow adding total balance")?;
		
		bond_token.balance = balance_value;
		bond_token.rewards_amount = rewards_value;
		<BondTokens<T>>::insert(key, bond_token.clone());

		<TotalBondTokenBalance>::insert(bond_token.symbol, new_total_balance);

		Ok(())
    }
}
