extern crate randomness_collective_flip as random;

use support::{decl_module, decl_storage, decl_event, ensure, dispatch::Result, traits::Randomness};
use system::ensure_signed;
use sr_std::prelude::*;
use sr_std::{
	convert::{TryInto},
};
use sr_std::str;
use sr_primitives::traits::{Hash, CheckedAdd};
use parity_codec::{Encode};

use node_primitives::{Balance, VerifyStatus, XtzStakeStage, XtzStakeData, ChainType, Symbol, constants::currency::*};
use token_balances::bondtoken;
use stafi_offchain_worker::tezosworker;
use stafi_multisig::multisigaddr;
use log::info;

pub trait Trait: system::Trait + balances::Trait + bondtoken::Trait + tezosworker::Trait + multisigaddr::Trait + stafi_staking_storage::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as XtzStaking {
		// Just a dummy storage item.
		pub StakeRecords get(stake_records): map T::Hash => Option<XtzStakeData<T::AccountId, T::Hash, Balance>>;

		pub AllStakeRecordsArray get(stake_by_index): map u64 => T::Hash;
        pub AllStakeRecordsCount get(all_stake_count): u64;

		pub OwnedStakeRecordsArray get(stake_of_owner_by_index): map (T::AccountId, u64) => T::Hash;
        pub OwnedStakeRecordsCount get(owned_stake_count): map T::AccountId => u64;

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
					info!("ddd {}.", value);
					Self::handle_init();
				}
			}
		}

		// Custom stake xtz
		pub fn custom_stake(origin, multi_sig_address: Vec<u8>, stake_amount: u128, tx_hash: Vec<u8>, block_hash: Vec<u8>, pub_key: Vec<u8>, sig: Vec<u8>) -> Result {
			let sender = ensure_signed(origin)?;

			ensure!(stake_amount > 0, "Stake amount must be greater than 0");

            ensure!(!<TransferInitCheckRecords>::exists(tx_hash.clone()), "This tx_hash exist");

			// ensure!(<multisigaddr::Module<T>>::check_multisig_address(ChainType::TEZOS, multi_sig_address.clone()), "Multi sig address is illegal");

			// TODO: Check tx hash
			// ensure!(tx_hash, "Stake amount must be greater than 0");
			// TODO: Check block hash
			// ensure!(block_hash, "Stake amount must be greater than 0");

			// pub_key verify sig
			// ensure!(stafi_crypto::tez::verify::verify_with_ed(&tx_hash, &sig, &pub_key), "Verify signature failed");
			
			let owned_stake_count = Self::owned_stake_count(&sender);
        	let new_owned_stake_count = owned_stake_count.checked_add(1).ok_or("Overflow adding a new owned stake")?;

			let all_stake_count = Self::all_stake_count();
        	let new_all_stake_count = all_stake_count.checked_add(1).ok_or("Overflow adding a new stake")?;

			// pub_key generate from
			let from = Self::pkh_from_pk(pub_key.clone());
			
			let nonce = <Nonce>::get();
			let random_seed = <random::Module<T>>::random_seed();
            let hash = (random_seed, &sender, nonce).using_encoded(<T as system::Trait>::Hashing::hash);

			let stake_data = XtzStakeData {
				id: hash,
				initiator: sender.clone(),
				multi_sig_address: multi_sig_address,
				stage: XtzStakeStage::Init,
				stake_amount: stake_amount,
				tx_hash: tx_hash.clone(),
				block_hash: block_hash.clone(),
				stake_account: from,
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

	fn pkh_from_pk(pub_key: Vec<u8>) -> Vec<u8> {
		let edpk_str = str::from_utf8(&pub_key).unwrap();
		let raw_pk_with_prefix = stafi_crypto::tez::base58::from_check(&edpk_str).unwrap();
        let pkh = stafi_crypto::tez::generator::pkh_from_rawpk(&raw_pk_with_prefix[4..]);

		pkh
	}

	fn handle_init() {
		let mut tmp_datas: Vec<XtzStakeData<T::AccountId, T::Hash, Balance>> = Vec::new();

        for (key, hash) in <TransferInitDataMapRecords<T>>::enumerate() {
			if tmp_datas.len() >= 50 {
				break;
			}

			if let Some(mut xtz_stake_data) = Self::stake_records(hash) {

				// let account_id = &xtz_stake_data.initiator;

				// xtz_stake_data.stage = XtzStakeStage::Completed;
				// <StakeRecords<T>>::insert(hash, xtz_stake_data.clone());

				// bondtoken::Module::<T>::create_bond_token(
				// 	account_id.clone(),
				// 	Symbol::XTZ,
				// 	xtz_stake_data.stake_amount,
				// 	xtz_stake_data.id,
				// 	xtz_stake_data.multi_sig_address
				// ).expect("Error adding xtz bond token");

				// <TransferInitDataMapRecords<T>>::remove(key);

				if xtz_stake_data.stage == XtzStakeStage::Completed {
					<TransferInitDataMapRecords<T>>::remove(key);
					<tezosworker::Module<T>>::remove_verified(xtz_stake_data.tx_hash);
					continue;
				}

				let (status, _) = <tezosworker::Module<T>>::verified(&xtz_stake_data.tx_hash);
				let enum_status = VerifyStatus::create(status);
				
				match enum_status {
					VerifyStatus::Confirmed => {
						let account_id = &xtz_stake_data.initiator;

						xtz_stake_data.stage = XtzStakeStage::Completed;
						<StakeRecords<T>>::insert(hash, xtz_stake_data.clone());

						bondtoken::Module::<T>::create_bond_token(
							account_id.clone(),
							Symbol::XTZ,
							xtz_stake_data.stake_amount,
							xtz_stake_data.id,
							xtz_stake_data.multi_sig_address
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
						<tezosworker::Module<T>>::remove_verified(xtz_stake_data.tx_hash);
					}
					VerifyStatus::NotFoundBlock | VerifyStatus::TxNotMatch => {
						<TransferInitCheckRecords>::remove(&xtz_stake_data.tx_hash);
						<TransferInitDataMapRecords<T>>::remove(key);
						<tezosworker::Module<T>>::remove_verified(xtz_stake_data.tx_hash);
					}
					VerifyStatus::Rollback | VerifyStatus::NotFoundTx => {
						<TransferInitDataMapRecords<T>>::remove(key);
						<tezosworker::Module<T>>::remove_verified(xtz_stake_data.tx_hash);
					}
					_ => {
						tmp_datas.push(xtz_stake_data);
					}
				}

			}
		}

		<stafi_staking_storage::Module<T>>::put_xtz_transfer_init_data_records(tmp_datas);
    }

}

