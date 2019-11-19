// Copyright 2019 Parity Technologies (UK) Ltd.
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

#![allow(missing_docs)]

use std::sync::Arc;

use client::blockchain::HeaderBackend;
use jsonrpc_core::{Result, Error, ErrorCode};
use jsonrpc_derive::rpc;
use stafi_primitives::{
	Block, BlockId, AccountId, Hash, StakesApi, XtzStakeStage
};
use codec::{Encode, Decode};
use sr_primitives::traits;
use hex;

use serde::{Serialize, Deserialize};
#[derive(Debug, Serialize, Deserialize)]
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct RpcXtzStakeData<AccountId, Hash> {
	pub id: Hash,
	pub initiator: AccountId,
	pub stage: XtzStakeStage,
	pub multi_sig_address: String,
	pub stake_amount: String,
}

#[rpc]
pub trait StakesRpcApi {
	#[rpc(name = "stake_xtz_gethash")]
	fn get_stake_hash(&self, account: AccountId) -> Result<Vec<Hash>>;
	#[rpc(name = "stake_xtz_getdata")]
	fn get_stake_data(&self, hash: Hash) -> Result<Option<RpcXtzStakeData<AccountId, Hash>>>;
}

pub struct Stakes<C> {
	client: Arc<C>,
}

impl<C> Stakes<C> {
	pub fn new(client: Arc<C>) -> Self {
		Stakes {
			client
		}
	}
}

impl<C> StakesRpcApi for Stakes<C>
where
	C: traits::ProvideRuntimeApi,
	C: HeaderBackend<Block>,
	C: Send + Sync + 'static,
	C::Api: StakesApi<Block>,
{
	fn get_stake_hash(&self, account: AccountId) -> Result<Vec<Hash>> {
		let api = self.client.runtime_api();
		let best = self.client.info().best_hash;
		let at = BlockId::hash(best);

		let hashes = api.get_stake_hash(&at, account).map_err(|e| Error {
			code: ErrorCode::ServerError(crate::constants::RUNTIME_ERROR),
			message: "Unable to query stake hash.".into(),
			data: Some(format!("{:?}", e).into()),
		})?;

		Ok(hashes)
	}

	fn get_stake_data(&self, hash: Hash) -> Result<Option<RpcXtzStakeData<AccountId, Hash>>> {
		let api = self.client.runtime_api();
		let best = self.client.info().best_hash;
		let at = BlockId::hash(best);

		let data = api.get_stake_data(&at, hash).map_err(|e| Error {
			code: ErrorCode::ServerError(crate::constants::RUNTIME_ERROR),
			message: "Unable to query stake data.".into(),
			data: Some(format!("{:?}", e).into()),
		})?;

		if let Some(data) = data {
			Ok(Some(RpcXtzStakeData {
				id: data.id,
				initiator: data.initiator,
				stage: data.stage,
				multi_sig_address: String::from("0x") + &hex::encode(data.multi_sig_address),
				stake_amount: data.stake_amount.to_string(),
			}))
		} else {
			Ok(None)
		}
	}
}
