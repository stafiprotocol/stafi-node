use crate as rdex_mining;
use crate::{Module, Trait};
use frame_support::{
    impl_outer_dispatch, impl_outer_event, impl_outer_origin, parameter_types, traits::Get,
    weights::Weight,
};
use sp_core::{H256, U256};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};
use sp_std::cell::RefCell;

pub(crate) type Balance = u128;

thread_local! {
    static EXISTENTIAL_DEPOSIT: RefCell<Balance> = RefCell::new(0);
}

impl_outer_origin! {
    pub enum Origin for Test where system = frame_system {}
}

impl_outer_event! {
    pub enum TestEvent for Test {
        frame_system<T>,
        rdex_mining<T>,
        rdex_swap<T>,
    }
}

impl_outer_dispatch! {
    pub enum Call for Test where origin: Origin {
        frame_system::System,
        self::RDexMining,
        self::RDexSwap,
    }
}
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
}
// For testing the pallet, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of pallets we want to use.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Test;

impl frame_system::Trait for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = U256;
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

impl Trait for Test {
    type Event = ();
    type Currency = Balances;
    type LpCurrency = LpBalances;
}

impl rdex_swap::Trait for Test {
    type Event = ();
    type RCurrency = RBalances;
    type Currency = Balances;
    type LpCurrency = LpBalances;
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

impl rdex_balances::Trait for Test {
    type Event = ();
}

impl rtoken_balances::Trait for Test {
    type Event = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (U256::from(42), 100),
            (U256::from(1), 100),
            (U256::from(2), 100),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
}

pub type RDexMining = Module<Test>;
pub type System = frame_system::Module<Test>;
pub type Balances = pallet_balances::Module<Test>;
pub type LpBalances = rdex_balances::Module<Test>;
pub type RDexSwap = rdex_swap::Module<Test>;
pub type RBalances = rtoken_balances::Module<Test>;

pub struct ExistentialDeposit;
impl Get<Balance> for ExistentialDeposit {
    fn get() -> Balance {
        EXISTENTIAL_DEPOSIT.with(|v| *v.borrow())
    }
}
