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
extern crate bip39;
extern crate bitcoin;
extern crate libsodium_sys as sodium;

use bip39::{Mnemonic, MnemonicType, Language, Seed};
use bitcoin::util::base58;
use std::mem;
use sodium::*;

pub struct KeyPair {
    pub mnemonic: String,
    pub sk: String,
    pub pk: String,
    pub pkh: String
}

pub fn generate_keypair() -> KeyPair {
    generate_keypair_with_password("mWcziEO9fE8kzGsV")
}

pub fn generate_keypair_with_password(password: &str) -> KeyPair {
    // create a new randomly generated mnemonic phrase
    let mnemonic = Mnemonic::new(MnemonicType::Words15, Language::English);
    generate_keypair_from_mnemonic(&mnemonic, password)
}

pub fn generate_keypair_from_mnemonic_str(mnemonic_str: &str,  password: &str) -> KeyPair  {
    let mnemonic = Mnemonic::from_phrase(mnemonic_str,  Language::English).map_err(|_| "Unexpected mnemonic").unwrap();
    generate_keypair_from_mnemonic(&mnemonic, password)
}

pub fn generate_keypair_from_mnemonic(mnemonic: &Mnemonic,  password: &str) -> KeyPair  {
    let seed = Seed::new(&mnemonic, password);

    // get the HD wallet seed as raw bytes
    let seed_bytes: &[u8] = &seed.as_bytes();

    let keypair = generate_keypair_from_seed(seed_bytes);

    KeyPair { mnemonic: mnemonic.phrase().to_string(), sk: keypair.0, pk: keypair.1 , pkh: keypair.2 }
}

fn keypair_from_raw_keypair(raw_sk: &[u8], raw_pk: &[u8]) -> (String, String, String) {
     // PubKey
    let mut pk = vec![13, 15, 37, 217]; // edpk
    pk.extend(raw_pk.clone().iter());
    let pk_string = base58::check_encode_slice(&pk);

    // PrivateKey
    let mut sk = vec![43, 246, 78, 7];
    sk.extend(raw_sk.clone().iter());
    let sk_string = base58::check_encode_slice(&sk);

    // pkh
    let mut pkh = vec![6, 161, 159]; // "tz1"
    let message_len = 20;
    let mut message: Vec<u8> = Vec::with_capacity(message_len);
    let tmp_data = raw_pk.clone();
    let tmp_data_ptr = tmp_data.as_ptr();
    let tmp_len: u64 = tmp_data.len() as u64;
    unsafe {
        let message_ptr = message.as_mut_ptr();
        mem::forget(message);
        crypto_generichash(
            message_ptr,
            message_len,
            tmp_data_ptr,
            tmp_len,
            vec![].as_ptr(),
            0,
        );
        message = Vec::from_raw_parts(message_ptr, message_len, message_len)
    }
    pkh.extend(message);
    let pkh_string = base58::check_encode_slice(&pkh);
    (sk_string, pk_string, pkh_string)
}

pub fn generate_keypair_from_seed(seed: &[u8]) -> (String, String, String) {
    let pub_len = 32;
    let private_len = 64;
    let mut pub_buffer:Vec<u8> = Vec::with_capacity(32);
    let mut private_buffer:Vec<u8> = Vec::with_capacity(64);
    unsafe {
        let pub_ptr = pub_buffer.as_mut_ptr();
        let private_ptr = private_buffer.as_mut_ptr();
        mem::forget(pub_buffer);
        mem::forget(private_buffer);

        crypto_sign_seed_keypair(pub_ptr, private_ptr, seed.as_ptr());
        pub_buffer = Vec::from_raw_parts(pub_ptr, pub_len, pub_len);
        private_buffer = Vec::from_raw_parts(private_ptr, private_len, private_len);
    }

    keypair_from_raw_keypair(&private_buffer, &pub_buffer)
}
