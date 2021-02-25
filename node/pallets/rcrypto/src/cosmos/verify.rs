// Copyright 2018 Stafi Protocol, Inc.
// This file is part of Stafi.

// Stafi is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Stafi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stafi.  If not, see <http://www.gnu.org/licenses/>.

extern crate bech32_no_std as bech32;
extern crate crypto;

use bech32::{FromBase32};
use crypto::{sha2::Sha256, digest::Digest};
use sp_std::str;
use sp_std::vec::Vec;
use sp_std::{
	convert::{TryInto},
};

pub fn verify_with_prefix(data: &[u8], sig_data: &[u8], pk: &[u8]) -> bool {
    let data_str = match str::from_utf8(data) {
        Ok(str) => str,
        Err(_) => return false,
    };
    let mut hasher = Sha256::new();
    hasher.input_str(data_str);
    let mut data_out = [0; 32];
    hasher.result(&mut data_out);

    // let sig_str = match str::from_utf8(sig_data) {
    //     Ok(str) => str,
    //     Err(_) => return false,
    // };
    // let sig_bytes = base64::decode(&sig_str).unwrap();
    // let sigature: &[u8] = &sig_bytes;

    let sigature: &[u8] = sig_data;

    let pk_str = match str::from_utf8(pk) {
        Ok(str) => str,
        Err(_) => return false,
    };
    let (_hrp, pk_data) = bech32::decode(&pk_str).unwrap();
    let prefixed_pubkey = Vec::<u8>::from_base32(&pk_data).unwrap();
    let pubkey = &prefixed_pubkey[5..];

    return verify(&data_out, &sigature, &pubkey);
}

pub fn verify(data: &[u8; 32], sig: &[u8], pk: &[u8]) -> bool {
    secp256k1::verify(&secp256k1::Message::parse(&data), 
        &secp256k1::Signature::parse(sig.try_into().unwrap()),
        &secp256k1::PublicKey::parse_compressed(pk.try_into().unwrap()).unwrap()
    )
}


#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1;

    #[test]
    fn test_cosmos_bech32_decode() {
        let pk_str = "cosmospub1addwnpepqgjng44c0h235k4deqg9ugllwnw9u5mq0usx5qkdcx6dnd4lzygpcrhupw2";
        let (hrp, data) = bech32::decode(&pk_str).unwrap();
        let prefixed_pubkey = Vec::<u8>::from_base32(&data).unwrap();
        let pubkey = &prefixed_pubkey[5..];
        assert_eq!(hrp, "cosmospub");
        assert_eq!(prefixed_pubkey, vec![235, 90, 233, 135, 33, 2, 37, 52, 86, 184, 125, 213, 26, 90, 173, 200, 16, 94, 35, 255, 116, 220, 94, 83, 96, 127, 32, 106, 2, 205, 193, 180, 217, 182, 191, 17, 16, 28]);
        assert_eq!(pubkey, vec![2, 37, 52, 86, 184, 125, 213, 26, 90, 173, 200, 16, 94, 35, 255, 116, 220, 94, 83, 96, 127, 32, 106, 2, 205, 193, 180, 217, 182, 191, 17, 16, 28]);
    }

    #[test]
    fn test_cosmos_base64_decode() {
        // let sig_str = "g8shCqAzvmD0QwmWKzVGvA2GlRWhrRI2ZEBAynSzX/g8JHmjrBNzuLVmMj0j+q0awqP33XBQXxi3L6fVmN4ZrA==";
        // let sig_bytes = base64::decode(&sig_str).unwrap();
        // assert_eq!(sig_bytes, vec![131, 203, 33, 10, 160, 51, 190, 96, 244, 67, 9, 150, 43, 53, 70, 188, 13, 134, 149, 21, 161, 173, 18, 54, 100, 64, 64, 202, 116, 179, 95, 248, 60, 36, 121, 163, 172, 19, 115, 184, 181, 102, 50, 61, 35, 250, 173, 26, 194, 163, 247, 221, 112, 80, 95, 24, 183, 47, 167, 213, 152, 222, 25, 172]);
    }

    #[test]
    fn test_cosmos_verify() {
        // let message = "test1";
        // let mut hasher = Sha256::new();
        // hasher.input_str(message);
        // let mut message_out = [0; 32];
        // hasher.result(&mut message_out);

        // let pk_str = "cosmospub1addwnpepqgjng44c0h235k4deqg9ugllwnw9u5mq0usx5qkdcx6dnd4lzygpcrhupw2";
        // let (hrp, data) = bech32::decode(&pk_str).unwrap();
        // let prefixed_pubkey = Vec::<u8>::from_base32(&data).unwrap();
        // let pubkey = &prefixed_pubkey[5..];

        // let sig_str = "g8shCqAzvmD0QwmWKzVGvA2GlRWhrRI2ZEBAynSzX/g8JHmjrBNzuLVmMj0j+q0awqP33XBQXxi3L6fVmN4ZrA==";
        // let sig_bytes = base64::decode(&sig_str).unwrap();

        // let sigature: &[u8] = &sig_bytes;

        // let verify_res = secp256k1::verify(&secp256k1::Message::parse(&message_out), 
        //     &secp256k1::Signature::parse(sigature.try_into().unwrap()),
        //     &secp256k1::PublicKey::parse_compressed(pubkey.try_into().unwrap()).unwrap()
        // );

        // assert_eq!(verify_res, true);
    }
}