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
extern crate crypto;
extern crate sp_std;

use super::base58;
use crypto::{blake2b, digest::*, ed25519};

use sp_std::default;
use sp_std::prelude::*;
use sp_std::vec;

pub struct SignatureData {
    pub sig: Vec<u8>,
    pub edsig: Vec<u8>,
    pub sbytes: Vec<u8>,
}

pub struct Sign {
    pub watermark: Vec<u8>,
}
impl default::Default for Sign {
    fn default() -> Self {
        Sign {
            watermark: [3].to_vec(),
        }
    }
}

impl Sign {
    pub fn sign(&self, data: Vec<u8>, sk_str: &str) -> SignatureData {
        self.sign_with_sk(data, base58::from_check(sk_str).unwrap())
    }

    pub fn sign_safe(&self, data: Vec<u8>, sk_str: &str) -> Result<SignatureData, &str> {
        let sk = match base58::from_check(sk_str) {
            Ok(sk) => sk,
            Err(_) => return Err("Invalid sk"),
        };
        Ok(self.sign_with_sk(data, sk))
    }

    pub fn preprocess(&self, data: Vec<u8>) -> (Vec<u8>, usize) {
        let watermark_generics: Vec<u8> = self.watermark.clone();
        let mut tmp_data = vec![];
        tmp_data.extend(watermark_generics);
        tmp_data.extend(data.clone());

        // Get hash of data with generic
        let message_len = 32;

        let mut hasher = blake2b::Blake2b::new(message_len);
        hasher.input(&tmp_data);
        let mut out = [0; 32];
        hasher.result(&mut out);

        (out.to_vec(), message_len)
    }

    pub fn sign_with_sk(&self, data: Vec<u8>, sk: Vec<u8>) -> SignatureData {
        let (message, _) = self.preprocess(data.clone());

        // Sk to sign
        // The sk has prefix "edsk", which need to be removed
        let sk_data: Vec<u8> = sk[4..].to_vec();

        let sig_bytes = ed25519::signature(&message, &sk_data).to_vec();

        // EDSIG
        // The edsig has prefix "edsig" = vec![9, 245, 205, 134, 18]
        let edsig_prefix: Vec<u8> = vec![9, 245, 205, 134, 18];
        let mut edsig_data: Vec<u8> = vec![];
        edsig_data.extend(edsig_prefix);
        edsig_data.extend(sig_bytes.clone());

        // sbytes = data appends signature
        let mut sbytes: Vec<u8> = data.clone();
        sbytes.extend(sig_bytes.clone());

        SignatureData {
            sig: sig_bytes.clone(),
            edsig: base58::check_encode_slice(&edsig_data),
            sbytes: sbytes,
        }
    }
}
