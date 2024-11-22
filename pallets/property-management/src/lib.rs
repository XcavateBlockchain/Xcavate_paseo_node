#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

use frame_support::{
	traits::{Currency, ExistenceRequirement::KeepAlive, ReservableCurrency},
	PalletId,
};

use frame_support::sp_runtime::{
	traits::{AccountIdConversion, CheckedAdd, CheckedDiv, CheckedMul, Zero, CheckedSub},
	Saturating,
};

use pallet_assets::Instance1;

use codec::Codec;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[cfg(feature = "runtime-benchmarks")]
	pub struct AssetHelper;

	#[cfg(feature = "runtime-benchmarks")]
	pub trait BenchmarkHelper<AssetId, T> {
		fn to_asset(i: u32) -> AssetId;
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl<T: Config> BenchmarkHelper<AssetId<T>, T> for AssetHelper {
		fn to_asset(i: u32) -> AssetId<T> {
			i.into()
		}
	}

	/// Info for the letting agent.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct LettingAgentInfo<T: Config> {
		pub account: AccountIdOf<T>,
		pub region: u32,
		pub locations: BoundedVec<LocationId<T>, T::MaxLocations>,
		pub assigned_properties: BoundedVec<u32, T::MaxProperties>,
		pub deposited: bool,
	}

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ pallet_xcavate_whitelist::Config
		+ pallet_nft_marketplace::Config
		+ pallet_assets::Config<Instance1>
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Type representing the weight of this pallet.
		type WeightInfo: WeightInfo;

		/// The reservable currency type.
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		/// The property management's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		#[cfg(feature = "runtime-benchmarks")]
		type Helper: crate::BenchmarkHelper<
			<Self as pallet_assets::Config<Instance1>>::AssetId,
			Self,
		>;

		/// Origin who can set a new letting agent.
		type AgentOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// The minimum amount of a letting agent that has to be staked.
		type LettingAgentDeposit: Get<BalanceOf<Self>>;

		/// The maximum amount of properties that can be assigned to a letting agent.
		#[pallet::constant]
		type MaxProperties: Get<u32>;

		/// The maximum amount of letting agents in a location.
		#[pallet::constant]
		type MaxLettingAgents: Get<u32>;

		/// The maximum amount of locations a letting agent can be assigned to.
		#[pallet::constant]
		type MaxLocations: Get<u32>;

		/// The Governance's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type GovernanceId: Get<PalletId>;

		/// The reserve a property needs to have.
		type PropertyReserve: Get<BalanceOf<Self>>;

		/// Asset id type from pallet assets.
		type AssetId: IsType<<Self as pallet_assets::Config<Instance1>>::AssetId>
			+ Parameter
			+ From<u32>
			+ Ord
			+ Copy;

		/// Multiplier for polkadot js.
		type PolkadotJsMultiplier: Get<BalanceOf<Self>>;
	}

	pub type AssetId<T> = <T as Config>::AssetId;

	pub type LocationId<T> = BoundedVec<u8, <T as pallet_nft_marketplace::Config>::PostcodeLimit>;

	/// Mapping from the real estate object to the letting agent.
	#[pallet::storage]
	pub type LettingStorage<T> = StorageMap<_, Blake2_128Concat, u32, AccountIdOf<T>, OptionQuery>;

	/// Mapping from account to currently stored balance.
	#[pallet::storage]
	pub type StoredFunds<T> =
		StorageMap<_, Blake2_128Concat, AccountIdOf<T>, BalanceOf<T>, ValueQuery>;

	/// Mapping of asset id to the stored balance for a property.
	#[pallet::storage]
	pub type PropertyReserve<T> =
		StorageMap<_, Blake2_128Concat, u32, BalanceOf<T>, ValueQuery>;

	/// Mapping of asset id to the stored debts of a property.
	#[pallet::storage]
	pub type PropertyDebts<T> =
		StorageMap<_, Blake2_128Concat, u32, BalanceOf<T>, ValueQuery>;

	/// Mapping from account to letting agent info
	#[pallet::storage]
	pub type LettingInfo<T: Config> =
		StorageMap<_, Blake2_128Concat, AccountIdOf<T>, LettingAgentInfo<T>, OptionQuery>;

	/// Mapping from region and location to the letting agents of this location.
	#[pallet::storage]
	pub type LettingAgentLocations<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Blake2_128Concat,
		LocationId<T>,
		BoundedVec<AccountIdOf<T>, T::MaxLettingAgents>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new letting agent got set.
		LettingAgentAdded { region: u32, who: T::AccountId },
		/// A letting agent deposited the necessary funds.
		Deposited { who: T::AccountId },
		/// A letting agent has been added to a location.
		LettingAgentAddedToLocation { who: T::AccountId, location: LocationId<T> },
		/// A letting agent has been added to a property.
		LettingAgentSet { asset_id: u32, who: T::AccountId },
		/// The rental income has been distributed.
		IncomeDistributed { asset_id: u32, amount: BalanceOf<T> },
		/// A user withdrew funds.
		WithdrawFunds { who: T::AccountId, amount: BalanceOf<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Error by convertion to balance type.
		ConversionError,
		/// Error by dividing a number.
		DivisionError,
		/// Error by multiplying a number.
		MultiplyError,
		ArithmeticOverflow,
		ArithmeticUnderflow,
		/// The caller has no funds stored.
		UserHasNoFundsStored,
		/// The pallet has not enough funds.
		NotEnoughFunds,
		/// The letting agent has already too many assigned properties.
		TooManyAssignedProperties,
		/// No letting agent could be selected.
		NoLettingAgentFound,
		/// The region is not registered.
		RegionUnknown,
		/// The location has already the maximum amount of letting agents.
		TooManyLettingAgents,
		/// The letting agent is already active in too many locations.
		TooManyLocations,
		/// The user is not a property owner and has no permission to deposit.
		NoPermission,
		/// The letting agent of this property is already set.
		LettingAgentAlreadySet,
		/// The real estate object could not be found.
		NoObjectFound,
		/// The account is not a letting agent of this location.
		AgentNotFound,
		/// The letting already deposited the necessary amount.
		AlreadyDeposited,
		/// The location is not registered.
		LocationUnknown,
		/// The letting agent is already assigned to this location.
		LettingAgentInLocation,
		/// The letting agent has no funds deposited.
		NotDeposited,
		/// The letting agent is already registered.
		LettingAgentExists,
		/// The property does not have enough reserves to make this proposal.
		NotEnoughReserves,
		/// This asset has no token.
		AssetNotFound,
		/// This letting agent has no location.
		NoLoactions,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Adds an account as a letting agent.
		///
		/// The origin must be the AgentOrigin.
		///
		/// Parameters:
		/// - `region`: The region number where the letting agent should be added to.
		/// - `location`: The location number where the letting agent should be added to.
		/// - `letting_agent`: The account of the letting_agent.
		///
		/// Emits `LettingAgentAdded` event when succesfful.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::add_letting_agent())]
		pub fn add_letting_agent(
			origin: OriginFor<T>,
			region: u32,
			location: LocationId<T>,
			letting_agent: AccountIdOf<T>,
		) -> DispatchResult {
			T::AgentOrigin::ensure_origin(origin)?;
			ensure!(
				pallet_nft_marketplace::RegionCollections::<T>::get(region).is_some(),
				Error::<T>::RegionUnknown
			);
			ensure!(
				pallet_nft_marketplace::LocationRegistration::<T>::get(
					region,
					location.clone()
				),
				Error::<T>::LocationUnknown
			);
			ensure!(
				!<LettingInfo<T>>::contains_key(letting_agent.clone()),
				Error::<T>::LettingAgentExists
			);
			let mut letting_info = LettingAgentInfo {
				account: letting_agent.clone(),
				region,
				locations: Default::default(),
				assigned_properties: Default::default(),
				deposited: Default::default(),
			};
			letting_info
				.locations
				.try_push(location)
				.map_err(|_| Error::<T>::TooManyLocations)?;
			LettingInfo::<T>::insert(letting_agent.clone(), letting_info);
			Self::deposit_event(Event::<T>::LettingAgentAdded { region, who: letting_agent });
			Ok(())
		}

		/// Lets the letting agent deposit the required amount, to be able to operate as a letting agent.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Emits `Deposited` event when succesfful.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::letting_agent_deposit())]
		pub fn letting_agent_deposit(origin: OriginFor<T>) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			LettingInfo::<T>::try_mutate(signer.clone(), |maybe_letting_info|{
				let letting_info = maybe_letting_info.as_mut().ok_or(Error::<T>::NoPermission)?;
				ensure!(!letting_info.deposited, Error::<T>::AlreadyDeposited);
				ensure!(letting_info.locations.len() > 0, Error::<T>::NoLoactions);
				ensure!(
					!LettingAgentLocations::<T>::get(
						letting_info.region,
						letting_info.locations[0].clone()
					)
					.contains(&signer),
					Error::<T>::LettingAgentInLocation
				);
				<T as pallet::Config>::Currency::reserve(
					&signer,
					<T as Config>::LettingAgentDeposit::get(),
				)?;
				letting_info.deposited = true;
				LettingAgentLocations::<T>::try_mutate(
					letting_info.region,
					letting_info.locations[0].clone(),
					|keys| {
						keys.try_push(signer.clone()).map_err(|_| Error::<T>::TooManyLettingAgents)?;
						Ok::<(), DispatchError>(())
					},
				)?;
				Ok::<(), DispatchError>(())
			})?;
			Self::deposit_event(Event::<T>::Deposited { who: signer });
			Ok(())
		}

		/// Adds a letting agent to a location.
		///
		/// The origin must be the AgentOrigin.
		///
		/// Parameters:
		/// - `location`: The location number where the letting agent should be added to.
		/// - `letting_agent`: The account of the letting_agent.
		///
		/// Emits `LettingAgentAddedToLocation` event when succesfful.
		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::add_letting_agent_to_location())]
		pub fn add_letting_agent_to_location(
			origin: OriginFor<T>,
			location: LocationId<T>,
			letting_agent: AccountIdOf<T>,
		) -> DispatchResult {
			T::AgentOrigin::ensure_origin(origin)?;
			LettingInfo::<T>::try_mutate(letting_agent.clone(), |maybe_letting_info| {
				let letting_info = maybe_letting_info.as_mut().ok_or(Error::<T>::NoLettingAgentFound)?;
				ensure!(letting_info.deposited, Error::<T>::NotDeposited);
				ensure!(
					pallet_nft_marketplace::LocationRegistration::<T>::get(
						letting_info.region,
						location.clone()
					),
					Error::<T>::LocationUnknown
				);
				ensure!(
					!LettingAgentLocations::<T>::get(letting_info.region, location.clone())
						.contains(&letting_agent),
					Error::<T>::LettingAgentInLocation
				);
				LettingAgentLocations::<T>::try_mutate(
					letting_info.region,
					location.clone(),
					|keys| {
						keys.try_push(letting_agent.clone())
							.map_err(|_| Error::<T>::TooManyLettingAgents)?;
						Ok::<(), DispatchError>(())
					},
				)?;
				letting_info
					.locations
					.try_push(location.clone())
					.map_err(|_| Error::<T>::TooManyLocations)?;
				Ok::<(), DispatchError>(())
			})?;
			Self::deposit_event(Event::<T>::LettingAgentAddedToLocation {
				who: letting_agent,
				location,
			});
			Ok(())
		}

		/// Sets a letting agent for a property.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `asset_id`: The asset id of the real estate object.
		///
		/// Emits `LettingAgentSet` event when succesfful.
		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_letting_agent())]
		pub fn set_letting_agent(origin: OriginFor<T>, asset_id: u32) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			ensure!(pallet_nft_marketplace::AssetIdDetails::<T>::get(asset_id).is_some(), Error::<T>::NoObjectFound);
			ensure!(LettingStorage::<T>::get(asset_id).is_none(), Error::<T>::LettingAgentAlreadySet);
			LettingInfo::<T>::try_mutate(signer.clone(), |maybe_letting_info|{
				let letting_info = maybe_letting_info.as_mut().ok_or(Error::<T>::AgentNotFound)?;
				LettingStorage::<T>::insert(asset_id, signer.clone());
				letting_info
					.assigned_properties
					.try_push(asset_id)
					.map_err(|_| Error::<T>::TooManyAssignedProperties)?;
				Ok::<(), DispatchError>(())
			})?;
			Self::deposit_event(Event::<T>::LettingAgentSet {
				asset_id,
				who: signer,
			});
			Ok(())
		}

		/// Lets the letting agent distribute the income for a property.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `asset_id`: The asset id of the property.
		/// - `amount`: The amount of funds that should be distributed.
		///
		/// Emits `IncomeDistributed` event when succesfful.
		#[pallet::call_index(4)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::distribute_income())]
		pub fn distribute_income(
			origin: OriginFor<T>,
			asset_id: u32,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			let letting_agent = LettingStorage::<T>::get(asset_id).ok_or(Error::<T>::NoLettingAgentFound)?;
			ensure!(letting_agent == signer, Error::<T>::NoPermission);
		
			let scaled_amount = amount
				.checked_mul(&Self::u64_to_balance_option(1)?)  // Modify the scale factor if needed
				.ok_or(Error::<T>::MultiplyError)?;
		
			<T as pallet::Config>::Currency::transfer(
				&signer,
				&Self::account_id(),
				scaled_amount.saturating_mul(
					<T as Config>::PolkadotJsMultiplier::get(),
				),
				KeepAlive,
			).map_err(|_| Error::<T>::NotEnoughFunds)?;
		
			let owner_list = pallet_nft_marketplace::PropertyOwner::<T>::get(asset_id);
			let mut governance_amount = BalanceOf::<T>::zero();
			let property_reserve = PropertyReserve::<T>::get(asset_id);
			let property_info = pallet_nft_marketplace::AssetIdDetails::<T>::get(asset_id)
				.ok_or(Error::<T>::NoObjectFound)?;
			let property_price = property_info.price;
		
			let property_price_converted: BalanceOf<T> = TryInto::<u64>::try_into(property_price)
				.map_err(|_| Error::<T>::ConversionError)?
				.try_into()
				.map_err(|_| Error::<T>::ConversionError)?;
		
			let required_reserve = property_price_converted
				.checked_div(&Self::u64_to_balance_option(25)?)
				.ok_or(Error::<T>::DivisionError)?
				.checked_div(&Self::u64_to_balance_option(12)?)
				.ok_or(Error::<T>::DivisionError)?;
		
			let property_debts = PropertyDebts::<T>::get(asset_id);
		
			// Pay property debts first
			let amount_to_pay_debts = core::cmp::min(amount, property_debts);
			if amount_to_pay_debts > BalanceOf::<T>::zero() {
				<T as pallet::Config>::Currency::transfer(
					&Self::account_id(),
					&letting_agent,
					amount_to_pay_debts.saturating_mul(
						<T as Config>::PolkadotJsMultiplier::get(),
					),
					KeepAlive,
				).map_err(|_| Error::<T>::NotEnoughFunds)?;
				let new_debts = property_debts.checked_sub(&amount_to_pay_debts)
					.ok_or(Error::<T>::ArithmeticUnderflow)?;
				PropertyDebts::<T>::insert(asset_id, new_debts);
		
				governance_amount = amount_to_pay_debts;
			}
		
			// Calculate remaining amount after paying debts
			let remaining_amount = amount.saturating_sub(governance_amount);
		
			// Fill property reserves with remaining amount
			if property_reserve < required_reserve {
				let missing_amount = required_reserve.saturating_sub(property_reserve);
				let reserve_amount = core::cmp::min(remaining_amount, missing_amount);
		
				if reserve_amount > BalanceOf::<T>::zero() {
					<T as pallet::Config>::Currency::transfer(
						&Self::account_id(),
						&Self::governance_account_id(),
						reserve_amount.saturating_mul(
							<T as Config>::PolkadotJsMultiplier::get(),
						),
						KeepAlive,
					).map_err(|_| Error::<T>::NotEnoughFunds)?;
		
					let new_property_reserve = property_reserve
						.checked_add(&reserve_amount)
						.ok_or(Error::<T>::ArithmeticOverflow)?;
					PropertyReserve::<T>::insert(asset_id, new_property_reserve);
				}
		
				governance_amount = governance_amount
					.checked_add(&reserve_amount)
					.ok_or(Error::<T>::ArithmeticOverflow)?;
			}
		
			// Distribute remaining amount to property owners
			let final_remaining_amount = amount.saturating_sub(governance_amount);
			let total_token = property_info.token_amount;
			for owner in owner_list {
				let token_amount = pallet_nft_marketplace::PropertyOwnerToken::<T>::get(
					asset_id,
					owner.clone(),
				);
				let amount_for_owner = Self::u64_to_balance_option(token_amount as u64)?
					.checked_mul(&final_remaining_amount)
					.ok_or(Error::<T>::MultiplyError)?
					.checked_div(&Self::u64_to_balance_option(total_token.into())?)
					.ok_or(Error::<T>::DivisionError)?;
				StoredFunds::<T>::try_mutate(owner.clone(), |old_funds| {
					*old_funds = old_funds
					.checked_add(&amount_for_owner)
					.ok_or(Error::<T>::ArithmeticOverflow)?;
					Ok::<(), DispatchError>(())
				})?;
			}
		
			Self::deposit_event(Event::<T>::IncomeDistributed {
				asset_id,
				amount: final_remaining_amount,
			});
			Ok(())
		}

		/// Lets a property owner withdraw the distributed funds.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Emits `WithdrawFunds` event when succesfful.
		#[pallet::call_index(5)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::withdraw_funds())]
		pub fn withdraw_funds(origin: OriginFor<T>) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			let amount = StoredFunds::<T>::take(signer.clone());
			ensure!(
				!amount.is_zero(),
				Error::<T>::UserHasNoFundsStored
			);
			<T as pallet::Config>::Currency::transfer(
				&Self::account_id(),
				&signer,
				amount.saturating_mul(
					<T as Config>::PolkadotJsMultiplier::get(),
				),
				KeepAlive,
			)
			.map_err(|_| Error::<T>::NotEnoughFunds)?;
			Self::deposit_event(Event::<T>::WithdrawFunds { who: signer, amount });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Get the account id of the pallet
		pub fn account_id() -> AccountIdOf<T> {
			<T as pallet::Config>::PalletId::get().into_account_truncating()
		}

		/// Get the account id of the governance pallet
		pub fn governance_account_id() -> AccountIdOf<T> {
			<T as pallet::Config>::GovernanceId::get().into_account_truncating()
		}

		/// Converts a u64 to a balance.
		pub fn u64_to_balance_option(input: u64) -> Result<BalanceOf<T>, Error<T>> {
			input.try_into().map_err(|_| Error::<T>::ConversionError)
		}

		/// Removes bad letting agents.
		pub fn remove_bad_letting_agent(
			asset_id: u32,
		) -> DispatchResult {
			LettingStorage::<T>::take(asset_id);
			Ok(())
		}

		/// Decreases the reserve of a property.
		pub fn decrease_reserves(asset_id: u32, amount: BalanceOf<T>) -> DispatchResult {
			PropertyReserve::<T>::try_mutate(asset_id, |property_reserve| -> Result<(), DispatchError> {
				ensure!(*property_reserve >= amount, Error::<T>::NotEnoughReserves);
				*property_reserve = property_reserve.saturating_sub(amount);				
				Ok(())
			})
		}

		/// Increases the debts of a property.
		pub fn increase_debts(asset_id: u32, amount: BalanceOf<T>) -> DispatchResult {
			PropertyDebts::<T>::try_mutate(asset_id, |property_debts| {
				*property_debts = property_debts.saturating_add(amount);
				Ok(())
			})
		}
	}
}

sp_api::decl_runtime_apis! {
    pub trait PropertyManagementApi<AccountId> 
	where
		AccountId: Codec
	{
        fn get_management_account_id() -> AccountId;
    }
}

