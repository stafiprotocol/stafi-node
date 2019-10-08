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
use token_balances::Symbol;
use log::info;


#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq)]
pub enum XtzStakeStage {
	// Init
	Init,
	// Transfer token to multi sig address
	Transfering,
	// Successful transfer
	Completed,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, PartialEq)]
pub struct XtzStakeTokenData {
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
pub struct XtzStakeData<AccountId, Hash> {
	// // identifier id
	pub id: Hash,
	// creator of stake
	pub initiator: AccountId,
	// multi sig address
	pub multi_sig_address: Vec<u8>,
	// Stage of stake
	pub stage: XtzStakeStage,
	// Token data of stake
	pub stake_token_data: XtzStakeTokenData,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, PartialEq)]
pub struct XtzTransferData<AccountId, Hash> {
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
	trait Store for Module<T: Trait> as XtzStaking {
		// Just a dummy storage item.
		pub StakeRecords get(stake_records): map (T::AccountId, T::Hash) => Option<XtzStakeData<T::AccountId, T::Hash>>;
		pub StakeDataHashRecords get(stake_data_hash_records): map T::AccountId => Vec<T::Hash>;
		pub TransferInitDataRecords get(transfer_init_data_records): linked_map (T::AccountId, T::Hash) => Option<XtzTransferData<T::AccountId, T::Hash>>;
		pub TransferingDataRecords get(transfering_data_records): linked_map (T::AccountId, T::Hash) => T::AccountId;
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

			let stake_token_data = XtzStakeTokenData {
				token_type: StakeTokenType::XTZ,
				token_decimals: 18,
				validator: validator,
				stake_amount: stake_amount,
				reward_amount: 0,
			};

			<StakeRecords<T>>::insert((sender.clone(), hash.clone()), XtzStakeData {
				id: hash.clone(),
				initiator: sender.clone(),
				multi_sig_address: multi_sig_address,
				stage: XtzStakeStage::Init,
				stake_token_data: stake_token_data.clone(),
			});

			let mut hashs = <StakeDataHashRecords<T>>::get(sender.clone());
			hashs.push(hash.clone());
			<StakeDataHashRecords<T>>::insert(sender.clone(), hashs);

			let transfer_data = XtzTransferData {
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
		info!("bbb {}.", 1);

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

			if let Some(mut xtz_stake_data) = Self::stake_records((account_id.clone(), hash.clone())) {
				if xtz_stake_data.stage == XtzStakeStage::Init {
					xtz_stake_data.stage = XtzStakeStage::Transfering;
					<StakeRecords<T>>::insert((account_id.clone(), hash.clone()), xtz_stake_data);

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

			if let Some(mut xtz_stake_data) = Self::stake_records((account_id.clone(), hash.clone())) {
				if xtz_stake_data.stage == XtzStakeStage::Transfering {
					xtz_stake_data.stage = XtzStakeStage::Completed;
					<StakeRecords<T>>::insert((account_id.clone(), hash.clone()), xtz_stake_data);
					<TransferingDataRecords<T>>::remove((account_id.clone(), hash.clone()));

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

					token_balances::Module::<T>::add_bond_token(account_id.clone(), Symbol::XtzBond, 10).expect("Error adding xtz bond token");
				}
			}
		}
    }
}

