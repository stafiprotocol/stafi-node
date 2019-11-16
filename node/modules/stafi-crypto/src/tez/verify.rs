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
extern crate crypto;

use bitcoin::util::base58;
use core::str;

use crypto::{ed25519};

pub fn verify_with_ed(data: &[u8], edsig: &[u8], edpk: &[u8]) -> bool {
    let edsig_str = str::from_utf8(edsig).unwrap();
    let edsig_bytes = base58::from_check(&edsig_str).unwrap();
    let edpk_str = str::from_utf8(edpk).unwrap();
    let pk = base58::from_check(&edpk_str).unwrap();
    verify(data, &edsig_bytes[5..], &pk[4..])
}

pub fn verify(data: &[u8], sig: &[u8], pk: &[u8]) -> bool {
     ed25519::verify(data, pk, sig)
}
