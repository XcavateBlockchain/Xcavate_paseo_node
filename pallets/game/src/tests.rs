use crate::{mock::*, Error, Event, PropertyInfoData};
use frame_support::{
	assert_noop, assert_ok,
	traits::{OnFinalize, OnInitialize},
};
use sp_runtime::{traits::BadOrigin, DispatchError, ModuleError};

fn practise_round(player: AccountId, game_id: u32) {
	assert_ok!(GameModule::play_game(
		RuntimeOrigin::signed(player.clone()),
		crate::DifficultyLevel::Practice,
	));
	assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed(player.clone()), 220000, game_id));
	System::assert_last_event(Event::AnswerSubmitted { player, game_id, guess: 220_000 }.into());
	assert_ok!(GameModule::check_result(
		RuntimeOrigin::root(),
		220_000,
		game_id,
		220_000,
		"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
	));
}

fn run_to_block(n: u64) {
	while System::block_number() < n {
		GameModule::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		GameModule::on_initialize(System::block_number());
	}
}


#[test]
fn setup_game_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
	});
}

#[test]
fn setup_game_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_noop!(GameModule::setup_game(RuntimeOrigin::signed([0; 32].into())), BadOrigin);
	});
}

#[test]
fn add_to_admins_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [0; 32].into()));
		assert_eq!(GameModule::admins().len(), 1);
	});
}

#[test]
fn add_to_admins_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_noop!(
			GameModule::add_to_admins(RuntimeOrigin::signed([0; 32].into()), [0; 32].into()),
			BadOrigin
		);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [0; 32].into()));
		assert_eq!(GameModule::admins().len(), 1);
		assert_noop!(
			GameModule::add_to_admins(RuntimeOrigin::root(), [0; 32].into()),
			Error::<Test>::AccountAlreadyAdmin
		);
	});
}

#[test]
fn remove_admins_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [0; 32].into()));
		assert_eq!(GameModule::admins().len(), 1);
		assert_ok!(GameModule::remove_from_admins(RuntimeOrigin::root(), [0; 32].into()));
		assert_eq!(GameModule::admins().len(), 0);
	});
}

#[test]
fn remove_admins_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_noop!(
			GameModule::remove_from_admins(RuntimeOrigin::root(), [0; 32].into()),
			Error::<Test>::NotAdmin
		);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [0; 32].into()));
		assert_eq!(GameModule::admins().len(), 1);
		assert_noop!(
			GameModule::remove_from_admins(RuntimeOrigin::signed([0; 32].into()), [0; 32].into()),
			BadOrigin
		);
	});
}

#[test]
fn play_game_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::game_properties().len(), 4);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[0; 32].into()
		));
		assert_eq!(Balances::free_balance(&([0; 32].into())), 10);
		assert_ok!(GameModule::give_points(RuntimeOrigin::root(), [0; 32].into(), 100));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_eq!(GameModule::game_info(1).unwrap().property.id, 147229391);
	});
}

#[test]
fn play_game_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::game_properties().len(), 4);
		assert_noop!(
			GameModule::play_game(
				RuntimeOrigin::signed([0; 32].into()),
				crate::DifficultyLevel::Player,
			),
			Error::<Test>::UserNotRegistered
		);
	});
}

#[test]
fn play_game_fails_no_active_round() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[0; 32].into()
		));
		assert_noop!(
			GameModule::play_game(
				RuntimeOrigin::signed([0; 32].into()),
				crate::DifficultyLevel::Practice,
			),
			Error::<Test>::NoActiveRound
		);
	});
}

#[test]
fn play_game_fails_not_enough_points() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::game_properties().len(), 4);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[0; 32].into()
		));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 10, 1));
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
	});
}

#[test]
fn submit_answer_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::game_properties().len(), 4);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[0; 32].into()
		));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 223_000, 1));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [0; 32].into(), game_id: 1, guess: 223_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			223_000,
			1,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		System::assert_last_event(Event::ResultChecked { game_id: 1, secret: "nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap(), points: 25, won: true, nft_received: false }.into());
		assert_eq!(GameModule::game_info(0).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 80);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 2));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [0; 32].into(), game_id: 2, guess: 220_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			220_000,
			2,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		System::assert_last_event(Event::ResultChecked { game_id: 2, secret: "nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap(), points: 100, won: true, nft_received: true }.into());
		assert_eq!(GameModule::game_info(1).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 180);
		assert_eq!(Nfts::owner(0, 0).unwrap(), [0; 32].into());
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().nfts.xorange, 1);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 0, 3));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [0; 32].into(), game_id: 3, guess: 0 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			0,
			3,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 155);
		for x in 4..8 {
			assert_ok!(GameModule::play_game(
				RuntimeOrigin::signed([0; 32].into()),
				crate::DifficultyLevel::Player,
			));
			assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, x));
			assert_ok!(GameModule::check_result(
				RuntimeOrigin::root(),
				220_000,
				x,
				220_000,
				"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
			));
		}
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().nfts.xorange, 5);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 835);
	});
}

#[test]
fn submit_answer_works_with_high_difference() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::game_properties().len(), 4);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[0; 32].into()
		));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 2_000, 1));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [0; 32].into(), game_id: 1, guess: 2_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			2_000,
			1,
			220_000_000_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		System::assert_last_event(Event::ResultChecked { game_id: 1, secret: "nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap(), points: 25, won: false, nft_received: false }.into());
		assert_eq!(GameModule::game_info(0).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 30);
	});
}

/* #[test]
fn game_expires_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::game_properties().len(), 4);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[0; 32].into()
		));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		run_to_block(20);
		System::assert_last_event(Event::NoAnswer { game_id: 1, points: 25 }.into());
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 30);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 2));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [0; 32].into(), game_id: 2, guess: 220_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			220_000,
			2,
			223_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 55);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Pro,
		));
		run_to_block(30);
		System::assert_last_event(Event::NoAnswer { game_id: 3, points: 50 }.into());
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 5);
	});
} */

#[test]
fn leaderboard_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::game_properties().len(), 4);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[0; 32].into()
		));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[1; 32].into()
		));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[2; 32].into()
		));
		practise_round([0; 32].into(), 0);
		practise_round([1; 32].into(), 1);
		practise_round([2; 32].into(), 2);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 230_000, 3));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [0; 32].into(), game_id: 3, guess: 230_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			230_000,
			3,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		System::assert_last_event(Event::ResultChecked { game_id: 3, secret: "nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap(), points: 15, won: true, nft_received: false }.into());
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([1; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([1; 32].into()), 225_000, 4));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [1; 32].into(), game_id: 4, guess: 225_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			225_000,
			4,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([2; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([2; 32].into()), 220_000, 5));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [2; 32].into(), game_id: 5, guess: 220_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			220_000,
			5,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_eq!(GameModule::game_info(0).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([2; 32].into()).unwrap().points, 155);
		assert_eq!(GameModule::users::<AccountId>([1; 32].into()).unwrap().points, 80);
		assert_eq!(GameModule::users::<AccountId>([1; 32].into()).unwrap().wins, 1);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 70);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().wins, 1);
		assert_eq!(GameModule::leaderboard().len(), 3);
		assert_eq!(GameModule::leaderboard()[0], ([2; 32].into(), 155));
		assert_eq!(GameModule::leaderboard()[1], ([1; 32].into(), 80));
		assert_eq!(GameModule::leaderboard()[2], ([0; 32].into(), 70));
	});
}

#[test]
fn submit_answer_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::game_properties().len(), 4);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[0; 32].into()
		));
		practise_round([0; 32].into(), 0);
		assert_noop!(
			GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 223_000, 1),
			Error::<Test>::NoActiveGame
		);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			230_000,
			1,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_noop!(
			GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 230_000, 1),
			Error::<Test>::NoActiveGame
		);
	});
}

#[test]
fn transfer_of_nft_does_not_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::game_properties().len(), 4);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[0; 32].into()
		));
		assert_ok!(GameModule::give_points(RuntimeOrigin::root(), [0; 32].into(), 100));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 1));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [0; 32].into(), game_id: 1, guess: 220_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			220_000,
			1,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_eq!(GameModule::game_info(1).is_none(), true);
		assert_eq!(Nfts::owner(0, 0).unwrap(), [0; 32].into());
		assert_noop!(
			Nfts::transfer(
				RuntimeOrigin::signed([0; 32].into()),
				0,
				0,
				sp_runtime::MultiAddress::Id([1; 32].into())
			),
			DispatchError::Module(ModuleError {
				index: 3,
				error: [12, 0, 0, 0],
				message: Some("ItemLocked")
			})
		);
	});
}

#[test]
fn list_nft_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::game_properties().len(), 4);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[0; 32].into()
		));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 1));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [0; 32].into(), game_id: 1, guess: 220_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			220_000,
			1,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_eq!(GameModule::game_info(1).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 155);
		assert_eq!(Nfts::owner(0, 0).unwrap(), [0; 32].into());
		assert_ok!(GameModule::list_nft(RuntimeOrigin::signed([0; 32].into()), 0, 0,));
		assert_eq!(Nfts::owner(0, 0).unwrap(), GameModule::account_id());
		assert_eq!(GameModule::listings(0).unwrap().owner, [0; 32].into());
	});
}

#[test]
fn list_nft_doesnt_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::game_properties().len(), 4);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[0; 32].into()
		));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 1));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [0; 32].into(), game_id: 1, guess: 220_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			220_000,
			1,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_eq!(GameModule::game_info(1).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 155);
		assert_eq!(Nfts::owner(0, 0).unwrap(), [0; 32].into());
		assert_noop!(
			GameModule::list_nft(RuntimeOrigin::signed([1; 32].into()), 0, 0,),
			Error::<Test>::NoPermission
		);
	});
}

#[test]
fn delist_nft_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::game_properties().len(), 4);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[0; 32].into()
		));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 1));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [0; 32].into(), game_id: 1, guess: 220_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			220_000,
			1,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_eq!(GameModule::game_info(0).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 155);
		assert_eq!(Nfts::owner(0, 0).unwrap(), [0; 32].into());
		assert_ok!(GameModule::list_nft(RuntimeOrigin::signed([0; 32].into()), 0, 0,));
		assert_eq!(Nfts::owner(0, 0).unwrap(), GameModule::account_id());
		assert_eq!(GameModule::listings(0).unwrap().owner, [0; 32].into());
		assert_ok!(GameModule::delist_nft(RuntimeOrigin::signed([0; 32].into()), 0,));
		assert_noop!(
			Nfts::transfer(
				RuntimeOrigin::signed([0; 32].into()),
				0,
				0,
				sp_runtime::MultiAddress::Id([1; 32].into())
			),
			DispatchError::Module(ModuleError {
				index: 3,
				error: [12, 0, 0, 0],
				message: Some("ItemLocked")
			})
		);
	});
}

#[test]
fn delist_nft_doesnt_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::game_properties().len(), 4);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[0; 32].into()
		));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 1));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [0; 32].into(), game_id: 1, guess: 220_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			220_000,
			1,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_eq!(GameModule::game_info(0).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 155);
		assert_eq!(Nfts::owner(0, 0).unwrap(), [0; 32].into());
		assert_ok!(GameModule::list_nft(RuntimeOrigin::signed([0; 32].into()), 0, 0,));
		assert_eq!(Nfts::owner(0, 0).unwrap(), GameModule::account_id());
		assert_eq!(GameModule::listings(0).unwrap().owner, [0; 32].into());
		assert_noop!(
			GameModule::delist_nft(RuntimeOrigin::signed([1; 32].into()), 0,),
			Error::<Test>::NoPermission
		);
	});
}

#[test]
fn make_offer_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::game_properties().len(), 4);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[0; 32].into()
		));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[1; 32].into()
		));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 1));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [0; 32].into(), game_id: 1, guess: 220_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			220_000,
			1,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_eq!(GameModule::game_info(0).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 155);
		assert_eq!(Nfts::owner(0, 0).unwrap(), [0; 32].into());
		practise_round([1; 32].into(), 2);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([1; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([1; 32].into()), 220_000, 3));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [1; 32].into(), game_id: 3, guess: 220_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			220_000,
			3,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_eq!(Nfts::owner(0, 1).unwrap(), [1; 32].into());
		assert_ok!(GameModule::list_nft(RuntimeOrigin::signed([0; 32].into()), 0, 0,));
		assert_eq!(Nfts::owner(0, 0).unwrap(), GameModule::account_id());
		assert_eq!(GameModule::listings(0).unwrap().owner, [0; 32].into());
		assert_ok!(GameModule::make_offer(RuntimeOrigin::signed([1; 32].into()), 0, 0, 1,));
		assert_eq!(GameModule::offers(0).unwrap().owner, [1; 32].into());
	});
}

#[test]
fn make_offer_doesnt_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::game_properties().len(), 4);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[0; 32].into()
		));
		assert_ok!(GameModule::give_points(RuntimeOrigin::root(), [0; 32].into(), 100));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 1));
		assert_noop!(
			GameModule::make_offer(RuntimeOrigin::signed([1; 32].into()), 0, 0, 0,),
			Error::<Test>::ListingDoesNotExist
		);
	});
}

#[test]
fn withdraw_offer_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::game_properties().len(), 4);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[0; 32].into()
		));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[1; 32].into()
		));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 1));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [0; 32].into(), game_id: 1, guess: 220_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			220_000,
			1,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_eq!(GameModule::game_info(0).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 155);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().wins, 1);
		assert_eq!(Nfts::owner(0, 0).unwrap(), [0; 32].into());
		practise_round([1; 32].into(), 2);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([1; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([1; 32].into()), 220_000, 3));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [1; 32].into(), game_id: 3, guess: 220_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			220_000,
			3,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_eq!(Nfts::owner(0, 1).unwrap(), [1; 32].into());
		assert_ok!(GameModule::list_nft(RuntimeOrigin::signed([0; 32].into()), 0, 0,));
		assert_eq!(Nfts::owner(0, 0).unwrap(), GameModule::account_id());
		assert_eq!(GameModule::listings(0).unwrap().owner, [0; 32].into());
		assert_ok!(GameModule::make_offer(RuntimeOrigin::signed([1; 32].into()), 0, 0, 1,));
		assert_eq!(GameModule::offers(0).unwrap().owner, [1; 32].into());
		assert_ok!(GameModule::withdraw_offer(RuntimeOrigin::signed([1; 32].into()), 0));
		assert_eq!(GameModule::offers(0).is_none(), true);
	});
}

#[test]
fn withdraw_offer_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::game_properties().len(), 4);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[0; 32].into()
		));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[1; 32].into()
		));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 1));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [0; 32].into(), game_id: 1, guess: 220_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			220_000,
			1,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_eq!(GameModule::game_info(0).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 155);
		assert_eq!(Nfts::owner(0, 0).unwrap(), [0; 32].into());
		practise_round([1; 32].into(), 2);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([1; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([1; 32].into()), 220_000, 3));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [1; 32].into(), game_id: 3, guess: 220_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			220_000,
			3,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_eq!(Nfts::owner(0, 1).unwrap(), [1; 32].into());
		assert_ok!(GameModule::list_nft(RuntimeOrigin::signed([0; 32].into()), 0, 0,));
		assert_eq!(Nfts::owner(0, 0).unwrap(), GameModule::account_id());
		assert_eq!(GameModule::listings(0).unwrap().owner, [0; 32].into());
		assert_noop!(
			GameModule::withdraw_offer(RuntimeOrigin::signed([0; 32].into()), 0),
			Error::<Test>::OfferDoesNotExist
		);
		assert_ok!(GameModule::make_offer(RuntimeOrigin::signed([1; 32].into()), 0, 0, 1,));
		assert_eq!(GameModule::offers(0).unwrap().owner, [1; 32].into());
		assert_noop!(
			GameModule::withdraw_offer(RuntimeOrigin::signed([0; 32].into()), 0),
			Error::<Test>::NoPermission
		);
	});
}

#[test]
fn handle_offer_accept_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::game_properties().len(), 4);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[0; 32].into()
		));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[1; 32].into()
		));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 1));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [0; 32].into(), game_id: 1, guess: 220_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			220_000,
			1,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 155);
		assert_eq!(GameModule::game_info(0).is_none(), true);
		assert_eq!(Nfts::owner(0, 0).unwrap(), [0; 32].into());
		practise_round([1; 32].into(), 2);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([1; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([1; 32].into()), 220_000, 3));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [1; 32].into(), game_id: 3, guess: 220_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			220_000,
			3,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 4));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [0; 32].into(), game_id: 4, guess: 220_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			220_000,
			4,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 275);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 5));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [0; 32].into(), game_id: 5, guess: 220_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			220_000,
			5,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().nfts.xorange, 3);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 495);
		assert_eq!(GameModule::users::<AccountId>([1; 32].into()).unwrap().nfts.xorange, 1);
		assert_eq!(Nfts::owner(0, 1).unwrap(), [1; 32].into());
		assert_ok!(GameModule::list_nft(RuntimeOrigin::signed([0; 32].into()), 0, 0,));
		assert_eq!(Nfts::owner(0, 0).unwrap(), GameModule::account_id());
		assert_eq!(GameModule::listings(0).unwrap().owner, [0; 32].into());
		assert_ok!(GameModule::make_offer(RuntimeOrigin::signed([1; 32].into()), 0, 0, 1,));
		assert_eq!(GameModule::offers(0).unwrap().owner, [1; 32].into());
		assert_ok!(GameModule::handle_offer(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			crate::Offer::Accept,
		));
		assert_eq!(Nfts::owner(0, 0).unwrap(), [1; 32].into());
		assert_eq!(Nfts::owner(0, 1).unwrap(), [0; 32].into());
		assert_eq!(GameModule::offers(0).is_none(), true);
		assert_eq!(GameModule::listings(0).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().nfts.xorange, 3);
		assert_eq!(GameModule::users::<AccountId>([1; 32].into()).unwrap().nfts.xorange, 1);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 495);
		assert_eq!(GameModule::users::<AccountId>([1; 32].into()).unwrap().points, 155);
		assert_noop!(
			Nfts::transfer(
				RuntimeOrigin::signed([0; 32].into()),
				0,
				1,
				sp_runtime::MultiAddress::Id([1; 32].into())
			),
			DispatchError::Module(ModuleError {
				index: 3,
				error: [12, 0, 0, 0],
				message: Some("ItemLocked")
			})
		);
		assert_noop!(
			Nfts::transfer(
				RuntimeOrigin::signed([1; 32].into()),
				0,
				0,
				sp_runtime::MultiAddress::Id([1; 32].into())
			),
			DispatchError::Module(ModuleError {
				index: 3,
				error: [12, 0, 0, 0],
				message: Some("ItemLocked")
			})
		);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		System::assert_last_event(
			Event::GameStarted { player: [0; 32].into(), game_id: 6 }.into(),
		);
		run_to_block(20);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			220_000,
			6,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().nfts.xorange, 3);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 470);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().wins, 3);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().losses, 1);
	});
}

#[test]
fn handle_offer_reject_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::game_properties().len(), 4);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[0; 32].into()
		));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[1; 32].into()
		));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 1));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [0; 32].into(), game_id: 1, guess: 220_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			220_000,
			1,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_eq!(GameModule::game_info(0).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 155);
		assert_eq!(Nfts::owner(0, 0).unwrap(), [0; 32].into());
		practise_round([1; 32].into(), 2);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([1; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([1; 32].into()), 220_000, 3));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [1; 32].into(), game_id: 3, guess: 220_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			220_000,
			3,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_eq!(Nfts::owner(0, 1).unwrap(), [1; 32].into());
		assert_ok!(GameModule::list_nft(RuntimeOrigin::signed([0; 32].into()), 0, 0,));
		assert_eq!(Nfts::owner(0, 0).unwrap(), GameModule::account_id());
		assert_eq!(GameModule::listings(0).unwrap().owner, [0; 32].into());
		assert_ok!(GameModule::make_offer(RuntimeOrigin::signed([1; 32].into()), 0, 0, 1,));
		assert_eq!(GameModule::offers(0).unwrap().owner, [1; 32].into());
		assert_ok!(GameModule::handle_offer(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			crate::Offer::Reject,
		));
		assert_eq!(Nfts::owner(0, 1).unwrap(), [1; 32].into());
		assert_eq!(GameModule::offers(0).is_none(), true);
		assert_eq!(GameModule::listings(0).is_some(), true);
		assert_noop!(
			Nfts::transfer(
				RuntimeOrigin::signed([1; 32].into()),
				0,
				1,
				sp_runtime::MultiAddress::Id([1; 32].into())
			),
			DispatchError::Module(ModuleError {
				index: 3,
				error: [12, 0, 0, 0],
				message: Some("ItemLocked")
			})
		);
	});
}

#[test]
fn handle_offer_doesnt_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::game_properties().len(), 4);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[0; 32].into()
		));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[1; 32].into()
		));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 1));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [0; 32].into(), game_id: 1, guess: 220_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			220_000,
			1,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_eq!(GameModule::game_info(0).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 155);
		assert_eq!(Nfts::owner(0, 0).unwrap(), [0; 32].into());
		practise_round([1; 32].into(), 2);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([1; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([1; 32].into()), 220_000, 3));
		System::assert_last_event(
			Event::AnswerSubmitted { player: [1; 32].into(), game_id: 3, guess: 220_000 }.into(),
		);
		assert_ok!(GameModule::check_result(
			RuntimeOrigin::root(),
			220_000,
			3,
			220_000,
			"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
		));
		assert_eq!(Nfts::owner(0, 1).unwrap(), [1; 32].into());
		assert_ok!(GameModule::list_nft(RuntimeOrigin::signed([0; 32].into()), 0, 0,));
		assert_eq!(Nfts::owner(0, 0).unwrap(), GameModule::account_id());
		assert_eq!(GameModule::listings(0).unwrap().owner, [0; 32].into());
		assert_noop!(
			GameModule::handle_offer(
				RuntimeOrigin::signed([0; 32].into()),
				0,
				crate::Offer::Reject,
			),
			Error::<Test>::OfferDoesNotExist
		);
	});
}

#[test]
fn play_multiple_rounds_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::game_properties().len(), 4);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[0; 32].into()
		));
		practise_round([0; 32].into(), 0);
		for x in 1..=20 {
			assert_ok!(GameModule::play_game(
				RuntimeOrigin::signed([0; 32].into()),
				crate::DifficultyLevel::Player,
			));
			assert_ok!(GameModule::submit_answer(
				RuntimeOrigin::signed([0; 32].into()),
				217_000,
				x
			));
			System::assert_last_event(
				Event::AnswerSubmitted { player: [0; 32].into(), game_id: x, guess: 217_000 }
					.into(),
			);
			assert_ok!(GameModule::check_result(
				RuntimeOrigin::root(),
				217_000,
				x,
				220_000,
				"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
			));
		}
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_eq!(GameModule::game_info(21).is_some(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 555);
	});
}

#[test]
fn add_property_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		let new_property = PropertyInfoData {
			id: 147031382,
			data: "nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap(),
		};
		assert_ok!(GameModule::add_property(RuntimeOrigin::root(), new_property));
		assert_eq!(GameModule::game_properties().len(), 5);
	});
}

#[test]
fn remove_property_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_ok!(GameModule::remove_property(RuntimeOrigin::root(), 146480642));
		assert_eq!(GameModule::game_properties().len(), 3);
	});
}

#[test]
fn request_token_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[0; 32].into()
		));
		System::set_block_number(100802);
		assert_ok!(GameModule::request_token(RuntimeOrigin::signed([0; 32].into())));
		assert_eq!(Balances::free_balance(&([0; 32].into())), 10);
	});
}

#[test]
fn request_token_doesnt_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_noop!(
			GameModule::request_token(RuntimeOrigin::signed([0; 32].into())),
			Error::<Test>::UserNotRegistered
		);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[0; 32].into()
		));
		assert_noop!(
			GameModule::request_token(RuntimeOrigin::signed([0; 32].into())),
			Error::<Test>::CantRequestToken
		);
	});
}

#[test]
fn check_result_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::game_properties().len(), 4);
		assert_ok!(GameModule::add_to_admins(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(GameModule::register_user(
			RuntimeOrigin::signed([4; 32].into()),
			[0; 32].into()
		));
		practise_round([0; 32].into(), 0);
		assert_noop!(
			GameModule::check_result(
				RuntimeOrigin::root(),
				223_000,
				0,
				220_000,
				"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
			),
			Error::<Test>::NoActiveGame
		);
		assert_noop!(
			GameModule::check_result(
				RuntimeOrigin::root(),
				223_000,
				1,
				220_000,
				"nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap()
			),
			Error::<Test>::NoActiveGame
		);
	});
}
