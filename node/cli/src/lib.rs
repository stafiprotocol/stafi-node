// Copyright 2019-2020 Stafi Protocol.
// This file is part of Stafi.

// Stafi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stafi.  If not, see <http://www.gnu.org/licenses/>.

//! Stafi CLI library.
//!
//! This package has two Cargo features:
//!
//! - `cli` (default): exposes functions that parse command-line options, then start and run the
//! node as a CLI application.

#![warn(missing_docs)]

pub mod chain_spec;

#[macro_use]
mod service;
#[cfg(feature = "cli")]
mod cli;
#[cfg(feature = "cli")]
mod command;

#[cfg(feature = "cli")]
pub use cli::*;
#[cfg(feature = "cli")]
pub use command::*;

pub mod testnet_fixtures;
pub mod mainnet_fixtures;