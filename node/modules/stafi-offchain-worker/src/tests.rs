// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Tests for the tezos worker module.

#![cfg(test)]
extern crate substrate_primitives as primitives;

use super::*;
use crate::mock::*;
use primitives::offchain::{
	OffchainExt,
	TransactionPoolExt,
	testing::{TestOffchainExt, TestTransactionPoolExt},
};

use node_primitives::VerifyStatus;
use tezosworker::tezos;

#[test]
fn test_offchain_local_storage() {
	let mut ext = new_test_ext(vec![0, 1, 2, 3]);
	let (offchain, _state) = TestOffchainExt::new();
	let (pool, _state) = TestTransactionPoolExt::new();
	ext.register_extension(OffchainExt::new(offchain));
	ext.register_extension(TransactionPoolExt::new(pool));

	ext.execute_with(|| {
		let key = "my_key".as_bytes().to_vec();
		let value = (15700 as u64).to_be_bytes().to_vec();
		tezos::set_value(&key, &value);

		assert_eq!(true, tezos::get_value(&key).is_some());
		let val = tezos::get_value(&key).unwrap();
		assert_eq!(true, tezos::vec8_to_u64(val) == 15700);

		let key = "your_key".as_bytes().to_vec();
		assert_eq!(true, tezos::get_value(&key).is_none());
	});
}

#[test]
#[ignore]
fn test_offchain_request_tezos() {
	let mut ext = new_test_ext(vec![0, 1, 2, 3]);
	let (offchain, _state) = TestOffchainExt::new();
	let (pool, _state) = TestTransactionPoolExt::new();
	ext.register_extension(OffchainExt::new(offchain));
	ext.register_extension(TransactionPoolExt::new(pool));

	ext.execute_with(|| {
		let host = "https://rpc.tezrpc.me".as_bytes().to_vec();
		let blockhash = "BKsxzJMXPxxJWRZcsgWG8AAegXNp2uUuUmMr8gzQcoEiGnNeCA6".as_bytes().to_vec();
		let txhash = "onv7i9LSacMXjhTdpgzmY4q6PxiZ18TZPq7KrRBRUVX7XJicSDi".as_bytes().to_vec();
		let from = "tz1SYq214SCBy9naR6cvycQsYcUGpBqQAE8d".as_bytes().to_vec();
		let to = "tz1S4MTpEV356QcuzkjQUdyZdAy36gPwPWXa".as_bytes().to_vec();
		let amount = 710391;

		let mut level = 0;
		let result = tezos::request_tezos(host, blockhash, txhash, from, to, amount, &mut level);

		assert_eq!(true, result == VerifyStatus::Confirmed);
		assert_eq!(true, level == 642208);
	});
}

