// Copyright 2019-2020 Stafi Protocol.
// This file is part of Stafi.

// Stafi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stafi.  If not, see <http://www.gnu.org/licenses/>.

use codec::{Encode, Decode};
use sp_runtime::RuntimeDebug;

/// Rtoken Identifier
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum RSymbol {
	/// rfis
	RFIS,
	/// rdot
    RDOT,
}

/// Rtoken Identifier
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum ChainType {
	/// substrate
	Substrate,
}

impl RSymbol {
	/// get chain type of rsymbol, eg: RDOT => Substrate
	pub fn chain_type(&self) -> ChainType {
		match self {
			RSymbol::RFIS | RSymbol::RDOT => ChainType::Substrate,
		}
	}
}