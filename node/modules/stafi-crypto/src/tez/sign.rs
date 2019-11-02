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

extern crate bitcoin;
extern crate libsodium_sys as sodium;

use bitcoin::util::base58;
use sodium::*;
use core::mem;

pub struct SignatureData {
    pub sig: Vec<u8>,
    pub edsig: String,
    pub sbytes: Vec<u8>,
}

pub fn sign(data: Vec<u8>, sk_str: &str) -> SignatureData {
    sign_with_sk(data, base58::from_check(sk_str).unwrap())
}

pub fn sign_with_sk(data: Vec<u8>, sk: Vec<u8>) -> SignatureData {
    let watermark_generics: Vec<u8> = [3].to_vec();
    let mut tmp_data = vec![];
    tmp_data.extend(watermark_generics);
    tmp_data.extend(data.clone());

    // Get hash of data with generic
    let message_len = 32;
    let mut message: Vec<u8> = Vec::with_capacity(message_len);
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

    // Signature
    let sig_len = 64;
    let mut sig_bytes: Vec<u8> = Vec::with_capacity(sig_len);

    // Sk to sign
    // The sk has prefix "edsk", which need to be removed
    let sk_data: Vec<u8> = sk[4..].to_vec();
    unsafe {
        let sig_bytes_ptr = sig_bytes.as_mut_ptr();
        mem::forget(sig_bytes);

        let mut siglen: u64 = 0;
        let siglen_ptr: *mut u64 = &mut siglen;
        let message_ptr = message.as_ptr();
        let sk_ptr = sk_data.as_ptr();
        crypto_sign_detached(
            sig_bytes_ptr,
            siglen_ptr,
            message_ptr,
            message_len as u64,
            sk_ptr,
        );
        let siglen_usize: usize = siglen as usize;
        sig_bytes = Vec::from_raw_parts(sig_bytes_ptr, siglen_usize, siglen_usize);
    }

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
