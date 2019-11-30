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

pub const BOND_REWARD_BLOCK_DURATION: u32 = 50;
pub const BOND_REWARD_MIN_BALANCE: u8 = 1;

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub struct BondToken<AccountId, Hash> {
	id: Hash,
	account_id: AccountId,
	symbol: Symbol,
	balance: Balance,
	capital_amount: Balance,
	rewards_amount: Balance,
	last_reward_block_num: BlockNumber,
	issue_block: BlockNumber,
	stake_id: Hash,
	stake_address: Vec<u8>,
	lock_ids: Vec<Hash>,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub struct LockBondData<Hash> {
	id: Hash,
	bond_id: Hash,
	lock_type: BondTokenLockType,
	lock_amount: Balance,
	lock_block: BlockNumber,
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
		pub BondTokens get(bond_token): map T::Hash => BondToken<T::AccountId, T::Hash>;

		pub TotalBondTokenBalance get(total_bond_token_balance): map Symbol => Balance;

		pub SymbolBondTokensArray get(symbol_bond_by_index): map (Symbol, u64) => T::Hash;
        pub SymbolBondTokensCount get(symbol_bond_count): map Symbol => u64;

		pub OwnedBondTokensArray get(bond_of_owner_by_index): map (T::AccountId, Symbol, u64) => T::Hash;
        pub OwnedBondTokensCount get(owned_bond_count): map (T::AccountId, Symbol) => u64;
		pub OwnedTotalBondBalance get(owned_total_bond_balance): map (T::AccountId, Symbol) => Balance;
		pub OwnedNextValidBondIndex get(owned_next_valid_bond_index): map (T::AccountId, Symbol) => u64;

		pub LockBondToken get(get_lock_bond_token): map T::Hash => LockBondData<T::Hash>;
		pub RedeemRecords get(redeem_records): map (T::AccountId, T::Hash) => Option<CustomRedeemData<T::AccountId, T::Hash>>;

		pub BondRewardsArray get(bond_reward_by_index): map (Symbol, u64) => BondReward;
        pub BondRewardsCount get(bond_rewards_count): map Symbol => u64;
		pub BondRewardsIndex get(bond_rewards_index): map (Symbol, BlockNumber) => u64;

		CreateNonce: u64;
		LockNonce: u64;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		fn on_finalize(block_num: T::BlockNumber) {
			let block_num_value: BlockNumber = block_num.try_into().ok().unwrap() as BlockNumber;

			if block_num_value % BOND_REWARD_BLOCK_DURATION == 0 {
				let symbol = Symbol::XTZ;
				let total_rewards: Balance = 10000000;
				let total_balance = Self::total_bond_token_balance(symbol);
				if total_rewards > 0 && total_balance > 0 {
					let bond_reward = BondReward {
						block_num: block_num_value,
						total_reward: total_rewards,
						total_balance: total_balance,
					};

					let bond_rewards_count = Self::bond_rewards_count(symbol);
					let new_bond_rewards_count = bond_rewards_count + 1;

					BondRewardsArray::insert((symbol, bond_rewards_count), bond_reward);
					BondRewardsCount::insert(symbol, new_bond_rewards_count);
					BondRewardsIndex::insert((symbol, block_num_value), bond_rewards_count);
				}
			}

		}

		pub fn transfer_specific_bond_token(origin, bond_id: T::Hash, to: T::AccountId, amount: Balance) -> Result {
			let sender = ensure_signed(origin)?;
			
			ensure!(amount > 0, "The amount to transfer must be greater than 0");

			ensure!(<BondTokens<T>>::exists(&bond_id), "This bond token does not exist");

			let bond_token = <BondTokens<T>>::get(&bond_id);
			ensure!(bond_token.account_id == sender, "You do not own this bond token");
			ensure!(bond_token.balance >= amount, "The balance of bond token is not enough");

			let symbol = bond_token.clone().symbol;

			let owned_bond_count = Self::owned_bond_count((to.clone(), symbol));
			owned_bond_count.checked_add(1).ok_or("Overflow adding a new owned bond token")?;

			let symbol_bond_count = Self::symbol_bond_count(symbol);
			symbol_bond_count.checked_add(1).ok_or("Overflow adding a new symbol bond token")?;

			// Claim sender's bond reward first.
			Self::claim_specific_bond_reward(bond_id.clone());

			Self::mint(to.clone(), symbol, amount, bond_token.clone().stake_id, bond_token.clone().stake_address);

			let mut bond_token_last = <BondTokens<T>>::get(&bond_id);
			bond_token_last.balance -= amount;
			<BondTokens<T>>::insert(bond_id, bond_token_last.clone());

			let owned_total_balance_key = (sender, symbol);
			let owned_total_balance = Self::owned_total_bond_balance(owned_total_balance_key.clone());
			let new_owned_total_balance = owned_total_balance - amount;
			<OwnedTotalBondBalance<T>>::insert(owned_total_balance_key, new_owned_total_balance);

			Ok(())
		}

		pub fn transfer_bond_token(origin, symbol: Symbol, _to: T::AccountId, amount: Balance) -> Result {
			let sender = ensure_signed(origin)?;
			
			ensure!(amount > 0, "The amount to transfer must be greater than 0");

			let owned_bond_count = Self::owned_bond_count((sender.clone(), symbol));
			ensure!(owned_bond_count > 0, "You do not own any bond token");

			let owned_total_balance = Self::owned_total_bond_balance((sender.clone(), symbol));
			ensure!(owned_total_balance >= amount, "You bond balance is not enough");

			// Claim sender's bond reward first.
			// Self::claim_specific_bond_reward(bond_id.clone());

			// Self::mint(to.clone(), symbol, amount, bond_token.clone().stake_id, bond_token.clone().stake_address);

			// let mut bond_token_last = <BondTokens<T>>::get(&bond_id);
			// bond_token_last.balance -= amount;
			// <BondTokens<T>>::insert(bond_id, bond_token_last.clone());

			Ok(())
		}

		pub fn claim_bond_reward(origin, symbol: Symbol) -> Result {
			let sender = ensure_signed(origin)?;
			
			let owned_bond_count = Self::owned_bond_count((sender.clone(), symbol));
			ensure!(owned_bond_count > 0, "You do not own any bond token");

			let bond_rewards_count = Self::bond_rewards_count(symbol);
			ensure!(bond_rewards_count > 0, "There is no bond reward");

			Self::claim_all_user_bond_reward(sender.clone(), symbol);

			Self::deposit_event(RawEvent::ClaimBondReward(sender, symbol));

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
		pub fn custom_redeem_batch(origin, _stake_address: Vec<u8>, amount: Balance, _original_account_id: Vec<u8>) -> Result {
			let _sender = ensure_signed(origin)?;

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
		ClaimBondReward(AccountId, Symbol),
	}
);

impl<T: Trait> Module<T> {

	pub fn create_bond_token(sender: T::AccountId, symbol: Symbol, amount: Balance, stake_id: T::Hash, stake_address: Vec<u8>) -> Result {
		ensure!(amount > 0, "The amount must be greater than 0");

		let total_balance = Self::total_bond_token_balance(symbol);
		let new_total_balance = total_balance.checked_add(amount).ok_or("Overflow adding total balance")?;

		let owned_bond_count = Self::owned_bond_count((sender.clone(), symbol));
        owned_bond_count.checked_add(1).ok_or("Overflow adding a new owned bond token")?;

		let symbol_bond_count = Self::symbol_bond_count(symbol);
        symbol_bond_count.checked_add(1).ok_or("Overflow adding a new symbol bond token")?;
		
		Self::mint(sender, symbol, amount, stake_id, stake_address);

		<TotalBondTokenBalance>::insert(symbol, new_total_balance);

		Ok(())
    }

	fn mint(sender: T::AccountId, symbol: Symbol, amount: Balance, stake_id: T::Hash, stake_address: Vec<u8>) {
		let owned_bond_count = Self::owned_bond_count((sender.clone(), symbol));
        let new_owned_bond_count = owned_bond_count + 1;

		let symbol_bond_count = Self::symbol_bond_count(symbol);
        let new_symbol_bond_count = symbol_bond_count + 1;

		let nonce = <CreateNonce>::get();
		let random_seed = <random::Module<T>>::random_seed();
		let bond_id = (random_seed, &sender, nonce).using_encoded(<T as system::Trait>::Hashing::hash);

		let block_num = <system::Module<T>>::block_number().try_into().ok().unwrap() as BlockNumber;
		let bond_token = BondToken {
			id: bond_id,
			account_id: sender.clone(),
			symbol: symbol,
			balance: amount,
			capital_amount: amount,
			rewards_amount: 0,
			last_reward_block_num: 0,
			issue_block: block_num,
			stake_id: stake_id,
			stake_address: stake_address,
			lock_ids: Vec::new(),
		};
		<BondTokens<T>>::insert(bond_id, bond_token);

		<OwnedBondTokensArray<T>>::insert((sender.clone(), symbol, owned_bond_count), &bond_id);
        <OwnedBondTokensCount<T>>::insert((sender.clone(), symbol), new_owned_bond_count);

		<SymbolBondTokensArray<T>>::insert((symbol, symbol_bond_count), &bond_id);
        <SymbolBondTokensCount>::insert(&symbol, new_symbol_bond_count);

		let owned_total_balance = Self::owned_total_bond_balance((sender.clone(), symbol));
		let new_owned_total_balance = owned_total_balance + amount;
		<OwnedTotalBondBalance<T>>::insert((sender.clone(), symbol), new_owned_total_balance);

		<CreateNonce>::mutate(|n| *n += 1);
    }

	fn claim_all_user_bond_reward(sender: T::AccountId, symbol: Symbol) {
		let owned_bond_count = Self::owned_bond_count((sender.clone(), symbol));
		if owned_bond_count <= 0 {
			return;
		}
		let bond_rewards_count = Self::bond_rewards_count(symbol);
		if bond_rewards_count <= 0 {
			return;
		}
		let begin = Self::owned_next_valid_bond_index((sender.clone(), symbol));
		if begin >= owned_bond_count {
			return;
		}
		//TODO: owned_bond_count is probably going to be very large
		for i in begin..owned_bond_count {
			let bond_id = Self::bond_of_owner_by_index((sender.clone(), symbol, i));
			Self::claim_specific_bond_reward(bond_id);
		}
	}

	fn claim_specific_bond_reward(bond_id: T::Hash) {
		let mut bond_token = <BondTokens<T>>::get(bond_id.clone());

		//TODO: BOND_REWARD_MIN_BALANCE shoule multiply precision
		if bond_token.balance < BOND_REWARD_MIN_BALANCE.into() {
			return;
		}

		let symbol = bond_token.symbol;
		let bond_rewards_count = Self::bond_rewards_count(symbol);
		if bond_rewards_count <= 0 {
			return;
		}

		let begin;
		if bond_token.last_reward_block_num > 0 {
			begin = Self::bond_rewards_index((symbol, bond_token.last_reward_block_num)) + 1;
		} else {
			let times = (bond_token.issue_block as f64 / BOND_REWARD_BLOCK_DURATION as f64).round() as BlockNumber;
			begin = Self::bond_rewards_index((symbol, times * BOND_REWARD_BLOCK_DURATION));
		}

		if begin >= bond_rewards_count {
			return;
		}
		
		let mut total_rewards_amount = 0;
		let mut last_reward_block_num = 0;
		for j in begin..bond_rewards_count {
			let bond_reward = Self::bond_reward_by_index((symbol, j));
			let rewards_amount = ((bond_reward.total_reward as f64) * (bond_token.balance as f64 / bond_reward.total_balance as f64)).floor() as Balance;
			total_rewards_amount += rewards_amount;
			last_reward_block_num = bond_reward.block_num;
		}

		if total_rewards_amount > 0 {
			Self::distribute_bond_rewards(bond_id.clone(), total_rewards_amount, last_reward_block_num).ok();
		} else {
			bond_token.last_reward_block_num = last_reward_block_num;
			<BondTokens<T>>::insert(bond_id.clone(), bond_token.clone());
		}
	}

	fn lock_bond_token(sender: T::AccountId, bond_id: T::Hash, amount: Balance, lock_type: BondTokenLockType) -> T::Hash {
		let nonce = <LockNonce>::get();
		let random_seed = <random::Module<T>>::random_seed();
		let lock_id = (random_seed, &sender, nonce).using_encoded(<T as system::Trait>::Hashing::hash);
		let block_num = <system::Module<T>>::block_number().try_into().ok().unwrap() as BlockNumber;
		let lock_bond_token = LockBondData {
			id: lock_id,
			bond_id: bond_id,
			lock_type: lock_type,
			lock_amount: amount,
			lock_block: block_num,
			status: BondTokenLockStatus::Locked
		};
		<LockBondToken<T>>::insert(lock_id, lock_bond_token);

		let mut bond_token = <BondTokens<T>>::get(bond_id);	
		bond_token.lock_ids.push(lock_id);
		bond_token.balance -= amount;

		<BondTokens<T>>::insert(bond_id, bond_token.clone());

		let owned_total_balance = Self::owned_total_bond_balance((sender.clone(), bond_token.symbol));
		let new_owned_total_balance = owned_total_balance - amount;
		<OwnedTotalBondBalance<T>>::insert((sender.clone(), bond_token.symbol), new_owned_total_balance);

		if bond_token.balance == 0 {
			Self::update_next_valid_bond_index(sender.clone(), bond_token.symbol, bond_id);
		}

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

	pub fn distribute_bond_rewards(bond_id: T::Hash, rewards_amount: Balance, last_reward_block_num: BlockNumber) -> Result {
		let key = bond_id;

		let mut bond_token = <BondTokens<T>>::get(key);

		let balance_value = bond_token.balance.checked_add(rewards_amount).ok_or("Overflow adding balance of bond")?;

		let rewards_value = bond_token.rewards_amount.checked_add(rewards_amount).ok_or("Overflow adding rewards of bond")?;

		let total_balance = Self::total_bond_token_balance(bond_token.symbol);
		let new_total_balance = total_balance.checked_add(rewards_amount).ok_or("Overflow adding total balance")?;
		
		bond_token.balance = balance_value;
		bond_token.rewards_amount = rewards_value;
		bond_token.last_reward_block_num = last_reward_block_num;
		<BondTokens<T>>::insert(key, bond_token.clone());

		<TotalBondTokenBalance>::insert(bond_token.symbol, new_total_balance);

		let owned_total_balance_key = (bond_token.account_id, bond_token.symbol);
		let owned_total_balance = Self::owned_total_bond_balance(owned_total_balance_key.clone());
		let new_owned_total_balance = owned_total_balance + rewards_amount;
		<OwnedTotalBondBalance<T>>::insert(owned_total_balance_key, new_owned_total_balance);

		Ok(())
    }

	fn update_next_valid_bond_index(sender: T::AccountId, symbol: Symbol, match_bond_id: T::Hash) {
		let next_valid_index = Self::owned_next_valid_bond_index((sender.clone(), symbol));
		let owned_bond_count = Self::owned_bond_count((sender.clone(), symbol));
		if next_valid_index <= (owned_bond_count - 1) {
			let next_valid_bond_id = Self::bond_of_owner_by_index((sender.clone(), symbol, next_valid_index));
			if next_valid_bond_id == match_bond_id {
				<OwnedNextValidBondIndex<T>>::insert((sender.clone(), symbol), next_valid_index + 1);
			}
		}
	}
}
