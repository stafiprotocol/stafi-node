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

use super::{balances, system, Codec, Decode, Encode};
use rstd::prelude::*;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq)]
pub enum TransactionType {
    TransferStafi,
}

impl Default for TransactionType {
    fn default() -> Self {
        TransactionType::TransferStafi
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(PartialEq, Eq, Clone, Encode, Decode, Default)]
pub struct Transaction {
    tx_type: TransactionType,
    data: Vec<u8>,
}

impl Transaction {
    pub fn new(tx_type: TransactionType, data: Vec<u8>) -> Self {
        Transaction { tx_type, data }
    }

    pub fn tx_type(&self) -> TransactionType {
        self.tx_type
    }

    pub fn data(&self) -> Vec<u8> {
        self.data.clone()
    }
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
pub struct Transfer<AccountId, Balance>
where
    AccountId: Codec,
    Balance: Codec,
{
    pub to: AccountId,
    pub value: Balance,
}

#[allow(unused)]
pub type TransferT<T> = Transfer<<T as system::Trait>::AccountId, <T as balances::Trait>::Balance>;
