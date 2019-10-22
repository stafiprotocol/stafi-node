//#![cfg_attr(not(feature = "std"), no_std)]

extern crate sr_std as rstd;
extern crate srml_support as support;
extern crate srml_system as system;

use support::{decl_module, decl_storage};
use rstd::prelude::*;
use system::{ensure_none, ensure_signed};
use inherents::{RuntimeString, InherentIdentifier, ProvideInherent, MakeFatalError, InherentData};

use stafi_primitives::{XtzTransferData, VerifiedData, VerifyStatus, TxHashType, BabeIdType};
use codec::{Encode, Decode};

pub mod tezos;
pub use tezos::INHERENT_IDENTIFIER;
#[cfg(feature = "std")]
pub use tezos::InherentDataProvider;

pub use tezos::InherentType;
pub use tezos::TXHASH_LEN;

//pub const ZERO_HASH: &'static [u8] = b"000000000000000000000000000000000000000000000000000";

pub trait Trait: system::Trait { }

decl_storage! {
	trait Store for Module<T: Trait> as TezosRpc {
		XtzTransferDataVec get(xtz_transfter_data_vec): Vec<XtzTransferData<T::AccountId, T::Hash>>;
		pub Verified get(verified): map TxHashType => (i8, u64);
		pub NodeResponse get(node_response): linked_map (TxHashType, BabeIdType) => Option<VerifiedData>;
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

		fn on_finalize() {
			let mut txhash_list: Vec<TxHashType> = Vec::new();
			for (k, _) in NodeResponse::enumerate() {
				let status = VerifyStatus::create(Verified::get(&k.0).0);
				if status != VerifyStatus::Confirmed && status != VerifyStatus::NotFound && status != VerifyStatus::Rollback && status != VerifyStatus::BadRequest {
					txhash_list.push(k.0);
				}
			}

			for txhash in txhash_list {
				let mut verified_counter = 0;
				let mut confirmed_counter = 0;
				let mut notfound_counter = 0;
				let mut rollback_counter = 0;
				let mut badreq_counter = 0;

				let mut timestamp = 0;
				let mut babe_num = 0;
				for (k, v) in NodeResponse::enumerate() {
					if txhash != k.0 {
						continue;
					}
					timestamp = v.timestamp;
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

				Verified::insert(txhash, (new_status as i8, timestamp));
			}
		}
	}
}

impl<T: Trait> Module<T> {}

fn extract_inherent_data(data: &InherentData) -> Result<InherentType, RuntimeString> {
	//data.get_data::<InherentType>(&INHERENT_IDENTIFIER)
	//	.map_err(|_| RuntimeString::from("Invalid inherent data encoding."))?
	//	.ok_or_else(|| "Inherent data is not provided.".into())
	
	let result = data.get_data::<InherentType>(&INHERENT_IDENTIFIER).unwrap();
	
	if let Some(s) = result {
		Ok(s)
	} else {
		Err(RuntimeString::from("error in get inherent data."))
	}
}

impl<T: Trait> ProvideInherent for Module<T> {
	type Call = Call<T>;
	type Error = MakeFatalError<RuntimeString>;
	const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;

	fn create_inherent(data: &InherentData) -> Option<Self::Call> {
        let data1 = extract_inherent_data(data);//.expect("Error in extracting inherent data.");

		let d = match data1 {
			Ok(data) => data,
			Err(_) => return None,
		};

		let verified_data_vec:Vec<VerifiedData> = Decode::decode(&mut &d[..]).unwrap();

		let call:Call<T> = Call::set_node_response(verified_data_vec[0].tx_hash.to_vec(), verified_data_vec[0].babe_id.to_vec(), verified_data_vec[0].clone());
		//for index in 1..verified_data_vec.len() {
		//	let txhash = &verified_data_vec[index].tx_hash;
		//	call = Call::set_verified(txhash.to_vec(), (verified_data_vec[index].status, verified_data_vec[index].timestamp));
		//}

		Some(call)
	}

	// TODO: Implement check_inherent.
}
