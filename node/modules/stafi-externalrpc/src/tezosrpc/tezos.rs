extern crate sr_std as rstd;

#[cfg(feature = "std")]
use inherents::{ProvideInherentData};
use inherents::{RuntimeString, InherentIdentifier, InherentData};
#[cfg(feature = "std")]
use primitives;
#[cfg(feature = "std")]
use hex;
#[cfg(feature = "std")]
use reqwest;
#[cfg(feature = "std")]
use serde_json::{Value};
#[cfg(feature = "std")]
use codec::{Encode, Decode};

use rstd::prelude::*;

use stafi_primitives::{AccountId, Hash, XtzTransferData, VerifiedData, VerifyStatus};

use babe_primitives::{AuthorityId, BabeAuthorityWeight};

pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"tezosrpc";
pub const RPC_REQUEST_INTERVAL: u64 = 60000; //1 minute
pub const TXHASH_LEN: u8 = 51;
pub const BLOCK_CONFIRMED: i64 = 3;
pub const TEZOS_BLOCK_DURATION: u64 = 60000;

pub type InherentType = Vec<u8>;

#[cfg(feature = "std")]
pub struct InherentDataProvider {
	url : String,
	host : String, 
	slot_duration : u64,
	blocks_confirmed: i64,
	babe_id: String,
}

#[cfg(feature = "std")]
impl InherentDataProvider {
	pub fn new(u: String, h: String, sd: u64, b: String) -> Self {
		Self {
			url : u,
			host : h,
			slot_duration : sd,
			blocks_confirmed: BLOCK_CONFIRMED,
			babe_id: b,
		}
	}
}

#[cfg(feature = "std")]
fn get_hexkey(key: &[u8]) -> String {
	let key = primitives::twox_128(key);
	let hexkey = hex::encode(key.to_vec());
	
	return hexkey;
}

#[cfg(feature = "std")]
fn get_maphexkey(key: &[u8], item_key: &[u8]) -> String {
	let encoded_item_key = Encode::encode(item_key);
	let mut total_key = key.to_vec();
	total_key.extend(encoded_item_key);

	let key = primitives::blake2_256(total_key.as_ref());
	let hexkey = hex::encode(key.to_vec());
	
	return hexkey;
}

//blake256hash(bytes("ModuleName" + " " + "StorageItem") + bytes(scale("StorageItemKey")))
//blake256hash(bytes("ModuleName" + " " + "StorageItem") + bytes(scale("FirstStorageItemKey"))) + blake256hash(bytes(scale("SecondStorageItemKey")))

#[cfg(feature = "std")]
fn get_map2hexkey(key: &[u8], item_key: &[u8], item2_key: &[u8]) -> String {
	let hexkey = get_maphexkey(key, item_key);
	
	let encoded_item2_key = Encode::encode(item2_key);
	let key2 = primitives::blake2_256(encoded_item2_key.as_ref()).to_vec();

	let hexkey2 = hexkey + &hex::encode(key2);
	
	return hexkey2;
}

#[cfg(feature = "std")]
fn get_value_from_storage(key: String, host: String) -> String {
	let params = format!("{{\"id\":1, \"jsonrpc\":\"2.0\", \"method\": \"state_getStorage\", \"params\":[\"0x{}\"]}}", key);

	let client = reqwest::Client::new();
	let result = client.post( &host[..])
		.header("Content-Type", "application/json")
		.body(params)
		.send();
	let mut resp = match result {
		Ok(r) => r,
		Err(_) => return "error".to_string(),
	};

	let text = resp.text().unwrap();
	let v: Value = serde_json::from_str(&text).unwrap();
	let result:String = serde_json::to_string(&v["result"]).unwrap();
	
	result
}

#[cfg(feature = "std")]
#[allow(dead_code)]
fn extract_number(result:String) -> u64 {
	if result == "null" || result == "error" {
		return 0;
	}

	let len = result.len();
	if len < 4 {
		return 0;
	}
	let mut s_result = String::new();
	for i in (3..len-1).step_by(2) {
		s_result.push_str(&result[len-i..len-i+2])
	}

	u64::from_str_radix(&s_result, 16).unwrap()
} 

#[cfg(feature = "std")]
fn extract_status_and_timestamp(result:String) -> (i8, u64) {
	if result == "null" {
		return (VerifyStatus::UnVerified as i8, 0);
	}

	if result == "error" {
		return (VerifyStatus::Error as i8, 0);
	}

	let len = result.len();
	if len < 6 {
		return (VerifyStatus::Error as i8, 0);
	}

	let status = i8::from_str_radix(&result[3..5], 16).unwrap();

	let mut s_result = String::new();
	for i in (5..len-3).step_by(2) {
		s_result.push_str(&result[len-i..len-i+2])
	}

	let timestamp: u64 = u64::from_str_radix(&s_result, 16).unwrap();

	(status, timestamp)
} 

#[cfg(feature = "std")]
fn decode_transfer_data(data: String) -> Vec<XtzTransferData<AccountId, Hash>> {
	let data1 = hex::decode(&data[3..data.len()-1]).unwrap();

    let result: Vec<XtzTransferData<AccountId, Hash>> = Decode::decode(&mut &data1[..]).unwrap();

	return result;
}

#[cfg(feature = "std")]
fn decode_authorities_data(data: String) -> Vec<(AuthorityId, BabeAuthorityWeight)> {
	let data1 = hex::decode(&data[3..data.len()-1]).unwrap();

    let result: Vec<(AuthorityId, BabeAuthorityWeight)> = Decode::decode(&mut &data1[..]).unwrap();

	return result;
}

#[cfg(feature = "std")]
impl ProvideInherentData for InherentDataProvider {
	fn inherent_identifier(&self) -> &'static InherentIdentifier {
		&INHERENT_IDENTIFIER
	}

	fn provide_inherent_data(&self, inherent_data: &mut InherentData) -> Result<(), RuntimeString> {
		use std::time::SystemTime;
		let verify_in_batch = false;
		if verify_in_batch {
			let now_millis:u64 = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64;
			let slot_number = now_millis / self.slot_duration;
			if slot_number % (RPC_REQUEST_INTERVAL/self.slot_duration) != 0 {
				sr_io::print_utf8(b"skip this slot");
				return Ok(());
			}
		}

		let babe_auth_key = get_hexkey(b"Babe Authorities");
		let babe_auth_key_str = get_value_from_storage(babe_auth_key, self.host.clone());
		//sr_io::print_utf8(&babe_auth_key_str.clone().into_bytes());
		if babe_auth_key_str == "null" {
			sr_io::print_utf8(b"babe_auth_key_str is null.");	
			return Ok(());
		}

		let mut found_babe_id = false;
		let babe_authorities = decode_authorities_data(babe_auth_key_str);
		for auth in babe_authorities.clone() {
			let bid = auth.0.to_string();
			if bid == self.babe_id {
				sr_io::print_utf8(b"the node is a validator");
				found_babe_id = true;
				break;
			}
		}

		if !found_babe_id {
			sr_io::print_utf8(b"the node isn't a validator");
			return Ok(());
		}

		let babe_num = &babe_authorities.len();

		let txhash_list_key = get_hexkey(b"XtzStaking TransferInitDataRecords");
		let transfer_data_str = get_value_from_storage(txhash_list_key, self.host.clone());
		if transfer_data_str == "null" {
			sr_io::print_utf8(b"transfer_data is null.");
			
			return Ok(());	
		}

		//sr_io::print_utf8(&transfer_data_str.clone().into_bytes());

		let mut verified_data_vec:Vec<VerifiedData> = Vec::new();

		let transfer_data_vec = decode_transfer_data(transfer_data_str);
		for transfer_data in transfer_data_vec {
			let txhash = String::from_utf8(transfer_data.tx_hash).unwrap();
			let blockhash = String::from_utf8(transfer_data.block_hash).unwrap();

			let verified_key = get_maphexkey(b"TezosRpc Verified", &txhash.clone().into_bytes());
			let result1 = get_value_from_storage(verified_key, self.host.clone());
			let (status, last_timestamp) = extract_status_and_timestamp(result1);
			let enum_status = VerifyStatus::create(status);
			if enum_status == VerifyStatus::Error {
				sr_io::print_utf8(b"error in reading verificaiton status.");
				continue;	
			}
			
			let verified_vec_key = get_map2hexkey(b"TezosRpc NodeResponse", &txhash.clone().into_bytes(), &self.babe_id.clone().into_bytes());
			let result3 = get_value_from_storage(verified_vec_key, self.host.clone());
			//sr_io::print_utf8(&result3.clone().into_bytes()); //TODO:

			if enum_status == VerifyStatus::Confirmed || enum_status == VerifyStatus::NotFound || enum_status == VerifyStatus::Rollback || enum_status == VerifyStatus::BadRequest {
				sr_io::print_utf8(&format!("{:}'s status is {:}.", txhash, enum_status as i8).into_bytes());
				continue;
			}

			let mut now_millis:u64 = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64;
			if enum_status == VerifyStatus::Verified && last_timestamp + self.blocks_confirmed as u64 * TEZOS_BLOCK_DURATION > now_millis {
				sr_io::print_utf8(b"status is Verified and timestamp + 60000 > now_millis.");
				continue;	
			}

			let mut new_status = VerifyStatus::Verified as i8;
			let mut cur_level:i64 = 0;
			let mut level:i64 = 0;
			let result2 = request_rpc2(self.url.clone(), blockhash, txhash.clone(), &mut level).unwrap_or_else(|_| false);
			if level < 0 {
				sr_io::print_utf8(b"bad rpc request.");
				new_status = VerifyStatus::BadRequest as i8;
			} else {
				let _ = request_rpc2(self.url.clone(), "head".to_string(), "".to_string(), &mut cur_level).unwrap_or_else(|_| false);
				if result2 {
					if cur_level > 0 && cur_level - level >= self.blocks_confirmed {
						new_status = VerifyStatus::Confirmed as i8;
					} 
				} else {
					new_status = VerifyStatus::NotFound as i8;
					if enum_status == VerifyStatus::Verified && cur_level > 0 && cur_level - level >= self.blocks_confirmed  {
						new_status = VerifyStatus::Rollback as i8;
					} 	
				}
			}

			now_millis = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64;
			let verified_data = VerifiedData {
				tx_hash: txhash.as_bytes().to_vec(),
				timestamp: now_millis,
				status: new_status,
				babe_id: self.babe_id.as_bytes().to_vec(),
				babe_num: *babe_num as u8,
			};
			sr_io::print_utf8(&format!("{} set tx {} new status: {}", self.babe_id, txhash, new_status).into_bytes());

			verified_data_vec.push(verified_data);

			if !verify_in_batch {break;}
		}
		
		if verified_data_vec.len() > 0 {
			let data = Encode::encode(&verified_data_vec);
			sr_io::print_hex(&data.clone());
			let _ = inherent_data.put_data(INHERENT_IDENTIFIER, &data);
		}

		Ok(())
	}

	fn error_to_string(&self, error: &[u8]) -> Option<String> {
		Some(format!("{:?}", error))
	}
}

#[cfg(feature = "std")]
fn request_rpc2(self_url: String, blockhash: String, txhash: String, level: &mut i64) -> Result<bool, RuntimeString> {
	//for test
	//return Ok(true);

	let url = format!("{}chains/main/blocks/{}", self_url, blockhash);
	reqwest::get(&url[..])
	.map_err(|error| {
		*level = -1;
		format!("{:?}", error).into()
	}).and_then(|mut resp| {
		resp.text()
		.map_err(|_| {
			*level = -2;
			"Could not get response body".into()
		}).and_then(|body| {
			let v: Value = serde_json::from_str(&body).unwrap_or_else(|_| serde_json::json!({}));
			let v_level: Value = v["header"]["level"].clone();
			if v_level.is_null() || !v_level.is_u64() {
				*level = -3;
				return Ok(false);
			}

			*level = v_level.as_i64().unwrap();

			if txhash == "" {
				return Ok(true);
			}

			let v_operations: Value = v["operations"].clone();
			if v_operations.is_null() || !v_operations.is_array() {
				return Ok(false);
			}

			let vl1 = v_operations.as_array().unwrap().len();
			if vl1 == 0 {
				return Ok(false);
			}
			
			let last_operation: Value = v_operations[vl1-1].clone();
			if !last_operation.is_array() {
				return Ok(false);
			}

			let mut found = false;
			for item in last_operation.as_array().unwrap() {
				let hash = serde_json::to_string(&item["hash"]).unwrap();
				//sr_io::print_utf8(&hash.clone().into_bytes());	
				if hash[1..hash.len()-1] == txhash {
					let contents: Value = item["contents"].clone();
					if !contents.is_null() && contents.is_array() {
						for content in contents.as_array().unwrap() {
							let kind: String = serde_json::to_string(&content["kind"]).unwrap();
							if kind == "\"transaction\"" {
								let status:String = serde_json::to_string(&content["metadata"]["operation_result"]["status"]).unwrap();
								if status == "\"applied\"" {
									sr_io::print_utf8(b"found tx on chain");
									found = true;
									break;
								}
							}
						}
					}
				}

				if found {break;}
			}
			
			return Ok(found);
		})
	})	
}