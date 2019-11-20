#[cfg(feature = "std")]
use inherents::{ProvideInherentData};
use inherents::{InherentIdentifier, InherentData};
#[cfg(feature = "std")]
use primitives;
#[cfg(feature = "std")]
use hex;
#[cfg(feature = "std")]
use reqwest;
#[cfg(feature = "std")]
use serde_json::{Value};

pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"irisdata";
pub const RPC_REQUEST_INTERVAL: u64 = 60000; //1 minute

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
fn extract_value(result:String) -> String {
	if result == "null" {
		return result;
	}

	let len = result.len();
	let mut start = 5; //skip "0xff
	if len > 256 {
		start = 7; //skip "0xffff
	}
	let v_result = hex::decode(&result[start..len-1]).unwrap();
	let s_result = String::from_utf8(v_result).unwrap();
	
	s_result
} 

#[cfg(feature = "std")]
impl ProvideInherentData for InherentDataProvider {
	fn inherent_identifier(&self) -> &'static InherentIdentifier {
		&INHERENT_IDENTIFIER
	}

	fn provide_inherent_data(&self, inherent_data: &mut InherentData) -> Result<(), inherents::Error> {
		use std::time::SystemTime;
		let now = SystemTime::now();
		let d:u64 = now.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64;
		let slot_number = d / self.slot_duration;
		if slot_number % (RPC_REQUEST_INTERVAL/self.slot_duration) != 0 {
			return inherent_data.put_data(INHERENT_IDENTIFIER, &String::from("skip slot"));	
		}

		let endpoint_key = get_hexkey(b"TemplateModule IrisnetUrl");
		let result = get_value_from_storage(endpoint_key, self.host.clone());
		let endpoint = extract_value(result);
		if endpoint == "null" {
			return inherent_data.put_data(INHERENT_IDENTIFIER, &endpoint);
		}
        let param_key = get_hexkey(b"TemplateModule IrisnetParam");
		let result1 = get_value_from_storage(param_key, self.host.clone());
		let param = extract_value(result1);
		if param == "null" {
			return inherent_data.put_data(INHERENT_IDENTIFIER, &param);
		}
        //for test
        //let endpoint = String::from("tx/broadcast");
		let url = format!("{}{}", self.url, endpoint);
		let client = reqwest::Client::new();
		let mut resp = client.post(&url[..])
			.header("Content-Type", "application/json")
			.body("{}") //.body(param)
			.send().unwrap();
		let text = resp.text().unwrap();
		let v: Value = serde_json::from_str(&text).unwrap();
		let result:Vec<u8> = serde_json::to_vec(&v["check_tx"]["log"]).unwrap();
		inherent_data.put_data(INHERENT_IDENTIFIER, &result)
	}

	fn error_to_string(&self, error: &[u8]) -> Option<String> {
		Some(format!("{:?}", error))
	}
}