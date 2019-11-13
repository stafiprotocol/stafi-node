extern crate srml_system as system;
extern crate srml_balances as balances;

use srml_support::{decl_module, decl_storage, decl_event, ensure, dispatch::Result};
use system::ensure_signed;
use sr_std::prelude::*;
use sr_std::{
	convert::{TryInto},
};
use sr_primitives::traits::{Hash, CheckedAdd};
use parity_codec::{Encode};

use stafi_primitives::{Balance, VerifyStatus, AtomStakeStage, ChainType, Symbol, constants::currency::*};
use stafi_primitives::AtomStakeData as CustomStakeData;
use token_balances::bondtoken;
use stafi_externalrpc::tezosrpc;
use stafi_multisig::multisigaddr;


pub trait Trait: system::Trait + balances::Trait + bondtoken::Trait + tezosrpc::Trait + multisigaddr::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as AtomStaking {
		// Just a dummy storage item.
		pub StakeRecords get(stake_records): map T::Hash => Option<CustomStakeData<T::AccountId, T::Hash, Balance>>;

		pub AllStakeRecordsArray get(stake_by_index): map u64 => T::Hash;
        pub AllStakeRecordsCount get(all_stake_count): u64;

		pub OwnedStakeRecordsArray get(stake_of_owner_by_index): map (T::AccountId, u64) => T::Hash;
        pub OwnedStakeRecordsCount get(owned_stake_count): map T::AccountId => u64;

		pub TransferInitDataRecords get(transfer_init_data_records): Vec<CustomStakeData<T::AccountId, T::Hash, Balance>>;
		pub TransferInitCheckRecords get(transfer_init_check_records): map Vec<u8> => bool;
		pub TransferInitDataMapRecords get(transfer_init_data_map_records): linked_map u64 => T::Hash;

		Nonce: u64;
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
				if value % 3 == 0 {
					Self::handle_init();
				}
			}
		}

		// Custom stake
		pub fn custom_stake(origin, multi_sig_address: Vec<u8>, stake_amount: u128, tx_hash: Vec<u8>, block_hash: Vec<u8>, pub_key: Vec<u8>, sig: Vec<u8>) -> Result {
			let sender = ensure_signed(origin)?;

			ensure!(stake_amount > 0, "Stake amount must be greater than 0");

            ensure!(!<TransferInitCheckRecords>::exists(tx_hash.clone()), "This tx_hash exist");

			ensure!(<multisigaddr::Module<T>>::check_multisig_address(ChainType::COSMOS, multi_sig_address.clone()), "Multi sig address is illegal");

			// TODO: Check tx hash
			// ensure!(tx_hash, "Stake amount must be greater than 0");
			// TODO: Check block hash
			// ensure!(block_hash, "Stake amount must be greater than 0");

			// TODO: pub_key verify sig
			// Self::check_sig(tx_hash.clone(), pub_key.clone(), sig.clone())?;
			
			let owned_stake_count = Self::owned_stake_count(&sender);
        	let new_owned_stake_count = owned_stake_count.checked_add(1).ok_or("Overflow adding a new owned stake")?;

			let all_stake_count = Self::all_stake_count();
        	let new_all_stake_count = all_stake_count.checked_add(1).ok_or("Overflow adding a new stake")?;

			// TODO: pub_key generate from
			let _from: Vec<u8> = Vec::new();
			
			let nonce = <Nonce>::get();
			let random_seed = <system::Module<T>>::random_seed();
            let hash = (random_seed, &sender, nonce).using_encoded(<T as system::Trait>::Hashing::hash);

			let stake_data = CustomStakeData {
				id: hash,
				initiator: sender.clone(),
				multi_sig_address: multi_sig_address,
				stage: AtomStakeStage::Init,
				stake_amount: stake_amount,
				tx_hash: tx_hash.clone(),
				block_hash: block_hash.clone(),
				stake_account: pub_key,
				sig: sig
			};

			<StakeRecords<T>>::insert(hash, stake_data.clone());

			<OwnedStakeRecordsArray<T>>::insert((sender.clone(), owned_stake_count), hash);
			<OwnedStakeRecordsCount<T>>::insert(&sender, new_owned_stake_count);

			<AllStakeRecordsArray<T>>::insert(all_stake_count, hash);
			<AllStakeRecordsCount>::put(new_all_stake_count);

			<TransferInitDataMapRecords<T>>::insert(all_stake_count, hash);

			<TransferInitCheckRecords>::insert(tx_hash, true);

			<Nonce>::mutate(|n| *n += 1);

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
		let mut tmp_datas: Vec<CustomStakeData<T::AccountId, T::Hash, Balance>> = Vec::new();

        for (key, hash) in <TransferInitDataMapRecords<T>>::enumerate() {
			if tmp_datas.len() >= 50 {
				break;
			}

			if let Some(mut stake_data) = Self::stake_records(hash) {

				let (status, _) = <tezosrpc::Module<T>>::verified(&stake_data.tx_hash);
				let enum_status = VerifyStatus::create(status);

				if stake_data.stage == AtomStakeStage::Completed {
					<TransferInitDataMapRecords<T>>::remove(key);
					<tezosrpc::Module<T>>::remove_verified(stake_data.tx_hash);
					continue;
				}
				
				match enum_status {
					VerifyStatus::Confirmed => {
						let account_id = &stake_data.initiator;

						stake_data.stage = AtomStakeStage::Completed;
						<StakeRecords<T>>::insert(hash, stake_data.clone());

						bondtoken::Module::<T>::create_bond_token(
							account_id.clone(),
							Symbol::XTZ,
							stake_data.stake_amount,
							stake_data.id,
							stake_data.multi_sig_address
						).expect("Error adding xtz bond token");

						// TODO: Add restrictive conditions to issue FIS token
						let free_balance = <balances::Module<T>>::free_balance(account_id.clone());
						let add_value: Balance = 100 * DOLLARS;
						if let Some(value) = add_value.try_into().ok() {
							// check
							match free_balance.checked_add(&value) {
								Some(total_value) => {
									balances::FreeBalance::<T>::insert(&account_id.clone(), total_value)
								},
								None => (),
							};
						}

						<TransferInitDataMapRecords<T>>::remove(key);
						<tezosrpc::Module<T>>::remove_verified(stake_data.tx_hash);
					}
					VerifyStatus::NotFoundBlock | VerifyStatus::TxNotMatch => {
						<TransferInitCheckRecords>::remove(&stake_data.tx_hash);
						<TransferInitDataMapRecords<T>>::remove(key);
						<tezosrpc::Module<T>>::remove_verified(stake_data.tx_hash);
					}
					VerifyStatus::Rollback | VerifyStatus::NotFoundTx => {
						<TransferInitDataMapRecords<T>>::remove(key);
						<tezosrpc::Module<T>>::remove_verified(stake_data.tx_hash);
					}
					_ => tmp_datas.push(stake_data),
				}

			}
		}

		<TransferInitDataRecords<T>>::put(tmp_datas);
    }

}

