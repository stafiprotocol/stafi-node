

/// Copyright 2019-2020 Stafi Protocol.
// This file is part of Stafi.

// Stafi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stafi.  If not, see <http://www.gnu.org/licenses/>.

use sp_runtime::{Perbill, traits::{BlakeTwo256, IdentityLookup}, testing::Header};
use sp_core::H256;
use frame_support::{assert_ok, impl_outer_origin, impl_outer_event, impl_outer_dispatch, parameter_types, weights::Weight};
use frame_system::{EnsureRoot};
use node_primitives::{ChainId, BlockNumber};
use crate as bridge_common;
use crate::{Module, Trait, ResourceId};

impl_outer_origin!{
	pub enum Origin for Test where system = frame_system {}
}
impl_outer_event!{
	pub enum TestEvent for Test {
		frame_system<T>,
		bridge_common<T>,
	}
}

impl_outer_dispatch! {
	pub enum Call for Test where origin: Origin {
		frame_system::System,
		self::BridgeCommon,
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
	type Event = TestEvent;
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
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}

parameter_types! {
	pub const ChainIdentity: ChainId = 1;
	pub const ProposalLifetime: BlockNumber = 50;
}

impl Trait for Test {
	type Event = TestEvent;
	type AdminOrigin = EnsureRoot<Self::AccountId>;
	type ChainIdentity = ChainIdentity;
	type Proposal = Call;
	type ProposalLifetime = ProposalLifetime;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	t.into()
}

pub type System = frame_system::Module<Test>;
pub type BridgeCommon = Module<Test>;

// Relayers
pub const RELAYER_A: u64 = 0x2;
pub const RELAYER_B: u64 = 0x3;
pub const RELAYER_C: u64 = 0x4;
// // pub const ENDOWED_BALANCE: u64 = 100_000_000;
pub const TEST_THRESHOLD: u32 = 2;

pub fn new_test_ext_initialized(
    src_id: ChainId,
    r_id: ResourceId,
    resource: Vec<u8>,
) -> sp_io::TestExternalities {
    let mut t = new_test_ext();
    t.execute_with(|| {
        // Set and check threshold
        assert_ok!(BridgeCommon::set_threshold(Origin::root(), TEST_THRESHOLD));
        assert_eq!(BridgeCommon::relayer_threshold(), TEST_THRESHOLD);
        // Add relayers
        assert_ok!(BridgeCommon::add_relayer(Origin::root(), RELAYER_A));
        assert_ok!(BridgeCommon::add_relayer(Origin::root(), RELAYER_B));
        assert_ok!(BridgeCommon::add_relayer(Origin::root(), RELAYER_C));
        // Whitelist chain
        assert_ok!(BridgeCommon::whitelist_chain(Origin::root(), src_id));
        // Set and check resource ID mapped to some junk data
        assert_ok!(BridgeCommon::add_resource(Origin::root(), r_id, resource));
        assert_eq!(BridgeCommon::resources(r_id).is_some(), true);
    });
    t
}