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

extern crate libsodium_sys as sodium;

use sodium::*;

pub fn verify(data: &[u8], sig: &[u8], pk: &[u8]) -> bool {
    let sig_ptr = sig.as_ptr();
    let data_ptr = data.as_ptr();
    let data_len = data.len();
    let pk_ptr = pk.as_ptr();
    let result;
    unsafe {
        result = crypto_sign_verify_detached(sig_ptr, data_ptr, data_len as u64, pk_ptr); 
    }
    return result == 0;
}