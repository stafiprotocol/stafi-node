// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Low-level types used throughout the Substrate code.

// #![warn(missing_docs)]

#![cfg_attr(not(feature = "std"), no_std)]
// rjson
#![feature(core_intrinsics)]

use sr_primitives::{
	generic, traits::{Verify, BlakeTwo256, IdentifyAccount}, OpaqueExtrinsic, MultiSignature
};

use rstd::prelude::*;

#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};
use parity_codec::{Encode, Decode};


/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them.
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Type used for expressing timestamp.
pub type Moment = u64;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = primitives::H256;

/// A timestamp: milliseconds since the unix epoch.
/// `u64` is enough to represent a duration of half a billion years, when the
/// time scale is milliseconds.
pub type Timestamp = u64;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;
/// Header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type.
pub type Block = generic::Block<Header, OpaqueExtrinsic>;
/// Block ID.
pub type BlockId = generic::BlockId<Block>;


/// A result of execution of a contract.
#[derive(Eq, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub enum ContractExecResult {
	/// The contract returned successfully.
	///
	/// There is a status code and, optionally, some data returned by the contract.
	Success {
		/// Status code returned by the contract.
		status: u8,
		/// Output data returned by the contract.
		///
		/// Can be empty.
		data: Vec<u8>,
	},
	/// The contract execution either trapped or returned an error.
	Error,
}


sr_api::decl_runtime_apis! {
	pub trait MultisigAddrApi {
		fn multisig_addr() -> Vec<MultisigAddr>;
	}

	pub trait StakesApi {
		fn get_stake_hash(account: AccountId) -> Vec<Hash>;
		fn get_stake_data(hash: Hash) -> Option<XtzStakeData<AccountId, Hash, Balance>>;
	}
}

pub mod constants;

pub mod stafistaking;
pub use stafistaking::{XtzStakeData, XtzStakeStage, AtomStakeData, AtomStakeStage};

pub mod externalrpc;
pub use externalrpc::{VerifiedData, OcVerifiedData, VerifyStatus, TxHashType, BabeIdType, HostData};

pub mod tokenbalances;
pub use tokenbalances::{BondTokenLockType, BondTokenLockStatus, CustomRedeemData};

pub mod multisig;
pub use multisig::{MultisigAddr};

pub mod chain;
pub use chain::{ChainType, StakeTokenType, Symbol};

pub mod rjson;

