extern crate srml_system as system;
extern crate srml_balances as balances;

use srml_support::{decl_module, decl_storage, decl_event, ensure, dispatch::Result};
use system::ensure_signed;
use sr_std::prelude::*;
use sr_std::{
	convert::{TryInto},
};
use sr_primitives::traits::Hash;
use sr_primitives::traits::CheckedAdd;
use parity_codec::{Encode, Decode};
use stafi_primitives::StakeTokenType;
use stafi_primitives::Balance;
use log::info;
use token_balances::Symbol;


#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq)]
pub enum AtomStakeStage {
	// Init
	Init,
	// Transfer token to multi sig address
	Transfering,
	// Successful transfer
	TransferSuccess,
	// Active staking stage
	Staking,
	// Completed staking stage
	Completed,
}

// TODO 
// put this struct into specific module
// #[cfg_attr(feature = "std", derive(Debug))]
// #[derive(Encode, Decode, PartialEq)]
// pub struct MultiSigAddress<Hash> {
// 	// public key
// 	pub public_key: Hash,
// 	// multi sig address
// 	pub address: Hash,
// }

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, PartialEq)]
pub struct AtomStakeTokenData {
	pub token_type: StakeTokenType,
	// decimals of token
	pub token_decimals: u32,
	// validator
	pub validator: Vec<u8>,
	// Amount of stake
	pub stake_amount: u128,
	// Reward of stake
	pub reward_amount: u128,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, PartialEq)]
pub struct AtomStakeData<AccountId, Hash> {
	// identifier id
	pub id: Hash,
	// creator of stake
	pub initiator: AccountId,
	// multi sig address
	pub multi_sig_address: Vec<u8>,
	// Stage of stake
	pub stage: AtomStakeStage,
	// Token data of stake
	pub stake_token_data: AtomStakeTokenData,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, PartialEq)]
pub struct AtomTransferData<AccountId, Hash> {
	// identifier id
	pub id: Hash,
	// creator of stake
	pub initiator: AccountId,
	// transaction message
	pub transfer_msg: Vec<u8>,
	// signatures of transaction
	pub signatures: Vec<u8>,
}

pub trait Trait: system::Trait + session::Trait + im_online::Trait + token_balances::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as AtomStaking {
		// Just a dummy storage item.
		pub StakeRecords get(stake_records): map (T::AccountId, T::Hash) => Option<AtomStakeData<T::AccountId, T::Hash>>;
		pub StakeDataHashRecords get(stake_data_hash_records): map T::AccountId => Vec<T::Hash>;
		pub TransferInitDataRecords get(transfer_init_data_records): linked_map (T::AccountId, T::Hash) => Option<AtomTransferData<T::AccountId, T::Hash>>;
		pub TransferingDataRecords get(transfering_data_records): linked_map (T::AccountId, T::Hash) => T::AccountId;
		pub TransferSuccessDataRecords get(transfer_success_data_records): linked_map (T::AccountId, T::Hash) => T::AccountId;
		pub StakingDataRecords get(staking_data_records): linked_map (T::AccountId, T::Hash) => T::AccountId;
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;

		// 
		pub fn custom_stake(origin, multi_sig_address: Vec<u8>, stake_amount: u128, validator: Vec<u8>, transfer_msg: Vec<u8>, signatures: Vec<u8>) -> Result {
			let sender = ensure_signed(origin)?;

			ensure!(stake_amount > 0, "Stake amount must be greater than 0");
			// // TODO: Check multi sig address
			// ensure!(stake_amount > 0, "Multi sig address is illegal");

			let random_seed = <system::Module<T>>::random_seed();
            let hash = (random_seed, &sender).using_encoded(<T as system::Trait>::Hashing::hash);

			let stake_token_data = AtomStakeTokenData {
				token_type: StakeTokenType::ATOM,
				token_decimals: 18,
				validator: validator,
				stake_amount: stake_amount,
				reward_amount: 0,
			};

			<StakeRecords<T>>::insert((sender.clone(), hash.clone()), AtomStakeData {
				id: hash.clone(),
				initiator: sender.clone(),
				multi_sig_address: multi_sig_address,
				stage: AtomStakeStage::Init,
				stake_token_data: stake_token_data,
			});

			let mut hashs = <StakeDataHashRecords<T>>::get(sender.clone());
			hashs.push(hash.clone());
			<StakeDataHashRecords<T>>::insert(sender.clone(), hashs);

			let transfer_data = AtomTransferData {
				id: hash.clone(),
				initiator: sender.clone(),
				transfer_msg: transfer_msg,
				signatures: signatures,
			};
			<TransferInitDataRecords<T>>::insert((sender.clone(), hash.clone()), transfer_data);

			// here we are raising the event
			Self::deposit_event(RawEvent::StakeInit(sender, hash));
			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId, Hash = <T as system::Trait>::Hash {
		StakeInit(AccountId, Hash),
	}
);


impl<T: Trait> session::OneSessionHandler<T::AccountId> for Module<T> {
	type Key = T::AuthorityId;

	fn on_genesis_session<'a, I: 'a>(_validators: I)
	where
		I: Iterator<Item = (&'a T::AccountId, Self::Key)>,
	{
		// ignore
	}

	fn on_new_session<'a, I: 'a>(_changed: bool, _validators: I, _next_validators: I)
	where
		I: Iterator<Item = (&'a T::AccountId, Self::Key)>,
	{
		// ignore
	}

	fn on_before_session_ending() {
		info!("aaa {}.", 1);

		Self::handle_staking();
		Self::handle_transfer_success();
		Self::handle_transfering();
		Self::handle_init();
	}

	fn on_disabled(_i: usize) {
		// ignore
	}
}

impl<T: Trait> Module<T> {
    // 
    fn handle_init() {
        for (key, _transfer_data) in <TransferInitDataRecords<T>>::enumerate() {
			let account_id = &key.0;
			let hash = &key.1;

			if let Some(mut atom_stake_data) = Self::stake_records((account_id.clone(), hash.clone())) {
				if atom_stake_data.stage == AtomStakeStage::Init {
					atom_stake_data.stage = AtomStakeStage::Transfering;
					<StakeRecords<T>>::insert((account_id.clone(), hash.clone()), atom_stake_data);

					<TransferInitDataRecords<T>>::remove((account_id.clone(), hash.clone()));
					<TransferingDataRecords<T>>::insert((account_id.clone(), hash.clone()), account_id.clone());
				}
			}
		}
    }

	// 
    fn handle_transfering() {
        for (key, _) in <TransferingDataRecords<T>>::enumerate() {
			let account_id = &key.0;
			let hash = &key.1;

			if let Some(mut atom_stake_data) = Self::stake_records((account_id.clone(), hash.clone())) {
				if atom_stake_data.stage == AtomStakeStage::Transfering {
					atom_stake_data.stage = AtomStakeStage::TransferSuccess;
					<StakeRecords<T>>::insert((account_id.clone(), hash.clone()), atom_stake_data);

					<TransferingDataRecords<T>>::remove((account_id.clone(), hash.clone()));
					<TransferSuccessDataRecords<T>>::insert((account_id.clone(), hash.clone()), account_id.clone());
				}
			}
		}
    }

	// 
    fn handle_transfer_success() {
        for (key, _) in <TransferSuccessDataRecords<T>>::enumerate() {
			let account_id = &key.0;
			let hash = &key.1;

			if let Some(mut atom_stake_data) = Self::stake_records((account_id.clone(), hash.clone())) {
				if atom_stake_data.stage == AtomStakeStage::TransferSuccess {
					atom_stake_data.stage = AtomStakeStage::Staking;
					<StakeRecords<T>>::insert((account_id.clone(), hash.clone()), atom_stake_data);

					<TransferSuccessDataRecords<T>>::remove((account_id.clone(), hash.clone()));
					<StakingDataRecords<T>>::insert((account_id.clone(), hash.clone()), account_id.clone());
				}
			}
		}
    }

	// 
    fn handle_staking() {
        for (key, _) in <StakingDataRecords<T>>::enumerate() {
			let account_id = &key.0;
			let hash = &key.1;

			if let Some(mut atom_stake_data) = Self::stake_records((account_id.clone(), hash.clone())) {
				if atom_stake_data.stage == AtomStakeStage::Staking {
					atom_stake_data.stage = AtomStakeStage::Completed;
					<StakeRecords<T>>::insert((account_id.clone(), hash.clone()), atom_stake_data);
					<StakingDataRecords<T>>::remove((account_id.clone(), hash.clone()));

					let free_balance = <balances::Module<T>>::free_balance(account_id.clone());
					let add_value: Balance = 10 * 1_000_000_000 * 1_000 * 1_000;
					if let Some(value) = add_value.try_into().ok() {
						// check
						match free_balance.checked_add(&value) {
							Some(b) => {
								balances::FreeBalance::<T>::insert(&account_id.clone(), b)
							},
							None => (),
						};
					}

					token_balances::Module::<T>::add_bond_token(account_id.clone(), Symbol::AtomBond, 10).expect("Error adding atom bond token");
				}
			}
		}
    }
}

