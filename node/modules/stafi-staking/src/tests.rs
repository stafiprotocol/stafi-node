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

//! Tests for the module.

#![cfg(test)]

use crate::mock::*;
use support::{assert_ok};

#[test]
fn test_custom_stake() {
	let mut ext = new_test_ext();

	ext.execute_with(|| {
		let account_id = 10;

		let multi_sig_address: Vec<u8> = Vec::new();
		let stake_amount: u128 = 11000000;
		let tx_hash: Vec<u8> = Vec::new();
		let block_hash: Vec<u8> = Vec::new();
		let pub_key: Vec<u8> = "edpktxQpBU6FcfwXzCaZHBmyk4vr91EVi7CghSw5SrE2tWoUoZZRUX".as_bytes().to_vec();
		let sig: Vec<u8> = Vec::new();

		let result = XtzStaking::custom_stake(
            Origin::signed(account_id),
            multi_sig_address,
            stake_amount,
            tx_hash,
			block_hash,
			pub_key,
			sig
        );

		assert_ok!(result);

		if let Some(stake_records) = XtzStaking::stake_records(XtzStaking::stake_of_owner_by_index((account_id, 0))) {
			assert_eq!(true, stake_records.initiator == account_id);
		}
		
	});
}
