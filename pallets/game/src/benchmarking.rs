//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as GameModule;
use frame_benchmarking::v2::*;
use frame_support::{
	assert_ok,
	traits::{OnFinalize, OnInitialize},
};
use frame_system::RawOrigin;

fn create_setup<T: Config>() -> T::AccountId {
	let caller: T::AccountId = whitelisted_caller();
	let admin: T::AccountId = account("admin", 0, 0);
	assert_ok!(GameModule::<T>::setup_game(RawOrigin::Root.into()));
	assert_ok!(GameModule::<T>::add_to_admins(RawOrigin::Root.into(), admin.clone()));
	assert_ok!(GameModule::<T>::register_user(RawOrigin::Signed(admin).into(), caller.clone()));
	caller
}

fn practise_round<T: Config>(caller: T::AccountId, game_id: u32) {
	assert_ok!(GameModule::<T>::play_game(
		RawOrigin::Signed(caller.clone()).into(),
		crate::DifficultyLevel::Practice
	));
	assert_ok!(GameModule::<T>::submit_answer(
		RawOrigin::Signed(caller.clone()).into(),
		20,
		game_id
	));
	assert_ok!(GameModule::<T>::check_result(
		RawOrigin::Root.into(),
		20,
		game_id,
		20,
		"test".as_bytes().to_vec().try_into().unwrap(),
	));
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn setup_game() {
		#[extrinsic_call]
		setup_game(RawOrigin::Root);
	}

	#[benchmark]
	fn register_user() {
		let caller: T::AccountId = account("caller", 0, 0);
		assert_ok!(GameModule::<T>::setup_game(RawOrigin::Root.into()));
		let admin: T::AccountId = account("admin", 0, 0);
		assert_ok!(GameModule::<T>::add_to_admins(RawOrigin::Root.into(), admin.clone()));
		#[extrinsic_call]
		register_user(RawOrigin::Signed(admin), caller.clone());

		assert!(GameModule::<T>::users(caller).is_some());
	}

	#[benchmark]
	fn give_points() {
		let caller = create_setup::<T>();
		#[extrinsic_call]
		give_points(RawOrigin::Root, caller.clone(), 100);

		assert_eq!(GameModule::<T>::users(caller).unwrap().points, 150);
	}

	#[benchmark]
	fn play_game() {
		let caller = create_setup::<T>();
		current_block::<T>(30u32.into());
		practise_round::<T>(caller.clone(), 0);
		#[extrinsic_call]
		play_game(RawOrigin::Signed(caller.clone()), crate::DifficultyLevel::Player);

		assert_eq!(GameModule::<T>::game_info(1).unwrap().player, caller);
	}

	#[benchmark]
	fn submit_answer() {
		let caller = create_setup::<T>();
		current_block::<T>(30u32.into());
		practise_round::<T>(caller.clone(), 0);
		assert_ok!(GameModule::<T>::play_game(
			RawOrigin::Signed(caller.clone()).into(),
			crate::DifficultyLevel::Player
		));
		#[extrinsic_call]
		submit_answer(RawOrigin::Signed(caller.clone()), 220000, 1);

		assert_ok!(GameModule::<T>::check_result(
			RawOrigin::Root.into(),
			220000,
			1,
			220000,
			"test".as_bytes().to_vec().try_into().unwrap(),
		));
		assert_eq!(GameModule::<T>::users::<AccountIdOf<T>>(caller).unwrap().nfts.xorange, 1);
	}

	#[benchmark]
	fn check_result() {
		let caller = create_setup::<T>();
		current_block::<T>(30u32.into());
		practise_round::<T>(caller.clone(), 0);
		assert_ok!(GameModule::<T>::play_game(
			RawOrigin::Signed(caller.clone()).into(),
			crate::DifficultyLevel::Player
		));
		assert_ok!(GameModule::<T>::submit_answer(
			RawOrigin::Signed(caller.clone()).into(),
			220000,
			1
		));
		#[extrinsic_call]
		check_result(
			RawOrigin::Root,
			220000,
			1,
			220000,
			"test".as_bytes().to_vec().try_into().unwrap(),
		);

		assert_eq!(GameModule::<T>::users::<AccountIdOf<T>>(caller).unwrap().nfts.xorange, 1);
	}

	#[benchmark]
	fn list_nft() {
		let caller = create_setup::<T>();
		current_block::<T>(30u32.into());
		practise_round::<T>(caller.clone(), 0);
		assert_ok!(GameModule::<T>::play_game(
			RawOrigin::Signed(caller.clone()).into(),
			crate::DifficultyLevel::Player
		));
		assert_ok!(GameModule::<T>::submit_answer(
			RawOrigin::Signed(caller.clone()).into(),
			220000,
			1
		));
		assert_ok!(GameModule::<T>::check_result(
			RawOrigin::Root.into(),
			220000,
			1,
			220000,
			"test".as_bytes().to_vec().try_into().unwrap(),
		));
		#[extrinsic_call]
		list_nft(RawOrigin::Signed(caller.clone()), 0.into(), 0.into());

		assert_eq!(GameModule::<T>::listings(0).unwrap().owner, caller);
	}

	#[benchmark]
	fn delist_nft() {
		let caller = create_setup::<T>();
		current_block::<T>(30u32.into());
		practise_round::<T>(caller.clone(), 0);
		assert_ok!(GameModule::<T>::play_game(
			RawOrigin::Signed(caller.clone()).into(),
			crate::DifficultyLevel::Player
		));
		assert_ok!(GameModule::<T>::submit_answer(
			RawOrigin::Signed(caller.clone()).into(),
			220000,
			1
		));
		assert_ok!(GameModule::<T>::check_result(
			RawOrigin::Root.into(),
			220000,
			1,
			220000,
			"test".as_bytes().to_vec().try_into().unwrap(),
		));
		assert_ok!(GameModule::<T>::list_nft(
			RawOrigin::Signed(caller.clone()).into(),
			0.into(),
			0.into()
		));
		#[extrinsic_call]
		delist_nft(RawOrigin::Signed(caller), 0);

		assert!(GameModule::<T>::listings(0).is_none());
	}

	#[benchmark]
	fn make_offer() {
		let caller = create_setup::<T>();
		current_block::<T>(30u32.into());
		practise_round::<T>(caller.clone(), 0);
		assert_ok!(GameModule::<T>::play_game(
			RawOrigin::Signed(caller.clone()).into(),
			crate::DifficultyLevel::Player
		));
		assert_ok!(GameModule::<T>::submit_answer(
			RawOrigin::Signed(caller.clone()).into(),
			220000,
			1
		));
		assert_ok!(GameModule::<T>::check_result(
			RawOrigin::Root.into(),
			220000,
			1,
			220000,
			"test".as_bytes().to_vec().try_into().unwrap(),
		));
		assert_ok!(GameModule::<T>::list_nft(
			RawOrigin::Signed(caller.clone()).into(),
			0.into(),
			0.into()
		));
		let caller2: T::AccountId = account("caller2", 0, 0);
		let admin: T::AccountId = account("admin", 0, 0);
		assert_ok!(GameModule::<T>::register_user(
			RawOrigin::Signed(admin).into(),
			caller2.clone()
		));
		practise_round::<T>(caller2.clone(), 2);
		assert_ok!(GameModule::<T>::play_game(
			RawOrigin::Signed(caller2.clone()).into(),
			crate::DifficultyLevel::Player
		));
		assert_ok!(GameModule::<T>::submit_answer(
			RawOrigin::Signed(caller2.clone()).into(),
			220000,
			3
		));
		assert_ok!(GameModule::<T>::check_result(
			RawOrigin::Root.into(),
			220000,
			3,
			220000,
			"test".as_bytes().to_vec().try_into().unwrap(),
		));
		#[extrinsic_call]
		make_offer(RawOrigin::Signed(caller2.clone()), 0, 0.into(), 1.into());

		assert_eq!(GameModule::<T>::offers(0).unwrap().owner, caller2);
	}

	#[benchmark]
	fn handle_offer() {
		let caller = create_setup::<T>();
		current_block::<T>(30u32.into());
		practise_round::<T>(caller.clone(), 0);
		assert_ok!(GameModule::<T>::play_game(
			RawOrigin::Signed(caller.clone()).into(),
			crate::DifficultyLevel::Player
		));
		assert_ok!(GameModule::<T>::submit_answer(
			RawOrigin::Signed(caller.clone()).into(),
			220000,
			1
		));
		assert_ok!(GameModule::<T>::check_result(
			RawOrigin::Root.into(),
			220000,
			1,
			220000,
			"test".as_bytes().to_vec().try_into().unwrap(),
		));
		assert_ok!(GameModule::<T>::list_nft(
			RawOrigin::Signed(caller.clone()).into(),
			0.into(),
			0.into()
		));
		let caller2: T::AccountId = account("caller2", 0, 0);
		let admin: T::AccountId = account("admin", 0, 0);
		assert_ok!(GameModule::<T>::register_user(
			RawOrigin::Signed(admin).into(),
			caller2.clone()
		));
		practise_round::<T>(caller2.clone(), 2);
		assert_ok!(GameModule::<T>::play_game(
			RawOrigin::Signed(caller2.clone()).into(),
			crate::DifficultyLevel::Player
		));
		assert_ok!(GameModule::<T>::submit_answer(
			RawOrigin::Signed(caller2.clone()).into(),
			220000,
			3
		));
		assert_ok!(GameModule::<T>::check_result(
			RawOrigin::Root.into(),
			220000,
			3,
			220000,
			"test".as_bytes().to_vec().try_into().unwrap(),
		));
		assert_eq!(
			GameModule::<T>::users::<AccountIdOf<T>>(caller2.clone()).unwrap().nfts.xorange,
			1
		);
		assert_ok!(GameModule::<T>::make_offer(
			RawOrigin::Signed(caller2.clone()).into(),
			0,
			0.into(),
			1.into()
		));

		#[extrinsic_call]
		handle_offer(RawOrigin::Signed(caller), 0, crate::Offer::Accept);

		assert_eq!(GameModule::<T>::offers(0).is_none(), true);
		assert_eq!(GameModule::<T>::listings(0).is_none(), true);
	}

	#[benchmark]
	fn add_property() {
		assert_ok!(GameModule::<T>::setup_game(RawOrigin::Root.into()));
		let new_property = PropertyInfoData {
			id: 147031382,
			data: "nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap(),
		};
		#[extrinsic_call]
		add_property(RawOrigin::Root, new_property);
		assert_eq!(GameModule::<T>::game_properties().len(), 5);
	}

	#[benchmark]
	fn remove_property() {
		assert_ok!(GameModule::<T>::setup_game(RawOrigin::Root.into()));
		#[extrinsic_call]
		remove_property(RawOrigin::Root, 146480642);

		assert_eq!(GameModule::<T>::game_properties().len(), 3);
	}

	#[benchmark]
	fn add_to_admins() {
		assert_ok!(GameModule::<T>::setup_game(RawOrigin::Root.into()));
		let new_admin: T::AccountId = account("new_admin", 1, 0);
		#[extrinsic_call]
		add_to_admins(RawOrigin::Root, new_admin);
	}

	#[benchmark]
	fn remove_from_admins() {
		assert_ok!(GameModule::<T>::setup_game(RawOrigin::Root.into()));
		let new_admin: T::AccountId = account("new_admin", 0, 0);
		assert_ok!(GameModule::<T>::add_to_admins(RawOrigin::Root.into(), new_admin.clone()));
		#[extrinsic_call]
		remove_from_admins(RawOrigin::Root, new_admin);
	}

	#[benchmark]
	fn request_token() {
		let caller = create_setup::<T>();
		current_block::<T>(30u32.into());
		current_block::<T>(100801u32.into());
		#[extrinsic_call]
		request_token(RawOrigin::Signed(caller));
	}

	impl_benchmark_test_suite!(GameModule, crate::mock::new_test_ext(), crate::mock::Test);
}

fn current_block<T: Config>(new_block: frame_system::pallet_prelude::BlockNumberFor<T>) {
	while frame_system::Pallet::<T>::block_number() < new_block {
		if frame_system::Pallet::<T>::block_number() > 0u32.into() {
			GameModule::<T>::on_initialize(frame_system::Pallet::<T>::block_number());
			frame_system::Pallet::<T>::on_finalize(frame_system::Pallet::<T>::block_number());
		}
		frame_system::Pallet::<T>::reset_events();
		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + 1u32.into(),
		);
		frame_system::Pallet::<T>::on_initialize(frame_system::Pallet::<T>::block_number());
		GameModule::<T>::on_initialize(frame_system::Pallet::<T>::block_number());
	}
}
