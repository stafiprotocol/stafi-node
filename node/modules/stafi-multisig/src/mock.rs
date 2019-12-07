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
extern crate sr_io as runtime_io;
extern crate pallet_balances as balances;
extern crate sr_std as rstd;

use rstd::marker::PhantomData;
use crate::{Module, Trait, SimpleMultiSigIdFor, MultiSigFor};
use sr_primitives::Perbill;
use sr_primitives::testing::{Header, UintAuthorityId, TestXt};
use sr_primitives::traits::{IdentityLookup, BlakeTwo256};
use primitives::H256;
use node_primitives::AccountId;
use frame_support::{impl_outer_origin, impl_outer_dispatch, parameter_types};
use system;

use node_primitives::{Moment};
use node_primitives::constants::time::*;
use pallet_balances::NegativeImbalance;

impl_outer_origin!{
	pub enum Origin for Runtime {}
}

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
    type Call = ();
    type Index = u64;
    type BlockNumber = u64;
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
	pub const ExistentialDeposit: u64 = 0;
	pub const TransferFee: u64 = 0;
	pub const CreationFee: u64 = 0;
}

impl balances::Trait for Runtime {
    type Balance = u64;
    type OnFreeBalanceZero = ();
    type OnNewAccount = ();
    type TransferPayment = ();
    type DustRemoval = ();
    type Event = ();
    type ExistentialDeposit = ExistentialDeposit;
    type TransferFee = TransferFee;
    type CreationFee = CreationFee;
}

impl Trait for Runtime {
    type MultiSig = SimpleMultiSigIdFor<Self>;
    type Event = ();
}

pub fn new_test_ext() -> runtime_io::TestExternalities {
    let mut t = system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

    t.into()
}

pub type MultiSigMock = Module<Runtime>;
pub type Balances = balances::Module<Runtime>;