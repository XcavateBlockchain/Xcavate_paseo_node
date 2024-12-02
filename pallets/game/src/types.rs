use crate::*;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;

/// Difficulty level of game enum.
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub enum DifficultyLevel {
	Practice,
	Player,
	Pro,
}

/// Offer enum.
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub enum Offer {
	Accept,
	Reject,
}

/// Nft color enum.
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub enum NftColor {
	Xorange,
	Xpink,
	Xblue,
	Xcyan,
	Xcoral,
	Xpurple,
	Xleafgreen,
	Xgreen,
}

impl NftColor {
	pub fn from_index(index: usize) -> Option<Self> {
		match index {
			0 => Some(NftColor::Xorange),
			1 => Some(NftColor::Xpink),
			2 => Some(NftColor::Xblue),
			3 => Some(NftColor::Xcyan),
			4 => Some(NftColor::Xcoral),
			5 => Some(NftColor::Xpurple),
			6 => Some(NftColor::Xleafgreen),
			7 => Some(NftColor::Xgreen),
			_ => None,
		}
	}
}

/// AccountId storage.
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub struct PalletIdStorage<T: Config> {
	pallet_id: AccountIdOf<T>,
}

/// Game Data.
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct GameData<T: Config> {
	pub difficulty: DifficultyLevel,
	pub player: AccountIdOf<T>,
	pub property: PropertyInfoData<T>,
	pub guess: Option<u32>,
}

/// Listing infos of a NFT.
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ListingInfo<CollectionId, ItemId, T: Config> {
	pub owner: AccountIdOf<T>,
	pub collection_id: CollectionId,
	pub item_id: ItemId,
}

/// Offer infos of a listing.
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct OfferInfo<CollectionId, ItemId, T: Config> {
	pub owner: AccountIdOf<T>,
	pub listing_id: u32,
	pub collection_id: CollectionId,
	pub item_id: ItemId,
}

/// Struct to store the property data for a game.
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(
	Encode,
	Decode,
	Clone,
	PartialEq,
	Eq,
	MaxEncodedLen,
	frame_support::pallet_prelude::RuntimeDebugNoBound,
	TypeInfo,
)]
#[scale_info(skip_type_params(T))]
pub struct PropertyInfoData<T: Config> {
	pub id: u32,
	pub data: BoundedVec<u8, <T as Config>::StringLimit>,
}

/// Struct for the user datas.
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct User<T: Config> {
	pub points: u32,
	pub wins: u32,
	pub losses: u32,
	pub practise_rounds: u8,
	pub last_played_round: u32,
	pub next_token_request: BlockNumberFor<T>,
	pub nfts: CollectedColors,
}

impl<T: pallet::Config> User<T> {
	pub fn add_nft_color(&mut self, color: NftColor) -> DispatchResult {
		self.nfts.add_nft_color(color)?;
		Ok(())
	}

	pub fn sub_nft_color(&mut self, color: NftColor) -> DispatchResult {
		self.nfts.sub_nft_color(color)?;
		Ok(())
	}

	pub fn has_four_of_all_colors(&self) -> bool {
		self.nfts.has_four_of_all_colors()
	}

	pub fn calculate_points(&mut self, color: NftColor) -> u32 {
		match color {
			NftColor::Xorange if self.nfts.xorange == 1 => 100,
			NftColor::Xorange if self.nfts.xorange == 2 => 120,
			NftColor::Xorange if self.nfts.xorange == 3 => 220,
			NftColor::Xorange if self.nfts.xorange == 4 => 340,
			NftColor::Xpink if self.nfts.xpink == 1 => 100,
			NftColor::Xpink if self.nfts.xpink == 2 => 120,
			NftColor::Xpink if self.nfts.xpink == 3 => 220,
			NftColor::Xpink if self.nfts.xpink == 4 => 340,
			NftColor::Xblue if self.nfts.xblue == 1 => 100,
			NftColor::Xblue if self.nfts.xblue == 2 => 120,
			NftColor::Xblue if self.nfts.xblue == 3 => 220,
			NftColor::Xblue if self.nfts.xblue == 4 => 340,
			NftColor::Xcyan if self.nfts.xcyan == 1 => 100,
			NftColor::Xcyan if self.nfts.xcyan == 2 => 120,
			NftColor::Xcyan if self.nfts.xcyan == 3 => 220,
			NftColor::Xcyan if self.nfts.xcyan == 4 => 340,
			NftColor::Xcoral if self.nfts.xcoral == 1 => 100,
			NftColor::Xcoral if self.nfts.xcoral == 2 => 120,
			NftColor::Xcoral if self.nfts.xcoral == 3 => 220,
			NftColor::Xcoral if self.nfts.xcoral == 4 => 340,
			NftColor::Xpurple if self.nfts.xpurple == 1 => 100,
			NftColor::Xpurple if self.nfts.xpurple == 2 => 120,
			NftColor::Xpurple if self.nfts.xpurple == 3 => 220,
			NftColor::Xpurple if self.nfts.xpurple == 4 => 340,
			NftColor::Xleafgreen if self.nfts.xleafgreen == 1 => 100,
			NftColor::Xleafgreen if self.nfts.xleafgreen == 2 => 120,
			NftColor::Xleafgreen if self.nfts.xleafgreen == 3 => 220,
			NftColor::Xleafgreen if self.nfts.xleafgreen == 4 => 340,
			NftColor::Xgreen if self.nfts.xgreen == 1 => 100,
			NftColor::Xgreen if self.nfts.xgreen == 2 => 120,
			NftColor::Xgreen if self.nfts.xgreen == 3 => 220,
			NftColor::Xgreen if self.nfts.xgreen == 4 => 340,
			_ => 0,
		}
	}

	pub fn subtracting_calculate_points(&mut self, color: NftColor) -> u32 {
		match color {
			NftColor::Xorange if self.nfts.xorange == 0 => 100,
			NftColor::Xorange if self.nfts.xorange == 1 => 120,
			NftColor::Xorange if self.nfts.xorange == 2 => 220,
			NftColor::Xorange if self.nfts.xorange == 3 => 340,
			NftColor::Xpink if self.nfts.xpink == 0 => 100,
			NftColor::Xpink if self.nfts.xpink == 1 => 120,
			NftColor::Xpink if self.nfts.xpink == 2 => 220,
			NftColor::Xpink if self.nfts.xpink == 3 => 340,
			NftColor::Xblue if self.nfts.xblue == 0 => 100,
			NftColor::Xblue if self.nfts.xblue == 1 => 120,
			NftColor::Xblue if self.nfts.xblue == 2 => 220,
			NftColor::Xblue if self.nfts.xblue == 3 => 340,
			NftColor::Xcyan if self.nfts.xcyan == 0 => 100,
			NftColor::Xcyan if self.nfts.xcyan == 1 => 120,
			NftColor::Xcyan if self.nfts.xcyan == 2 => 220,
			NftColor::Xcyan if self.nfts.xcyan == 3 => 340,
			NftColor::Xcoral if self.nfts.xcoral == 0 => 100,
			NftColor::Xcoral if self.nfts.xcoral == 1 => 120,
			NftColor::Xcoral if self.nfts.xcoral == 2 => 220,
			NftColor::Xcoral if self.nfts.xcoral == 3 => 340,
			NftColor::Xpurple if self.nfts.xpurple == 0 => 100,
			NftColor::Xpurple if self.nfts.xpurple == 1 => 120,
			NftColor::Xpurple if self.nfts.xpurple == 2 => 220,
			NftColor::Xpurple if self.nfts.xpurple == 3 => 340,
			NftColor::Xleafgreen if self.nfts.xleafgreen == 0 => 100,
			NftColor::Xleafgreen if self.nfts.xleafgreen == 1 => 120,
			NftColor::Xleafgreen if self.nfts.xleafgreen == 2 => 220,
			NftColor::Xleafgreen if self.nfts.xleafgreen == 3 => 340,
			NftColor::Xgreen if self.nfts.xgreen == 0 => 100,
			NftColor::Xgreen if self.nfts.xgreen == 1 => 120,
			NftColor::Xgreen if self.nfts.xgreen == 2 => 220,
			NftColor::Xgreen if self.nfts.xgreen == 3 => 340,
			_ => 0,
		}
	}
}

#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo, Default)]
#[scale_info(skip_type_params(T))]
pub struct CollectedColors {
	pub xorange: u32,
	pub xpink: u32,
	pub xblue: u32,
	pub xcyan: u32,
	pub xcoral: u32,
	pub xpurple: u32,
	pub xleafgreen: u32,
	pub xgreen: u32,
}

impl CollectedColors {
	pub fn add_nft_color(&mut self, color: NftColor) -> DispatchResult {
		match color {
			NftColor::Xorange => {
				self.xorange = self.xorange.checked_add(1).ok_or("Arithmetic overflow")?;
				Ok(())
			},
			NftColor::Xpink => {
				self.xpink = self.xpink.checked_add(1).ok_or("Arithmetic overflow")?;
				Ok(())
			},
			NftColor::Xblue => {
				self.xblue = self.xblue.checked_add(1).ok_or("Arithmetic overflow")?;
				Ok(())
			},
			NftColor::Xcyan => {
				self.xcyan = self.xcyan.checked_add(1).ok_or("Arithmetic overflow")?;
				Ok(())
			},
			NftColor::Xcoral => {
				self.xcoral = self.xcoral.checked_add(1).ok_or("Arithmetic overflow")?;
				Ok(())
			},
			NftColor::Xpurple => {
				self.xpurple = self.xpurple.checked_add(1).ok_or("Arithmetic overflow")?;
				Ok(())
			},
			NftColor::Xleafgreen => {
				self.xleafgreen = self.xleafgreen.checked_add(1).ok_or("Arithmetic overflow")?;
				Ok(())
			},
			NftColor::Xgreen => {
				self.xgreen = self.xgreen.checked_add(1).ok_or("Arithmetic overflow")?;
				Ok(())
			},
		}
	}

	pub fn sub_nft_color(&mut self, color: NftColor) -> DispatchResult {
		match color {
			NftColor::Xorange => {
				self.xorange = self.xorange.checked_sub(1).ok_or("Arithmetic underflow")?;
				Ok(())
			},
			NftColor::Xpink => {
				self.xpink = self.xpink.checked_sub(1).ok_or("Arithmetic underflow")?;
				Ok(())
			},
			NftColor::Xblue => {
				self.xblue = self.xblue.checked_sub(1).ok_or("Arithmetic underflow")?;
				Ok(())
			},
			NftColor::Xcyan => {
				self.xcyan = self.xcyan.checked_sub(1).ok_or("Arithmetic underflow")?;
				Ok(())
			},
			NftColor::Xcoral => {
				self.xcoral = self.xcoral.checked_sub(1).ok_or("Arithmetic underflow")?;
				Ok(())
			},
			NftColor::Xpurple => {
				self.xpurple = self.xpurple.checked_sub(1).ok_or("Arithmetic underflow")?;
				Ok(())
			},
			NftColor::Xleafgreen => {
				self.xleafgreen = self.xleafgreen.checked_sub(1).ok_or("Arithmetic underflow")?;
				Ok(())
			},
			NftColor::Xgreen => {
				self.xgreen = self.xgreen.checked_sub(1).ok_or("Arithmetic underflow")?;
				Ok(())
			},
		}
	}

	pub fn has_four_of_all_colors(&self) -> bool {
		self.xorange >= 4 &&
			self.xpink >= 4 &&
			self.xblue >= 4 &&
			self.xcyan >= 4 &&
			self.xcoral >= 4 &&
			self.xpurple >= 4 &&
			self.xleafgreen >= 4 &&
			self.xgreen >= 4
	}
}
