#![cfg_attr(not(feature = "std"), no_std)]
extern crate session;
extern crate paint_balances as balances;
extern crate paint_support as support;
extern crate paint_system as system;

use hex_literal::hex;
use im_online;
use parity_codec::{Decode, Encode};
use sr_primitives::traits::{CheckedAdd};
use sr_std::{convert::TryInto, str};
use node_primitives::{AccountId, Balance};
use substrate_primitives::{crypto::UncheckedInto, sr25519::Public};
//use log::info;
use support::{
	decl_event, decl_module, decl_storage,
};
pub trait Trait: system::Trait + session::Trait + im_online::Trait + balances::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

const ALLOCATION_BALANCES: u64 = 100000000;
const ALLOCATION_TEAM_PER: u8 = 10;
const ALLOCATION_FUND_PER: u8 = 20;
const ALLOCATION_INVESTOR_PER: u8 = 12;
const ALLOCATION_RESERVE_PER: u8 = 10;
const ALLOCATION_CUT_OFF_SECOND: u64 = 11447460000; //365.5 days per year
const SECS_PER_BLOCK: u8 = 2;
const EPOCH_DURATION_IN_BLOCKS: u32 = 300;

struct AllocationData {
	account_id: AccountId,
	rate: u8,
}
decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		Something get(something): Option<u32>;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		fn deposit_event() = default;
	}
}

impl<T: Trait> Module<T> {
	fn get_cut_off_height() -> u64 {
		ALLOCATION_CUT_OFF_SECOND / (SECS_PER_BLOCK as u64)
	}

	fn get_send_account_session() -> u64 {
		let height: u64 = Self::get_cut_off_height();
		ALLOCATION_BALANCES / (height / (EPOCH_DURATION_IN_BLOCKS as u64))
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
		SomethingStored(u32, AccountId),
	}
);

// impl<T: Trait> session::OneSessionHandler<T::AccountId> for Module<T> {
// 	type Key = T::AuthorityId;
// 	fn on_new_session<'a, I: 'a>(_changed: bool, _validators: I, _queued_validators: I)
// 	where
// 		I: Iterator<Item = (&'a T::AccountId, T::AuthorityId)>,
// 	{
// 	}

// 	fn on_before_session_ending() {
// 		// team account
// 		let team_account_public: Public =
// 			hex!["d43b38b84b60b06e7f1a00d892dcff67ea69dc1dc2f837fdb6a27344b63c9279"]
// 				.unchecked_into();

// 		let team_account = AllocationData {
// 			account_id: team_account_public,
// 			rate: ALLOCATION_TEAM_PER,
// 		};
// 		//fundation
// 		let fun_account_public: Public =
// 			hex!["e489771ea3c4f10cb28698d21d5382ce3c5a673f47bade2b7325718701ad4b0c"]
// 				.unchecked_into();
// 		let fun_account = AllocationData {
// 			account_id: fun_account_public,
// 			rate: ALLOCATION_FUND_PER,
// 		};

// 		//Investor
// 		let investor_account_public: Public =
// 			hex!["7e35cbeea9f986613567088dbdb56da124f6511e339a44fd53a127b7653cff34"]
// 				.unchecked_into();
// 		let investor_account = AllocationData {
// 			account_id: investor_account_public,
// 			rate: ALLOCATION_INVESTOR_PER,
// 		};


// 		//Reserve
// 		let reserve_account_public: Public =
// 			hex!["63410a24555c6f0c5ba8f0d27f85740dca150d9a0d67e3fa8502d5d9e6a4fafe"]
// 				.unchecked_into();
// 		let reserve_account = AllocationData {
// 			account_id: reserve_account_public,
// 			rate: ALLOCATION_RESERVE_PER,
// 		};


// 		let allocation_arr = [team_account, fun_account, investor_account, reserve_account];
// 		for val in allocation_arr.iter() {
// 			let account_id: T::AccountId = val
// 				.account_id
// 				.using_encoded(|mut s| Decode::decode(&mut s))
// 				.expect("Panic");
// 			let all_amount: u64 = Self::get_send_account_session();
// 			//let account_amount: u64 = all_amount / 100 * val.rate as u64;
// 			let free_balance =
// 				<balances::Module<T>>::free_balance::<T::AccountId>(account_id.clone());
// 			let add_value: Balance = all_amount as u128  * 1_000_000_000 * 1_000 * 1_000 / 100 * val.rate as u128;
// 			if let Some(value) = add_value.try_into().ok() {
// 				// check
// 				match free_balance.checked_add(&value) {
// 					Some(b) => balances::FreeBalance::<T>::insert::<T::AccountId, T::Balance>(
// 						account_id.clone(),
// 						b + value,
// 					),
// 					None => (),
// 				};
// 			}
// 		}
// 	}

// 	fn on_genesis_session<'a, I: 'a>(_validators: I)
// 	where
// 		I: Iterator<Item = (&'a T::AccountId, T::AuthorityId)>,
// 	{
// 	}

// 	fn on_disabled(_i: usize) {}
// }
