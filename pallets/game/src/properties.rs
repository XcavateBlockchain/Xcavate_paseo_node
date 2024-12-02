use crate::*;
use frame_support::pallet_prelude::*;

impl<T: Config> Pallet<T> {
	pub(crate) fn create_game_properties() -> DispatchResult {
		let new_property = PropertyInfoData {
			id: 147229391,
			data: "nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap(),
		};
		GameProperties::<T>::try_append(new_property.clone())
			.map_err(|_| Error::<T>::TooManyTest)?;
		let new_property = PropertyInfoData {
			id: 146480642,
			data: "nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap(),
		};
		GameProperties::<T>::try_append(new_property.clone())
			.map_err(|_| Error::<T>::TooManyTest)?;
		let new_property = PropertyInfoData {
			id: 147031382,
			data: "nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap(),
		};
		GameProperties::<T>::try_append(new_property.clone())
			.map_err(|_| Error::<T>::TooManyTest)?;
		let new_property = PropertyInfoData {
			id: 147031382,
			data: "nfdjakl;fueif;janf,dnfm,dhfhfdksks".as_bytes().to_vec().try_into().unwrap(),
		};
		GameProperties::<T>::try_append(new_property.clone())
			.map_err(|_| Error::<T>::TooManyTest)?;
		Ok(())
	}
}
