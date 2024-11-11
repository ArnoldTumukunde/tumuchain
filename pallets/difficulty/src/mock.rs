use crate::pallet;
use frame_support::{
    parameter_types,
    traits::{ConstU32, ConstU128, OnFinalize, Time},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        DifficultyPallet: pallet::{Pallet, Call, Storage, Event<T>, Config},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
}

impl pallet::Config for Test {
    type RuntimeEvent = Event;
    type WeightInfo = ();
    type TimeProvider = MockTimeProvider;
    type TargetBlockTime = ConstU128<10>;
    type DampFactor = ConstU128<2>;
    type ClampFactor = ConstU128<2>;
    type MaxDifficulty = ConstU128<u128::MAX>;
    type MinDifficulty = ConstU128<1>;
}

pub struct MockTimeProvider;
impl Time for MockTimeProvider {
    type Moment = u64;

    fn now() -> Self::Moment {
        1000
    }

    fn block_number() -> Self::Moment {
        1
    }
}

pub fn new_test_ext() -> frame_support::testing::TestExternalities {
    let t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
    let mut ext = frame_support::testing::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
    });
    ext
}