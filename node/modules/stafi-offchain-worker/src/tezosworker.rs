//#![cfg_attr(not(feature = "std"), no_std)]

extern crate sr_std as rstd;
extern crate srml_support as support;
extern crate srml_system as system;

use support::{decl_module, decl_storage, decl_event};
use rstd::prelude::*;
use system::{ensure_none, ensure_signed, ensure_root};
use stafi_primitives::{XtzTransferData, VerifiedData, VerifyStatus, TxHashType, BabeIdType, HostData};
use codec::Decode;

use app_crypto::{KeyTypeId, RuntimeAppPublic};
use system::offchain::SubmitSignedTransaction;
/// only for debug
fn debug(msg: &str) {
	// let msg = format!("\x1b[34m{}", msg);
	sr_io::print_utf8(msg.as_bytes());
}

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"orin");
pub const BUFFER_LEN: usize = 40960;
pub const BUF_LEN: usize = 2048;

pub mod sr25519 {
	mod app_sr25519 {
		use app_crypto::{app_crypto, sr25519};
		app_crypto!(sr25519, super::super::KEY_TYPE);

		impl From<Signature> for sr_primitives::AnySignature {
			fn from(sig: Signature) -> Self {
				sr25519::Signature::from(sig).into()
			}
		}
	}

	/// An oracle signature using sr25519 as its crypto.
	// pub type AuthoritySignature = app_sr25519::Signature;

	/// An oracle identifier using sr25519 as its crypto.
	pub type AuthorityId = app_sr25519::Public;
}

pub trait Trait: system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	/// A dispatchable call type.
	type Call: From<Call<Self>>;
	/// A transaction submitter.
	//type SubmitTransaction: SubmitUnsignedTransaction<Self, <Self as Trait>::Call>;
	type SubmitTransaction: SubmitSignedTransaction<Self, <Self as Trait>::Call>;
	/// The local keytype
	type KeyType: RuntimeAppPublic + From<Self::AccountId> + Into<Self::AccountId> + Clone;
}

decl_storage! {
	trait Store for Module<T: Trait> as TezosWorker {
		XtzTransferDataVec get(xtz_transfter_data_vec): Vec<XtzTransferData<T::AccountId, T::Hash>>;
		pub Verified get(verified): map TxHashType => (i8, u64);
		pub NodeResponse get(node_response): linked_map (TxHashType, BabeIdType) => Option<VerifiedData>;
		RpcHost get(rpc_host): Vec<HostData>;
		BlocksConfirmed: u8;
		BlockDuration: u64;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn set_xtz_transfer_data(origin, xtd: XtzTransferData<T::AccountId, T::Hash>) {
			let _who = ensure_signed(origin)?;
			let mut v:Vec<XtzTransferData<T::AccountId, T::Hash>> = Vec::new();
			v.push(xtd); 
			<XtzTransferDataVec<T>>::put(v);	
		}

		fn set_node_response(origin, txhash: TxHashType, babe_key: BabeIdType, v_data: VerifiedData) {
			let _who = ensure_none(origin)?;
			
			NodeResponse::insert((txhash, babe_key), v_data);
		}

		fn add_rpc_host(origin, host:Vec<u8>) {
			let _who = ensure_signed(origin)?;
			
			let host_data = HostData {
				host: host,
				weight: 1,
			};

			let mut v: Vec<HostData> = RpcHost::get();
			v.push(host_data);
			RpcHost::put(v);
		}

		fn remove_rpc_host(origin, host:Vec<u8>) {
			let _who = ensure_root(origin)?;

			let v: Vec<HostData> = RpcHost::get().into_iter().filter(|x| x.host != host).collect();
			RpcHost::put(v);
		}

		fn set_blocks_confirmed(origin, blocks:u8) {
			let _who = ensure_root(origin)?;
			
			BlocksConfirmed::put(blocks);
		}

		fn set_block_duration(origin, duration:u64) {
			let _who = ensure_root(origin)?;
			
			BlockDuration::put(duration);
		}

		fn on_finalize() {
			let mut txhash_list: Vec<TxHashType> = Vec::new();
			let mut response_list: Vec<((TxHashType, BabeIdType), VerifiedData)> = Vec::new();
			for (k, v) in NodeResponse::enumerate() {
				let status = VerifyStatus::create(Verified::get(&k.0).0);
				if status != VerifyStatus::Confirmed && status != VerifyStatus::NotFound && status != VerifyStatus::Rollback && status != VerifyStatus::BadRequest {
					txhash_list.push(k.0.clone());
					response_list.push((k, v));
				}
			}

			for txhash in txhash_list {
				let mut timestamp = 0;

				let new_status = get_new_status(txhash.clone(), response_list.clone(), &mut timestamp);
				if new_status != VerifyStatus::UnVerified {
					Verified::insert(txhash, (new_status as i8, timestamp));
				}
			}
		}

		fn offchain_worker(now: T::BlockNumber) {
			debug("in offchain worker");
			if let Some(key) = Self::authority_id() {
                debug("sign...");

                let call = Call::add_rpc_host([0x33,0x34].to_vec());
		        let _ = T::SubmitTransaction::sign_and_submit(call, key.clone().into());
            }
		}
	}
}

fn get_new_status(txhash: TxHashType, response: Vec<((TxHashType, BabeIdType), VerifiedData)>, timestamp: &mut u64) -> VerifyStatus {
	let mut verified_counter = 0;
	let mut confirmed_counter = 0;
	let mut notfound_counter = 0;
	let mut rollback_counter = 0;
	let mut badreq_counter = 0;

	let mut babe_num = 0;
	for (k, v) in response {
		if txhash != k.0 {
			continue;
		}

		*timestamp = v.timestamp;
		babe_num = v.babe_num;

		match VerifyStatus::create(v.status) {
			VerifyStatus::Verified => verified_counter = verified_counter + 1,
			VerifyStatus::Confirmed => confirmed_counter = confirmed_counter + 1,
			VerifyStatus::NotFound => notfound_counter = notfound_counter + 1,
			VerifyStatus::Rollback => rollback_counter = rollback_counter + 1,
			VerifyStatus::BadRequest => badreq_counter = badreq_counter + 1,
			_ => (),
		}
	}

	let mut new_status = VerifyStatus::UnVerified;

	if verified_counter >= (babe_num + 2)/2 {
		new_status = VerifyStatus::Verified;
	} else if confirmed_counter >= (babe_num + 2)/2 {
		new_status = VerifyStatus::Confirmed;
	} else if notfound_counter >= (babe_num + 2)/2 {
		new_status = VerifyStatus::NotFound;
	} else if rollback_counter >= (babe_num + 2)/2 {
		new_status = VerifyStatus::Rollback;
	} else if badreq_counter >= (babe_num + 2)/2 {
		new_status = VerifyStatus::BadRequest;
	}

	return new_status;
}

impl<T: Trait> Module<T> {
	pub fn remove_verified(txhash: TxHashType) {
		if Verified::exists(&txhash) {
			Verified::remove(&txhash);
			for (k, _) in NodeResponse::enumerate() {
				if k.0 == txhash {
					NodeResponse::remove(k);
				}
			}
		}
	}

	fn authority_id() -> Option<T::AccountId> {
		let local_keys = T::KeyType::all().iter().map(
			|i| (*i).clone().into()
		).collect::<Vec<T::AccountId>>();

		if local_keys.len() > 0 {
			Some(local_keys[0].clone())
		} else {
			None
		}
	}
}

decl_event!(
    pub enum Event<T>
    where
        <T as system::Trait>::AccountId,
    {
        SetAuthority(AccountId),
    }
);