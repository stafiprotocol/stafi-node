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

//! Substrate CLI library.
//!
//! This package has two Cargo features:
//!
//! - `cli` (default): exposes functions that parse command-line options, then start and run the
//! node as a CLI application.
//!

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

pub mod chain_spec;

#[macro_use]
mod service;
#[cfg(feature = "cli")]
mod cli;
#[cfg(feature = "cli")]
mod factory_impl;

mod fixtures;

#[cfg(feature = "cli")]
pub use cli::*;

/// The chain specification option.
#[derive(Clone, Debug, PartialEq)]
pub enum ChainSpec {
	 /// Whatever the current runtime is, with just Alice as an auth.
    Development,
    /// Whatever the current runtime is, with simple Alice/Bob auths.
    LocalTestnet,
    /// Stafi testnet.
    Stafi,
    /// Stafi testnet configuration (intermediate build process)
    StafiTestnetConfiguration,
}

/// Get a chain config from a spec setting.
impl ChainSpec {
	pub(crate) fn load(self) -> Result<chain_spec::ChainSpec, String> {
		Ok(match self {
			ChainSpec::StafiTestnetConfiguration => chain_spec::stafi_testnet_config(),
            ChainSpec::Stafi => chain_spec::stafi_config()?,
            ChainSpec::Development => chain_spec::development_config(),
            ChainSpec::LocalTestnet => chain_spec::local_testnet_config(),
		})
	}

	pub(crate) fn from(s: &str) -> Option<Self> {
		match s {
			"dev" => Some(ChainSpec::Development),
            "local" => Some(ChainSpec::LocalTestnet),
            "test" => Some(ChainSpec::StafiTestnetConfiguration),
            "stafi" => Some(ChainSpec::Stafi),
            "" => Some(ChainSpec::Stafi),
            _ => None,
		}
	}
}

fn load_spec(id: &str) -> Result<Option<chain_spec::ChainSpec>, String> {
	Ok(match ChainSpec::from(id) {
		Some(spec) => Some(spec.load()?),
		None => None,
	})
}