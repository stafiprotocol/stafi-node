//#![cfg_attr(not(feature = "std"), no_std)]

extern crate sr_std as rstd;
extern crate srml_support as support;
extern crate srml_system as system;

use support::{decl_module, decl_storage};
use rstd::prelude::*;
use system::{ensure_none, ensure_root};
use inherents::{RuntimeString, InherentIdentifier, ProvideInherent, MakeFatalError, InherentData};

use stafi_primitives::{VerifiedData, VerifyStatus, TxHashType, BabeIdType, HostData};
use codec::Decode;

pub mod tezos;
pub use tezos::INHERENT_IDENTIFIER;
#[cfg(feature = "std")]
pub use tezos::InherentDataProvider;

pub use tezos::InherentType;

pub trait Trait: system::Trait { }

decl_storage! {
	trait Store for Module<T: Trait> as TezosRpc {
		pub Verified get(verified): map TxHashType => (i8, u64);
		VerifiedBak get(verified_bak): Vec<(TxHashType, i8, u64)>;
		pub NodeResponse get(node_response): linked_map TxHashType => Vec<VerifiedData>;
		RpcHost get(rpc_host): Vec<HostData>;
		BlocksConfirmed: u8;
		BlockDuration: u64;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn set_node_response(origin, txhash: TxHashType, babe_id: BabeIdType, v_data: VerifiedData) {
			let _who = ensure_none(origin)?;
			
			let mut vd: Vec<VerifiedData> = NodeResponse::get(&txhash).into_iter().filter(|x| x.babe_id != babe_id).collect();
			vd.push(v_data);
			NodeResponse::insert(txhash, vd);
		}

		fn add_rpc_host(origin, host:Vec<u8>) {
			let _who = ensure_root(origin)?;
			
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
			let mut response_list: Vec<(TxHashType, Vec<VerifiedData>)> = Vec::new();
			for (k, v) in NodeResponse::enumerate() {
				let txhash = k;
				let status = VerifyStatus::create(Verified::get(txhash.clone()).0);
				if status != VerifyStatus::Confirmed && status != VerifyStatus::NotFoundTx && status != VerifyStatus::Rollback && status != VerifyStatus::NotFoundBlock {
					response_list.push((txhash, v));
				}
			}

			for (k, v) in response_list {
				let mut timestamp = 0;
				let txhash = k;

				let new_status = get_new_status(v.clone(), &mut timestamp);
				if new_status != VerifyStatus::UnVerified {
					let status = new_status as i8;
					Verified::insert(txhash.clone(), (status, timestamp));
					let mut vb = VerifiedBak::get();
					vb.push((txhash, status, timestamp));
					VerifiedBak::put(vb);
				}
			}
		}
	}
}

fn get_new_status(vd: Vec<VerifiedData>, timestamp: &mut u64) -> VerifyStatus {
	let mut verified_counter = 0;
	let mut confirmed_counter = 0;
	let mut notfoundtx_counter = 0;
	let mut rollback_counter = 0;
	let mut notfoundblock_counter = 0;
	let mut notresponse_counter = 0;

	let mut babe_num = 0;
	for v in vd {
		*timestamp = v.timestamp;
		babe_num = v.babe_num;

		match VerifyStatus::create(v.status) {
			VerifyStatus::Verified => verified_counter = verified_counter + 1,
			VerifyStatus::Confirmed => confirmed_counter = confirmed_counter + 1,
			VerifyStatus::NotFoundTx => notfoundtx_counter = notfoundtx_counter + 1,
			VerifyStatus::Rollback => rollback_counter = rollback_counter + 1,
			VerifyStatus::NotFoundBlock => notfoundblock_counter = notfoundblock_counter + 1,
			VerifyStatus::NotResponse => notresponse_counter = notresponse_counter + 1,
			_ => (),
		}
	}

	let mut new_status = VerifyStatus::UnVerified;

	if verified_counter >= (babe_num + 2)/2 {
		new_status = VerifyStatus::Verified;
	} else if confirmed_counter >= (babe_num + 2)/2 {
		new_status = VerifyStatus::Confirmed;
	} else if notfoundtx_counter >= (babe_num + 2)/2 {
		new_status = VerifyStatus::NotFoundTx;
	} else if rollback_counter >= (babe_num + 2)/2 {
		new_status = VerifyStatus::Rollback;
	} else if notfoundblock_counter >= (babe_num + 2)/2 {
		new_status = VerifyStatus::NotFoundBlock;
	} else if notresponse_counter >= (babe_num + 2)/2 {
		new_status = VerifyStatus::NotResponse;
	} 

	return new_status;
}

impl<T: Trait> Module<T> {
	pub fn remove_verified(txhash: TxHashType) {
		if Verified::exists(&txhash) {
			Verified::remove(&txhash);
			NodeResponse::remove(&txhash);
		}
	}	
}

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

#[cfg(test)]
mod tests {
	use super::*;
	use std::time::SystemTime;
	use stafi_primitives::{VerifiedData, VerifyStatus, TxHashType, BabeIdType};

	#[test]
	fn test_get_new_status() {
		let now_millis:u64 = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64;
		
		let txhash = b"transaction_hash".to_vec();
		
		let babe_id_list = [b"babe1".to_vec(), b"babe2".to_vec(), b"babe3".to_vec()];

		let mut response: Vec<VerifiedData> = Vec::new();

		response.push(VerifiedData {
			tx_hash: txhash.clone(),
			timestamp: now_millis,
			status: VerifyStatus::Confirmed as i8,
			babe_id: babe_id_list[0].clone(),
			babe_num: babe_id_list.len() as u8,
		});

		response.push(VerifiedData {
			tx_hash: txhash.clone(),
			timestamp: now_millis,
			status: VerifyStatus::NotFoundTx as i8,
			babe_id: babe_id_list[1].clone(),
			babe_num: babe_id_list.len() as u8,
		});

		response.push(VerifiedData {
			tx_hash: txhash.clone(),
			timestamp: now_millis,
			status: VerifyStatus::Confirmed as i8,
			babe_id: babe_id_list[2].clone(),
			babe_num: babe_id_list.len() as u8,
		});

		let mut timestamp = 0;

		let new_status = get_new_status(response.clone(), &mut timestamp);

		assert_eq!(new_status, VerifyStatus::Confirmed);
		assert_eq!(timestamp, now_millis);
	}
}
