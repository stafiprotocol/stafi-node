// Copyright 2019-2020 Stafi Protocol.
// This file is part of Stafi.

// Stafi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stafi.  If not, see <http://www.gnu.org/licenses/>.

//! A list of the different weight modules for our runtime.

pub mod frame_system;
pub mod pallet_balances;
pub mod pallet_treasury;
pub mod pallet_collective;
pub mod pallet_democracy;
pub mod pallet_identity;
pub mod pallet_indices;
pub mod pallet_im_online;
pub mod pallet_multisig;
pub mod pallet_proxy;
pub mod pallet_scheduler;
pub mod pallet_session;
pub mod pallet_staking;
pub mod pallet_timestamp;
pub mod pallet_utility;
pub mod pallet_vesting;
pub mod pallet_elections_phragmen;
