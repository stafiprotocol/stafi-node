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
}

#[cfg(feature = "std")]
impl InherentDataProvider {
	pub fn new(u: String, h: String, sd: u64) -> Self {
		Self {
			url : u,
			host : h,
			slot_duration : sd,
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
	let mut resp = client.post( &host[..])
		.header("Content-Type", "application/json")
		.body(params)
		.send().unwrap();
	let text = resp.text().unwrap();
	
	let v: Value = serde_json::from_str(&text).unwrap();	

	let result:String = serde_json::to_string(&v["result"]).unwrap();
	
	result
}

#[cfg(feature = "std")]
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
		let now = SystemTime::now();
		let now_millis:u64 = now.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64;
		let slot_number = now_millis / self.slot_duration;
		if slot_number % (RPC_REQUEST_INTERVAL/self.slot_duration) != 0 {
			sr_io::print_utf8(b"skip this slot");
			return Ok(());
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
			let status = extract_number(result1);
			if status == 1 {
				sr_io::print_utf8(b"status is 1.");
				continue;
			}

			let result2 = request_rpc2(self.url.clone(), blockhash, txhash.clone()).unwrap();
			if result2 {
				let verified_data = VerifiedData {
					tx_hash: txhash.as_bytes().to_vec()
				};

				verified_data_vec.push(verified_data);
			}
		}
		
		if verified_data_vec.len() > 0 {
			let data = Encode::encode(&verified_data_vec);
			sr_io::print_hex(&data.clone());
			inherent_data.put_data(INHERENT_IDENTIFIER, &data);
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

	let url = format!("{}chains/main/blocks1/{}", self_url, blockhash);
	reqwest::get(&url[..])
	.map_err(|error| {
			format!("{:?}", error).into()
	}).and_then(|mut resp| {
		resp.text()
		.map_err(|_| {
			"Could not get response body".into()
		}).and_then(|body| {
			let v: Value = serde_json::from_str(&body).unwrap();
			let v_operations = serde_json::to_string(&v["operations"]).unwrap();
			let v1: Value = serde_json::from_str(&v_operations).unwrap();
			let vl1 = v1.as_array().unwrap().len();
			let last_operation = serde_json::to_string(&v1[vl1-1]).unwrap();
			let v2: Value = serde_json::from_str(&last_operation).unwrap();
			let vl2 = v2.as_array().unwrap().len();
			let mut found = false;
			for index in 0..vl2 {
				let hash = serde_json::to_string(&v2[index]["hash"]).unwrap();
				//sr_io::print_utf8(&hash.clone().into_bytes());	
				if hash[1..hash.len()-1] == txhash {
					sr_io::print_utf8(b"found tx on chian");
					found = true;
					break;
				}
			}
			
			return Ok(found);
		})
	})	
}