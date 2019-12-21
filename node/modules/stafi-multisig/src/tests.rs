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
#![cfg(test)]
extern crate substrate_primitives as primitives;

#[macro_use]
use hex_literal::hex;
use primitives::{crypto::UncheckedInto, sr25519::Public};
use primitives::offchain::{
	OffchainExt,
	testing::{TestOffchainExt, TestTransactionPoolExt},
	TransactionPoolExt,
};

use node_primitives::AccountId;

use crate::mock::*;

use super::*;

#[test]
fn test_owner() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        let deployer: AccountId =
            hex!["d43b38b84b60b06e7f1a00d892dcff67ea69dc1dc2f837fdb6a27344b63c9279"]
                .into();
        let account_a: AccountId =
            hex!["e489771ea3c4f10cb28698d21d5382ce3c5a673f47bade2b7325718701ad4b0c"]
                .into();


        let account_b: AccountId =
            hex!["7e35cbeea9f986613567088dbdb56da124f6511e339a44fd53a127b7653cff34"]
                .into();

        let account_c: AccountId =
            hex!["63410a24555c6f0c5ba8f0d27f85740dca150d9a0d67e3fa8502d5d9e6a4fafe"]
                .into();

        let origin = Origin::signed(deployer.clone());
        Balances::make_free_balance_be(&deployer, 500);
        for account in &[account_a.clone(), account_b.clone()] {
            Balances::make_free_balance_be(&account, 0);
        }
        let owners: Vec<_> = [account_a.clone(), account_b.clone()].iter().map(|i| (i.clone(), true)).collect();
        let result = MultiSigMock::deploy(origin, owners, 1, 10);
        let multisig_addr = MultiSigMock::multi_sig_list_item_for((deployer.clone(), 0)).unwrap();

        let result = MultiSigMock::is_owner_for(Origin::signed(deployer.clone()), multisig_addr.clone());
        assert_ok!(result);

        let result = MultiSigMock::is_owner_for(Origin::signed(account_c.clone()), multisig_addr.clone());
        assert_eq!(result.is_err(), true);
    });
}

#[test]
fn test_not_owner() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        let deployer: AccountId =
            hex!["d43b38b84b60b06e7f1a00d892dcff67ea69dc1dc2f837fdb6a27344b63c9279"]
                .into();
        let account_a: AccountId =
            hex!["e489771ea3c4f10cb28698d21d5382ce3c5a673f47bade2b7325718701ad4b0c"]
                .into();


        let account_b: AccountId =
            hex!["7e35cbeea9f986613567088dbdb56da124f6511e339a44fd53a127b7653cff34"]
                .into();

        let account_c: AccountId =
            hex!["63410a24555c6f0c5ba8f0d27f85740dca150d9a0d67e3fa8502d5d9e6a4fafe"]
                .into();

        let origin = Origin::signed(deployer.clone());
        Balances::make_free_balance_be(&deployer, 500);
        for account in &[account_a.clone(), account_b.clone()] {
            Balances::make_free_balance_be(&account, 0);
        }
        let owners: Vec<_> = [account_a.clone(), account_b.clone()].iter().map(|i| (i.clone(), true)).collect();
        let result = MultiSigMock::deploy(origin, owners, 1, 10);
        let multisig_addr = MultiSigMock::multi_sig_list_item_for((deployer.clone(), 0)).unwrap();

        let result = MultiSigMock::is_owner_for(Origin::signed(account_c.clone()), multisig_addr.clone());
        assert_eq!(result.is_err(), true);
    });
}

#[test]
fn test_create_multisig() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        let deployer: AccountId =
            hex!["d43b38b84b60b06e7f1a00d892dcff67ea69dc1dc2f837fdb6a27344b63c9279"]
                .into();
        let account_a: AccountId =
            hex!["e489771ea3c4f10cb28698d21d5382ce3c5a673f47bade2b7325718701ad4b0c"]
                .into();


        let account_b: AccountId =
            hex!["7e35cbeea9f986613567088dbdb56da124f6511e339a44fd53a127b7653cff34"]
                .into();

        let account_c: AccountId =
            hex!["63410a24555c6f0c5ba8f0d27f85740dca150d9a0d67e3fa8502d5d9e6a4fafe"]
                .into();

        let origin = Origin::signed(deployer.clone());
        Balances::make_free_balance_be(&deployer, 500);
        for account in &[account_a.clone(), account_b.clone(), account_c.clone()] {
            Balances::make_free_balance_be(&account, 0);
        }
        let owners: Vec<_> = [account_a.clone(), account_b.clone(), account_c.clone()].iter().map(|i| (i.clone(), true)).collect();
        let result = MultiSigMock::deploy(origin, owners, 2, 10);
        // Test Deploy
		assert_ok!(result);
		assert_eq!(490, Balances::total_balance(&deployer));
        let multisig_addr = MultiSigMock::multi_sig_list_item_for((deployer.clone(), 0)).unwrap();
        assert_eq!(10, Balances::total_balance(&multisig_addr));

        // Test Transfer
        assert_eq!(0, Balances::total_balance(&account_c));
        let transfer = MultiSigMock::transfer(Origin::signed(deployer.clone()), multisig_addr.clone(), TransactionType::TransferStafi, account_c.clone(), 1);
        assert_ok!(transfer);
        let transefer_id = MultiSigMock::pending_list_item_for((multisig_addr.clone(), 0)).unwrap();
        let confirm = MultiSigMock::confirm(Origin::signed(account_a.clone()), multisig_addr.clone(), transefer_id);
        assert_ok!(confirm);
        assert_eq!(9, Balances::total_balance(&multisig_addr));
        assert_eq!(1, Balances::total_balance(&account_c));
    });
}

#[test]
fn test_cancel_multisig() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        let deployer: AccountId =
            hex!["d43b38b84b60b06e7f1a00d892dcff67ea69dc1dc2f837fdb6a27344b63c9279"]
                .into();
        let account_a: AccountId =
            hex!["e489771ea3c4f10cb28698d21d5382ce3c5a673f47bade2b7325718701ad4b0c"]
                .into();


        let account_b: AccountId =
            hex!["7e35cbeea9f986613567088dbdb56da124f6511e339a44fd53a127b7653cff34"]
                .into();

        let account_c: AccountId =
            hex!["63410a24555c6f0c5ba8f0d27f85740dca150d9a0d67e3fa8502d5d9e6a4fafe"]
                .into();

        let origin = Origin::signed(deployer.clone());
        Balances::make_free_balance_be(&deployer, 500);
        for account in &[account_a.clone(), account_b.clone(), account_c.clone()] {
            Balances::make_free_balance_be(&account, 0);
        }
        let owners: Vec<_> = [account_a.clone(), account_b.clone(), account_c.clone()].iter().map(|i| (i.clone(), true)).collect();
        let result = MultiSigMock::deploy(origin, owners, 2, 10);
        let multisig_addr = MultiSigMock::multi_sig_list_item_for((deployer.clone(), 0)).unwrap();

        let transfer = MultiSigMock::transfer(Origin::signed(deployer.clone()), multisig_addr.clone(), TransactionType::TransferStafi, account_c.clone(), 1);
        let transefer_id = MultiSigMock::pending_list_item_for((multisig_addr.clone(), 0)).unwrap();
        // Cancel
        let cancel = MultiSigMock::remove_multi_sig_for(Origin::signed(account_c.clone()), multisig_addr.clone(), transefer_id.clone());
        assert_ok!(cancel);

        let confirm = MultiSigMock::confirm(Origin::signed(account_a.clone()), multisig_addr.clone(), transefer_id);
        assert_eq!(confirm.is_err(), true);
    });
}
