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
use super::sign;
use crypto::ed25519;
use sp_std::str;
use sp_std::vec::Vec;

pub fn verify_with_ed(data: &[u8], edsig: &[u8], edpk: &[u8], use_default_signer: bool) -> bool {
    let edsig_str = match str::from_utf8(edsig) {
        Ok(str) => str,
        Err(_) => return false,
    };
    let edsig_bytes = match base58::from_check(&edsig_str) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };
    let edpk_str = match str::from_utf8(edpk) {
        Ok(str) => str,
        Err(_) => return false,
    };
    let pk = match base58::from_check(&edpk_str) {
        Ok(pk) => pk,
        Err(_) => return false,
    };
    let signer;
    if use_default_signer {
        signer = sign::Sign::default();
    } else {
        signer = sign::Sign {
            watermark: Vec::new(),
        };
    }
    let (message, _) = signer.preprocess(data.to_vec());
    verify(&message, &edsig_bytes[5..], &pk[4..])
}

pub fn verify(data: &[u8], sig: &[u8], pk: &[u8]) -> bool {
    ed25519::verify(data, pk, sig)
}
