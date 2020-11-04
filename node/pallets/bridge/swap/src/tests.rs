// Copyright 2019-2020 Stafi Protocol.
// This file is part of Stafi.

// Stafi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stafi.  If not, see <http://www.gnu.org/licenses/>.

//! Tests for the module.

use super::*;
use super::mock::*;
use frame_support::{assert_ok, assert_noop};
use node_primitives::{ETH_CHAIN_ID};

#[test]
fn transfer_native_should_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			BridgeSwap::transfer_native(Origin::signed(42), 10, vec![11, 21], ETH_CHAIN_ID),
			Error::<Test>::InvalidChainId,
		);

		assert_ok!(BridgeCommon::set_is_pasued(Origin::root(), true));
		assert_noop!(
			BridgeSwap::transfer_native(Origin::signed(42), 10, vec![11, 21], ETH_CHAIN_ID),
			Error::<Test>::ServicePaused,
		);

		assert_ok!(BridgeCommon::set_is_pasued(Origin::root(), false));
		assert_noop!(
			BridgeSwap::transfer_native(Origin::signed(42), 10, vec![11, 21], ETH_CHAIN_ID),
			Error::<Test>::InvalidChainId,
		);

		assert_ok!(BridgeCommon::whitelist_chain(Origin::root(), ETH_CHAIN_ID));
		assert_noop!(
			BridgeSwap::transfer_native(Origin::signed(42), 10, vec![11, 21], ETH_CHAIN_ID),
			Error::<Test>::InvalidEthereumAddress,
		);

		let eth_address = vec![11, 21, 31, 43, 88, 120, 43, 54, 55, 99, 54, 98, 23, 24, 54, 64, 29, 94, 26, 75];
		assert_noop!(
			BridgeSwap::transfer_native(Origin::signed(42), 10, eth_address.clone(), ETH_CHAIN_ID),
			Error::<Test>::InvalidChainFee,
		);

		assert_ok!(BridgeCommon::set_proxy_accounts(Origin::root(), 40));
		let chain_fees = 10;
		assert_ok!(BridgeCommon::set_chain_fees(Origin::signed(40), ETH_CHAIN_ID, chain_fees));
		assert_noop!(
			BridgeSwap::transfer_native(Origin::signed(42), 10, eth_address.clone(), ETH_CHAIN_ID),
			Error::<Test>::InvalidFeesRecipientAccount,
		);

		let recipient_account = 2;
		assert_ok!(BridgeCommon::set_fees_recipient_account(Origin::root(), recipient_account));
		assert_noop!(
			BridgeSwap::transfer_native(Origin::signed(42), 10, eth_address.clone(), ETH_CHAIN_ID),
			pallet_balances::Error::<Test, _>::InsufficientBalance,
		);

		assert_noop!(
			BridgeSwap::transfer_native(Origin::signed(1), 100, eth_address.clone(), ETH_CHAIN_ID),
			pallet_balances::Error::<Test, _>::InsufficientBalance,
		);

		assert_ok!(BridgeSwap::transfer_native(Origin::signed(1), 80, eth_address.clone(), ETH_CHAIN_ID));
		assert_eq!(Balances::free_balance(BridgeCommon::account_id()), 80);
		assert_eq!(Balances::free_balance(&recipient_account), chain_fees);
	});
}
