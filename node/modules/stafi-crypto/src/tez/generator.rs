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

#[cfg(feature = "std")]
extern crate bip39;

#[cfg(feature = "std")]
extern crate bitcoin;

#[cfg(feature = "std")]
extern crate libsodium_sys as sodium;

#[cfg(feature = "std")]
use bip39::{Mnemonic, MnemonicType, Language, Seed};
#[cfg(feature = "std")]
extern crate rstd;
extern crate ed25519_dalek;
extern crate blake2_rfc;
extern crate crypto;

use crypto::{ed25519};

#[cfg(feature = "std")]
use bitcoin::util::base58;
use rstd::mem;
#[cfg(feature = "std")]
use sodium::*;

use rstd::vec::Vec;
use rstd::str;
use ed25519_dalek::Keypair;
use blake2_rfc::blake2b;

pub struct KeyPair {
    pub mnemonic: Vec<u8>,
    pub sk: Vec<u8>,
    pub pk: Vec<u8>,
    pub pkh: Vec<u8>
}

#[cfg(feature = "std")]
pub fn generate_keypair() -> KeyPair {
    generate_keypair_with_password("mWcziEO9fE8kzGsV")
}

#[cfg(feature = "std")]
pub fn generate_keypair_with_password(password: &str) -> KeyPair {
    // create a new randomly generated mnemonic phrase
    let mnemonic = Mnemonic::new(MnemonicType::Words15, Language::English);
    generate_keypair_from_mnemonic(&mnemonic, password)
}

#[cfg(feature = "std")]
pub fn generate_keypair_from_mnemonic_str(mnemonic_str: &str,  password: &str) -> KeyPair  {
    let mnemonic = Mnemonic::from_phrase(mnemonic_str,  Language::English).map_err(|_| "Unexpected mnemonic").unwrap();
    generate_keypair_from_mnemonic(&mnemonic, password)
}

#[cfg(feature = "std")]
pub fn generate_keypair_from_mnemonic(mnemonic: &Mnemonic,  password: &str) -> KeyPair  {
    let seed = Seed::new(&mnemonic, password);

    // get the HD wallet seed as raw bytes
    let seed_bytes: &[u8] = &seed.as_bytes();

    let keypair = generate_keypair_from_seed(seed_bytes);

    KeyPair { mnemonic: mnemonic.phrase().to_string().as_bytes().to_vec(), 
    sk: keypair.0.as_bytes().to_vec(),
    pk: keypair.1.as_bytes().to_vec(),
     pkh: keypair.2.as_bytes().to_vec() }
}

#[cfg(feature = "std")]
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
    let pkh_string = pkh_from_rawpk(raw_pk);
    (sk_string, pk_string, pkh_string)
}

#[cfg(feature = "std")]
pub fn pkh_from_rawpk(raw_pk: &[u8]) -> String {
    let mut pkh = vec![6, 161, 159]; // "tz1"
    let message_len = 20;
    let tmp_data = raw_pk.clone();
    let new_hash = blake2b::blake2b(message_len, &[], tmp_data);
    pkh.extend(new_hash.as_bytes());
    let pkh_string = base58::check_encode_slice(&pkh);
    pkh_string
}

#[cfg(feature = "std")]
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
    let mut pk: [u8;32] = [0; 32];
    pk.copy_from_slice(&pub_buffer);
    let mut sk: [u8;64] = [0; 64];
    sk.copy_from_slice(&private_buffer);
    let keypair = ed25519::keypair(seed);

    keypair_from_raw_keypair(&keypair.0, &keypair.1)
}
