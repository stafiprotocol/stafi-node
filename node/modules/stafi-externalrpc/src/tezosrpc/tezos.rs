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

use stafi_primitives::{AccountId, Hash, Balance, XtzStakeData, VerifiedData, VerifyStatus, HostData};

use babe_primitives::{AuthorityId, BabeAuthorityWeight};

pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"tezosrpc";
pub const RPC_REQUEST_INTERVAL: u64 = 60000; //1 minute
pub const TEZOS_TXHASH_LEN: u8 = 51;
pub const TEZOS_BLOCK_CONFIRMED: u8 = 3;
pub const TEZOS_BLOCK_DURATION: u64 = 60000;
pub const TEZOS_RPC_HOST: &'static [u8] = b"https://rpc.tezrpc.me";

pub type InherentType = Vec<u8>;

#[cfg(feature = "std")]
pub struct InherentDataProvider {
	node_rpc_host : String, 
	slot_duration : u64,
	babe_id: String,
}

#[cfg(feature = "std")]
impl InherentDataProvider {
	pub fn new(h: String, sd: u64, b: String) -> Self {
		Self {
			node_rpc_host : h,
			slot_duration : sd,
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

	let key = primitives::blake2_256(&total_key);
	let hexkey = hex::encode(key.to_vec());
	
	return hexkey;
}

#[cfg(feature = "std")]
#[allow(dead_code)]
fn get_map2hexkey(key: &[u8], item_key: &[u8], item2_key: &[u8]) -> String {
	let hexkey = get_maphexkey(key, item_key);
	
	let encoded_item2_key = Encode::encode(item2_key);
	let key2 = primitives::blake2_256(&encoded_item2_key);

	let hexkey2 = hexkey + &hex::encode(key2.to_vec());
	
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
fn decode_stake_data(data: String) -> Vec<XtzStakeData<AccountId, Hash, Balance>> {
	let data1 = hex::decode(&data[3..data.len()-1]).unwrap();

    let result: Vec<XtzStakeData<AccountId, Hash, Balance>> = Decode::decode(&mut &data1[..]).unwrap();

	return result;
}

#[cfg(feature = "std")]
fn decode_node_response_data(data: String) -> Vec<VerifiedData> {
	let data1 = hex::decode(&data[3..data.len()-1]).unwrap();

    let result: Vec<VerifiedData> = Decode::decode(&mut &data1[..]).unwrap();

	return result;
}

#[cfg(feature = "std")]
fn decode_host_data(data: String) -> Vec<HostData> {
	let mut result: Vec<HostData> = Vec::new();
	if data != "null" {
		let data1 = hex::decode(&data[3..data.len()-1]).unwrap();
    	result = Decode::decode(&mut &data1[..]).unwrap();
	}

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
		let mut now_millis:u64 = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64;
		let verify_in_batch = false;
		if verify_in_batch {
			let slot_number = now_millis / self.slot_duration;
			if slot_number % (RPC_REQUEST_INTERVAL/self.slot_duration) != 0 {
				sr_io::print_utf8(b"skip this slot");
				return Ok(());
			}
		}

		let babe_auth_key = get_hexkey(b"Babe Authorities");
		let babe_auth_key_str = get_value_from_storage(babe_auth_key, self.node_rpc_host.clone());
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

		let stake_data_key = get_hexkey(b"XtzStaking TransferInitDataRecords");
		let stake_data_str = get_value_from_storage(stake_data_key, self.node_rpc_host.clone());
		if stake_data_str == "null" {
			sr_io::print_utf8(b"stake_data is null.");
			
			return Ok(());	
		}

		//sr_io::print_utf8(&transfer_data_str.clone().into_bytes());
		
		let blocks_confirmed_key = get_hexkey(b"TezosRpc BlocksConfirmed");
		let blocks_confirmed_str = get_value_from_storage(blocks_confirmed_key, self.node_rpc_host.clone());
		let mut blocks_confirmed = extract_number(blocks_confirmed_str) as u8;
		if blocks_confirmed == 0 {
			blocks_confirmed = TEZOS_BLOCK_CONFIRMED;
		}
		
		let block_duration_key = get_hexkey(b"TezosRpc BlockDuration");
		let block_duration_str = get_value_from_storage(block_duration_key, self.node_rpc_host.clone());
		let mut block_duration = extract_number(block_duration_str);
		if block_duration == 0 {
			block_duration = TEZOS_BLOCK_DURATION;
		}
		
		let tezos_rpc_host_key = get_hexkey(b"TezosRpc RpcHost");
		let tezos_rpc_host_str = get_value_from_storage(tezos_rpc_host_key, self.node_rpc_host.clone());
		let mut tezos_rpc_host_list = decode_host_data(tezos_rpc_host_str);
		if tezos_rpc_host_list.len() == 0 {
			tezos_rpc_host_list.push(HostData {
				host: TEZOS_RPC_HOST.to_vec(),
				weight: 1
			});
		} 
		let tezos_rpc_host = String::from_utf8(tezos_rpc_host_list[(now_millis % tezos_rpc_host_list.len() as u64) as usize].host.clone()).unwrap();	
		sr_io::print_utf8(&format!("current tezos rpc host is {}", tezos_rpc_host.clone()).into_bytes());

		let mut verified_data_vec:Vec<VerifiedData> = Vec::new();

		let stake_data_vec = decode_stake_data(stake_data_str);
		for stake_data in stake_data_vec {
			let txhash = String::from_utf8(stake_data.tx_hash).unwrap();
			let blockhash = String::from_utf8(stake_data.block_hash).unwrap();
			//if txhash.len() != TEZOS_TXHASH_LEN {
			//	sr_io::print_utf8(b"bad tx hash.");
			//	continue;
			//}

			let verified_key = get_maphexkey(b"TezosRpc Verified", &txhash.clone().into_bytes());
			let result1 = get_value_from_storage(verified_key, self.node_rpc_host.clone());
			let (status, last_timestamp) = extract_status_and_timestamp(result1);
			let enum_status = VerifyStatus::create(status);
			if enum_status == VerifyStatus::Error {
				sr_io::print_utf8(b"error in reading verificaiton status.");
				continue;	
			}
			
			if enum_status == VerifyStatus::Confirmed || enum_status == VerifyStatus::NotFoundTx || enum_status == VerifyStatus::Rollback || enum_status == VerifyStatus::NotFoundBlock || enum_status == VerifyStatus::TxNotMatch {
				sr_io::print_utf8(&format!("{:}'s status is {:}.", txhash, enum_status as i8).into_bytes());
				continue;
			}

			now_millis = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64;
			if enum_status == VerifyStatus::Verified && last_timestamp + blocks_confirmed as u64 * block_duration > now_millis {
				sr_io::print_utf8(b"status is Verified and timestamp + 60000 > now_millis.");
				continue;	
			}
			
			let node_response_key = get_maphexkey(b"TezosRpc NodeResponse", &txhash.clone().into_bytes());
			let node_response_str = get_value_from_storage(node_response_key, self.node_rpc_host.clone());
			if node_response_str != "null" {
				let node_response_data = decode_node_response_data(node_response_str);
				let mut node_status_set = false;
				for vd in node_response_data {
					if vd.babe_id == self.babe_id.as_bytes().to_vec() {
						let node_status = VerifyStatus::create(vd.status);
						if node_status != VerifyStatus::UnVerified {
							sr_io::print_utf8(&format!("{:} {:}'s status is {:}.", self.babe_id, txhash, node_status as i8).into_bytes());
							node_status_set = true;
						}
						break;
					}
				}
				if node_status_set {
					continue;
				}
			}

			let from = String::from_utf8(stake_data.stake_account).unwrap();
			let to = String::from_utf8(stake_data.multi_sig_address).unwrap();
			let amount = stake_data.stake_amount as u128;
			
			let mut new_status = VerifyStatus::Verified as i8;
			let mut cur_level:i64 = 0;
			let mut level:i64 = 0;
			let result2 = request_rpc2(tezos_rpc_host.clone(), blockhash, txhash.clone(), from, to, amount, &mut level).unwrap_or_else(|_| false);
			if level < 0 {
				sr_io::print_utf8(&format!("rpc request failed({}).", level).into_bytes());
				if level == -4 {
					new_status = VerifyStatus::TxNotMatch as i8;
				} else if level == -3 {
					new_status = VerifyStatus::NotFoundBlock as i8;
				} else {
					new_status = VerifyStatus::NotResponse as i8;
				}
			} else {
				let _ = request_rpc2(tezos_rpc_host.clone(), "head".to_string(), "".to_string(), "".to_string(), "".to_string(), 0, &mut cur_level).unwrap_or_else(|_| false);
				if result2 {
					if cur_level > 0 && cur_level - level > 0 && (cur_level - level) as u8 >= blocks_confirmed {
						new_status = VerifyStatus::Confirmed as i8;
					} 
				} else {
					new_status = VerifyStatus::NotFoundTx as i8;
					if enum_status == VerifyStatus::Verified && cur_level > 0 && cur_level - level > 0 && (cur_level - level) as u8 >= blocks_confirmed  {
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
			//sr_io::print_hex(&data.clone());
			let _ = inherent_data.put_data(INHERENT_IDENTIFIER, &data);
		}

		Ok(())
	}

	fn error_to_string(&self, error: &[u8]) -> Option<String> {
		Some(format!("{:?}", error))
	}
}

#[cfg(feature = "std")]
fn request_rpc2(self_url: String, blockhash: String, txhash: String, from: String, to: String, stake_amount: u128, level: &mut i64) -> Result<bool, RuntimeString> {
	let url = format!("{}/chains/main/blocks/{}", self_url, blockhash);
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
				let hash = serde_json::to_string(&item["hash"]).unwrap_or_else(|_|String::from("\"\""));
				//sr_io::print_utf8(&hash.clone().into_bytes());	
				if hash[1..hash.len()-1] == txhash {
					let contents: Value = item["contents"].clone();
					if !contents.is_null() && contents.is_array() {
						for content in contents.as_array().unwrap() {
							let kind: String = serde_json::to_string(&content["kind"]).unwrap_or_else(|_|String::from("\"\""));
							let source: String = serde_json::to_string(&content["source"]).unwrap_or_else(|_|String::from("\"\""));
							let destination: String = serde_json::to_string(&content["destination"]).unwrap_or_else(|_|String::from("\"\""));
							let amount_str: String = serde_json::to_string(&content["amount"]).unwrap_or_else(|_|String::from("\"0\""));
							let amount: u128 = amount_str[1..amount_str.len()-1].parse().unwrap_or_else(|_|0);
							if kind != "\"transaction\"" || &source[1..source.len()-1] != from || &destination[1..destination.len()-1] != to || amount != stake_amount {
								*level = -4;
								break;
							}
							
							let status:String = serde_json::to_string(&content["metadata"]["operation_result"]["status"]).unwrap_or_else(|_|String::from(""));
							if status == "\"applied\"" {
								sr_io::print_utf8(b"found tx on chain");
								found = true;

								break;
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_get_hexkey() {
		let result = get_hexkey(b"Sudo Key");
		assert_eq!(result, "50a63a871aced22e88ee6466fe5aa5d9")
	}

	#[test]
	fn test_get_maphexkey() {
		let result = get_maphexkey(b"TezosRpc Verified", b"onv7i9LSacMXjhTdpgzmY4q6PxiZ18TZPq7KrRBRUVX7XJicSDi");
		assert_eq!(result, "1c5f64057a95d792855eebea477b2c17887b4d3fa87413463cb500012f79b56d")
	}

	#[test]
	fn test_request_rpc2() {
		let tezos_url = String::from("https://rpc.tezrpc.me/");
		let block_hash = String::from("BKsxzJMXPxxJWRZcsgWG8AAegXNp2uUuUmMr8gzQcoEiGnNeCA6");
		let tx_hash = String::from("onv7i9LSacMXjhTdpgzmY4q6PxiZ18TZPq7KrRBRUVX7XJicSDi");
		let from = String::from("tz1SYq214SCBy9naR6cvycQsYcUGpBqQAE8d");
		let to = String::from("tz1S4MTpEV356QcuzkjQUdyZdAy36gPwPWXa");
		let amount = 710391;
		let mut level = 0;
		let result = request_rpc2(tezos_url, block_hash, tx_hash, from, to, amount, &mut level).unwrap_or_else(|_| false);
		assert_eq!(result, true);
		assert_eq!(level, 642208);

		let tezos_url = String::from("https://rpc.tezrpc.me/");
		let block_hash = String::from("ab12");
		let tx_hash = String::from("cd34");
		let mut level = 0;
		let result = request_rpc2(tezos_url, block_hash, tx_hash, String::from(""), String::from(""), 0, &mut level).unwrap_or_else(|_| false);
		assert_eq!(result, false);
		assert_eq!(level <= 0, true);
	}
}