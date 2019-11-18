extern crate sr_io as runtime_io;
extern crate sr_std as rstd;
extern crate substrate_primitives as primitives;

use primitives::offchain::{Duration, HttpRequestId, HttpRequestStatus};
use rstd::result::Result;
use rstd::vec::Vec;
use sr_primitives::traits::{Member, SaturatedConversion};
use stafi_primitives::rjson;
use stafi_primitives::rjson::{JsonValue, JsonArray, JsonObject};

pub const BUFFER_LEN: usize = 40960;
pub const BUF_LEN: usize = 2048;

/// only for debug
fn debug(msg: &str) {
    runtime_io::print_utf8(msg.as_bytes());
}

enum RequestError {
    AddHeaderFailed,
    BadRequest,
    IoError,
    Invalid,
    Failed,
    Deadline,
    InvalidBlockId,
    ReadBodyError,
}

pub fn request_tezos() -> u32 {
    let res = request_tezos_buf();
    match res {
        Ok(buf) => {
            return 1;//parse_result(buf, "onv7i9LSacMXjhTdpgzmY4q6PxiZ18TZPq7KrRBRUVX7XJicSDi");
        }
        Err(err) => {
            return 0;
        }
    }
}

fn request_tezos_buf() -> Result<[u8; BUFFER_LEN], RequestError> {
    let uri = "https://rpc.tezrpc.me/chains/main/blocks/BKsxzJMXPxxJWRZcsgWG8AAegXNp2uUuUmMr8gzQcoEiGnNeCA";

    let mut counter = 0;

    loop {
        let res = http_request_get(uri, None);
        match res {
            Ok(buf) => {
                runtime_io::print_utf8(b"request_tezos_value return");
                return Ok(buf); //parse_result(buf, "onv7i9LSacMXjhTdpgzmY4q6PxiZ18TZPq7KrRBRUVX7XJicSDi");
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
                //return 0;
            }
        }
    }

}

fn http_request_get(
    uri: &str,
    header: Option<(&str, &str)>,
) -> Result<[u8; BUFFER_LEN], RequestError> {
    // TODO: extract id, maybe use for other place
    let id: HttpRequestId = runtime_io::http_request_start("GET", uri, &[0]).unwrap();
    let deadline = runtime_io::timestamp().add(Duration::from_millis(60_000));

    if let Some((name, value)) = header {
        match runtime_io::http_request_add_header(id, name, value) {
            Ok(_) => (),
            Err(_) => return Err(RequestError::AddHeaderFailed),
        };
    }

    match runtime_io::http_response_wait(&[id], Some(deadline))[0] {
        HttpRequestStatus::Finished(200) => (),
        HttpRequestStatus::Invalid => return Err(RequestError::Invalid),
        HttpRequestStatus::DeadlineReached => return Err(RequestError::Deadline),
        HttpRequestStatus::IoError => return Err(RequestError::IoError),
        HttpRequestStatus::Finished(400) => { return Err(RequestError::InvalidBlockId); }
        HttpRequestStatus::Finished(num) => {
            runtime_io::print_num(num as u64);
            return Err(RequestError::BadRequest);
        }
        _ => {
            debug("request failed");
            return Err(RequestError::Failed);
        }
    }

    let mut res: [u8; BUFFER_LEN] = [0; BUFFER_LEN];
    let mut offset : usize = 0;

    loop {
        // set a fix len for result
        let mut buf = Vec::with_capacity(BUF_LEN as usize);
        buf.resize(BUF_LEN as usize, 0);

        let len = runtime_io::http_response_read_body(id, &mut buf, Some(deadline));
        match len {
            Ok(len) => {
                if len > 0 {
                    res[offset..offset + len].copy_from_slice(&buf[..len]);
                    offset = offset + len;
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

fn parse_result(res: [u8; BUFFER_LEN], txid: &str) -> u32 {
    if let Ok(data) = core::str::from_utf8(&res) {
        runtime_io::print_utf8(&res);
        let data_array: Vec<char> = data.chars().collect();
        let mut index:usize = 0;
        let o = rjson::parse::<JsonValue, JsonArray, JsonObject, JsonValue>(&*data_array, &mut index).unwrap_or(JsonValue::Null);
        runtime_io::print_num(index as u64);
        //Self::parse_json(&o);
        let v = rjson::get_value_by_keys(&o, "header.level");
        if let Some(v) = v {
            if rjson::is_number(v) {
                let n = rjson::get_number(&v);
                if let Some(n) = n {
                    debug("found level node");
                    runtime_io::print_num(n as u64);

                    let v = rjson::get_value_by_key_recursively(&o, "operations");
                    if let Some(ops) = v {
                        if rjson::is_array(ops) {
                            debug("found operations");
                            let op_array = rjson::get_array(&ops);
                            runtime_io::print_num(op_array.len() as u64);
                            let mut found = false;
                            for op_arr in op_array {
                                if rjson::is_array(op_arr) {
                                    let arr = rjson::get_array(&op_arr);
                                    for node in arr {
                                        found = rjson::find_object_by_key_and_value(&node, "hash", txid);
                                        if found {
                                            debug("found tx");

                                            let contents = rjson::get_value_by_key(&node, "contents");
                                            if let Some(contents) = contents {
                                                if rjson::is_array(contents) {
                                                    debug("found contents");
                                                    let content_array = rjson::get_array(contents);
                                                    for content in content_array {
                                                        let kind = rjson::get_value_by_key(content, "kind");
                                                        if let Some(kind) = kind {
                                                            debug("found kind");
                                                            debug(rjson::get_string(&kind).unwrap());
                                                        }

                                                        let status = rjson::get_value_by_keys(content, "metadata.operation_result.status");
                                                        if let Some(status) = status {
                                                            debug("found status");
                                                            debug(rjson::get_string(&status).unwrap());
                                                        }

                                                    }
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
                        }
                    }

                    return n as u32;
                }
            }
        } else {
            debug("not found");
        }

        return 0;

    } else {
        return 0;
    }
}