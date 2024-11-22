use crate::ItemId;
use crate::{mock::*, Error};
use frame_support::BoundedVec;
use frame_support::{assert_noop, assert_ok};
use crate::{RegionCollections, LocationRegistration, ListedToken, NextNftId,
	OngoingObjectListing, NextAssetId, RegisteredNftDetails, TokenOwner, TokenBuyer,
	TokenListings, OngoingOffers, PropertyOwnerToken, PropertyOwner, PropertyLawyer,
	RealEstateLawyer};

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

// create_new_region function
#[test]
fn create_new_region_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_eq!(RegionCollections::<Test>::get(0).unwrap(), 0);
		assert_eq!(RegionCollections::<Test>::get(1).unwrap(), 1);
	})
}

// create_new_location function
#[test]
fn create_new_location_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![9, 10]));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 1, bvec![9, 10]));
		assert_eq!(
			LocationRegistration::<Test>::get::<u32, BoundedVec<u8, Postcode>>(
				0,
				bvec![10, 10]
			),
			true
		);
		assert_eq!(
			LocationRegistration::<Test>::get::<u32, BoundedVec<u8, Postcode>>(0, bvec![9, 10]),
			true
		);
		assert_eq!(
			LocationRegistration::<Test>::get::<u32, BoundedVec<u8, Postcode>>(1, bvec![9, 10]),
			true
		);
		assert_eq!(
			LocationRegistration::<Test>::get::<u32, BoundedVec<u8, Postcode>>(
				1,
				bvec![10, 10]
			),
			false
		);
		assert_eq!(
			LocationRegistration::<Test>::get::<u32, BoundedVec<u8, Postcode>>(1, bvec![8, 10]),
			false
		);
	})
}

#[test]
fn create_new_location_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_noop!(
			NftMarketplace::create_new_location(RuntimeOrigin::root(), 1, bvec![10, 10]),
			Error::<Test>::RegionUnknown
		);
	})
}

// register_lawyer function
#[test]
fn register_lawyer_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_eq!(RealEstateLawyer::<Test>::get::<AccountId>([0; 32].into()), false);
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [0; 32].into()));
		assert_eq!(RealEstateLawyer::<Test>::get::<AccountId>([0; 32].into()), true);
	})
}

#[test]
fn register_lawyer_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [0; 32].into()));
		assert_noop!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [0; 32].into()), Error::<Test>::LawyerAlreadyRegistered);
	})
}

// list_object function
#[test]
fn list_object_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_eq!(ListedToken::<Test>::get(0).unwrap(), 100);
		assert_eq!(NextNftId::<Test>::get(0), 1);
		assert_eq!(NextNftId::<Test>::get(1), 0);
		assert_eq!(NextAssetId::<Test>::get(), 1);
		assert_eq!(OngoingObjectListing::<Test>::get(0).is_some(), true);
		assert_eq!(RegisteredNftDetails::<Test>::get(0, 0).is_some(), true);
		assert_eq!(Uniques::owner(0, 0).unwrap(), NftMarketplace::account_id());
	})
}

#[test]
fn list_object_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_noop!(
			NftMarketplace::list_object(
				RuntimeOrigin::signed([0; 32].into()),
				0,
				bvec![10, 10],
				10_000,
				100,
				bvec![22, 22]
			),
			Error::<Test>::RegionUnknown
		);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_noop!(
			NftMarketplace::list_object(
				RuntimeOrigin::signed([0; 32].into()),
				0,
				bvec![10, 10],
				10_000,
				100,
				bvec![22, 22]
			),
			Error::<Test>::LocationUnknown
		);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_noop!(
			NftMarketplace::list_object(
				RuntimeOrigin::signed([0; 32].into()),
				0,
				bvec![10, 10],
				10_000,
				251,
				bvec![22, 22]
			),
			Error::<Test>::TooManyToken
		);
	})
}

// buy_token function
#[test]
fn buy_token_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 30));
		assert_eq!(ListedToken::<Test>::get(0).unwrap(), 70);
		assert_eq!(TokenOwner::<Test>::get::<AccountId, ItemId<Test>>([1; 32].into(), 0).token_amount, 30);
		assert_eq!(TokenBuyer::<Test>::get(0).len(), 1);
		assert_eq!(Balances::free_balance(&([1; 32].into())), 15_000_000);
		assert_eq!(Assets::balance(1, &[1; 32].into()), 1_188_000);
	})
}



#[test]
fn buy_token_doesnt_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_noop!(
			NftMarketplace::buy_token(RuntimeOrigin::signed([0; 32].into()), 1, 1),
			Error::<Test>::TokenNotForSale
		);
	})
}

#[test]
fn buy_token_doesnt_work_2() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_noop!(
			NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 101),
			Error::<Test>::NotEnoughTokenAvailable
		);
	})
}

#[test]
fn listing_and_selling_multiple_objects() {
	new_test_ext().execute_with(|| {
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [3; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [2; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([3; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([2; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 1, 80));
		assert_eq!(PropertyLawyer::<Test>::get(1).is_some(), false);
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 1, 20));
		assert_eq!(PropertyLawyer::<Test>::get(1).is_some(), true);
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			1,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			1,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			1,
			true,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([11; 32].into()),
			1,
			true,
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 2, 10));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 2, 10));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([2; 32].into()), 2, 30));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([3; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([2; 32].into()), 0, 33));
		assert_eq!(ListedToken::<Test>::get(0).unwrap(), 67);
		assert_eq!(ListedToken::<Test>::get(2).unwrap(), 50);
		assert_eq!(ListedToken::<Test>::get(3).unwrap(), 100);
		assert_eq!(TokenOwner::<Test>::get::<AccountId, ItemId<Test>>([2; 32].into(), 2).token_amount, 30);
		assert_eq!(TokenBuyer::<Test>::get(2).len(), 2);
		assert_eq!(TokenOwner::<Test>::get::<AccountId, ItemId<Test>>([1; 32].into(), 1).token_amount, 0);
		assert_eq!(TokenBuyer::<Test>::get(1).len(), 0);
		assert_eq!(PropertyOwnerToken::<Test>::get::<u32, AccountId>(2, [1; 32].into()), 100);
	});
}

// lawyer_claim_property function
#[test]
fn claim_property_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_eq!(PropertyLawyer::<Test>::get(0).unwrap().real_estate_developer_lawyer, Some([10; 32].into()));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_eq!(PropertyLawyer::<Test>::get(0).unwrap().spv_lawyer, Some([11; 32].into()));
	})
}

#[test]
fn claim_property_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 99));
		assert_noop!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		), Error::<Test>::InvalidIndex);
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 1));
		assert_noop!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([9; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		), Error::<Test>::NoPermission);
		assert_noop!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			11_000,
		), Error::<Test>::CostsTooHigh);
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_eq!(PropertyLawyer::<Test>::get(0).unwrap().real_estate_developer_lawyer, Some([10; 32].into()));
		assert_noop!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		), Error::<Test>::LawyerJobTaken);
		assert_eq!(PropertyLawyer::<Test>::get(0).unwrap().spv_lawyer, None);
	})
}

// remove_from_case function
#[test]
fn remove_from_case_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [12; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_eq!(PropertyLawyer::<Test>::get(0).unwrap().real_estate_developer_lawyer, Some([10; 32].into()));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_eq!(PropertyLawyer::<Test>::get(0).unwrap().spv_lawyer, Some([11; 32].into()));
		assert_ok!(NftMarketplace::remove_from_case(
			RuntimeOrigin::signed([10; 32].into()),
			0,
		));
		assert_eq!(PropertyLawyer::<Test>::get(0).unwrap().real_estate_developer_lawyer, None);
		assert_eq!(PropertyLawyer::<Test>::get(0).unwrap().spv_lawyer, Some([11; 32].into()));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([12; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_eq!(PropertyLawyer::<Test>::get(0).unwrap().real_estate_developer_lawyer, Some([12; 32].into()));
	})
}

#[test]
fn remove_from_case_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_noop!(NftMarketplace::remove_from_case(
			RuntimeOrigin::signed([10; 32].into()),
			0,
		), Error::<Test>::NoPermission);
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_noop!(NftMarketplace::remove_from_case(
			RuntimeOrigin::signed([10; 32].into()),
			1,
		), Error::<Test>::InvalidIndex);
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			true,
		));
		assert_noop!(NftMarketplace::remove_from_case(
			RuntimeOrigin::signed([10; 32].into()),
			0,
		), Error::<Test>::AlreadyConfirmed);
	})
}

// lawyer_confirm_documents function
#[test]
fn distributes_nfts_and_funds() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_eq!(PropertyLawyer::<Test>::get(0).unwrap().real_estate_developer_lawyer, Some([10; 32].into()));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_eq!(PropertyLawyer::<Test>::get(0).unwrap().spv_lawyer, Some([11; 32].into()));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			true,
		));
		assert_eq!(PropertyLawyer::<Test>::get(0).unwrap().real_estate_developer_status, crate::DocumentStatus::Approved);
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			true,
		));
		assert_eq!(PropertyLawyer::<Test>::get(1).is_some(), false);
		assert_eq!(Assets::balance(1, &[0; 32].into()), 20_990_000);
		assert_eq!(Assets::balance(1, &NftMarketplace::treasury_account_id()), 12000);
		assert_eq!(Assets::balance(1, &[1; 32].into()), 460_000);
		assert_eq!(Assets::balance(1, &[10; 32].into()), 34_000);
		assert_eq!(Assets::balance(1, &[11; 32].into()), 4_000);
		assert_eq!(RegisteredNftDetails::<Test>::get(0, 0).unwrap().spv_created, true);
		assert_eq!(ListedToken::<Test>::get(0), None);
		assert_eq!(TokenOwner::<Test>::get::<AccountId, ItemId<Test>>([1; 32].into(), 0).token_amount, 0);
		assert_eq!(TokenBuyer::<Test>::get(0).len(), 0);
		assert_eq!(Assets::balance(0, &[1; 32].into()), 100);
	})
}

#[test]
fn reject_contract_and_refund() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_eq!(PropertyLawyer::<Test>::get(0).unwrap().real_estate_developer_lawyer, Some([10; 32].into()));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_eq!(PropertyLawyer::<Test>::get(0).unwrap().spv_lawyer, Some([11; 32].into()));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			false,
		));
		assert_eq!(PropertyLawyer::<Test>::get(0).unwrap().real_estate_developer_status, crate::DocumentStatus::Rejected);
		assert_eq!(pallet_nfts::Item::<Test>::get(0, 0).is_none(), false);
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			false,
		));
		assert_eq!(PropertyLawyer::<Test>::get(1).is_some(), false);
		assert_eq!(Assets::balance(1, &[0; 32].into()), 20_000_000);
		assert_eq!(Assets::balance(1, &NftMarketplace::treasury_account_id()), 6000);
		assert_eq!(Assets::balance(1, &[1; 32].into()), 1_490_000);
		assert_eq!(Assets::balance(1, &[11; 32].into()), 4_000);
		assert_eq!(RegisteredNftDetails::<Test>::get(0, 0).is_none(), true);
		assert_eq!(ListedToken::<Test>::get(0), None);
		assert_eq!(TokenOwner::<Test>::get::<AccountId, ItemId<Test>>([1; 32].into(), 0).token_amount, 0);
		assert_eq!(TokenBuyer::<Test>::get(0).len(), 0);
		assert_eq!(pallet_nfts::Item::<Test>::get(0, 0).is_none(), true);
	})
}

#[test]
fn second_attempt_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_eq!(PropertyLawyer::<Test>::get(0).unwrap().real_estate_developer_lawyer, Some([10; 32].into()));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_eq!(PropertyLawyer::<Test>::get(0).unwrap().spv_lawyer, Some([11; 32].into()));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			true,
		));
		assert_eq!(PropertyLawyer::<Test>::get(0).unwrap().real_estate_developer_status, crate::DocumentStatus::Approved);
		assert_eq!(pallet_nfts::Item::<Test>::get(0, 0).is_none(), false);
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			false,
		));
		assert_eq!(PropertyLawyer::<Test>::get(0).unwrap().second_attempt, true);
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			false,
		));
		assert_eq!(PropertyLawyer::<Test>::get(0).unwrap().real_estate_developer_status, crate::DocumentStatus::Rejected);
		assert_eq!(pallet_nfts::Item::<Test>::get(0, 0).is_none(), false);
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			true,
		));
		assert_eq!(Assets::balance(1, &[0; 32].into()), 20_000_000);
		assert_eq!(Assets::balance(1, &NftMarketplace::treasury_account_id()), 6000);
		assert_eq!(Assets::balance(1, &[1; 32].into()), 1_490_000);
		assert_eq!(Assets::balance(1, &[11; 32].into()), 4_000);
		assert_eq!(RegisteredNftDetails::<Test>::get(0, 0).is_none(), true);
		assert_eq!(ListedToken::<Test>::get(0), None);
		assert_eq!(TokenOwner::<Test>::get::<AccountId, ItemId<Test>>([1; 32].into(), 0).token_amount, 0);
		assert_eq!(TokenBuyer::<Test>::get(0).len(), 0);
		assert_eq!(pallet_nfts::Item::<Test>::get(0, 0).is_none(), true);
	})
}

#[test]
fn lawyer_confirm_documents_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [12; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_eq!(PropertyLawyer::<Test>::get(0).unwrap().real_estate_developer_lawyer, Some([10; 32].into()));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_eq!(PropertyLawyer::<Test>::get(0).unwrap().spv_lawyer, Some([11; 32].into()));
		assert_noop!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			1,
			false,
		), Error::<Test>::InvalidIndex);
		assert_noop!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([12; 32].into()),
			0,
			false,
		), Error::<Test>::NoPermission);
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			false,
		));
		assert_noop!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			true,
		), Error::<Test>::AlreadyConfirmed);
	})
}

// list_token function
#[test]
fn relist_a_nft() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			true,
		));
		assert_eq!(RegisteredNftDetails::<Test>::get(0, 0).unwrap().spv_created, true);
		assert_ok!(NftMarketplace::relist_token(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			0,
			1000,
			1
		));
		assert_eq!(TokenListings::<Test>::get(1).is_some(), true);
		assert_eq!(TokenListings::<Test>::get(1).unwrap().item_id, 0);
		assert_eq!(Assets::balance(0, NftMarketplace::account_id()), 1);
		assert_eq!(Assets::balance(0, &[1; 32].into()), 99);
	})
}

#[test]
fn relist_nfts_not_created_with_marketplace_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Uniques::create(
			RuntimeOrigin::signed([0; 32].into()),
			sp_runtime::MultiAddress::Id([0; 32].into()),
			Default::default()
		));
		assert_ok!(Uniques::mint(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			0,
			sp_runtime::MultiAddress::Id([0; 32].into()),
			None
		));
		assert_noop!(
			NftMarketplace::relist_token(RuntimeOrigin::signed([0; 32].into()), 0, 0, 1000, 1),
			Error::<Test>::RegionUnknown
		);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_noop!(
			NftMarketplace::relist_token(RuntimeOrigin::signed([0; 32].into()), 0, 0, 1000, 1),
			Error::<Test>::NftNotFound
		);
	})
}

#[test]
fn relist_a_nft_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			true,
		));
		assert_eq!(RegisteredNftDetails::<Test>::get(0, 0).unwrap().spv_created, true);
		assert_noop!(
			NftMarketplace::relist_token(RuntimeOrigin::signed([0; 32].into()), 0, 0, 1000, 1),
			Error::<Test>::NotEnoughFunds
		);
	})
}

// buy_relisted_token function
#[test]
fn buy_relisted_token_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [3; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			true,
		));
		assert_eq!(Assets::balance(1, &([0; 32].into())), 20990000);
		assert_eq!(Assets::balance(1, &NftMarketplace::treasury_account_id()), 12000);
		assert_eq!(Assets::balance(1, &([1; 32].into())), 460_000);
		assert_eq!(RegisteredNftDetails::<Test>::get(0, 0).unwrap().spv_created, true);
		assert_ok!(NftMarketplace::relist_token(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			0,
			1000,
			3
		));
		assert_ok!(NftMarketplace::buy_relisted_token(RuntimeOrigin::signed([3; 32].into()), 1, 2));
		assert_eq!(Assets::balance(1, &([3; 32].into())), 3_000);
		assert_eq!(Assets::balance(0, &[3; 32].into()), 2);
		assert_eq!(TokenListings::<Test>::get(1).is_some(), true);
		assert_ok!(NftMarketplace::buy_relisted_token(RuntimeOrigin::signed([3; 32].into()), 1, 1));
		assert_eq!(Assets::balance(1, &([3; 32].into())), 2_000);
		assert_eq!(TokenListings::<Test>::get(1).is_some(), false);
		assert_ok!(NftMarketplace::relist_token(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			0,
			500,
			1
		));
		assert_ok!(NftMarketplace::buy_relisted_token(RuntimeOrigin::signed([3; 32].into()), 2, 1));
		assert_eq!(TokenListings::<Test>::get(0).is_some(), false);
		assert_eq!(PropertyOwner::<Test>::get(0).len(), 2);
		assert_eq!(PropertyOwnerToken::<Test>::get::<u32, AccountId>(0, [1; 32].into()), 96);
		assert_eq!(PropertyOwnerToken::<Test>::get::<u32, AccountId>(0, [3; 32].into()), 4);
		assert_eq!(Assets::balance(1, &([1; 32].into())), 463_465);
		assert_eq!(Assets::balance(1, &([3; 32].into())), 1_500);
		assert_eq!(Assets::balance(0, &[1; 32].into()), 96);
		assert_eq!(Assets::balance(0, &[3; 32].into()), 4);
	})
}

#[test]
fn buy_relisted_token_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [3; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			true,
		));
		assert_eq!(Assets::balance(1, &([0; 32].into())), 20990000);
		assert_eq!(Assets::balance(1, &NftMarketplace::treasury_account_id()), 12_000);
		assert_eq!(Assets::balance(1, &([1; 32].into())), 460_000);
		assert_eq!(RegisteredNftDetails::<Test>::get(0, 0).unwrap().spv_created, true);
		assert_noop!(
			NftMarketplace::buy_relisted_token(RuntimeOrigin::signed([3; 32].into()), 1, 1),
			Error::<Test>::TokenNotForSale
		);
	})
}

// make_offer function
#[test]
fn make_offer_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [2; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::relist_token(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			0,
			500,
			1
		));
		assert_ok!(NftMarketplace::make_offer(RuntimeOrigin::signed([2; 32].into()), 1, 2000, 1));
		assert_eq!(TokenListings::<Test>::get(1).is_some(), true);
		assert_eq!(OngoingOffers::<Test>::get::<u32, AccountId>(1, [2; 32].into()).is_some(), true);
		assert_eq!(Assets::balance(1, &([2; 32].into())), 1_148_000);
		assert_eq!(Assets::balance(1, &NftMarketplace::account_id()), 2000);
	})
}

#[test]
fn make_offer_fails() {
	new_test_ext().execute_with(|| {
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [2; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [3; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			true,
		));
		assert_noop!(
			NftMarketplace::make_offer(RuntimeOrigin::signed([2; 32].into()), 1, 200, 1),
			Error::<Test>::TokenNotForSale
		);
		assert_ok!(NftMarketplace::relist_token(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			0,
			500,
			1
		));
		assert_noop!(
			NftMarketplace::make_offer(RuntimeOrigin::signed([2; 32].into()), 1, 200, 2),
			Error::<Test>::NotEnoughTokenAvailable
		);
		assert_ok!(NftMarketplace::make_offer(RuntimeOrigin::signed([2; 32].into()), 1, 200, 1));
		assert_ok!(NftMarketplace::make_offer(RuntimeOrigin::signed([3; 32].into()), 1, 300, 1));
		assert_noop!(
			NftMarketplace::make_offer(RuntimeOrigin::signed([2; 32].into()), 1, 400, 1),
			Error::<Test>::OnlyOneOfferPerUser
		);
		assert_eq!(OngoingOffers::<Test>::get::<u32, AccountId>(1, [2; 32].into()).unwrap().token_price, 200);
		assert_eq!(OngoingOffers::<Test>::get::<u32, AccountId>(1, [3; 32].into()).unwrap().token_price, 300);
	})
}

// handle_offer function
#[test]
fn handle_offer_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [2; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [3; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::relist_token(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			0,
			5000,
			20
		));
		assert_ok!(NftMarketplace::make_offer(RuntimeOrigin::signed([2; 32].into()), 1, 200, 1));
		assert_ok!(NftMarketplace::make_offer(RuntimeOrigin::signed([3; 32].into()), 1, 150, 1));
		assert_ok!(NftMarketplace::handle_offer(
			RuntimeOrigin::signed([1; 32].into()),
			1,
			[2; 32].into(),
			crate::Offer::Reject
		));
		assert_ok!(NftMarketplace::cancel_offer(RuntimeOrigin::signed([3; 32].into()), 1));
		assert_eq!(Assets::balance(1, &([2; 32].into())), 1_150_000);
		assert_eq!(TokenListings::<Test>::get(1).is_some(), true);
		assert_eq!(OngoingOffers::<Test>::get::<u32, AccountId>(1, [2; 32].into()).is_none(), true);
		assert_ok!(NftMarketplace::make_offer(RuntimeOrigin::signed([2; 32].into()), 1, 2000, 10));
		assert_eq!(Assets::balance(1, &([2; 32].into())), 1_130_000);
		assert_eq!(Assets::balance(1, &NftMarketplace::account_id()), 20000);
		assert_ok!(NftMarketplace::handle_offer(
			RuntimeOrigin::signed([1; 32].into()),
			1,
			[2; 32].into(),
			crate::Offer::Accept
		));
		assert_eq!(TokenListings::<Test>::get(1).unwrap().amount, 10);
		assert_eq!(OngoingOffers::<Test>::get::<u32, AccountId>(1, [2; 32].into()).is_none(), true);
		assert_eq!(Assets::balance(1, &NftMarketplace::account_id()), 0);
		assert_eq!(Assets::balance(0, &([1; 32].into())), 80);
		assert_eq!(Assets::balance(0, &([2; 32].into())), 10);
		assert_eq!(Assets::balance(0, &NftMarketplace::account_id()), 10);
		assert_eq!(Assets::balance(1, &([1; 32].into())), 479_800);
		assert_eq!(Assets::balance(1, &([2; 32].into())), 1_130_000);
	})
}

#[test]
fn handle_offer_fails() {
	new_test_ext().execute_with(|| {
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [2; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			true,
		));
		assert_noop!(
			NftMarketplace::handle_offer(
				RuntimeOrigin::signed([1; 32].into()),
				1,
				[2; 32].into(),
				crate::Offer::Reject
			),
			Error::<Test>::TokenNotForSale
		);
		assert_ok!(NftMarketplace::relist_token(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			0,
			5000,
			2
		));
		assert_noop!(
			NftMarketplace::handle_offer(
				RuntimeOrigin::signed([1; 32].into()),
				1,
				[2; 32].into(),
				crate::Offer::Reject
			),
			Error::<Test>::InvalidIndex
		);
		assert_ok!(NftMarketplace::make_offer(RuntimeOrigin::signed([2; 32].into()), 1, 200, 1));
		assert_noop!(
			NftMarketplace::handle_offer(
				RuntimeOrigin::signed([2; 32].into()),
				1,
				[2; 32].into(),
				crate::Offer::Accept
			),
			Error::<Test>::NoPermission
		);
	})
}

// cancel_offer function

#[test]
fn cancel_offer_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [2; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::relist_token(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			0,
			500,
			1
		));
		assert_ok!(NftMarketplace::make_offer(RuntimeOrigin::signed([2; 32].into()), 1, 2000, 1));
		assert_eq!(TokenListings::<Test>::get(1).is_some(), true);
		assert_eq!(OngoingOffers::<Test>::get::<u32, AccountId>(1, [2; 32].into()).is_some(), true);
		assert_eq!(Assets::balance(1, &([2; 32].into())), 1_148_000);
		assert_eq!(Assets::balance(1, &NftMarketplace::account_id()), 2000);
		assert_ok!(NftMarketplace::cancel_offer(RuntimeOrigin::signed([2; 32].into()), 1));
		assert_eq!(TokenListings::<Test>::get(1).is_some(), true);
		assert_eq!(OngoingOffers::<Test>::get::<u32, AccountId>(1, [2; 32].into()).is_some(), false);
		assert_eq!(Assets::balance(1, &([2; 32].into())), 1_150_000);
		assert_eq!(Assets::balance(1, &NftMarketplace::account_id()), 0);
	})
}

#[test]
fn cancel_offer_fails() {
	new_test_ext().execute_with(|| {
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [2; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::relist_token(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			0,
			500,
			1
		));
		assert_noop!(
			NftMarketplace::cancel_offer(RuntimeOrigin::signed([2; 32].into()), 1),
			Error::<Test>::InvalidIndex
		);
		assert_ok!(NftMarketplace::make_offer(RuntimeOrigin::signed([2; 32].into()), 1, 2000, 1));
		assert_eq!(TokenListings::<Test>::get(1).is_some(), true);
		assert_eq!(OngoingOffers::<Test>::get::<u32, AccountId>(1, [2; 32].into()).is_some(), true);
		assert_eq!(Assets::balance(1, &([2; 32].into())), 1_148_000);
		assert_eq!(Assets::balance(1, &NftMarketplace::account_id()), 2000);
		assert_noop!(
			NftMarketplace::cancel_offer(RuntimeOrigin::signed([1; 32].into()), 1),
			Error::<Test>::InvalidIndex
		);
	})
}

// upgrade_listing function
#[test]
fn upgrade_price_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::relist_token(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			0,
			1000,
			1
		));
		assert_ok!(NftMarketplace::upgrade_listing(RuntimeOrigin::signed([1; 32].into()), 1, 300));
		assert_eq!(TokenListings::<Test>::get(1).unwrap().token_price, 300);
	})
}

#[test]
fn upgrade_price_fails_if_not_owner() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::relist_token(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			0,
			1000,
			1
		));
		assert_noop!(
			NftMarketplace::upgrade_listing(RuntimeOrigin::signed([4; 32].into()), 1, 300),
			Error::<Test>::NoPermission
		);
	})
}

// upgrade_object function
#[test]
fn upgrade_object_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::upgrade_object(RuntimeOrigin::signed([0; 32].into()), 0, 30000));
		assert_eq!(OngoingObjectListing::<Test>::get(0).unwrap().token_price, 30000);
	})
}

#[test]
fn upgrade_object_and_distribute_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [2; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 50));
		assert_ok!(NftMarketplace::upgrade_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			20_000
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([2; 32].into()), 0, 50));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			true,
		));
		assert_eq!(Assets::balance(1, &([0; 32].into())), 21485000);
		assert_eq!(Assets::balance(1, &NftMarketplace::treasury_account_id()), 22000);
		assert_eq!(Assets::balance(1, &([1; 32].into())), 980_000);
		assert_eq!(Assets::balance(1, &([2; 32].into())), 110_000);

		assert_eq!(RegisteredNftDetails::<Test>::get(0, 0).unwrap().spv_created, true);
		assert_eq!(ListedToken::<Test>::get(0), None);
	})
}

#[test]
fn upgrade_single_nft_from_listed_object_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_noop!(
			NftMarketplace::upgrade_listing(RuntimeOrigin::signed([0; 32].into()), 0, 300),
			Error::<Test>::TokenNotForSale
		);
	})
}

#[test]
fn upgrade_object_for_relisted_nft_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([0; 32].into()), 0, 100));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::relist_token(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			0,
			1000,
			1
		));
		assert_noop!(
			NftMarketplace::upgrade_object(RuntimeOrigin::signed([0; 32].into()), 1, 300),
			Error::<Test>::TokenNotForSale
		);
	})
}

#[test]
fn upgrade_unknown_collection_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_noop!(
			NftMarketplace::upgrade_object(RuntimeOrigin::signed([0; 32].into()), 0, 300),
			Error::<Test>::TokenNotForSale
		);
	})
}

// delist_token function
#[test]
fn delist_single_token_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [2; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::relist_token(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			0,
			1000,
			1
		));
		assert_ok!(NftMarketplace::delist_token(RuntimeOrigin::signed([1; 32].into()), 1));
		assert_eq!(TokenListings::<Test>::get(0), None);
		assert_ok!(NftMarketplace::relist_token(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			0,
			1000,
			3
		));
		assert_ok!(NftMarketplace::buy_relisted_token(RuntimeOrigin::signed([2; 32].into()), 2, 2));
		assert_ok!(NftMarketplace::delist_token(RuntimeOrigin::signed([1; 32].into()), 2));
		assert_eq!(Assets::balance(0, &[2; 32].into()), 2);
		assert_eq!(Assets::balance(0, &[1; 32].into()), 98);
	})
}
 
#[test]
fn delist_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([11; 32].into()),
			0,
			true,
		));
		assert_ok!(NftMarketplace::relist_token(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			0,
			1000,
			1
		));
		assert_noop!(
			NftMarketplace::delist_token(RuntimeOrigin::signed([4; 32].into()), 1),
			Error::<Test>::NoPermission
		);
		assert_noop!(
			NftMarketplace::delist_token(RuntimeOrigin::signed([1; 32].into()), 2),
			Error::<Test>::TokenNotForSale
		);
	})
}

#[test]
fn listing_objects_in_different_regions() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_region(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 0, bvec![10, 10]));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 1, bvec![10, 10]));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root(), 2, bvec![10, 10]));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(XcavateWhitelist::add_to_whitelist(RuntimeOrigin::root(), [2; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [10; 32].into()));
		assert_ok!(NftMarketplace::register_lawyer(RuntimeOrigin::root(), [11; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			1,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			2,
			bvec![10, 10],
			10_000,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 1, 100));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			1,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			1,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			1,
			true,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([11; 32].into()),
			1,
			true,
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([2; 32].into()), 2, 100));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([10; 32].into()),
			2,
			crate::LegalProperty::RealEstateDeveloperSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_claim_property(
			RuntimeOrigin::signed([11; 32].into()),
			2,
			crate::LegalProperty::SpvSide,
			4_000,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([10; 32].into()),
			2,
			true,
		));
		assert_ok!(NftMarketplace::lawyer_confirm_documents(
			RuntimeOrigin::signed([11; 32].into()),
			2,
			true,
		));
		assert_eq!(RegisteredNftDetails::<Test>::get(1, 0).unwrap().spv_created, true);
		assert_eq!(RegisteredNftDetails::<Test>::get(2, 0).unwrap().spv_created, true);
		assert_ok!(NftMarketplace::relist_token(
			RuntimeOrigin::signed([1; 32].into()),
			1,
			0,
			1000,
			100
		));
		assert_ok!(NftMarketplace::buy_relisted_token(
			RuntimeOrigin::signed([2; 32].into()),
			3,
			100
		));
		assert_eq!(Assets::balance(2, &[2; 32].into()), 100);
		assert_eq!(Assets::balance(3, &[2; 32].into()), 100);
	})
}
 