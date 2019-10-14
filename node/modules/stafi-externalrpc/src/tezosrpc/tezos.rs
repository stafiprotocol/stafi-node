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

use stafi_primitives::{AccountId, Hash, XtzTransferData, VerifiedData};

pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"tezosrpc";
pub const RPC_REQUEST_INTERVAL: u64 = 60000; //1 minute
pub const TXHASH_LEN: u8 = 51;

pub type InherentType = Vec<u8>;

#[cfg(feature = "std")]
pub struct InherentDataProvider {
	url : String,
	host : String, 
	slot_duration : u64,
	block_duration: u64,
}

#[cfg(feature = "std")]
impl InherentDataProvider {
	pub fn new(u: String, h: String, sd: u64) -> Self {
		Self {
			url : u,
			host : h,
			slot_duration : sd,
			block_duration: 60000 //1 minute
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
		Err(_) => return "null".to_string(),
	};

	let text = resp.text().unwrap();
	let v: Value = serde_json::from_str(&text).unwrap();
	let result:String = serde_json::to_string(&v["result"]).unwrap();
	
	result
}

#[cfg(feature = "std")]
#[allow(dead_code)]
fn extract_number(result:String) -> u64 {
	if result == "null" {
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
		return (0, 0);
	}

	let len = result.len();
	if len < 6 {
		return (0, 0);
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
			if status == 2 {
				sr_io::print_utf8(b"status is 2.");
				continue;
			}

			let mut now_millis:u64 = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64;
			if status == 1 && last_timestamp + self.block_duration > now_millis {
				sr_io::print_utf8(b"status is 1 and timestamp + 60000 > now_millis.");
				continue;	
			}

			let result2 = request_rpc2(self.url.clone(), blockhash, txhash.clone()).unwrap_or_else(|_| false);
			if result2 {
				now_millis = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64;
				let verified_data = VerifiedData {
					tx_hash: txhash.as_bytes().to_vec(),
					timestamp: now_millis,
					status: status + 1
				};

				verified_data_vec.push(verified_data);
			}

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
fn request_rpc2(self_url: String, blockhash: String, txhash: String) -> Result<bool, RuntimeString> {
	//for test
	//return Ok(true);

	let url = format!("{}chains/main/blocks/{}", self_url, blockhash);
	reqwest::get(&url[..])
	.map_err(|error| {
			format!("{:?}", error).into()
	}).and_then(|mut resp| {
		resp.text()
		.map_err(|_| {
			"Could not get response body".into()
		}).and_then(|body| {
			let v: Value = serde_json::from_str(&body).unwrap_or_else(|_| serde_json::json!({}));
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
									sr_io::print_utf8(b"found tx on chian");
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