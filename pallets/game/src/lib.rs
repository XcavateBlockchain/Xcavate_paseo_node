#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;
pub mod functions;
pub mod properties;
pub mod types;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as pallet_nfts::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

use frame_support::{
	traits::{Currency, Incrementable, ReservableCurrency},
	PalletId,
};

use frame_support::sp_runtime::{
	traits::{AccountIdConversion, StaticLookup},
	Saturating,
};

use pallet_nfts::{
	CollectionConfig, CollectionSetting, CollectionSettings, ItemConfig, ItemSettings, MintSettings,
};

use frame_system::RawOrigin;

use enumflags2::BitFlags;

use frame_support::traits::Randomness;

pub use types::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_nfts::Config //+ pallet_babe::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The currency type.
		type Currency: Currency<AccountIdOf<Self>> + ReservableCurrency<AccountIdOf<Self>>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;
		/// Origin who can create a new game.
		type GameOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// Collection id type from pallet nfts.
		type CollectionId: IsType<<Self as pallet_nfts::Config>::CollectionId>
			+ Parameter
			+ From<u32>
			+ Ord
			+ Copy
			+ MaxEncodedLen
			+ Encode;
		/// Item id type from pallet nfts.
		type ItemId: IsType<<Self as pallet_nfts::Config>::ItemId>
			+ Parameter
			+ From<u32>
			+ Ord
			+ Copy
			+ MaxEncodedLen
			+ Encode;
		/// The maximum amount of properties.
		#[pallet::constant]
		type MaxProperty: Get<u32> + Clone + PartialEq + Eq;
		/// The marketplace's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
		/// The maximum amount of games that can be played at the same time.
		#[pallet::constant]
		type MaxOngoingGames: Get<u32>;
		/// Randomness used for choosing a random property.
		type GameRandomness: Randomness<Self::Hash, BlockNumberFor<Self>>;
		/// The maximum length of data stored in string.
		#[pallet::constant]
		type StringLimit: Get<u32>;
		/// The maximum length of leaderboard.
		#[pallet::constant]
		type LeaderboardLimit: Get<u32>;
		#[pallet::constant]
		type MaxAdmins: Get<u32>;
		/// The amount of time until player can request more token.
		type RequestLimit: Get<BlockNumberFor<Self>>;
	}

	pub type CollectionId<T> = <T as Config>::CollectionId;
	pub type ItemId<T> = <T as Config>::ItemId;

	/// The id of the current round.
	#[pallet::storage]
	#[pallet::getter(fn current_round)]
	pub(super) type CurrentRound<T> = StorageValue<_, u32, ValueQuery>;

	/// Bool if there is a round currently ongoing.
	#[pallet::storage]
	#[pallet::getter(fn round_active)]
	pub(super) type RoundActive<T> = StorageValue<_, bool, ValueQuery>;

	/// A mapping of a round to the winner of the round.
	#[pallet::storage]
	#[pallet::getter(fn round_champion)]
	pub(super) type RoundChampion<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, AccountIdOf<T>, OptionQuery>;

	/// The next item id in a collection.
	#[pallet::storage]
	pub(super) type NextColorId<T: Config> =
		StorageMap<_, Blake2_128Concat, <T as pallet::Config>::CollectionId, u32, ValueQuery>;

	/// Mapping of a collection to the correlated color.
	#[pallet::storage]
	#[pallet::getter(fn collection_color)]
	pub(super) type CollectionColor<T: Config> =
		StorageMap<_, Blake2_128Concat, <T as pallet::Config>::CollectionId, NftColor, OptionQuery>;

	/// The next id of listings.
	#[pallet::storage]
	pub(super) type NextListingId<T> = StorageValue<_, u32, ValueQuery>;

	/// The next id of offers.
	#[pallet::storage]
	#[pallet::getter(fn next_offer_id)]
	pub(super) type NextOfferId<T> = StorageValue<_, u32, ValueQuery>;

	/// The next id of game.
	#[pallet::storage]
	#[pallet::getter(fn game_id)]
	pub type GameId<T> = StorageValue<_, u32, ValueQuery>;

	/// The leaderboard of the game.
	#[pallet::storage]
	#[pallet::getter(fn leaderboard)]
	pub type Leaderboard<T> = StorageValue<
		_,
		BoundedVec<(AccountIdOf<T>, u32), <T as Config>::LeaderboardLimit>,
		ValueQuery,
	>;

	/// Mapping of an account id to the user data of the account.
	#[pallet::storage]
	#[pallet::getter(fn users)]
	pub type Users<T> = StorageMap<_, Blake2_128Concat, AccountIdOf<T>, User<T>, OptionQuery>;

	/// Mapping of game id to the game info.
	#[pallet::storage]
	#[pallet::getter(fn game_info)]
	pub type GameInfo<T: Config> = StorageMap<_, Blake2_128Concat, u32, GameData<T>, OptionQuery>;

	/// Mapping of listing id to the listing data.
	#[pallet::storage]
	#[pallet::getter(fn listings)]
	pub type Listings<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u32,
		ListingInfo<<T as pallet::Config>::CollectionId, <T as pallet::Config>::ItemId, T>,
		OptionQuery,
	>;

	/// Mapping of offer id to the offer data.
	#[pallet::storage]
	#[pallet::getter(fn offers)]
	pub type Offers<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u32,
		OfferInfo<<T as pallet::Config>::CollectionId, <T as pallet::Config>::ItemId, T>,
		OptionQuery,
	>;

	/// A List of test properties
	#[pallet::storage]
	#[pallet::getter(fn game_properties)]
	pub type GameProperties<T: Config> =
		StorageValue<_, BoundedVec<PropertyInfoData<T>, T::MaxProperty>, ValueQuery>;

	/// Vector of admins who can register users.
	#[pallet::storage]
	#[pallet::getter(fn admins)]
	pub type Admins<T: Config> =
		StorageValue<_, BoundedVec<AccountIdOf<T>, T::MaxAdmins>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A user has received points.
		PointsReceived { receiver: AccountIdOf<T>, amount: u32 },
		/// A game has started.
		GameStarted { player: AccountIdOf<T>, game_id: u32 },
		/// An answer has been submitted.
		AnswerSubmitted { player: AccountIdOf<T>, game_id: u32, guess: u32 },
		/// The result has been checked.
		ResultChecked { game_id: u32, secret: BoundedVec<u8, <T as Config>::StringLimit>, points: u32, won: bool, nft_received: bool },
		/// No Answer has been submitted.
		NoAnswer { game_id: u32, points: u32 },
		/// A nft has been listed.
		NftListed { owner: AccountIdOf<T>, collection_id: CollectionId<T>, item_id: ItemId<T> },
		/// A nft has been delisted.
		NftDelisted { owner: AccountIdOf<T>, collection_id: CollectionId<T>, item_id: ItemId<T> },
		/// An offer has been made.
		OfferMade {
			owner: AccountIdOf<T>,
			listing_id: u32,
			collection_id: CollectionId<T>,
			item_id: ItemId<T>,
		},
		/// An offer has been withdrawn.
		OfferWithdrawn { owner: AccountIdOf<T>, offer_id: u32 },
		/// An offer has been handled.
		OfferHandeld { offer_id: u32, offer: Offer },
		/// A new player has been registered.
		NewPlayerRegistered { player: AccountIdOf<T> },
		/// A new admins has been added.
		NewAdminAdded { new_admin: AccountIdOf<T> },
		/// An admin has been removed.
		AdminRemoved { admin: AccountIdOf<T> },
		/// The user received token.
		TokenReceived { player: AccountIdOf<T> },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// A player has not enough points to play.
		NotEnoughPoints,
		ConversionError,
		ArithmeticOverflow,
		ArithmeticUnderflow,
		MultiplyError,
		DivisionError,
		/// There are too many games active.
		TooManyGames,
		/// The caller has no permission.
		NoThePlayer,
		/// This is not an active game.
		NoActiveGame,
		NoPermission,
		/// This listing is not listed.
		ListingDoesNotExist,
		/// This offer does not exist.
		OfferDoesNotExist,
		/// There are too many test properties.
		TooManyTest,
		/// No property could be found.
		NoProperty,
		/// The user has not yet been registered.
		UserNotRegistered,
		/// The user has already made 5 practise rounds.
		TooManyPractise,
		/// The user has not yet made a practise round.
		NoPractise,
		InvalidIndex,
		/// The color for this collection is not known.
		CollectionUnknown,
		/// There is no active round at the moment.
		NoActiveRound,
		/// The player is already registered.
		PlayerAlreadyRegistered,
		/// The account is already an admin.
		AccountAlreadyAdmin,
		/// This account is not an admin.
		NotAdmin,
		/// There are already enough admins.
		TooManyAdmins,
		/// The user has to wait to request token.
		CantRequestToken,
		/// There has been no guess from the player.
		NoGuess,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates the setup for a new game.
		///
		/// The origin must be the sudo.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::setup_game())]
		pub fn setup_game(origin: OriginFor<T>) -> DispatchResult {
			T::GameOrigin::ensure_origin(origin)?;
			for x in 0..8 {
				if pallet_nfts::NextCollectionId::<T>::get().is_none() {
					pallet_nfts::NextCollectionId::<T>::set(
						<T as pallet_nfts::Config>::CollectionId::initial_value(),
					);
				};
				let collection_id = pallet_nfts::NextCollectionId::<T>::get().unwrap();
				let next_collection_id = collection_id.increment();
				pallet_nfts::NextCollectionId::<T>::set(next_collection_id);
				let collection_id: CollectionId<T> = collection_id.into();
				let pallet_id: AccountIdOf<T> =
					<T as pallet::Config>::PalletId::get().into_account_truncating();
				pallet_nfts::Pallet::<T>::do_create_collection(
					collection_id.into(),
					pallet_id.clone(),
					pallet_id.clone(),
					Self::default_collection_config(),
					T::CollectionDeposit::get(),
					pallet_nfts::Event::Created {
						creator: pallet_id.clone(),
						owner: pallet_id,
						collection: collection_id.into(),
					},
				)?;
				let color = NftColor::from_index(x).ok_or(Error::<T>::InvalidIndex)?;
				CollectionColor::<T>::insert(collection_id, color);
			}
			Self::create_game_properties()?;
			let mut round = CurrentRound::<T>::get();
			round = round.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
			CurrentRound::<T>::put(round);
			RoundActive::<T>::put(true);
			Ok(())
		}

		/// Registers a player and gives him initialy 50 points.
		///
		/// The origin must be the sudo.
		///
		/// Parameters:
		/// - `player`: The AccountId of the user who gets registered.
		///
		/// Emits `NewPlayerRegistered` event when succesfful.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::register_user())]
		pub fn register_user(origin: OriginFor<T>, player: AccountIdOf<T>) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			ensure!(Admins::<T>::get().contains(&signer), Error::<T>::NoPermission);
			ensure!(Users::<T>::get(player.clone()).is_none(), Error::<T>::PlayerAlreadyRegistered);
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			let next_request =
				current_block_number.saturating_add(<T as Config>::RequestLimit::get());
			let user = User {
				points: 50,
				wins: Default::default(),
				losses: Default::default(),
				practise_rounds: Default::default(),
				last_played_round: Default::default(),
				next_token_request: next_request,
				nfts: CollectedColors::default(),
			};
			<T as pallet::Config>::Currency::make_free_balance_be(
				&player,
				/*10000000000000 */10u64.try_into().map_err(|_| Error::<T>::ConversionError)?,
			);
			Users::<T>::insert(player.clone(), user);
			frame_system::Pallet::<T>::inc_providers(&player);
			Self::deposit_event(Event::<T>::NewPlayerRegistered { player });
			Ok(())
		}

		/// Gives points to a user.
		///
		/// The origin must be the sudo.
		///
		/// Parameters:
		/// - `receiver`: The AccountId of the user who gets points.
		///
		/// Emits `LocationCreated` event when succesfful.
		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::give_points())]
		pub fn give_points(
			origin: OriginFor<T>,
			receiver: AccountIdOf<T>,
			amount: u32,
		) -> DispatchResult {
			T::GameOrigin::ensure_origin(origin)?;
			let mut user =
				Users::<T>::get(receiver.clone()).ok_or(Error::<T>::UserNotRegistered)?;
			user.points = user.points.checked_add(amount).ok_or(Error::<T>::ArithmeticOverflow)?;
			Users::<T>::insert(receiver.clone(), user);
			Self::deposit_event(Event::<T>::PointsReceived { receiver, amount: 100 });
			Ok(())
		}

		/// Starts a game for the player.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `game_type`: The difficulty level of the game.
		///
		/// Emits `GameStarted` event when succesfful.
		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::play_game())]
		#[pallet::feeless_if(|origin: &OriginFor<T>, game_type: &DifficultyLevel| -> bool {
			if let Ok(signer) = ensure_signed(origin.clone()) {
				match game_type {
					DifficultyLevel::Pro => {
						if let Some(user) = Users::<T>::get(&signer) {
							if user.points >= 50 {
								return true;
							}
						}
					},
					DifficultyLevel::Player => {
						if let Some(user) = Users::<T>::get(&signer) {
							if user.points >= 25 {
								return true;
							}
						}
					},
					DifficultyLevel::Practice => {
						return true;
					}
				}
			}
			false
		})]
		pub fn play_game(origin: OriginFor<T>, game_type: DifficultyLevel) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			Self::check_enough_points(signer.clone(), game_type.clone())?;
			ensure!(RoundActive::<T>::get(), Error::<T>::NoActiveRound);
			let mut user = Users::<T>::get(signer.clone()).ok_or(Error::<T>::UserNotRegistered)?;
			let current_round = CurrentRound::<T>::get();
			if current_round != user.last_played_round {
				user.nfts = Default::default();
				user.last_played_round = current_round;
				Users::<T>::insert(signer.clone(), user);
			}
			let game_id = GameId::<T>::get();
			
			let (hashi, _) = T::GameRandomness::random(&[(game_id % 256) as u8]);
			let u32_value = u32::from_le_bytes(
				hashi.as_ref()[4..8].try_into().map_err(|_| Error::<T>::ConversionError)?,
			);
			let mut game_properties = GameProperties::<T>::take();
			let random_number = u32_value as usize % game_properties.len();
			let property = game_properties[random_number].clone();
			let game_datas =
				GameData { difficulty: game_type, player: signer.clone(), property: property, guess: None };
			game_properties.retain(|property| property.id as usize != random_number);
			GameProperties::<T>::put(game_properties);
			GameInfo::<T>::insert(game_id, game_datas);
			let next_game_id = game_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
			GameId::<T>::put(next_game_id);
			// Submit the encrypted property data and delete the price
			Self::deposit_event(Event::<T>::GameStarted { player: signer, game_id });
			Ok(())
		}

		/// Checks the answer of the player and handles rewards accordingly.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `guess`: The answer of the player.
		/// - `game_id`: The id of the game that the player wants to answer to.
		///
		/// Emits `AnswerSubmitted` event when succesfful.
		#[pallet::call_index(4)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::submit_answer())]
		#[pallet::feeless_if(|origin: &OriginFor<T>, guess: &u32, game_id: &u32| -> bool {
			if let Ok(signer) = ensure_signed(origin.clone()) {
				if let Some(game_info) = GameInfo::<T>::get(*game_id) {
					if signer == game_info.player {
						return true;
					}
				}
			}
			false
		})]
		pub fn submit_answer(origin: OriginFor<T>, guess: u32, game_id: u32) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			let mut game_info = GameInfo::<T>::get(game_id).ok_or(Error::<T>::NoActiveGame)?;
			ensure!(signer == game_info.player, Error::<T>::NoThePlayer);
			game_info.guess = Some(guess);
			GameInfo::<T>::insert(game_id, game_info);
			Self::deposit_event(Event::<T>::AnswerSubmitted { player: signer, game_id, guess });
			Ok(())
		}

		/// Checks the answer of the player and handles rewards accordingly.
		///
		/// The origin must be root.
		///
		/// Parameters:
		/// - `guess`: The answer of the player.
		/// - `game_id`: The id of the game that the result should be compared to.
		/// - `price`: The price of the property.
		/// - `secret`: The secret to decrypt the price and property data.
		///
		/// Emits `ResultChecked` event when succesfful.
		#[pallet::call_index(5)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::check_result())]
		pub fn check_result(
			origin: OriginFor<T>,
			guess: u64,
			game_id: u32,
			price: u64,
			secret: BoundedVec<u8, <T as Config>::StringLimit>,
		) -> DispatchResult {
			T::GameOrigin::ensure_origin(origin)?;
			let game_info = GameInfo::<T>::take(game_id).ok_or(Error::<T>::NoActiveGame)?;
			if game_info.guess.is_none() {
				Self::no_answer_result(game_info, game_id)?;
			} else {
				let difference_value = ((price as i64)
					.checked_sub(guess as i64)
					.ok_or(Error::<T>::ArithmeticUnderflow)?)
					.checked_mul(1000)
					.ok_or(Error::<T>::MultiplyError)?
					.checked_div(price as i64)
					.ok_or(Error::<T>::DivisionError)?
					.abs();
				Self::do_check_result(
					difference_value.try_into().map_err(|_| Error::<T>::ConversionError)?,
					game_id,
					game_info,
					secret,
				)?;
			}
			Ok(())
		}

		/// Lists a nft from the user.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `collection_id`: The collection id of the nft that will be listed.
		/// - `item_id`: The item id of the nft that will be listed.
		///
		/// Emits `NftListed` event when succesfful.
		#[pallet::call_index(6)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::list_nft())]
		pub fn list_nft(
			origin: OriginFor<T>,
			collection_id: CollectionId<T>,
			item_id: ItemId<T>,
		) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;
			let pallet_lookup = <T::Lookup as StaticLookup>::unlookup(Self::account_id());
			ensure!(
				pallet_nfts::Pallet::<T>::owner(collection_id.into(), item_id.into()) ==
					Some(signer.clone()),
				Error::<T>::NoPermission
			);
			let pallet_origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
			pallet_nfts::Pallet::<T>::unlock_item_transfer(
				pallet_origin,
				collection_id.into(),
				item_id.into(),
			)?;
			pallet_nfts::Pallet::<T>::transfer(
				origin,
				collection_id.into(),
				item_id.into(),
				pallet_lookup,
			)?;
			let listing_info = ListingInfo { owner: signer.clone(), collection_id, item_id };
			let mut listing_id = NextListingId::<T>::get();
			Listings::<T>::insert(listing_id, listing_info);
			listing_id = listing_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
			NextListingId::<T>::put(listing_id);
			Self::deposit_event(Event::<T>::NftListed { owner: signer, collection_id, item_id });
			Ok(())
		}

		/// Delists a nft from the user.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `listing_id`: The listing id of the listing.
		///
		/// Emits `NftDelisted` event when succesfful.
		#[pallet::call_index(7)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::delist_nft())]
		pub fn delist_nft(origin: OriginFor<T>, listing_id: u32) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;
			let listing_info =
				Listings::<T>::take(listing_id).ok_or(Error::<T>::ListingDoesNotExist)?;
			ensure!(listing_info.owner == signer, Error::<T>::NoPermission);
			pallet_nfts::Pallet::<T>::do_transfer(
				listing_info.collection_id.into(),
				listing_info.item_id.into(),
				signer.clone(),
				|_, _| Ok(()),
			)?;
			let pallet_origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
			pallet_nfts::Pallet::<T>::lock_item_transfer(
				pallet_origin,
				listing_info.collection_id.into(),
				listing_info.item_id.into(),
			)?;
			Self::deposit_event(Event::<T>::NftDelisted {
				owner: signer,
				collection_id: listing_info.collection_id,
				item_id: listing_info.item_id,
			});
			Ok(())
		}

		/// Makes an offer for a nft listing.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `listing_id`: The listing id of the listing.
		/// - `collection_id`: The collection id of the nft that will be offered.
		/// - `item_id`: The item id of the nft that will be offered.
		///
		/// Emits `OfferMade` event when succesfful.
		#[pallet::call_index(8)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::make_offer())]
		pub fn make_offer(
			origin: OriginFor<T>,
			listing_id: u32,
			collection_id: CollectionId<T>,
			item_id: ItemId<T>,
		) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;
			ensure!(Listings::<T>::get(listing_id).is_some(), Error::<T>::ListingDoesNotExist);
			let pallet_lookup = <T::Lookup as StaticLookup>::unlookup(Self::account_id());
			let pallet_origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
			pallet_nfts::Pallet::<T>::unlock_item_transfer(
				pallet_origin,
				collection_id.into(),
				item_id.into(),
			)?;
			pallet_nfts::Pallet::<T>::transfer(
				origin,
				collection_id.into(),
				item_id.into(),
				pallet_lookup,
			)?;
			let offer_info =
				OfferInfo { owner: signer.clone(), listing_id, collection_id, item_id };
			let offer_id = NextOfferId::<T>::get();
			Offers::<T>::insert(offer_id, offer_info);
			let offer_id = offer_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
			NextOfferId::<T>::put(offer_id);
			Self::deposit_event(Event::<T>::OfferMade {
				owner: signer,
				listing_id,
				collection_id,
				item_id,
			});
			Ok(())
		}

		/// Withdraw an offer.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `offer_id`: The id of the offer.
		///
		/// Emits `OfferWithdrawn` event when succesfful.
		#[pallet::call_index(9)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::make_offer())]
		pub fn withdraw_offer(origin: OriginFor<T>, offer_id: u32) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;
			let offer_details = Offers::<T>::get(offer_id).ok_or(Error::<T>::OfferDoesNotExist)?;
			ensure!(offer_details.owner == signer, Error::<T>::NoPermission);
			pallet_nfts::Pallet::<T>::do_transfer(
				offer_details.collection_id.into(),
				offer_details.item_id.into(),
				signer.clone(),
				|_, _| Ok(()),
			)?;
			let pallet_origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
			pallet_nfts::Pallet::<T>::lock_item_transfer(
				pallet_origin,
				offer_details.collection_id.into(),
				offer_details.item_id.into(),
			)?;
			Offers::<T>::take(offer_id).ok_or(Error::<T>::OfferDoesNotExist)?;
			Self::deposit_event(Event::<T>::OfferWithdrawn { owner: signer, offer_id });
			Ok(())
		}

		/// Handles an offer for a nft listing.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `offer_id`: The id of the offer.
		/// - `offer`: Must be either Accept or Reject.
		///
		/// Emits `OfferHandeld` event when succesfful.
		#[pallet::call_index(10)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::handle_offer())]
		pub fn handle_offer(origin: OriginFor<T>, offer_id: u32, offer: Offer) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;
			let offer_details = Offers::<T>::take(offer_id).ok_or(Error::<T>::OfferDoesNotExist)?;
			let listing_details = Listings::<T>::get(offer_details.listing_id)
				.ok_or(Error::<T>::ListingDoesNotExist)?;
			ensure!(listing_details.owner == signer, Error::<T>::NoPermission);
			let pallet_origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
			if offer == Offer::Accept {
				pallet_nfts::Pallet::<T>::do_transfer(
					listing_details.collection_id.into(),
					listing_details.item_id.into(),
					offer_details.owner.clone(),
					|_, _| Ok(()),
				)?;
				pallet_nfts::Pallet::<T>::lock_item_transfer(
					pallet_origin.clone(),
					listing_details.collection_id.into(),
					listing_details.item_id.into(),
				)?;
				pallet_nfts::Pallet::<T>::do_transfer(
					offer_details.collection_id.into(),
					offer_details.item_id.into(),
					listing_details.owner.clone(),
					|_, _| Ok(()),
				)?;
				pallet_nfts::Pallet::<T>::lock_item_transfer(
					pallet_origin,
					offer_details.collection_id.into(),
					offer_details.item_id.into(),
				)?;
				Listings::<T>::take(offer_details.listing_id)
					.ok_or(Error::<T>::ListingDoesNotExist)?;

				Self::swap_user_points(
					offer_details.owner.clone(),
					listing_details.collection_id,
					offer_details.collection_id,
				)?;
				Self::swap_user_points(
					signer.clone(),
					offer_details.collection_id,
					listing_details.collection_id,
				)?;
			} else {
				pallet_nfts::Pallet::<T>::do_transfer(
					offer_details.collection_id.into(),
					offer_details.item_id.into(),
					offer_details.owner,
					|_, _| Ok(()),
				)?;
				pallet_nfts::Pallet::<T>::lock_item_transfer(
					pallet_origin,
					offer_details.collection_id.into(),
					offer_details.item_id.into(),
				)?;
			}
			Self::deposit_event(Event::<T>::OfferHandeld { offer_id, offer });
			Ok(())
		}

		/// Add a new property and the price.
		///
		/// The origin must be the sudo.
		///
		/// Parameters:
		/// - `property`: The new property that will be added.
		/// - `price`: The price of the property that will be added.
		#[pallet::call_index(11)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::add_property())]
		pub fn add_property(origin: OriginFor<T>, property: PropertyInfoData<T>) -> DispatchResult {
			T::GameOrigin::ensure_origin(origin)?;
			GameProperties::<T>::try_append(property.clone())
				.map_err(|_| Error::<T>::TooManyTest)?;
			Ok(())
		}

		/// Remove a new property and the price.
		///
		/// The origin must be the sudo.
		///
		/// Parameters:
		/// - `id`: The id of the property that should be removed.
		#[pallet::call_index(12)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::remove_property())]
		pub fn remove_property(origin: OriginFor<T>, id: u32) -> DispatchResult {
			T::GameOrigin::ensure_origin(origin)?;
			let mut properties = GameProperties::<T>::take();
			properties.retain(|property| property.id != id);
			GameProperties::<T>::put(properties);
			Ok(())
		}

		/// Adds an account to the admins.
		///
		/// The origin must be the sudo.
		///
		/// Parameters:
		/// - `new_admin`: The address of the new account added to the list.
		///
		/// Emits `NewAdminAdded` event when succesfful
		#[pallet::call_index(13)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::add_to_admins())]
		pub fn add_to_admins(origin: OriginFor<T>, new_admin: AccountIdOf<T>) -> DispatchResult {
			T::GameOrigin::ensure_origin(origin)?;
			ensure!(!Admins::<T>::get().contains(&new_admin), Error::<T>::AccountAlreadyAdmin,);
			Admins::<T>::try_append(new_admin.clone()).map_err(|_| Error::<T>::TooManyAdmins)?;
			Self::deposit_event(Event::<T>::NewAdminAdded { new_admin });
			Ok(())
		}

		/// Removes an account from the admins.
		///
		/// The origin must be the sudo.
		///
		/// Parameters:
		/// - `admin`: The address of the admin removed from the admins.
		///
		/// Emits `UserRemoved` event when succesfful
		#[pallet::call_index(14)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::remove_from_admins())]
		pub fn remove_from_admins(origin: OriginFor<T>, admin: AccountIdOf<T>) -> DispatchResult {
			T::GameOrigin::ensure_origin(origin)?;
			ensure!(Admins::<T>::get().contains(&admin), Error::<T>::NotAdmin);
			let mut admins = Admins::<T>::get();
			let index = admins.iter().position(|x| *x == admin).ok_or(Error::<T>::InvalidIndex)?;
			admins.remove(index);
			Admins::<T>::put(admins);
			Self::deposit_event(Event::<T>::AdminRemoved { admin });
			Ok(())
		}

		/// Lets the player request token to play.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Emits `TokenReceived` event when succesfful.
		#[pallet::call_index(15)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::request_token())]
		pub fn request_token(origin: OriginFor<T>) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			let mut user = Users::<T>::get(signer.clone()).ok_or(Error::<T>::UserNotRegistered)?;
			ensure!(user.next_token_request < current_block_number, Error::<T>::CantRequestToken);
			let next_request =
				current_block_number.saturating_add(<T as Config>::RequestLimit::get());
			user.next_token_request = next_request;
			<T as pallet::Config>::Currency::make_free_balance_be(
				&signer,
				/* 10000000000000 */10u64.try_into().map_err(|_| Error::<T>::ConversionError)?,
			);
			Users::<T>::insert(signer.clone(), user);
			Self::deposit_event(Event::<T>::TokenReceived { player: signer });
			Ok(())
		}
	}
}
