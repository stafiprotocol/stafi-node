extern crate sr_io as runtime_io;
extern crate sr_std as rstd;
extern crate substrate_primitives as primitives;

use primitives::offchain::{Duration, HttpRequestId, HttpRequestStatus, StorageKind};
use rstd::result::Result;
use rstd::vec::Vec;
use node_primitives::{rjson, VerifyStatus};
use node_primitives::rjson::{JsonValue, JsonArray, JsonObject};
use log::info;

pub const BUFFER_LEN: usize = 40960;
pub const BUF_LEN: usize = 2048;

/// only for debug
fn debug(msg: &str) {
    info!("{}", msg);
}

enum RequestError {
    AddHeaderFailed,
    BadRequest,
    IoError,
    Invalid,
    Deadline,
    InvalidBlockId,
    ReadBodyError,
}

pub fn request_tezos(host: Vec<u8>, blockhash: Vec<u8>, txhash: Vec<u8>, from: Vec<u8>, to: Vec<u8>, stake_amount: u128, level: &mut i64) -> VerifyStatus {

    let uri = [core::str::from_utf8(&host).unwrap(), "/chains/main/blocks/", core::str::from_utf8(&blockhash).unwrap()].join("");
    debug(&uri);

    let res = request_tezos_buf(&uri);
    match res {
        Ok(buf) => {
            let ret = parse_result(buf, core::str::from_utf8(&blockhash).unwrap(), core::str::from_utf8(&txhash).unwrap(),core::str::from_utf8(&from).unwrap(), core::str::from_utf8(&to).unwrap(), stake_amount, level);
            return ret;
        }
        Err(_err) => {
            return VerifyStatus::NotResponse;
        }
    }
}

fn request_tezos_buf(uri: &str) -> Result<[u8; BUFFER_LEN], RequestError> {
    let mut counter = 0;

    loop {
        let res = http_request_get(uri, None);
        match res {
            Ok(buf) => {
                runtime_io::misc::print_utf8(b"request_tezos_value return");
                return Ok(buf);
            }
            Err(err) => {
                debug("request_tezos_value error");
                match err {
                    RequestError::IoError => {
                        debug("request_tezos_value io error");
                        counter += 1;
                        if counter > 3 {
                            return Err(RequestError::IoError);
                        }
                    }
                    RequestError::InvalidBlockId => {
                        debug("request_tezos_value invalid block id");
                        return Err(RequestError::InvalidBlockId);
                    }
                    _ => return Err(RequestError::BadRequest),
                }
            }
        }
    }

}

fn http_request_get(
    uri: &str,
    header: Option<(&str, &str)>,
) -> Result<[u8; BUFFER_LEN], RequestError> {
    let id: HttpRequestId = runtime_io::offchain::http_request_start("GET", uri, &[0]).unwrap();
    let deadline = runtime_io::offchain::timestamp().add(Duration::from_millis(90_000));

    if let Some((name, value)) = header {
        match runtime_io::offchain::http_request_add_header(id, name, value) {
            Ok(_) => (),
            Err(_) => return Err(RequestError::AddHeaderFailed),
        };
    }

    match runtime_io::offchain::http_response_wait(&[id], Some(deadline))[0] {
        HttpRequestStatus::Finished(200) => (),
        HttpRequestStatus::Invalid => return Err(RequestError::Invalid),
        HttpRequestStatus::DeadlineReached => return Err(RequestError::Deadline),
        HttpRequestStatus::IoError => return Err(RequestError::IoError),
        HttpRequestStatus::Finished(400) => { return Err(RequestError::InvalidBlockId); }
        HttpRequestStatus::Finished(num) => {
            runtime_io::misc::print_num(num as u64);
            return Err(RequestError::BadRequest);
        }
    }

    let mut res: [u8; BUFFER_LEN] = [0; BUFFER_LEN];
    let mut offset : usize = 0;

    loop {
        let mut buf = Vec::with_capacity(BUF_LEN as usize);
        buf.resize(BUF_LEN as usize, 0);

        let http_res = runtime_io::offchain::http_response_read_body(id, &mut buf, Some(deadline));
        match http_res {
            Ok(len) => {
                if len > 0 {
                    res[offset..offset + len as usize].copy_from_slice(&buf[..len as usize]);
                    offset = offset + len as usize;
                }
                if len == 0 {
                    return Ok(res);
                }
            }
            Err(_) => {
                //debug("read body error");
                return Err(RequestError::ReadBodyError);
            }
        }
    }
}

fn parse_result(res: [u8; BUFFER_LEN], blockhash: &str, txid: &str, from: &str, to: &str, stake_amount: u128, level: &mut i64) -> VerifyStatus {
    let data = core::str::from_utf8(&res).unwrap_or("");
    if data == "" {
        return VerifyStatus::Error;
    }

    //runtime_io::print_utf8(&res);
    let data_array: Vec<char> = data.chars().collect();
    let mut index:usize = 0;
    let o = rjson::parse::<JsonValue, JsonArray, JsonObject, JsonValue>(&*data_array, &mut index).unwrap_or(JsonValue::None);
    if rjson::is_none(&o) {
        return VerifyStatus::Error;
    }

    //runtime_io::print_num(index as u64);
    //Self::parse_json(&o);
    let v = rjson::get_value_by_keys(&o, "header.level").unwrap_or(&JsonValue::None);
    if rjson::is_none(&v) || !rjson::is_number(v){
        return VerifyStatus::NotFoundBlock;
    }

    let n = rjson::get_number(&v).unwrap_or(0.0);
    if n == 0.0 {
        return VerifyStatus::NotFoundBlock;
    }

    debug("found level node");
    //runtime_io::print_num(n as u64);
    *level = n as i64;

    if blockhash == "head" {
        return VerifyStatus::TxOk;
    }

    let ops = rjson::get_value_by_key_recursively(&o, "operations").unwrap_or(&JsonValue::None);
    if rjson::is_none(&ops) || !rjson::is_array(ops){
        return VerifyStatus::NotFoundTx;
    }

    debug("found operations");
    let op_array = rjson::get_array(&ops);
    //runtime_io::print_num(op_array.len() as u64);
    let mut found = false;
    for op_arr in op_array {
        if rjson::is_array(op_arr) {
            let arr = rjson::get_array(&op_arr);
            for node in arr {
                found = rjson::find_object_by_key_and_value(&node, "hash", txid);
                if found {
                    debug("found tx");

                    let contents = rjson::get_value_by_key(&node, "contents").unwrap_or(&JsonValue::None);
                    if rjson::is_none(&contents) || !rjson::is_array(contents){
                        break;
                    }

                    debug("found contents");
                    let content_array = rjson::get_array(contents);
                    for content in content_array {
                        let kind = rjson::get_value_by_key(content, "kind").unwrap_or(&JsonValue::None);
                        if rjson::is_none(&kind) || !rjson::is_string(kind) || rjson::get_string(&kind).unwrap_or("") != "transaction" {
                            debug("not transaction");
                            return VerifyStatus::TxNotMatch;
                        }

                        let source = rjson::get_value_by_key(content, "source").unwrap_or(&JsonValue::None);
                        if rjson::is_none(&source) || rjson::get_string(&source).unwrap_or("") != from {
                            debug("source not not match");
                            return VerifyStatus::TxNotMatch;
                        }

                        let destination = rjson::get_value_by_key(content, "destination").unwrap_or(&JsonValue::None);
                        if rjson::is_none(&destination) || rjson::get_string(&destination).unwrap_or("") != to {
                            debug("destination not match");
                            return VerifyStatus::TxNotMatch;
                        }

                        let amount = rjson::get_value_by_key(content, "amount").unwrap_or(&JsonValue::None);
                        if rjson::is_none(&amount) {
                            debug("amount not exist");
                            return VerifyStatus::TxNotMatch;
                        }

                        let amount = rjson::get_string(&amount).unwrap_or("").parse::<u128>().unwrap_or(0);
                        if amount != stake_amount {
                            debug("amount not match");
                            return VerifyStatus::TxNotMatch;
                        }

                        let status = rjson::get_value_by_keys(content, "metadata.operation_result.status").unwrap_or(&JsonValue::None);
                        if rjson::is_none(&status) || rjson::get_string(&status).unwrap_or("") != "applied" {
                            debug("not applied");
                            return VerifyStatus::TxNotMatch;
                        }
                    }

                    break;
                }
            }

        }
        if found {
            break;
        }
    }
    //found
    if found {
        return VerifyStatus::TxOk;
    }

    return VerifyStatus::NotFoundTx;
}

//local storage
pub fn set_value(key: &[u8], value: &[u8]) {
    runtime_io::offchain::local_storage_set(StorageKind::PERSISTENT, key, value);
}

pub fn get_value(key: &[u8]) -> Option<Vec<u8>> {
    runtime_io::offchain::local_storage_get(StorageKind::PERSISTENT, key)
}

pub fn vec8_to_u64(v: Vec<u8>) -> u64 {
    let mut a: [u8; 8] = [0; 8];
    for i in 0..8 {
        a[i] = v[i];
    }
    u64::from_be_bytes(a)
}