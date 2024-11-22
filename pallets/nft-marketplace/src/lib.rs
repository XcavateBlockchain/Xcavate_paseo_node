#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

use frame_support::{
	traits::{Currency, Incrementable, ReservableCurrency},
	PalletId,
};

use frame_support::sp_runtime::traits::{
	AccountIdConversion, CheckedAdd, CheckedSub, CheckedDiv, CheckedMul, StaticLookup,
};

use enumflags2::BitFlags;

use pallet_nfts::{
	CollectionConfig, CollectionSetting, CollectionSettings, ItemConfig, ItemSettings, MintSettings,
};

use frame_system::RawOrigin;

use parity_scale_codec::Codec;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

type AssetBalanceOf<T> = <T as pallet_assets::Config>::Balance;

type FrationalizedNftBalanceOf<T> = <T as pallet_nft_fractionalization::Config>::AssetBalance;

type CurrencyBalanceOf<T> = <<T as pallet_nfts::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[cfg(feature = "runtime-benchmarks")]
	pub struct NftHelper;

	#[cfg(feature = "runtime-benchmarks")]
	pub trait BenchmarkHelper<AssetId, T> {
		fn to_asset(i: u32) -> AssetId;
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl<T: Config>
		BenchmarkHelper<FractionalizedAssetId<T>, T> for NftHelper
	{
		fn to_asset(i: u32) -> FractionalizedAssetId<T> {
			i.into()
		}
	}

	/// Infos regarding a listed nft of a real estate object on the marketplace.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct NftDetails<T: Config> {
		pub spv_created: bool,
		pub asset_id: u32,
		pub region: u32,
		pub location: LocationId<T>,
	}

	/// Infos regarding the listing of a real estate object.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct NftListingDetails<Balance, ItemId, CollectionId, T: Config> {
		pub real_estate_developer: AccountIdOf<T>,
		pub token_price: Balance,
		pub collected_funds: Balance,
		pub collected_tax: Balance,
		pub collected_fees: Balance,
		pub asset_id: u32,
		pub item_id: ItemId,
		pub collection_id: CollectionId,
		pub token_amount: u32,
	}

	/// Infos regarding the listing of a token.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct TokenListingDetails<Balance, ItemId, CollectionId, T: Config> {
		pub seller: AccountIdOf<T>,
		pub token_price: Balance,
		pub asset_id: u32,
		pub item_id: ItemId,
		pub collection_id: CollectionId,
		pub amount: u32,
	}

	/// Infos regarding the asset id.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct AssetDetails<ItemId, CollectionId, T: Config> {
		pub collection_id: CollectionId,
		pub item_id: ItemId,
		pub region: u32,
		pub location: LocationId<T>,
		pub price: AssetBalanceOf<T>,
		pub token_amount: u32,
	}

	/// Infos regarding an offer.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct OfferDetails<Balance, T: Config> {
		pub buyer: AccountIdOf<T>,
		pub token_price: Balance,
		pub amount: u32,
	}

	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct PropertyLawyerDetails<T: Config> {
		pub real_estate_developer_lawyer: Option<AccountIdOf<T>>,
		pub spv_lawyer: Option<AccountIdOf<T>>,
		pub real_estate_developer_status: DocumentStatus,
		pub spv_status: DocumentStatus,
		pub real_estate_developer_lawyer_costs: AssetBalanceOf<T>,
		pub spv_lawyer_costs: AssetBalanceOf<T>,
		pub second_attempt: bool,
	}

	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo, Default)]
	#[scale_info(skip_type_params(T))]
	pub struct TokenOwnerDetails<Balance> {
		pub token_amount: u32,
		pub paid_funds: Balance,
		pub paid_tax: Balance,
	}

	impl<Balance, T: Config> OfferDetails<Balance, T>
	where
		Balance: CheckedMul + TryFrom<u64>,
	{
		pub fn get_total_amount(&self) -> Result<Balance, Error<T>> {
			let amount_in_balance: Balance = (self.amount as u64)
				.try_into()
				.map_err(|_| Error::<T>::ConversionError)?;
	
			self.token_price
				.checked_mul(&amount_in_balance)
				.ok_or(Error::<T>::MultiplyError)
		}
	}

	/// Offer enum.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub enum Offer {
		Accept,
		Reject,
	}

	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub enum LegalProperty {
		RealEstateDeveloperSide,
		SpvSide,
	}

	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub enum DocumentStatus {
		Pending,
		Approved,
		Rejected,
	}

	/// AccountId storage.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub struct PalletIdStorage<T: Config> {
		pallet_id: AccountIdOf<T>,
	}

	/// The module configuration trait.
	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ pallet_nfts::Config
		+ pallet_xcavate_whitelist::Config
		+ pallet_assets::Config
		+ pallet_nft_fractionalization::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Type representing the weight of this pallet.
		type WeightInfo: WeightInfo;

		/// The currency type.
		type Currency: Currency<AccountIdOf<Self>> + ReservableCurrency<AccountIdOf<Self>>;

		/// The marketplace's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		#[cfg(feature = "runtime-benchmarks")]
		type Helper: crate::BenchmarkHelper<
			<Self as pallet_assets::Config>::AssetId,
			Self,
		>;

		/// The maximum amount of token of a nft.
		#[pallet::constant]
		type MaxNftToken: Get<u32>;

		/// Origin who can unlock new locations.
		type LocationOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Collection id type from pallet nfts.
		type CollectionId: IsType<<Self as pallet_nfts::Config>::CollectionId>
			+ Parameter
			+ From<u32>
			+ Default
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

		/// Collection id type from pallet nft fractionalization.
		type FractionalizeCollectionId: IsType<<Self as pallet_nft_fractionalization::Config>::NftCollectionId>
			+ Parameter
			+ From<CollectionId<Self>>
			+ Ord
			+ Copy
			+ MaxEncodedLen
			+ Encode;

		/// Item id type from pallet nft fractionalization.
		type FractionalizeItemId: IsType<<Self as pallet_nft_fractionalization::Config>::NftId>
			+ Parameter
			+ From<ItemId<Self>>
			+ Ord
			+ Copy
			+ MaxEncodedLen
			+ Encode;

		/// Asset id type from pallet nft fractionalization.
		type AssetId: IsType<<Self as pallet_nft_fractionalization::Config>::AssetId>
			+ Parameter
			+ From<u32>
			+ Ord
			+ Copy;

		/// Asset id type from pallet assets.
		type AssetId2: IsType<<Self as pallet_assets::Config>::AssetId>
			+ Parameter
			+ From<u32>
			+ Ord
			+ Copy;

		/// The Trasury's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type TreasuryId: Get<PalletId>;

		/// The CommunityProjects's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type CommunityProjectsId: Get<PalletId>;

		/// The maximum length of data stored in for post codes.
		#[pallet::constant]
		type PostcodeLimit: Get<u32>;
	}

	pub type FractionalizedAssetId<T> = <T as Config>::AssetId;
	pub type AssetId<T> = <T as Config>::AssetId2;
	pub type CollectionId<T> = <T as Config>::CollectionId;
	pub type ItemId<T> = <T as Config>::ItemId;
	pub type FractionalizeCollectionId<T> = <T as Config>::FractionalizeCollectionId;
	pub type FractionalizeItemId<T> = <T as Config>::FractionalizeItemId;
	pub type RegionId = u32;
	pub type ListingId = u32;
	pub type LocationId<T> = BoundedVec<u8, <T as Config>::PostcodeLimit>;

	pub(super) type NftListingDetailsType<T> = NftListingDetails<
		AssetBalanceOf<T>,
		<T as pallet::Config>::ItemId,
		<T as pallet::Config>::CollectionId,
		T,
	>;

	pub(super) type ListingDetailsType<T> = TokenListingDetails<
		AssetBalanceOf<T>,
		<T as pallet::Config>::ItemId,
		<T as pallet::Config>::CollectionId,
		T,
	>;

	/// Id for the next nft in a collection.
	#[pallet::storage]
	pub(super) type NextNftId<T: Config> =
		StorageMap<_, Blake2_128Concat, <T as pallet::Config>::CollectionId, u32, ValueQuery>;

	/// Id of the possible next asset that would be used for
	/// Nft fractionalization.
	#[pallet::storage]
	pub(super) type NextAssetId<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Id of the next region.
	#[pallet::storage]
	pub(super) type NextRegionId<T: Config> = StorageValue<_, RegionId, ValueQuery>;

	/// True if a location is registered.
	#[pallet::storage]
	pub type LocationRegistration<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		RegionId,
		Blake2_128Concat,
		LocationId<T>,
		bool,
		ValueQuery,
	>;

	/// The Id for the next token listing.
	#[pallet::storage]
	pub(super) type NextListingId<T: Config> = StorageValue<_, ListingId, ValueQuery>;

	/// Mapping of a collection id to the region.
	#[pallet::storage]
	pub type RegionCollections<T: Config> =
		StorageMap<_, Blake2_128Concat, RegionId, <T as pallet::Config>::CollectionId, OptionQuery>;

	/// Mapping from the Nft to the Nft details.
	#[pallet::storage]
	pub(super) type RegisteredNftDetails<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		<T as pallet::Config>::CollectionId,
		Blake2_128Concat,
		<T as pallet::Config>::ItemId,
		NftDetails<T>,
		OptionQuery,
	>;

	/// Mapping from the nft to the ongoing nft listing details.
	#[pallet::storage]
	pub(super) type OngoingObjectListing<T: Config> =
		StorageMap<_, Blake2_128Concat, ListingId, NftListingDetailsType<T>, OptionQuery>;

	/// Mapping of the nft to the amount of listed token.
	#[pallet::storage]
	pub(super) type ListedToken<T: Config> = StorageMap<_, Blake2_128Concat, ListingId, u32, OptionQuery>;

	/// Mapping of the listing to the buyer of the sold token.
	#[pallet::storage]
	pub(super) type TokenBuyer<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		ListingId,
		BoundedVec<AccountIdOf<T>, T::MaxNftToken>,
		ValueQuery,
	>;

	/// Double mapping of the account id of the token owner
	/// and the listing to the amount of token.
	#[pallet::storage]
	pub(super) type TokenOwner<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		AccountIdOf<T>,
		Blake2_128Concat,
		ListingId,
		TokenOwnerDetails<AssetBalanceOf<T>>,
		ValueQuery,
	>;

	/// Mapping of the listing id to the listing details of a token listing.
	#[pallet::storage]
	pub(super) type TokenListings<T: Config> =
		StorageMap<_, Blake2_128Concat, ListingId, ListingDetailsType<T>, OptionQuery>;

	/// Mapping of the assetid to the vector of token holder.
	#[pallet::storage]
	pub type PropertyOwner<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u32,
		BoundedVec<AccountIdOf<T>, T::MaxNftToken>,
		ValueQuery,
	>;

	/// Mapping of assetid and accountid to the amount of token an account is holding of the asset.
	#[pallet::storage]
	pub type PropertyOwnerToken<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Blake2_128Concat,
		AccountIdOf<T>,
		u32,
		ValueQuery,
	>;

	/// Mapping of the assetid to the collectionid and nftid.
	#[pallet::storage]
	pub type AssetIdDetails<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u32,
		AssetDetails<<T as pallet::Config>::ItemId, <T as pallet::Config>::CollectionId, T>,
		OptionQuery,
	>;

	/// Mapping from listing to offer details.
	#[pallet::storage]
	pub(super) type OngoingOffers<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		ListingId,
		Blake2_128Concat,
		AccountIdOf<T>,
		OfferDetails<AssetBalanceOf<T>, T>,
		OptionQuery,
	>;

	/// Stores the lawyer info.
	#[pallet::storage]
	pub(super) type RealEstateLawyer<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		AccountIdOf<T>,
		bool,
		ValueQuery,
	>;

	#[pallet::storage]
	pub type PropertyLawyer<T: Config> = StorageMap<
		_, 
		Blake2_128Concat,
		ListingId,
		PropertyLawyerDetails<T>,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new object has been listed on the marketplace.
		ObjectListed {
			collection_index: <T as pallet::Config>::CollectionId,
			item_index: <T as pallet::Config>::ItemId,
			price: AssetBalanceOf<T>,
			seller: AccountIdOf<T>,
		},
		/// A token has been bought.
		TokenBought { asset_id: u32, buyer: AccountIdOf<T>, price: AssetBalanceOf<T> },
		/// Token from listed object have been bought.
		TokenBoughtObject { asset_id: u32, buyer: AccountIdOf<T>, amount: u32, price: AssetBalanceOf<T> },
		/// Token have been listed.
		TokenListed { asset_id: u32, price: AssetBalanceOf<T>, seller: AccountIdOf<T> },
		/// The price of the token listing has been updated.
		ListingUpdated { listing_index: ListingId, new_price: AssetBalanceOf<T> },
		/// The nft has been delisted.
		ListingDelisted { listing_index: ListingId },
		/// The price of the listed object has been updated.
		ObjectUpdated { listing_index: ListingId, new_price: AssetBalanceOf<T> },
		/// New region has been created.
		RegionCreated { region_id: u32, collection_id: CollectionId<T> },
		/// New location has been created.
		LocationCreated { region_id: u32, location_id: LocationId<T> },
		/// A new offer has been made.
		OfferCreated { listing_id: ListingId, price: AssetBalanceOf<T> },
		/// An offer has been cancelled.
		OfferCancelled { listing_id: ListingId, account_id: AccountIdOf<T> },
		/// A lawyer has been registered.
		LawyerRegistered { lawyer: AccountIdOf<T> },
		/// A lawyer claimed a property.
		LawyerClaimedProperty { lawyer: AccountIdOf<T>, listing_id: ListingId, legal_side: LegalProperty},
		/// A lawyer stepped back from a legal case.
		LawyerRemovedFromCase { lawyer: AccountIdOf<T>, listing_id: ListingId },
		/// Documents have been approved or rejected.
		DocumentsConfirmed { signer: AccountIdOf<T>, listing_id: ListingId, approve: bool },
		/// The property nft got burned.
		PropertyNftBurned { collection_id: CollectionId<T>, item_id: ItemId<T>, asset_id: u32 },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// This index is not taken.
		InvalidIndex,
		/// The buyer doesn't have enough funds.
		NotEnoughFunds,
		/// Not enough token available to buy.
		NotEnoughTokenAvailable,
		/// Error by converting a type.
		ConversionError,
		/// Error by dividing a number.
		DivisionError,
		/// Error by multiplying a number.
		MultiplyError,
		/// No sufficient permission.
		NoPermission,
		/// The SPV has already been created.
		SpvAlreadyCreated,
		/// User did not pass the kyc.
		UserNotWhitelisted,
		ArithmeticUnderflow,
		ArithmeticOverflow,
		/// The token is not for sale.
		TokenNotForSale,
		/// The nft has not been registered on the marketplace.
		NftNotFound,
		/// There are already too many token buyer.
		TooManyTokenBuyer,
		/// This Region is not known.
		RegionUnknown,
		/// The location is already registered.
		LocationRegistered,
		/// The location is not registered.
		LocationUnknown,
		/// The object can not be divided in so many token.
		TooManyToken,
		/// A user can only make one offer per listing.
		OnlyOneOfferPerUser,
		/// The lawyer has already been registered.
		LawyerAlreadyRegistered,
		/// The lawyer job has already been taken.
		LawyerJobTaken,
		/// A lawyer has not been set.
		LawyerNotFound,
		/// The lawyer already submitted his answer.
		AlreadyConfirmed,
		/// The costs of the lawyer can't be that high.
		CostsTooHigh,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates a new region for the marketplace.
		/// This function calls the nfts-pallet to create a new collection.
		///
		/// The origin must be the LocationOrigin.
		///
		/// Emits `RegionCreated` event when succesfful.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::create_new_region())]
		pub fn create_new_region(origin: OriginFor<T>) -> DispatchResult {
			T::LocationOrigin::ensure_origin(origin)?;
			let collection_id = pallet_nfts::NextCollectionId::<T>::mutate(|maybe_id| {
				let current_collection_id = maybe_id.unwrap_or_else(|| {
					let initial_value = <T as pallet_nfts::Config>::CollectionId::initial_value();
					*maybe_id = initial_value;
					initial_value.expect("Failed to get the initial value")
				});
				let next_collection_id = current_collection_id.increment();
				*maybe_id = next_collection_id;
				current_collection_id
			});
			let collection_id: CollectionId<T> = collection_id.into();
			let pallet_id: AccountIdOf<T> =
				Self::account_id();
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
			let mut region_id = NextRegionId::<T>::get();
			RegionCollections::<T>::insert(region_id, collection_id);
			region_id = region_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
			NextRegionId::<T>::put(region_id);
			Self::deposit_event(Event::<T>::RegionCreated { region_id, collection_id });
			Ok(())
		}

		/// Creates a new location for a region.
		///
		/// The origin must be the LocationOrigin.
		///
		/// Parameters:
		/// - `region`: The region where the new location should be created.
		/// - `location`: The postcode of the new location.
		///
		/// Emits `LocationCreated` event when succesfful.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::create_new_location())]
		pub fn create_new_location(
			origin: OriginFor<T>,
			region: RegionId,
			location: LocationId<T>,
		) -> DispatchResult {
			T::LocationOrigin::ensure_origin(origin)?;
			ensure!(RegionCollections::<T>::get(region).is_some(), Error::<T>::RegionUnknown);
			ensure!(
				!LocationRegistration::<T>::get(region, location.clone()),
				Error::<T>::LocationRegistered
			);
			LocationRegistration::<T>::insert(region, location.clone(), true);
			Self::deposit_event(Event::<T>::LocationCreated {
				region_id: region,
				location_id: location,
			});
			Ok(())
		}

		/// List a real estate object. A new nft gets minted.
		/// This function calls the nfts-pallet to mint a new nft and sets the Metadata.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `region`: The region where the object is located.
		/// - `location`: The location where the object is located.
		/// - `token_price`: The price of a single token.
		/// - `token_amount`: The amount of tokens for a object.
		/// - `data`: The Metadata of the nft.
		///
		/// Emits `ObjectListed` event when succesfful
		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::list_object())]
		pub fn list_object(
			origin: OriginFor<T>,
			region: RegionId,
			location: LocationId<T>,
			token_price: AssetBalanceOf<T>,
			token_amount: u32,
			data: BoundedVec<u8, <T as pallet_nfts::Config>::StringLimit>,
		) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;
			ensure!(
				pallet_xcavate_whitelist::Pallet::<T>::whitelisted_accounts(signer.clone()),
				Error::<T>::UserNotWhitelisted
			);
			ensure!(token_amount <= T::MaxNftToken::get(), Error::<T>::TooManyToken);
			let collection_id: CollectionId<T> =
				RegionCollections::<T>::get(region).ok_or(Error::<T>::RegionUnknown)?;
			ensure!(
				LocationRegistration::<T>::get(region, location.clone()),
				Error::<T>::LocationUnknown
			);
			let mut next_item_id = NextNftId::<T>::get(collection_id);
			let mut asset_number: u32 = NextAssetId::<T>::get();
			let mut asset_id: AssetId<T> = asset_number.into();
			while pallet_assets::Pallet::<T>::maybe_total_supply(asset_id.into())
				.is_some()
			{
				asset_number = asset_number.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
				asset_id = asset_number.into();
			}
			let asset_id: FractionalizedAssetId<T> = asset_number.into();
			let item_id: ItemId<T> = next_item_id.into();
			let mut listing_id = NextListingId::<T>::get();
			let nft = NftListingDetails {
				real_estate_developer: signer.clone(),
				token_price,
				collected_funds: Default::default(),
				collected_tax: Default::default(),
				collected_fees: Default::default(),
				asset_id: asset_number,
				item_id,
				collection_id,
				token_amount,
			};
			let pallet_account = Self::account_id();
			pallet_nfts::Pallet::<T>::do_mint(
				collection_id.into(),
				item_id.into(),
				Some(pallet_account.clone()),
				pallet_account.clone(),
				Self::default_item_config(),
				|_, _| Ok(()),
			)?;
			let pallet_origin: OriginFor<T> = RawOrigin::Signed(pallet_account.clone()).into();
			pallet_nfts::Pallet::<T>::set_metadata(
				pallet_origin.clone(),
				collection_id.into(),
				item_id.into(),
				data.clone(),
			)?;
			let registered_nft_details = NftDetails {
				spv_created: false,
				asset_id: asset_number,
				region,
				location: location.clone(),
			};
			RegisteredNftDetails::<T>::insert(collection_id, item_id, registered_nft_details);
			OngoingObjectListing::<T>::insert(listing_id, nft.clone());
			ListedToken::<T>::insert(listing_id, token_amount);

			let user_lookup = <T::Lookup as StaticLookup>::unlookup(pallet_account);
			let nft_balance: FrationalizedNftBalanceOf<T> = token_amount.into();
			let fractionalize_collection_id = FractionalizeCollectionId::<T>::from(collection_id);
			let fractionalize_item_id = FractionalizeItemId::<T>::from(item_id);
			pallet_nft_fractionalization::Pallet::<T>::fractionalize(
				pallet_origin.clone(),
				fractionalize_collection_id.into(),
				fractionalize_item_id.into(),
				asset_id.into(),
				user_lookup,
				nft_balance,
			)?;
			let property_price = token_price
				.checked_mul(&Self::u64_to_balance_option(token_amount as u64)?)
				.ok_or(Error::<T>::MultiplyError)?;
			let asset_details =
				AssetDetails { collection_id, item_id, region, location, price: property_price, token_amount };
			AssetIdDetails::<T>::insert(asset_number, asset_details);
			next_item_id = next_item_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
			asset_number = asset_number.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
			NextNftId::<T>::insert(collection_id, next_item_id);
			NextAssetId::<T>::put(asset_number);
			listing_id = Self::next_listing_id(listing_id)?;
			NextListingId::<T>::put(listing_id);

			Self::deposit_event(Event::<T>::ObjectListed {
				collection_index: collection_id,
				item_index: item_id,
				price: token_price,
				seller: signer,
			});
			Ok(())
		}

		/// Buy listed token from the marketplace.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `listing_id`: The listing that the investor wants to buy token from.
		/// - `amount`: The amount of token that the investor wants to buy.
		///
		/// Emits `TokenBoughtObject` event when succesfful.
		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::buy_token())]
		pub fn buy_token(origin: OriginFor<T>, listing_id: ListingId, amount: u32) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;
			ensure!(
				pallet_xcavate_whitelist::Pallet::<T>::whitelisted_accounts(signer.clone()),
				Error::<T>::UserNotWhitelisted
			);

			ListedToken::<T>::try_mutate_exists(listing_id, |maybe_listed_token| {
				let listed_token = maybe_listed_token.as_mut().ok_or(Error::<T>::TokenNotForSale)?;
				ensure!(*listed_token >= amount, Error::<T>::NotEnoughTokenAvailable);
				let mut nft_details =
					OngoingObjectListing::<T>::get(listing_id).ok_or(Error::<T>::InvalidIndex)?;
				ensure!(
					!RegisteredNftDetails::<T>::get(nft_details.collection_id, nft_details.item_id)
						.ok_or(Error::<T>::InvalidIndex)?
						.spv_created,
					Error::<T>::SpvAlreadyCreated
				);

				let transfer_price = nft_details
					.token_price
					.checked_mul(&Self::u64_to_balance_option(amount as u64)?)
					.ok_or(Error::<T>::MultiplyError)?;

				let fee = transfer_price
					.checked_mul(&Self::u64_to_balance_option(1)?)
					.ok_or(Error::<T>::MultiplyError)?
					.checked_div(&Self::u64_to_balance_option(100)?) 
					.ok_or(Error::<T>::DivisionError)?;
				
				let tax = transfer_price
					.checked_mul(&Self::u64_to_balance_option(3)?)
					.ok_or(Error::<T>::MultiplyError)?
					.checked_div(&Self::u64_to_balance_option(100)?) 
					.ok_or(Error::<T>::DivisionError)?;
				
				let total_transfer_price = transfer_price
					.checked_add(&fee)
					.ok_or(Error::<T>::ArithmeticOverflow)?
					.checked_add(&tax)
					.ok_or(Error::<T>::ArithmeticOverflow)?;

				Self::transfer_funds(signer.clone(), Self::account_id(), total_transfer_price)?;
				*listed_token =
					listed_token.checked_sub(amount).ok_or(Error::<T>::ArithmeticUnderflow)?;
				if !TokenBuyer::<T>::get(listing_id).contains(&signer) {
					TokenBuyer::<T>::try_mutate(listing_id, |keys| {
						keys.try_push(signer.clone()).map_err(|_| Error::<T>::TooManyTokenBuyer)?;
						Ok::<(), DispatchError>(())
					})?;
				}
				TokenOwner::<T>::try_mutate_exists(signer.clone(), listing_id, |maybe_token_owner_details| {
					let token_owner_details = maybe_token_owner_details.get_or_insert( TokenOwnerDetails {
						token_amount: 0,
						paid_funds: Default::default(),
						paid_tax: Default::default(),
					});
					token_owner_details.token_amount = token_owner_details.token_amount
						.checked_add(amount)
						.ok_or(Error::<T>::ArithmeticOverflow)?;
					token_owner_details.paid_funds = token_owner_details.paid_funds
						.checked_add(&transfer_price)
						.ok_or(Error::<T>::ArithmeticOverflow)?;
					token_owner_details.paid_tax = token_owner_details.paid_tax
						.checked_add(&tax)
						.ok_or(Error::<T>::ArithmeticOverflow)?;

					Ok::<(), DispatchError>(())
				})?;
				nft_details.collected_funds = nft_details
					.collected_funds
					.checked_add(&transfer_price)
					.ok_or(Error::<T>::ArithmeticOverflow)?;
				nft_details.collected_tax = nft_details
					.collected_tax
					.checked_add(&tax)
					.ok_or(Error::<T>::ArithmeticOverflow)?;
				nft_details.collected_fees = nft_details
					.collected_fees
					.checked_add(&fee)
					.ok_or(Error::<T>::ArithmeticOverflow)?;
				OngoingObjectListing::<T>::insert(listing_id, nft_details.clone());
				if *listed_token == 0 {
					let property_lawyer_details = PropertyLawyerDetails {
						real_estate_developer_lawyer: None,
						spv_lawyer: None,
						real_estate_developer_status: DocumentStatus::Pending,
						spv_status: DocumentStatus::Pending,
						real_estate_developer_lawyer_costs: Default::default(),
						spv_lawyer_costs: Default::default(),
						second_attempt: false,
					};
					PropertyLawyer::<T>::insert(listing_id, property_lawyer_details);
					*maybe_listed_token = None;
				} 
				Self::deposit_event(Event::<T>::TokenBoughtObject {
					asset_id: nft_details.asset_id,
					buyer: signer.clone(),
					amount,
					price: transfer_price,
				});
				Ok::<(), DispatchError>(())
			})?;
			Ok(())
		}

		/// Relist token on the marketplace.
		/// The nft must be registered on the marketplace.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `region`: The region where the object is located.
		/// - `item_id`: The item id of the nft.
		/// - `token_price`: The price of a single token.
		/// - `amount`: The amount of token of the real estate object that should be listed.
		///
		/// Emits `TokenListed` event when succesfful
		#[pallet::call_index(4)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::relist_token())]
		pub fn relist_token(
			origin: OriginFor<T>,
			region: RegionId,
			item_id: <T as pallet::Config>::ItemId,
			token_price: AssetBalanceOf<T>,
			amount: u32,
		) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;

			ensure!(
				pallet_xcavate_whitelist::Pallet::<T>::whitelisted_accounts(signer.clone()),
				Error::<T>::UserNotWhitelisted
			);
			let collection_id: CollectionId<T> =
				RegionCollections::<T>::get(region).ok_or(Error::<T>::RegionUnknown)?;

			let nft_details = RegisteredNftDetails::<T>::get(collection_id, item_id)
				.ok_or(Error::<T>::NftNotFound)?;
			ensure!(
				LocationRegistration::<T>::get(region, nft_details.location),
				Error::<T>::LocationUnknown
			);
			let pallet_lookup = <T::Lookup as StaticLookup>::unlookup(Self::account_id());
			let asset_id: AssetId<T> = nft_details.asset_id.into();
			let token_amount = amount.into();
			pallet_assets::Pallet::<T>::transfer(
				origin,
				asset_id.into().into(),
				pallet_lookup,
				token_amount,
			)
			.map_err(|_| Error::<T>::NotEnoughFunds)?;
			let mut listing_id = NextListingId::<T>::get();
			let token_listing = TokenListingDetails {
				seller: signer.clone(),
				token_price,
				asset_id: nft_details.asset_id,
				item_id,
				collection_id,
				amount,
			};
			TokenListings::<T>::insert(listing_id, token_listing);
			listing_id = Self::next_listing_id(listing_id)?;
			NextListingId::<T>::put(listing_id);

			Self::deposit_event(Event::<T>::TokenListed {
				asset_id: nft_details.asset_id,
				price: token_price,
				seller: signer,
			});
			Ok(())
		}

		/// Buy token from the marketplace.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `listing_id`: The listing that the investor wants to buy from.
		/// - `amount`: The amount of token the investor wants to buy.
		///
		/// Emits `TokenBought` event when succesfful.
		#[pallet::call_index(5)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::buy_relisted_token())]
		pub fn buy_relisted_token(
			origin: OriginFor<T>,
			listing_id: ListingId,
			amount: u32,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			ensure!(
				pallet_xcavate_whitelist::Pallet::<T>::whitelisted_accounts(origin.clone()),
				Error::<T>::UserNotWhitelisted
			);
			let listing_details =
				TokenListings::<T>::take(listing_id).ok_or(Error::<T>::TokenNotForSale)?;
			ensure!(listing_details.amount >= amount, Error::<T>::NotEnoughTokenAvailable);
			let price = listing_details
				.token_price
				.checked_mul(&Self::u64_to_balance_option(amount.into())?)
				.ok_or(Error::<T>::MultiplyError)?;
			Self::buying_token_process(
				listing_id,
				origin.clone(),
				origin,
				listing_details,
				price,
				amount,
			)?;
			Ok(())
		}

		/// Created an offer for a token listing.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `listing_id`: The listing that the investor wants to buy from.
		/// - `offer_price`: The offer price for token that are offered.
		/// - `amount`: The amount of token that the investor wants to buy.
		///
		/// Emits `OfferCreated` event when succesfful.
		#[pallet::call_index(6)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::make_offer())]
		pub fn make_offer(
			origin: OriginFor<T>,
			listing_id: ListingId,
			offer_price: AssetBalanceOf<T>,
			amount: u32,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			ensure!(
				pallet_xcavate_whitelist::Pallet::<T>::whitelisted_accounts(signer.clone()),
				Error::<T>::UserNotWhitelisted
			);
			ensure!(OngoingOffers::<T>::get(listing_id, signer.clone()).is_none(), Error::<T>::OnlyOneOfferPerUser);
			let listing_details =
				TokenListings::<T>::get(listing_id).ok_or(Error::<T>::TokenNotForSale)?;
			ensure!(listing_details.amount >= amount, Error::<T>::NotEnoughTokenAvailable);
			let price = offer_price
				.checked_mul(&Self::u64_to_balance_option(amount.into())?)
				.ok_or(Error::<T>::MultiplyError)?;
			Self::transfer_funds(signer.clone(), Self::account_id(), price)?;
			let offer_details = OfferDetails { buyer: signer.clone(), token_price: offer_price, amount };
			OngoingOffers::<T>::insert(listing_id, signer, offer_details);
			Self::deposit_event(Event::<T>::OfferCreated { listing_id, price: offer_price });
			Ok(())
		}

		/// Lets the investor handle an offer.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `listing_id`: The listing that the investor wants to buy from.
		/// - `offeror`: AccountId of the person that the seller wants to handle the offer from.
		/// - `offer`: Enum for offer which is either Accept or Reject.
		#[pallet::call_index(7)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::handle_offer())]
		pub fn handle_offer(
			origin: OriginFor<T>,
			listing_id: ListingId,
			offeror: AccountIdOf<T>,
			offer: Offer,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			ensure!(
				pallet_xcavate_whitelist::Pallet::<T>::whitelisted_accounts(signer.clone()),
				Error::<T>::UserNotWhitelisted
			);
			let listing_details =
				TokenListings::<T>::get(listing_id).ok_or(Error::<T>::TokenNotForSale)?;
			ensure!(listing_details.seller == signer, Error::<T>::NoPermission);
			let offer_details =
				OngoingOffers::<T>::take(listing_id, offeror).ok_or(Error::<T>::InvalidIndex)?;
			ensure!(listing_details.amount >= offer_details.amount, Error::<T>::NotEnoughTokenAvailable);
			let price = offer_details.get_total_amount()?;
			let pallet_account = Self::account_id();
			match offer {
				Offer::Accept => {
					Self::buying_token_process(
						listing_id,
						pallet_account,
						offer_details.buyer,
						listing_details,
						price,
						offer_details.amount,
					)?;
				}
				Offer::Reject => {
					Self::transfer_funds(pallet_account, offer_details.buyer, price)?;
				}
			}
			Ok(())
		}

		/// Lets the investor cancel an offer.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `listing_id`: The listing that the investor wants to buy from.
		///
		/// Emits `OfferCancelled` event when succesfful.
		#[pallet::call_index(8)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::cancel_offer())]
		pub fn cancel_offer(
			origin: OriginFor<T>,
			listing_id: ListingId,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			let offer_details =
				OngoingOffers::<T>::take(listing_id, signer.clone()).ok_or(Error::<T>::InvalidIndex)?;
			ensure!(offer_details.buyer == signer.clone(), Error::<T>::NoPermission);
			let price = offer_details.get_total_amount()?;
			Self::transfer_funds(Self::account_id(), offer_details.buyer, price)?;
			Self::deposit_event(Event::<T>::OfferCancelled { listing_id, account_id: signer.clone() });
			Ok(())
		}

		/// Upgrade the price from a listing.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `listing_id`: The listing that the seller wants to update.
		/// - `new_price`: The new price of the nft.
		///
		/// Emits `ListingUpdated` event when succesfful.
		#[pallet::call_index(9)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::upgrade_listing())]
		pub fn upgrade_listing(
			origin: OriginFor<T>,
			listing_id: ListingId,
			new_price: AssetBalanceOf<T>,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			ensure!(
				pallet_xcavate_whitelist::Pallet::<T>::whitelisted_accounts(signer.clone()),
				Error::<T>::UserNotWhitelisted
			);
			let _ = TokenListings::<T>::try_mutate(listing_id, |maybe_listing_details| {
				let listing_details = maybe_listing_details.as_mut().ok_or(Error::<T>::TokenNotForSale)?;
				ensure!(listing_details.seller == signer, Error::<T>::NoPermission);
				listing_details.token_price = new_price;
				Ok::<(), DispatchError>(())
			})?;
			Self::deposit_event(Event::<T>::ListingUpdated {
				listing_index: listing_id,
				new_price,
			});
			Ok(())
		}

		/// Upgrade the price from a listed object.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `listing_id`: The listing that the seller wants to update.
		/// - `new_price`: The new price of the object.
		///
		/// Emits `ObjectUpdated` event when succesfful.
		#[pallet::call_index(10)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::upgrade_object())]
		pub fn upgrade_object(
			origin: OriginFor<T>,
			listing_id: ListingId,
			new_price: AssetBalanceOf<T>,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			ensure!(
				pallet_xcavate_whitelist::Pallet::<T>::whitelisted_accounts(signer.clone()),
				Error::<T>::UserNotWhitelisted
			);
			ensure!(ListedToken::<T>::contains_key(listing_id), Error::<T>::TokenNotForSale);
			let _ = OngoingObjectListing::<T>::try_mutate(listing_id, |maybe_nft_details| {
				let nft_details = maybe_nft_details.as_mut().ok_or(Error::<T>::InvalidIndex)?;
				ensure!(nft_details.real_estate_developer == signer.clone(), Error::<T>::NoPermission);
				ensure!(
					!RegisteredNftDetails::<T>::get(nft_details.collection_id, nft_details.item_id)
						.ok_or(Error::<T>::InvalidIndex)?
						.spv_created,
					Error::<T>::SpvAlreadyCreated
				);
				nft_details.token_price = new_price;
				Ok::<(), DispatchError>(())
			})?;
			Self::deposit_event(Event::<T>::ObjectUpdated { listing_index: listing_id, new_price });
			Ok(())
		}

		/// Delist the choosen listing from the marketplace.
		/// Works only for relisted token.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `listing_id`: The listing that the seller wants to delist.
		///
		/// Emits `ListingDelisted` event when succesfful.
		#[pallet::call_index(11)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::delist_token())]
		pub fn delist_token(origin: OriginFor<T>, listing_id: ListingId) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			ensure!(
				pallet_xcavate_whitelist::Pallet::<T>::whitelisted_accounts(signer.clone()),
				Error::<T>::UserNotWhitelisted
			);
			let listing_details =
				TokenListings::<T>::take(listing_id).ok_or(Error::<T>::TokenNotForSale)?;
			ensure!(listing_details.seller == signer, Error::<T>::NoPermission);
			let user_lookup = <T::Lookup as StaticLookup>::unlookup(signer.clone());
			let asset_id: AssetId<T> = listing_details.asset_id.into();
			let token_amount = listing_details.amount.into();
			let pallet_origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
			pallet_assets::Pallet::<T>::transfer(
				pallet_origin,
				asset_id.into().into(),
				user_lookup,
				token_amount,
			)
			.map_err(|_| Error::<T>::NotEnoughFunds)?;
			Self::deposit_event(Event::<T>::ListingDelisted { listing_index: listing_id });
			Ok(())
		}

		/// Registers a new lawyer.
		///
		/// The origin must be the LocationOrigin.
		///
		/// Parameters:
		/// - `lawyer`: The lawyer that should be registered.
		///
		/// Emits `LawyerRegistered` event when succesfful.
		#[pallet::call_index(12)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
		pub fn register_lawyer(
			origin: OriginFor<T>,
			lawyer: AccountIdOf<T>,
		) -> DispatchResult {
			T::LocationOrigin::ensure_origin(origin)?;
			ensure!(!RealEstateLawyer::<T>::get(lawyer.clone()), Error::<T>::LawyerAlreadyRegistered);
			RealEstateLawyer::<T>::insert(lawyer.clone(), true);
			Self::deposit_event(Event::<T>::LawyerRegistered {lawyer});
			Ok(())
		}

		/// Lets a lawyer claim a property to handle the legal work.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `listing_id`: The listing from the property.
		/// - `legal_side`: The side that the lawyer wants to represent.
		/// - `costs`: The costs thats the lawyer demands for his work.
		///
		/// Emits `LawyerClaimedProperty` event when succesfful.
		#[pallet::call_index(13)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
		pub fn lawyer_claim_property(
			origin: OriginFor<T>,
			listing_id: ListingId,
			legal_side: LegalProperty,
			costs: AssetBalanceOf<T>,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			ensure!(RealEstateLawyer::<T>::get(signer.clone()), Error::<T>::NoPermission);
			let mut property_lawyer_details = PropertyLawyer::<T>::get(listing_id).ok_or(Error::<T>::InvalidIndex)?;
			let nft_details =
				OngoingObjectListing::<T>::get(listing_id).ok_or(Error::<T>::InvalidIndex)?;
			ensure!(nft_details.collected_fees >= costs, Error::<T>::CostsTooHigh);

			match legal_side {
				LegalProperty::RealEstateDeveloperSide => {
					ensure!(property_lawyer_details.real_estate_developer_lawyer.is_none(), Error::<T>::LawyerJobTaken);
					ensure!(property_lawyer_details.spv_lawyer != Some(signer.clone()), Error::<T>::NoPermission);
					property_lawyer_details.real_estate_developer_lawyer = Some(signer.clone());
					property_lawyer_details.real_estate_developer_lawyer_costs = costs;
					PropertyLawyer::<T>::insert(listing_id, property_lawyer_details);
				}
				LegalProperty::SpvSide => {
					ensure!(property_lawyer_details.spv_lawyer.is_none(), Error::<T>::LawyerJobTaken);
					ensure!(property_lawyer_details.real_estate_developer_lawyer != Some(signer.clone()), Error::<T>::NoPermission);
					property_lawyer_details.spv_lawyer = Some(signer.clone());
					property_lawyer_details.spv_lawyer_costs = costs;
					PropertyLawyer::<T>::insert(listing_id, property_lawyer_details);
				}
			}
			Self::deposit_event(Event::<T>::LawyerClaimedProperty {lawyer: signer, listing_id, legal_side});
			Ok(())
		}

		/// Lets a lawyer step back from a case.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `listing_id`: The listing from the property.
		///
		/// Emits `LawyerRemovedFromCase` event when succesfful.
		#[pallet::call_index(14)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
		pub fn remove_from_case(
			origin: OriginFor<T>,
			listing_id: ListingId,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			ensure!(RealEstateLawyer::<T>::get(signer.clone()), Error::<T>::NoPermission);
			let mut property_lawyer_details = PropertyLawyer::<T>::get(listing_id).ok_or(Error::<T>::InvalidIndex)?;
			if property_lawyer_details.real_estate_developer_lawyer == Some(signer.clone()) {
				ensure!(property_lawyer_details.real_estate_developer_status == DocumentStatus::Pending,
					Error::<T>::AlreadyConfirmed);
				property_lawyer_details.real_estate_developer_lawyer = None;
			} else if property_lawyer_details.spv_lawyer == Some(signer.clone()) {
				ensure!(property_lawyer_details.spv_status == DocumentStatus::Pending,
					Error::<T>::AlreadyConfirmed);
				property_lawyer_details.spv_lawyer = None;
			} else {
				return Err(Error::<T>::NoPermission.into());
			}
			PropertyLawyer::<T>::insert(listing_id, property_lawyer_details);	
			Self::deposit_event(Event::<T>::LawyerRemovedFromCase {lawyer: signer, listing_id});	
			Ok(())
		}

		/// Lets a lawyer confirm a legal case.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `listing_id`: The listing from the property.
		/// - `approve`: Approves or Rejects the case.
		///
		/// Emits `DocumentsConfirmed` event when succesfful.
		#[pallet::call_index(15)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
		pub fn lawyer_confirm_documents(
			origin: OriginFor<T>,
			listing_id: ListingId,
			approve: bool,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;

			let mut property_lawyer_details = PropertyLawyer::<T>::take(listing_id).ok_or(Error::<T>::InvalidIndex)?;
			if property_lawyer_details.real_estate_developer_lawyer == Some(signer.clone()) {
				ensure!(property_lawyer_details.real_estate_developer_status == DocumentStatus::Pending,
					Error::<T>::AlreadyConfirmed);
				property_lawyer_details.real_estate_developer_status = if approve {
					DocumentStatus::Approved
				} else {
					DocumentStatus::Rejected
				};
				Self::deposit_event(Event::<T>::DocumentsConfirmed { signer, listing_id, approve });
			} else if property_lawyer_details.spv_lawyer == Some(signer.clone()) {
				ensure!(property_lawyer_details.spv_status == DocumentStatus::Pending,
					Error::<T>::AlreadyConfirmed);
				property_lawyer_details.spv_status = if approve {
					DocumentStatus::Approved
				} else {
					DocumentStatus::Rejected
				};
				Self::deposit_event(Event::<T>::DocumentsConfirmed {signer, listing_id, approve});
			} else {
				return Err(Error::<T>::NoPermission.into());
			}

			let developer_status = property_lawyer_details.real_estate_developer_status.clone();
			let spv_status = property_lawyer_details.spv_status.clone();

			match (developer_status, spv_status) {
				(DocumentStatus::Approved, DocumentStatus::Approved) => {
					Self::execute_deal(
						listing_id, 
						property_lawyer_details
					)?;
				}
				(DocumentStatus::Rejected, DocumentStatus::Rejected) => {
					Self::burn_tokens_and_nfts(listing_id)?;
					Self::refund_investors(listing_id, property_lawyer_details)?;
				}
				(DocumentStatus::Approved, DocumentStatus::Rejected) => {
					if !property_lawyer_details.second_attempt {
						property_lawyer_details.spv_status = DocumentStatus::Pending;
						property_lawyer_details.real_estate_developer_status = DocumentStatus::Pending;
						property_lawyer_details.second_attempt = true;
						PropertyLawyer::<T>::insert(listing_id, property_lawyer_details);
					} else {
						Self::burn_tokens_and_nfts(listing_id)?;
						Self::refund_investors(listing_id, property_lawyer_details)?;
					}
				}
				(DocumentStatus::Rejected, DocumentStatus::Approved) => {
					if !property_lawyer_details.second_attempt {
						property_lawyer_details.spv_status = DocumentStatus::Pending;
						property_lawyer_details.real_estate_developer_status = DocumentStatus::Pending;
						property_lawyer_details.second_attempt = true;
						PropertyLawyer::<T>::insert(listing_id, property_lawyer_details);
					} else {
						Self::burn_tokens_and_nfts(listing_id)?;
						Self::refund_investors(listing_id, property_lawyer_details)?;
					}
				}
				_ => {
					PropertyLawyer::<T>::insert(listing_id, property_lawyer_details);
				}
			}
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Get the account id of the pallet
		pub fn account_id() -> AccountIdOf<T> {
			<T as pallet::Config>::PalletId::get().into_account_truncating()
		}

		/// Get the account id of the treasury pallet
		pub fn treasury_account_id() -> AccountIdOf<T> {
			T::TreasuryId::get().into_account_truncating()
		}

		/// Get the account id of the community pallet
		pub fn community_account_id() -> AccountIdOf<T> {
			T::CommunityProjectsId::get().into_account_truncating()
		}

		pub fn next_listing_id(listing_id: ListingId) -> Result<ListingId, Error<T>> {
			listing_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)
		}

		/// Sends the token to the new owners and the funds to the real estate developer once all 100 token
		/// of a collection are sold.
		fn execute_deal(listing_id: u32, property_lawyer_details: PropertyLawyerDetails<T>) -> DispatchResult {
			let list = <TokenBuyer<T>>::take(listing_id);
			let pallet_account = Self::account_id();
			let nft_details =
				OngoingObjectListing::<T>::take(listing_id).ok_or(Error::<T>::InvalidIndex)?;
			let price = nft_details.collected_funds;
			let treasury_id = Self::treasury_account_id();
			let seller_part = price
				.checked_mul(&Self::u64_to_balance_option(99)?)
				.ok_or(Error::<T>::MultiplyError)?
				.checked_div(&Self::u64_to_balance_option(100)?)
				.ok_or(Error::<T>::DivisionError)?;
			let tax = nft_details.collected_tax;
			let treasury_fees = price
				.checked_div(&Self::u64_to_balance_option(100)?)
				.ok_or(Error::<T>::DivisionError)?
				.checked_add(&nft_details.collected_fees)
				.ok_or(Error::<T>::ArithmeticOverflow)?
				.checked_sub(&property_lawyer_details.real_estate_developer_lawyer_costs)
				.ok_or(Error::<T>::ArithmeticUnderflow)?
				.checked_sub(&property_lawyer_details.spv_lawyer_costs)
				.ok_or(Error::<T>::ArithmeticUnderflow)?;
			//Self::transfer_funds(sender.clone(), treasury_id, fees)?;
			let real_estate_developer_lawyer_id = match property_lawyer_details.real_estate_developer_lawyer {
				Some(account_id) => account_id,
				None => return Err(Error::<T>::LawyerNotFound.into()),
			};
			let spv_lawyer_id = match property_lawyer_details.spv_lawyer {
				Some(account_id) => account_id,
				None => return Err(Error::<T>::LawyerNotFound.into()),
			};
			let real_estate_developer_part = tax
				.checked_add(&property_lawyer_details.real_estate_developer_lawyer_costs)
				.ok_or(Error::<T>::ArithmeticOverflow)?;

			Self::transfer_funds(pallet_account.clone(), real_estate_developer_lawyer_id, real_estate_developer_part)?;
			Self::transfer_funds(pallet_account.clone(), spv_lawyer_id, property_lawyer_details.spv_lawyer_costs)?;
			Self::transfer_funds(pallet_account.clone(), treasury_id, treasury_fees)?;
			Self::transfer_funds(pallet_account.clone(), nft_details.real_estate_developer, seller_part)?;
			let origin: OriginFor<T> = RawOrigin::Signed(pallet_account).into();
			let asset_id: AssetId<T> = nft_details.asset_id.into();
			for owner in list {
				let user_lookup = <T::Lookup as StaticLookup>::unlookup(owner.clone());
				let token_details: TokenOwnerDetails<AssetBalanceOf<T>> = TokenOwner::<T>::take(owner.clone(), listing_id);
				//let token: u64 = TokenOwner::<T>::take(owner.clone(), listing_id) as u64;
				let token_amount = token_details.token_amount.try_into().map_err(|_| Error::<T>::ConversionError)?;
				pallet_assets::Pallet::<T>::transfer(
					origin.clone(),
					asset_id.into().into(),
					user_lookup,
					token_amount,
				)
				.map_err(|_| Error::<T>::NotEnoughFunds)?;
				PropertyOwner::<T>::try_mutate(nft_details.asset_id, |keys| {
					keys.try_push(owner.clone()).map_err(|_| Error::<T>::TooManyTokenBuyer)?;
					Ok::<(), DispatchError>(())
				})?;
				PropertyOwnerToken::<T>::insert(nft_details.asset_id, owner, token_details.token_amount as u32)
			}
			let mut registered_nft_details =
				RegisteredNftDetails::<T>::get(nft_details.collection_id, nft_details.item_id)
					.ok_or(Error::<T>::InvalidIndex)?;
			registered_nft_details.spv_created = true;
			RegisteredNftDetails::<T>::insert(
				nft_details.collection_id,
				nft_details.item_id,
				registered_nft_details,
			);
			Ok(())
		}

		fn burn_tokens_and_nfts(listing_id: ListingId) -> DispatchResult {
			let nft_details =
				OngoingObjectListing::<T>::get(listing_id).ok_or(Error::<T>::InvalidIndex)?;
			let pallet_account = Self::account_id();
			let pallet_origin: OriginFor<T> = RawOrigin::Signed(pallet_account.clone()).into();
			let user_lookup = <T::Lookup as StaticLookup>::unlookup(pallet_account);
			let fractionalize_collection_id = FractionalizeCollectionId::<T>::from(nft_details.collection_id);
			let fractionalize_item_id = FractionalizeItemId::<T>::from(nft_details.item_id);
			let fractionalize_asset_id = FractionalizedAssetId::<T>::from(nft_details.asset_id);
			pallet_nft_fractionalization::Pallet::<T>::unify(
				pallet_origin.clone(),
				fractionalize_collection_id.into(),
				fractionalize_item_id.into(),
				fractionalize_asset_id.into(),
				user_lookup,
			)?;
			pallet_nfts::Pallet::<T>::burn(
				pallet_origin,
				nft_details.collection_id.into(),
				nft_details.item_id.into(),
			)?;
			Self::deposit_event(Event::<T>::PropertyNftBurned { 
				collection_id: nft_details.collection_id, 
				item_id: nft_details.item_id,
				asset_id: nft_details.asset_id, 
			});
			RegisteredNftDetails::<T>::take(nft_details.collection_id, nft_details.item_id)
				.ok_or(Error::<T>::InvalidIndex)?;
			Ok(())
		}

		fn refund_investors(listing_id: ListingId, property_lawyer_details: PropertyLawyerDetails<T>) -> DispatchResult {
			let list = <TokenBuyer<T>>::take(listing_id);
			let pallet_account = Self::account_id();
			let nft_details =
				OngoingObjectListing::<T>::take(listing_id).ok_or(Error::<T>::InvalidIndex)?;
			let fees = nft_details.collected_fees;
			let treasury_id = Self::treasury_account_id();
			let treasury_amount = fees
				.checked_sub(&property_lawyer_details.spv_lawyer_costs)
				.ok_or(Error::<T>::ArithmeticUnderflow)?;
			Self::transfer_funds(pallet_account.clone(), treasury_id, treasury_amount)?;
			let spv_lawyer_id = match property_lawyer_details.spv_lawyer {
				Some(account_id) => account_id,
				None => return Err(Error::<T>::LawyerNotFound.into()),
			};
			Self::transfer_funds(pallet_account.clone(), spv_lawyer_id, property_lawyer_details.spv_lawyer_costs)?;
			for owner in list {
				let token_details: TokenOwnerDetails<AssetBalanceOf<T>> = TokenOwner::<T>::take(owner.clone(), listing_id);
				let refund_amount = token_details.paid_funds
					.checked_add(&token_details.paid_tax)
					.ok_or(Error::<T>::ArithmeticOverflow)?;
				Self::transfer_funds(pallet_account.clone(), owner.clone(), refund_amount)?;
				PropertyOwner::<T>::take(nft_details.asset_id);
				PropertyOwnerToken::<T>::take(nft_details.asset_id, owner);
			}
			Ok(())
		}

		fn buying_token_process(
			listing_id: u32,
			transfer_from: AccountIdOf<T>,
			account: AccountIdOf<T>,
			mut listing_details: ListingDetailsType<T>,
			price: AssetBalanceOf<T>,
			amount: u32,
		) -> DispatchResult {
			Self::calculate_fees(price, transfer_from.clone(), listing_details.seller.clone())?;
			let user_lookup = <T::Lookup as StaticLookup>::unlookup(account.clone());
			let asset_id: AssetId<T> = listing_details.asset_id.into();
			let token_amount = amount.into();
			let pallet_origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
			pallet_assets::Pallet::<T>::transfer(
				pallet_origin,
				asset_id.into().into(),
				user_lookup,
				token_amount,
			)
			.map_err(|_| Error::<T>::NotEnoughFunds)?;
			let mut old_token_owner_amount = PropertyOwnerToken::<T>::take(
				listing_details.asset_id,
				listing_details.seller.clone(),
			);
			old_token_owner_amount = old_token_owner_amount
				.checked_sub(amount)
				.ok_or(Error::<T>::ArithmeticUnderflow)?;
			if old_token_owner_amount == 0 {
				let mut owner_list = PropertyOwner::<T>::take(listing_details.asset_id);
				let index = owner_list
					.iter()
					.position(|x| *x == listing_details.seller.clone())
					.ok_or(Error::<T>::InvalidIndex)?;
				owner_list.remove(index);
				PropertyOwner::<T>::insert(listing_details.asset_id, owner_list);
			} else {
				PropertyOwnerToken::<T>::insert(
					listing_details.asset_id,
					listing_details.seller.clone(),
					old_token_owner_amount,
				);
			}
			if PropertyOwner::<T>::get(listing_details.asset_id).contains(&account) {
				let mut buyer_token_amount =
					PropertyOwnerToken::<T>::take(listing_details.asset_id, account.clone());
				buyer_token_amount =
					buyer_token_amount.checked_add(amount).ok_or(Error::<T>::ArithmeticOverflow)?;
				PropertyOwnerToken::<T>::insert(
					listing_details.asset_id,
					account.clone(),
					buyer_token_amount,
				);
			} else {
				PropertyOwner::<T>::try_mutate(listing_details.asset_id, |keys| {
					keys.try_push(account.clone()).map_err(|_| Error::<T>::TooManyTokenBuyer)?;
					Ok::<(), DispatchError>(())
				})?;
				PropertyOwnerToken::<T>::insert(listing_details.asset_id, account.clone(), amount);
			}
			listing_details.amount = listing_details
				.amount
				.checked_sub(amount)
				.ok_or(Error::<T>::ArithmeticUnderflow)?;
			if listing_details.amount > 0 {
				TokenListings::<T>::insert(listing_id, listing_details.clone());
			}
			Self::deposit_event(Event::<T>::TokenBought {
				asset_id: listing_details.asset_id,
				buyer: account.clone(),
				price: listing_details.token_price,
			});
			Ok(())
		}

		fn calculate_fees(
			price: AssetBalanceOf<T>,
			sender: AccountIdOf<T>,
			receiver: AccountIdOf<T>,
		) -> DispatchResult {
			let fees = price
				.checked_div(&Self::u64_to_balance_option(100)?)
				.ok_or(Error::<T>::DivisionError)?;
			let treasury_id = Self::treasury_account_id();
			let seller_part = price
				.checked_mul(&Self::u64_to_balance_option(99)?)
				.ok_or(Error::<T>::MultiplyError)?
				.checked_div(&Self::u64_to_balance_option(100)?)
				.ok_or(Error::<T>::DivisionError)?;
			Self::transfer_funds(sender.clone(), treasury_id, fees)?;
			Self::transfer_funds(sender, receiver, seller_part)?;
			Ok(())
		}

		/// Set the default collection configuration for creating a collection.
		fn default_collection_config() -> CollectionConfig<
			CurrencyBalanceOf<T>,
			BlockNumberFor<T>,
			<T as pallet_nfts::Config>::CollectionId,
		> {
			Self::collection_config_from_disabled_settings(
				CollectionSetting::DepositRequired.into(),
			)
		}

		fn collection_config_from_disabled_settings(
			settings: BitFlags<CollectionSetting>,
		) -> CollectionConfig<
			CurrencyBalanceOf<T>,
			BlockNumberFor<T>,
			<T as pallet_nfts::Config>::CollectionId,
		> {
			CollectionConfig {
				settings: CollectionSettings::from_disabled(settings),
				max_supply: None,
				mint_settings: MintSettings::default(),
			}
		}

		/// Set the default item configuration for minting a nft.
		fn default_item_config() -> ItemConfig {
			ItemConfig { settings: ItemSettings::all_enabled() }
		}

		/// Converts a u64 to a balance.
		pub fn u64_to_balance_option(input: u64) -> Result<AssetBalanceOf<T>, Error<T>> {
			input.try_into().map_err(|_| Error::<T>::ConversionError)
		}

		fn transfer_funds(
			from: AccountIdOf<T>,
			to: AccountIdOf<T>,
			amount: AssetBalanceOf<T>,
		) -> DispatchResult {
			let u32_amunt =
				TryInto::<u32>::try_into(amount).map_err(|_| Error::<T>::ConversionError)?;
			let origin: OriginFor<T> = RawOrigin::Signed(from).into();
			let account_lookup = <T::Lookup as StaticLookup>::unlookup(to);
			let asset_id: AssetId<T> = 1.into();
			let token_amount = u32_amunt.into();
			Ok(pallet_assets::Pallet::<T>::transfer(
				origin,
				asset_id.into().into(),
				account_lookup,
				token_amount,
			)
			.map_err(|_| Error::<T>::NotEnoughFunds)?)
		}
	}
}

sp_api::decl_runtime_apis! {
    pub trait NftMarketplaceApi<AccountId> 
	where
		AccountId: Codec
	{
        fn get_marketplace_account_id() -> AccountId;
    }
}