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
use node_primitives::{
	Block, BlockId, ChainType
};
pub use node_primitives::MultisigAddrApi;
use codec::{Encode, Decode};
use sr_primitives::traits;
use hex;

use serde::{Serialize, Deserialize};

const RUNTIME_ERROR: i64 = 1;

#[derive(Debug, Serialize, Deserialize)]
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct RpcMultisigAddr {
	pub chain_type: ChainType,
	pub multisig_addr: String,
}

#[rpc]
pub trait MultisigsApi {
	#[rpc(name = "multisig_getaddr")]
	fn get_addr(&self, chainType: ChainType) -> Result<Vec<RpcMultisigAddr>>;
}

pub struct Multisigs<C> {
	client: Arc<C>,
}

impl<C> Multisigs<C> {
	pub fn new(client: Arc<C>) -> Self {
		Multisigs {
			client
		}
	}
}

impl<C> MultisigsApi for Multisigs<C>
where
	C: traits::ProvideRuntimeApi,
	C: HeaderBackend<Block>,
	C: Send + Sync + 'static,
	C::Api: MultisigAddrApi<Block>,
{
	fn get_addr(&self, chainType: ChainType) -> Result<Vec<RpcMultisigAddr>> {
		let api = self.client.runtime_api();
		let best = self.client.info().best_hash;
		let at = BlockId::hash(best);

		let addrs = api.multisig_addr(&at, chainType).map_err(|e| Error {
			code: ErrorCode::ServerError(RUNTIME_ERROR),
			message: "Unable to query multisig address.".into(),
			data: Some(format!("{:?}", e).into()),
		})?;

		let mut result:Vec<RpcMultisigAddr> = Vec::new();
        for addr in addrs.clone() {

            let ret = RpcMultisigAddr {
                chain_type: addr.chain_type,
                multisig_addr: String::from("0x") + &hex::encode(addr.multisig_addr),
            };

            result.push(ret);
        }

		Ok(result)
	}
}
