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
	Block, BlockId, BroadcastsApi
};
use codec::{Encode, Decode};
use sr_primitives::{traits, OpaqueExtrinsic};
use sr_primitives::traits::{Block as BlockT};
use hex;

#[rpc]
pub trait BroadcastsRpcApi {
	#[rpc(name = "apply_extrinsic")]
	fn apply_extrinsic(&self, payload: String) -> Result<u8>;
}

pub struct Broadcasts<C> {
	client: Arc<C>,
}

impl<C> Broadcasts<C> {
	pub fn new(client: Arc<C>) -> Self {
		Broadcasts {
			client
		}
	}
}

impl<C> BroadcastsRpcApi for Broadcasts<C>
where
	C: traits::ProvideRuntimeApi,
	C: HeaderBackend<Block>,
	C: Send + Sync + 'static,
	C::Api: BroadcastsApi<Block>,
{
	fn apply_extrinsic(&self, payload: String) -> Result<u8> {
		let api = self.client.runtime_api();
		let best = self.client.info().best_hash;
		let at = BlockId::hash(best);

        println!("payload:{}", payload);
        let v = hex::decode(payload).unwrap().to_vec();
        println!("vec:{:?}", v);
        //let extrinsic = <Block as BlockT>::Extrinsic::decode(&mut &*v).unwrap();
        //let extrinsic = <Block as BlockT>::Extrinsic::encode(&OpaqueExtrinsic(v));
        let extrinsic: <Block as BlockT>::Extrinsic = OpaqueExtrinsic(v);
		//println!("extrinsic:{:?}", extrinsic);
        
		let ret = api.apply_extrinsic(&at, extrinsic).map_err(|e| Error {
			code: ErrorCode::ServerError(crate::constants::RUNTIME_ERROR),
			message: "Unable to apply extrinsic.".into(),
			data: Some(format!("{:?}", e).into()),
		})?;

		Ok(1)
	}
}
