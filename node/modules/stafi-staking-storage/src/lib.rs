// Copyright 2018 Stafi Protocol, Inc.
// This file is part of Stafi.

// Stafi is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Stafi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stafi.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

use support::{decl_module, decl_storage};
use node_primitives::{Balance, XtzStakeData};
use sr_std::prelude::*;

pub trait Trait: system::Trait {}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as StakingStorage {
		pub XtzTransferInitDataRecords get(xtz_transfer_init_data_records): Vec<XtzStakeData<T::AccountId, T::Hash, Balance>>;
	}
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {}
}

impl<T: Trait> Module<T> {

	pub fn put_xtz_transfer_init_data_records(datas: Vec<XtzStakeData<T::AccountId, T::Hash, Balance>>) {
		<XtzTransferInitDataRecords<T>>::put(datas);
    }

}