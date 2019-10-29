extern crate srml_system as system;
extern crate srml_balances as balances;

use srml_support::{decl_module, decl_storage, decl_event, ensure, dispatch::Result};
use system::ensure_signed;
use sr_std::prelude::*;
use sr_std::{
	convert::{TryInto},
};
use sr_primitives::traits::{Hash, CheckedAdd};
use parity_codec::{Encode, Decode};

#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};

use stafi_primitives::{Balance, XtzTransferData, VerifyStatus, Symbol, constants::currency::*};
use token_balances::bondtoken;
use stafi_externalrpc::tezosrpc;
use log::info;


#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq)]
pub enum XtzStakeStage {
	// Init - Transfer token to multi sig address
	Init,
	// Successful transfer
	Completed,
}

#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq)]
pub struct XtzStakeData<AccountId, Hash> {
	// identifier id
	pub id: Hash,
	// creator of stake
	pub initiator: AccountId,
	// Stage of stake
	pub stage: XtzStakeStage,
	// multi sig address
	pub multi_sig_address: Vec<u8>,
	// Token data of stake
	pub stake_amount: Balance,
}

pub trait Trait: system::Trait + bondtoken::Trait + tezosrpc::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as XtzStaking {
		// Just a dummy storage item.
		pub StakeRecords get(stake_records): map (T::AccountId, T::Hash) => Option<XtzStakeData<T::AccountId, T::Hash>>;
		pub StakeDataHashRecords get(stake_data_hash_records): map T::AccountId => Vec<T::Hash>;
		pub TransferInitDataRecords get(transfer_init_data_records): Vec<XtzTransferData<T::AccountId, T::Hash>>;
		pub TransferInitCheckRecords get(transfer_init_check_records): map Vec<u8> => bool;
		pub TransferInitDataMapRecords get(transfer_init_data_map_records): linked_map Vec<u8> => Option<XtzTransferData<T::AccountId, T::Hash>>;
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;

		fn on_finalize(n: T::BlockNumber) {
			if let Some(value) = n.try_into().ok() {
				info!("ddd {}.", value);
				if value % 3 == 0 {
					Self::handle_init();
				}
			}
		}

		// Custom stake xtz
		pub fn custom_stake(origin, multi_sig_address: Vec<u8>, stake_amount: Balance, tx_hash: Vec<u8>, block_hash: Vec<u8>) -> Result {
			let sender = ensure_signed(origin)?;

			// Check that the tx_hash exists
            ensure!(!<TransferInitDataMapRecords<T>>::exists(tx_hash.clone()), "This tx_hash exist");

			// TODO: Check multi sig address
			// ensure!(multi_sig_address > 0, "Multi sig address is illegal");
			ensure!(stake_amount > 0, "Stake amount must be greater than 0");
			// TODO: Check tx hash
			// ensure!(tx_hash, "Stake amount must be greater than 0");
			// TODO: Check block hash
			// ensure!(block_hash, "Stake amount must be greater than 0");
			
			let random_seed = <system::Module<T>>::random_seed();
            let hash = (random_seed, &sender).using_encoded(<T as system::Trait>::Hashing::hash);

			<StakeRecords<T>>::insert((sender.clone(), hash.clone()), XtzStakeData {
				id: hash.clone(),
				initiator: sender.clone(),
				multi_sig_address: multi_sig_address,
				stage: XtzStakeStage::Init,
				stake_amount: stake_amount,
			});

			let mut hashs = <StakeDataHashRecords<T>>::get(sender.clone());
			hashs.push(hash.clone());
			<StakeDataHashRecords<T>>::insert(sender.clone(), hashs);

			let transfer_data = XtzTransferData {
				id: hash.clone(),
				initiator: sender.clone(),
				tx_hash: tx_hash.clone(),
				block_hash: block_hash,
			};

			<TransferInitDataMapRecords<T>>::insert(tx_hash.clone(), transfer_data.clone());

			<TransferInitCheckRecords>::insert(tx_hash, true);

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

impl<T: Trait> Module<T> {

	fn handle_init() {
		let mut tmp_datas: Vec<XtzTransferData<T::AccountId, T::Hash>> = Vec::new();

        for (key, transfer_data) in <TransferInitDataMapRecords<T>>::enumerate() {
			let account_id = &transfer_data.initiator;
			let hash = &transfer_data.id;

			if let Some(mut xtz_stake_data) = Self::stake_records((account_id.clone(), hash.clone())) {

				let (status, _) = <tezosrpc::Module<T>>::verified(&transfer_data.tx_hash);

				let enum_status = VerifyStatus::create(status);

				if xtz_stake_data.stage == XtzStakeStage::Init && enum_status == VerifyStatus::Confirmed {
					xtz_stake_data.stage = XtzStakeStage::Completed;
					<StakeRecords<T>>::insert((account_id.clone(), hash.clone()), xtz_stake_data.clone());

					<TransferInitDataMapRecords<T>>::remove(key);

					let free_balance = <balances::Module<T>>::free_balance(account_id.clone());
					let add_value: Balance = 100 * DOLLARS;
					if let Some(value) = add_value.try_into().ok() {
						// check
						match free_balance.checked_add(&value) {
							Some(b) => {
								balances::FreeBalance::<T>::insert(&account_id.clone(), b)
							},
							None => (),
						};
					}

					bondtoken::Module::<T>::add_bond_token(account_id.clone(), Symbol::XtzBond, xtz_stake_data.stake_amount, xtz_stake_data.id).expect("Error adding xtz bond token");
				} else {
					tmp_datas.push(transfer_data);
				}
			}
		}

		<TransferInitDataRecords<T>>::put(tmp_datas);
    }

}

