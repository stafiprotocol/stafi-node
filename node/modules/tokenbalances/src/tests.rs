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

use super::*;
use crate::mock::*;
use support::{assert_ok};

use node_primitives::Symbol;

#[test]
fn test_register_token() {
	let mut ext = new_test_ext();

	ext.execute_with(|| {
		let token_desc: TokenDesc = Vec::new();
		let precision: Precision = 6;
		let symbol = Symbol::XTZ;
		let result = TokenBalances::register_token(
            Origin::signed(11),
            symbol,
            token_desc,
            precision
        );
		assert_ok!(result);

		assert_eq!(true, TokenBalances::token_info(symbol).precision == precision);
	});
}

#[test]
fn test_set_free_token() {
	let mut ext = new_test_ext();

	ext.execute_with(|| {
		let account_id = 10;
		let free_balance = 120;
		let symbol = Symbol::XTZ;
		let result = TokenBalances::set_free_token(
            Origin::signed(11),
            account_id,
            symbol,
            free_balance
        );
		assert_ok!(result);

		assert_eq!(true, TokenBalances::token_free_balance((account_id, symbol)) == free_balance);
	});
}

