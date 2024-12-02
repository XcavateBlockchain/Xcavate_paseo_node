use crate::*;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;

impl<T: Config> Pallet<T> {
	/// Get the account id of the pallet
	pub fn account_id() -> AccountIdOf<T> {
		<T as pallet::Config>::PalletId::get().into_account_truncating()
	}

	/// checks if the signer has enough points to start a game.
	pub fn check_enough_points(
		signer: AccountIdOf<T>,
		game_type: DifficultyLevel,
	) -> DispatchResult {
		if game_type == DifficultyLevel::Pro {
			ensure!(
				Self::users(signer.clone())
					.ok_or(Error::<T>::UserNotRegistered)?
					.practise_rounds > 0,
				Error::<T>::NoPractise
			);
			ensure!(
				Self::users(signer).ok_or(Error::<T>::UserNotRegistered)?.points >= 50,
				Error::<T>::NotEnoughPoints
			);
		} else if game_type == DifficultyLevel::Player {
			ensure!(
				Self::users(signer.clone())
					.ok_or(Error::<T>::UserNotRegistered)?
					.practise_rounds > 0,
				Error::<T>::NoPractise
			);
			ensure!(
				Self::users(signer).ok_or(Error::<T>::UserNotRegistered)?.points >= 25,
				Error::<T>::NotEnoughPoints
			);
		} else {
			ensure!(
				Self::users(signer).ok_or(Error::<T>::UserNotRegistered)?.practise_rounds < 5,
				Error::<T>::TooManyPractise
			);
		}
		Ok(())
	}

	/// checks the answer and distributes the rewards accordingly.
	pub fn do_check_result(difference: u64, game_id: u32, game_info: GameData<T>, secret: BoundedVec<u8, <T as Config>::StringLimit> ) -> DispatchResult {
		ensure!(game_info.guess.is_some(), Error::<T>::NoGuess);
		if game_info.difficulty == DifficultyLevel::Pro {
			match difference {
				0..=10 => {
					let (hashi, _) = T::GameRandomness::random(&[game_id as u8]);
					let u32_value = u32::from_le_bytes(
						hashi.as_ref()[4..8].try_into().map_err(|_| Error::<T>::ConversionError)?,
					);
					let random_number = (u32_value % 8)
						.checked_add(
							8 * (Self::current_round()
								.checked_sub(1)
								.ok_or(Error::<T>::ArithmeticUnderflow)?),
						)
						.ok_or(Error::<T>::ArithmeticOverflow)?;
					let collection_id: <T as pallet::Config>::CollectionId = random_number.into();
					let next_item_id = NextColorId::<T>::get(collection_id);
					let item_id: ItemId<T> = next_item_id.into();
					let next_item_id =
						next_item_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
					NextColorId::<T>::insert(collection_id, next_item_id);
					pallet_nfts::Pallet::<T>::do_mint(
						collection_id.into(),
						item_id.into(),
						Some(Self::account_id()),
						game_info.player.clone(),
						Self::default_item_config(),
						|_, _| Ok(()),
					)?;
					let pallet_origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
					pallet_nfts::Pallet::<T>::lock_item_transfer(
						pallet_origin,
						collection_id.into(),
						item_id.into(),
					)?;
					let mut user = Self::users(game_info.player.clone())
						.ok_or(Error::<T>::UserNotRegistered)?;
					let color = Self::collection_color(collection_id)
						.ok_or(Error::<T>::CollectionUnknown)?;
					user.add_nft_color(color.clone())?;
					let points = user.calculate_points(color);
					user.points =
						user.points.checked_add(points).ok_or(Error::<T>::ArithmeticOverflow)?;
					user.wins = user.wins.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
					Users::<T>::insert(game_info.player.clone(), user.clone());
					if user.has_four_of_all_colors() {
						Self::end_game(game_info.player.clone())?;
					}
					Self::deposit_event(Event::<T>::ResultChecked { game_id, secret, points, won: true, nft_received: true });
				},
				11..=30 => {
					let mut user = Self::users(game_info.player.clone())
						.ok_or(Error::<T>::UserNotRegistered)?;
					user.points =
						user.points.checked_add(50).ok_or(Error::<T>::ArithmeticOverflow)?;
					user.wins = user.wins.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
					Users::<T>::insert(game_info.player.clone(), user);
					Self::deposit_event(Event::<T>::ResultChecked { game_id, secret, points: 50, won: true, nft_received: false });
				},
				31..=50 => {
					let mut user = Self::users(game_info.player.clone())
						.ok_or(Error::<T>::UserNotRegistered)?;
					user.points =
						user.points.checked_add(30).ok_or(Error::<T>::ArithmeticOverflow)?;
					user.wins = user.wins.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
					Users::<T>::insert(game_info.player.clone(), user);
					Self::deposit_event(Event::<T>::ResultChecked { game_id, secret, points: 30, won: true, nft_received: false });
				},
				51..=100 => {
					let mut user = Self::users(game_info.player.clone())
						.ok_or(Error::<T>::UserNotRegistered)?;
					user.points =
						user.points.checked_add(10).ok_or(Error::<T>::ArithmeticOverflow)?;
					user.wins = user.wins.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
					Users::<T>::insert(game_info.player.clone(), user);
					Self::deposit_event(Event::<T>::ResultChecked { game_id, secret, points: 10, won: true, nft_received: false });
				},
				101..=150 => {
					let mut user = Self::users(game_info.player.clone())
						.ok_or(Error::<T>::UserNotRegistered)?;
					user.points =
						user.points.checked_sub(10).ok_or(Error::<T>::ArithmeticUnderflow)?;
					user.losses = user.losses.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
					Users::<T>::insert(game_info.player.clone(), user);
					Self::deposit_event(Event::<T>::ResultChecked { game_id, secret, points: 10, won: false, nft_received: false });
				},
				151..=200 => {
					let mut user = Self::users(game_info.player.clone())
						.ok_or(Error::<T>::UserNotRegistered)?;
					user.points =
						user.points.checked_sub(20).ok_or(Error::<T>::ArithmeticUnderflow)?;
					user.losses = user.losses.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
					Users::<T>::insert(game_info.player.clone(), user);
					Self::deposit_event(Event::<T>::ResultChecked { game_id, secret, points: 20, won: false, nft_received: false });
				},
				201..=250 => {
					let mut user = Self::users(game_info.player.clone())
						.ok_or(Error::<T>::UserNotRegistered)?;
					user.points =
						user.points.checked_sub(30).ok_or(Error::<T>::ArithmeticUnderflow)?;
					user.losses = user.losses.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
					Users::<T>::insert(game_info.player.clone(), user);
					Self::deposit_event(Event::<T>::ResultChecked { game_id, secret, points: 30, won: false, nft_received: false });
				},
				251..=300 => {
					let mut user = Self::users(game_info.player.clone())
						.ok_or(Error::<T>::UserNotRegistered)?;
					user.points =
						user.points.checked_sub(40).ok_or(Error::<T>::ArithmeticUnderflow)?;
					user.losses = user.losses.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
					Users::<T>::insert(game_info.player.clone(), user);
					Self::deposit_event(Event::<T>::ResultChecked { game_id, secret, points: 40, won: false, nft_received: false });
				},
				_ => {
					let mut user = Self::users(game_info.player.clone())
						.ok_or(Error::<T>::UserNotRegistered)?;
					user.points =
						user.points.checked_sub(50).ok_or(Error::<T>::ArithmeticUnderflow)?;
					user.losses = user.losses.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
					Users::<T>::insert(game_info.player.clone(), user);
					Self::deposit_event(Event::<T>::ResultChecked { game_id, secret, points: 50, won: false, nft_received: false });
				},
			}
		} else if game_info.difficulty == DifficultyLevel::Player {
			match difference {
				0..=10 => {
					let (hashi, _) = T::GameRandomness::random(&[game_id as u8]);
					let u32_value = u32::from_le_bytes(
						hashi.as_ref()[4..8].try_into().map_err(|_| Error::<T>::ConversionError)?,
					);
					let random_number = (u32_value % 8)
						.checked_add(
							8 * (Self::current_round()
								.checked_sub(1)
								.ok_or(Error::<T>::ArithmeticUnderflow)?),
						)
						.ok_or(Error::<T>::ArithmeticOverflow)?;
					let collection_id: <T as pallet::Config>::CollectionId = random_number.into();
					let next_item_id = NextColorId::<T>::get(collection_id);
					let item_id: ItemId<T> = next_item_id.into();
					let next_item_id =
						next_item_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
					NextColorId::<T>::insert(collection_id, next_item_id);
					pallet_nfts::Pallet::<T>::do_mint(
						collection_id.into(),
						item_id.into(),
						Some(Self::account_id()),
						game_info.player.clone(),
						Self::default_item_config(),
						|_, _| Ok(()),
					)?;
					let pallet_origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
					pallet_nfts::Pallet::<T>::lock_item_transfer(
						pallet_origin,
						collection_id.into(),
						item_id.into(),
					)?;
					let mut user = Self::users(game_info.player.clone())
						.ok_or(Error::<T>::UserNotRegistered)?;
					let color = Self::collection_color(collection_id)
						.ok_or(Error::<T>::CollectionUnknown)?;
					user.add_nft_color(color.clone())?;
					let points = user.calculate_points(color);
					user.points =
						user.points.checked_add(points).ok_or(Error::<T>::ArithmeticOverflow)?;
					user.wins = user.wins.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
					Users::<T>::insert(game_info.player.clone(), user.clone());
					if user.has_four_of_all_colors() {
						Self::end_game(game_info.player.clone())?;
					}
					Self::deposit_event(Event::<T>::ResultChecked { game_id, secret, points, won: true, nft_received: true });
				},
				11..=30 => {
					let mut user = Self::users(game_info.player.clone())
						.ok_or(Error::<T>::UserNotRegistered)?;
					user.points =
						user.points.checked_add(25).ok_or(Error::<T>::ArithmeticUnderflow)?;
					user.wins = user.wins.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
					Users::<T>::insert(game_info.player.clone(), user);
					Self::deposit_event(Event::<T>::ResultChecked { game_id, secret, points: 25, won: true, nft_received: false });
				},
				31..=50 => {
					let mut user = Self::users(game_info.player.clone())
						.ok_or(Error::<T>::UserNotRegistered)?;
					user.points =
						user.points.checked_add(15).ok_or(Error::<T>::ArithmeticUnderflow)?;
					user.wins = user.wins.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
					Users::<T>::insert(game_info.player.clone(), user);
					Self::deposit_event(Event::<T>::ResultChecked { game_id, secret, points: 15, won: true, nft_received: false });
				},
				51..=100 => {
					let mut user = Self::users(game_info.player.clone())
						.ok_or(Error::<T>::UserNotRegistered)?;
					user.points =
						user.points.checked_add(5).ok_or(Error::<T>::ArithmeticUnderflow)?;
					user.wins = user.wins.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
					Users::<T>::insert(game_info.player.clone(), user);
					Self::deposit_event(Event::<T>::ResultChecked { game_id, secret, points: 5, won: true, nft_received: false });
				},
				101..=150 => {
					let mut user = Self::users(game_info.player.clone())
						.ok_or(Error::<T>::UserNotRegistered)?;
					user.points =
						user.points.checked_sub(5).ok_or(Error::<T>::ArithmeticUnderflow)?;
					user.losses = user.losses.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
					Users::<T>::insert(game_info.player.clone(), user);
					Self::deposit_event(Event::<T>::ResultChecked { game_id, secret, points: 5, won: false, nft_received: false });
				},
				151..=200 => {
					let mut user = Self::users(game_info.player.clone())
						.ok_or(Error::<T>::UserNotRegistered)?;
					user.points =
						user.points.checked_sub(10).ok_or(Error::<T>::ArithmeticUnderflow)?;
					user.losses = user.losses.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
					Users::<T>::insert(game_info.player.clone(), user);
					Self::deposit_event(Event::<T>::ResultChecked { game_id, secret, points: 10, won: false, nft_received: false });
				},
				201..=250 => {
					let mut user = Self::users(game_info.player.clone())
						.ok_or(Error::<T>::UserNotRegistered)?;
					user.points =
						user.points.checked_sub(15).ok_or(Error::<T>::ArithmeticUnderflow)?;
					user.losses = user.losses.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
					Users::<T>::insert(game_info.player.clone(), user);
					Self::deposit_event(Event::<T>::ResultChecked { game_id, secret, points: 15, won: false, nft_received: false });
				},
				251..=300 => {
					let mut user = Self::users(game_info.player.clone())
						.ok_or(Error::<T>::UserNotRegistered)?;
					user.points =
						user.points.checked_sub(20).ok_or(Error::<T>::ArithmeticUnderflow)?;
					user.losses = user.losses.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
					Users::<T>::insert(game_info.player.clone(), user);
					Self::deposit_event(Event::<T>::ResultChecked { game_id, secret, points: 20, won: false, nft_received: false });
				},
				_ => {
					let mut user = Self::users(game_info.player.clone())
						.ok_or(Error::<T>::UserNotRegistered)?;
					user.points =
						user.points.checked_sub(25).ok_or(Error::<T>::ArithmeticUnderflow)?;
					user.losses = user.losses.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
					Users::<T>::insert(game_info.player.clone(), user);
					Self::deposit_event(Event::<T>::ResultChecked { game_id, secret, points: 25, won: false, nft_received: false });
				},
			}
		} else {
			let mut user =
				Self::users(game_info.player.clone()).ok_or(Error::<T>::UserNotRegistered)?;
			user.points = user.points.checked_add(5).ok_or(Error::<T>::ArithmeticUnderflow)?;
			user.practise_rounds =
				user.practise_rounds.checked_add(1).ok_or(Error::<T>::ArithmeticUnderflow)?;
			Users::<T>::insert(game_info.player.clone(), user);
			Self::deposit_event(Event::<T>::ResultChecked { game_id, secret, points: 5, won: true, nft_received: false });
		}
		let user = Self::users(game_info.player.clone()).ok_or(Error::<T>::UserNotRegistered)?;
		Self::update_leaderboard(game_info.player, user.points)?;
		Ok(())
	}

	pub fn update_leaderboard(user_id: AccountIdOf<T>, new_points: u32) -> DispatchResult {
		let mut leaderboard = Self::leaderboard();
		let leaderboard_size = leaderboard.len();

		if let Some((_, user_points)) = leaderboard.iter_mut().find(|(id, _)| *id == user_id) {
			*user_points = new_points;
			leaderboard.sort_by(|a, b| b.1.cmp(&a.1));
			Leaderboard::<T>::put(leaderboard);
			return Ok(());
		}
		if new_points > 0 &&
			(leaderboard_size < 10 ||
				new_points > leaderboard.last().map(|(_, points)| *points).unwrap_or(0))
		{
			if leaderboard.len() >= 10 {
				leaderboard.pop();
			}
			leaderboard
				.try_push((user_id, new_points))
				.map_err(|_| Error::<T>::InvalidIndex)?;
			leaderboard.sort_by(|a, b| b.1.cmp(&a.1));
			Leaderboard::<T>::put(leaderboard);
		}
		Ok(())
	}

	pub fn swap_user_points(
		nft_holder: AccountIdOf<T>,
		collection_id_add: CollectionId<T>,
		collection_id_sub: CollectionId<T>,
	) -> DispatchResult {
		let mut user = Self::users(nft_holder.clone()).ok_or(Error::<T>::UserNotRegistered)?;
		let color_add =
			Self::collection_color(collection_id_add).ok_or(Error::<T>::CollectionUnknown)?;
		let color_sub =
			Self::collection_color(collection_id_sub).ok_or(Error::<T>::CollectionUnknown)?;
		user.add_nft_color(color_add.clone())?;
		let points = user.calculate_points(color_add);
		user.points = user.points.checked_add(points).ok_or(Error::<T>::ArithmeticOverflow)?;
		user.sub_nft_color(color_sub.clone())?;
		let points = user.subtracting_calculate_points(color_sub);
		user.points = user.points.checked_sub(points).ok_or(Error::<T>::ArithmeticOverflow)?;
		Users::<T>::insert(nft_holder.clone(), user.clone());
		Self::update_leaderboard(nft_holder.clone(), user.points)?;
		if user.has_four_of_all_colors() {
			Self::end_game(nft_holder)?;
		}
		Ok(())
	}

	/// Handles the case if the player did not answer on time.
	pub fn no_answer_result(game_info: GameData<T>, game_id: u32) -> DispatchResult {
		if game_info.difficulty == DifficultyLevel::Pro {
			let mut user =
				Self::users(game_info.player.clone()).ok_or(Error::<T>::UserNotRegistered)?;
			user.points = user.points.checked_sub(50).ok_or(Error::<T>::ArithmeticUnderflow)?;
			user.losses = user.losses.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
			Users::<T>::insert(game_info.player.clone(), user);
			Self::deposit_event(Event::<T>::NoAnswer { game_id, points: 50 });
		} else if game_info.difficulty == DifficultyLevel::Player {
			let mut user =
				Self::users(game_info.player.clone()).ok_or(Error::<T>::UserNotRegistered)?;
			user.points = user.points.checked_sub(25).ok_or(Error::<T>::ArithmeticUnderflow)?;
			user.losses = user.losses.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
			Users::<T>::insert(game_info.player.clone(), user);
			Self::deposit_event(Event::<T>::NoAnswer { game_id, points: 25 });
		}
		Ok(())
	}

	pub fn end_game(winner: AccountIdOf<T>) -> DispatchResult {
		RoundActive::<T>::put(false);
		RoundChampion::<T>::insert(Self::current_round(), winner);
		Ok(())
	}

	/// Set the default collection configuration for creating a collection.
	pub fn default_collection_config(
	) -> CollectionConfig<BalanceOf<T>, BlockNumberFor<T>, <T as pallet_nfts::Config>::CollectionId>
	{
		Self::collection_config_from_disabled_settings(CollectionSetting::DepositRequired.into())
	}

	pub fn collection_config_from_disabled_settings(
		settings: BitFlags<CollectionSetting>,
	) -> CollectionConfig<BalanceOf<T>, BlockNumberFor<T>, <T as pallet_nfts::Config>::CollectionId>
	{
		CollectionConfig {
			settings: CollectionSettings::from_disabled(settings),
			max_supply: None,
			mint_settings: MintSettings::default(),
		}
	}

	/// Set the default item configuration for minting a nft.
	pub fn default_item_config() -> ItemConfig {
		ItemConfig { settings: ItemSettings::all_enabled() }
	}
}
