// Copyright 2019-2021 Stafi Protocol.
// This file is part of Stafi.

// Stafi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stafi.  If not, see <http://www.gnu.org/licenses/>.

use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

/// Rtoken Identifier
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum RSymbol {
	/// rFIS
	RFIS,
	/// rDOT
	RDOT,
	/// rKSM
	RKSM,
	/// rATOM
	RATOM,
	/// rSOL
	RSOL,
	/// rMatic
	RMATIC,
	/// rBNB
	RBNB,
	/// rETH
	RETH,
}

/// Chain Type
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum ChainType {
	/// substrate
	Substrate,
	/// tendermint
	Tendermint,
	/// solana
	Solana,
	/// ethereum
	Ethereum,
}

impl RSymbol {
	/// get chain type of rsymbol, eg: RDOT => Substrate
	pub fn chain_type(&self) -> ChainType {
		match self {
			RSymbol::RFIS | RSymbol::RDOT | RSymbol::RKSM => ChainType::Substrate,
			RSymbol::RATOM => ChainType::Tendermint,
			RSymbol::RSOL => ChainType::Solana,
			RSymbol::RMATIC | RSymbol::RBNB | RSymbol::RETH => ChainType::Ethereum,
		}
	}
}
