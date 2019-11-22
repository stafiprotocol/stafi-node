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
extern crate crypto;
extern crate sr_std;

use super::base58;
use bip39::{Language, Mnemonic, MnemonicType, Seed};
use crypto::ed25519;
use crypto::{blake2b, digest::*};
use sr_std::prelude::*;

pub struct KeyPair {
    pub mnemonic: Vec<u8>,
    pub sk: Vec<u8>,
    pub pk: Vec<u8>,
    pub pkh: Vec<u8>,
}

pub fn generate_keypair() -> KeyPair {
    generate_keypair_with_password("mWcziEO9fE8kzGsV")
}

pub fn generate_keypair_with_password(password: &str) -> KeyPair {
    // create a new randomly generated mnemonic phrase
    let mnemonic = Mnemonic::new(MnemonicType::Words15, Language::English);
    generate_keypair_from_mnemonic(&mnemonic, password)
}

pub fn generate_keypair_from_mnemonic_str(mnemonic_str: &str, password: &str) -> KeyPair {
    let mnemonic = Mnemonic::from_phrase(mnemonic_str, Language::English)
        .map_err(|_| "Unexpected mnemonic")
        .unwrap();
    generate_keypair_from_mnemonic(&mnemonic, password)
}

pub fn generate_keypair_from_mnemonic(mnemonic: &Mnemonic, password: &str) -> KeyPair {
    let seed = Seed::new(&mnemonic, password);

    // get the HD wallet seed as raw bytes
    let seed_bytes: &[u8] = &seed.as_bytes();

    let keypair = generate_keypair_from_seed(seed_bytes);

    KeyPair {
        mnemonic: mnemonic.phrase().to_string().as_bytes().to_vec(),
        sk: keypair.0.as_bytes().to_vec(),
        pk: keypair.1.as_bytes().to_vec(),
        pkh: keypair.2.as_bytes().to_vec(),
    }
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
    let pkh_string = pkh_from_rawpk(raw_pk);
    (sk_string, pk_string, pkh_string)
}

pub fn pkh_from_rawpk(raw_pk: &[u8]) -> String {
    let mut pkh = vec![6, 161, 159]; // "tz1"
    let message_len = 20;
    let tmp_data = raw_pk.clone();
    let mut hasher = blake2b::Blake2b::new(message_len);
    hasher.input(&tmp_data);
    let mut hash = [0; 20];
    hasher.result(&mut hash);
    pkh.extend(&hash);
    let pkh_string = base58::check_encode_slice(&pkh);
    pkh_string
}

pub fn generate_keypair_from_seed(seed: &[u8]) -> (String, String, String) {
    let (sk, pk) = ed25519::keypair(&seed[..32]);

    keypair_from_raw_keypair(&sk, &pk)
}
