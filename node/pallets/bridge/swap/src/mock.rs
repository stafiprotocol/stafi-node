/// Copyright 2019-2020 Stafi Protocol.
// This file is part of Stafi.

// Stafi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stafi.  If not, see <http://www.gnu.org/licenses/>.

use sp_std::{cell::RefCell};
use sp_io::hashing::blake2_128;
use sp_runtime::{Perbill, traits::{BlakeTwo256, IdentityLookup}, testing::Header};
use sp_core::H256;
use frame_support::{impl_outer_origin, impl_outer_dispatch, parameter_types, traits::{Get}, weights::Weight};
use frame_system::{EnsureRoot};
use node_primitives::{ChainId, BlockNumber};
use crate::{Module, Trait};

pub(crate) type Balance = u128;

impl_outer_origin!{
	pub enum Origin for Test where system = frame_system {}
}

impl_outer_dispatch! {
	pub enum Call for Test where origin: Origin {
		frame_system::System,
		balances::Balances,
		rtoken_balances::RBalances,
		bridge_common::BridgeCommon,
		self::BridgeSwap,
	}
}

// For testing the pallet, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of pallets we want to use.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Test;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}

impl frame_system::Trait for Test {
	type BaseCallFilter = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type DbWeight = ();
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type MaximumExtrinsicWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type PalletInfo = ();
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}

thread_local! {
	static EXISTENTIAL_DEPOSIT: RefCell<Balance> = RefCell::new(0);
}

pub struct ExistentialDeposit;
impl Get<Balance> for ExistentialDeposit {
	fn get() -> Balance {
		EXISTENTIAL_DEPOSIT.with(|v| *v.borrow())
	}
}

impl pallet_balances::Trait for Test {
	type MaxLocks = ();
	type Balance = Balance;
	type DustRemoval = ();
	type Event = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

impl rtoken_balances::Trait for Test {
	type Event = ();
}

parameter_types! {
	pub const ChainIdentity: ChainId = 1;
	pub const ProposalLifetime: BlockNumber = 50;
}

impl bridge_common::Trait for Test {
	type Event = ();
	type AdminOrigin = EnsureRoot<Self::AccountId>;
	type ChainIdentity = ChainIdentity;
	type Proposal = Call;
	type ProposalLifetime = ProposalLifetime;
}

parameter_types! {
	pub NativeTokenId: bridge_common::ResourceId = bridge_common::derive_resource_id(1, &blake2_128(b"FIS"));
}

impl Trait for Test {
	type Currency = Balances;
	type RCurrency = RBalances;
	type NativeTokenId = NativeTokenId;
	type BridgeOrigin = bridge_common::EnsureBridge<Test>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	pallet_balances::GenesisConfig::<Test> {
				balances: vec![
					(1, 100),
				],
			}.assimilate_storage(&mut t).unwrap();

	t.into()
}

pub type System = frame_system::Module<Test>;
pub type Balances = pallet_balances::Module<Test>;
pub type RBalances = rtoken_balances::Module<Test>;
pub type BridgeCommon = bridge_common::Module<Test>;
pub type BridgeSwap = Module<Test>;

// Relayers
pub const RELAYER_A: u64 = 0x2;
pub const RELAYER_B: u64 = 0x3;
pub const RELAYER_C: u64 = 0x4;
pub const TEST_THRESHOLD: u32 = 2;
