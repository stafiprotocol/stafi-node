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

pub mod atomstaking;
pub use atomstaking::{Module, Trait, RawEvent, Event};
pub use atomstaking::{AtomStakeStage, AtomStakeTokenData, AtomStakeData};


pub mod xtzstaking;
pub use xtzstaking::{Module as XtzModule, Trait as XtzTrait, RawEvent as XtzRawEvent, Event as XtzEvent};
pub use xtzstaking::{XtzStakeStage, XtzStakeTokenData, XtzStakeData};
