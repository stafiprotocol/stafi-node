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
extern crate substrate_primitives as primitives;
extern crate sr_io as runtime_io;

use crate::tezosworker::{Module, Trait, GenesisConfig};
use sr_primitives::Perbill;
use sr_primitives::testing::{Header, UintAuthorityId, TestXt};
use sr_primitives::traits::{IdentityLookup, BlakeTwo256};
use primitives::H256;
use support::{impl_outer_origin, impl_outer_dispatch, parameter_types};
use system;

use node_primitives::{Moment};
use node_primitives::constants::time::*;

impl_outer_origin!{
	pub enum Origin for Runtime {}
}

impl_outer_dispatch! {
	pub enum Call for Runtime where origin: Origin {
		tezosworker::TezosWorker,
	}
}

type DummyValidatorId = u64;
type DummyAccountId = u64;

/// An extrinsic type used for tests.
pub type Extrinsic = TestXt<Call, ()>;
type SubmitTransaction = system::offchain::TransactionSubmitter<(), Call, Extrinsic>;

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
	type Call = Call;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = babe::AuthorityId;
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
	pub const EpochDuration: u64 = EPOCH_DURATION_IN_SLOTS;
	pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
}

impl babe::Trait for Runtime {
	type EpochDuration = EpochDuration;
	type ExpectedBlockTime = ExpectedBlockTime;
	type EpochChangeTrigger = babe::ExternalTrigger;
}

impl timestamp::Trait for Runtime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
}

impl stafi_staking_storage::Trait for Runtime {

}

impl Trait for Runtime {
	type Event = ();
	type Call = Call;
	type SubmitTransaction = SubmitTransaction;
}

pub fn new_test_ext(authorities: Vec<DummyValidatorId>, auth_account: Vec<DummyAccountId>) -> runtime_io::TestExternalities {
    let mut t = system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
	let p: babe::AuthorityId = UintAuthorityId(1 as u64).to_public_key();

	babe::GenesisConfig {
        authorities: authorities.into_iter().map(|a| (UintAuthorityId(a).to_public_key(), 1)).collect(),
    }.assimilate_storage::<Runtime>(&mut t).unwrap();

	GenesisConfig::<Runtime> {
		auth_account: auth_account.into_iter().map(|a| UintAuthorityId(a).to_public_key()).collect(),
	}.assimilate_storage(&mut t).unwrap();

    t.into()
}

/// TezosWorker module.
pub type TezosWorker = Module<Runtime>;
pub type System = system::Module<Runtime>;
pub type Timestamp = timestamp::Module<Runtime>;
pub type Babe = babe::Module<Runtime>;

