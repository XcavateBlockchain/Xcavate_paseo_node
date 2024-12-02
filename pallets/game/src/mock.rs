use crate as pallet_game;
use frame_support::{parameter_types, traits::AsEnsureOriginWithArg, PalletId, derive_impl};
use pallet_nfts::PalletFeatures;
use sp_core::ConstU32;
use sp_runtime::{
	traits::{AccountIdLookup, BlakeTwo256, IdentifyAccount, Verify},
	BuildStorage, MultiSignature,
};
pub type BlockNumber = u64;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Signature = MultiSignature;
pub type AccountPublic = <Signature as Verify>::Signer;
use frame_system::EnsureRoot;

type Block = frame_system::mocking::MockBlock<Test>;

pub const MILLISECS_PER_BLOCK: u64 = 6000;
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		GameModule: pallet_game,
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Nfts: pallet_nfts::{Pallet, Call, Storage, Event<T>},
		RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip,
	}
);

parameter_types! {
	pub const BlockHashCount: BlockNumber = 2400;
}

#[derive_impl(frame_system::config_preludes::ParaChainDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
	type RuntimeCall = RuntimeCall;
	type Nonce = u32;
	type Block = Block;
	type Hash = sp_core::H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = AccountIdLookup<AccountId, ()>;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u32>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = frame_support::traits::Everything;
	type SystemWeightInfo = ();
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type RuntimeTask = ();
}

impl pallet_balances::Config for Test {
	type Balance = u32;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU32<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = ();
	// Holds are used with COLLATOR_LOCK_ID and DELEGATOR_LOCK_ID
	type MaxFreezes = ConstU32<0>;
}

parameter_types! {
	pub Features: PalletFeatures = PalletFeatures::all_enabled();
	pub const ApprovalsLimit: u32 = 20;
	pub const ItemAttributesApprovalsLimit: u32 = 20;
	pub const MaxTips: u32 = 10;
	pub const MaxDeadlineDuration: BlockNumber = 12 * 30 * DAYS;
	pub const MaxAttributesPerCall: u32 = 10;
}

impl pallet_nfts::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = u32;
	type ItemId = u32;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<Self::AccountId>>;
	type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type Locker = ();
	type CollectionDeposit = ConstU32<2>;
	type ItemDeposit = ConstU32<1>;
	type MetadataDepositBase = ConstU32<1>;
	type AttributeDepositBase = ConstU32<1>;
	type DepositPerByte = ConstU32<1>;
	type StringLimit = ConstU32<50>;
	type KeyLimit = ConstU32<50>;
	type ValueLimit = ConstU32<50>;
	type WeightInfo = ();
	type ApprovalsLimit = ApprovalsLimit;
	type ItemAttributesApprovalsLimit = ItemAttributesApprovalsLimit;
	type MaxTips = MaxTips;
	type MaxDeadlineDuration = MaxDeadlineDuration;
	type MaxAttributesPerCall = MaxAttributesPerCall;
	type Features = Features;
	type OffchainSignature = Signature;
	type OffchainPublic = AccountPublic;
}

impl pallet_insecure_randomness_collective_flip::Config for Test {}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct MaxProperties;

impl sp_core::Get<u32> for MaxProperties {
	fn get() -> u32 {
		100
	}
}

parameter_types! {
	pub const GamePalletId: PalletId = PalletId(*b"py/rlxdl");
	pub const MaxOngoingGame: u32 = 200;
	pub const LeaderLimit: u32 = 10;
	pub const MaxAdmin: u32 = 10;
	pub const RequestLimits: BlockNumber = 180;
}

/// Configure the pallet-game in pallets/game.
impl pallet_game::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type WeightInfo = pallet_game::weights::SubstrateWeight<Test>;
	type GameOrigin = EnsureRoot<Self::AccountId>;
	type CollectionId = u32;
	type ItemId = u32;
	type MaxProperty = MaxProperties;
	type PalletId = GamePalletId;
	type MaxOngoingGames = MaxOngoingGame;
	type GameRandomness = RandomnessCollectiveFlip;
	type StringLimit = ConstU32<5000>;
	type LeaderboardLimit = LeaderLimit;
	type MaxAdmins = MaxAdmin;
	type RequestLimit = RequestLimits;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut test = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(GameModule::account_id(), 1_000_000)],
	}
	.assimilate_storage(&mut test)
	.unwrap();

	test.into()
}
