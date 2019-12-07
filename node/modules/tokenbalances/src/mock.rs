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

//! Test utilities

#![cfg(test)]
extern crate sr_io as runtime_io;

use crate::{
	Module, Trait, SymbolString, DescString
};
use sr_primitives::Perbill;
use sr_primitives::testing::{Header};
use sr_primitives::traits::{IdentityLookup, BlakeTwo256};
use primitives::H256;
use support::{impl_outer_origin, parameter_types};
use system;

impl_outer_origin!{
	pub enum Origin for Runtime {}
}

pub type AccountId = u64;
pub type Balance = u64;


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Runtime;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: u32 = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
	pub const MinimumPeriod: u64 = 1;
}

impl system::Trait for Runtime {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Call = ();
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
}

parameter_types! {
	pub const TransferFee: Balance = 0;
	pub const CreationFee: Balance = 0;
	pub const ExistentialDeposit: Balance = 0;
}
impl balances::Trait for Runtime {
	type Balance = Balance;
	type OnFreeBalanceZero = ();
	type OnNewAccount = ();
	type Event = ();
	type TransferPayment = ();
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type TransferFee = TransferFee;
	type CreationFee = CreationFee;
}

impl Trait for Runtime {
	const STAFI_SYMBOL: SymbolString = b"FIS";
    const STAFI_TOKEN_DESC: DescString = b"STAFI";
	type Event = ();
	type TokenBalance = u128;
}

pub fn new_test_ext() -> runtime_io::TestExternalities {
    let t = system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

    t.into()
}

pub type TokenBalances = Module<Runtime>;

