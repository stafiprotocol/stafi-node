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

#[test]
fn whitelist_chain_should_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			BridgeCommon::whitelist_chain(Origin::signed(42), 2),
			sp_runtime::traits::BadOrigin,
		);
		assert_noop!(
			BridgeCommon::whitelist_chain(Origin::root(), 1),
			Error::<Test>::InvalidChainId,
		);
		assert_eq!(BridgeCommon::chains(2), None);
		assert_ok!(BridgeCommon::whitelist_chain(Origin::root(), 2));
		assert_eq!(BridgeCommon::chains(2), Some(0));
		assert_noop!(
			BridgeCommon::whitelist_chain(Origin::root(), 2),
			Error::<Test>::ChainAlreadyWhitelisted,
		);
	});
}

#[test]
fn set_proxy_accounts_should_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			BridgeCommon::set_proxy_accounts(Origin::signed(42), 1),
			sp_runtime::traits::BadOrigin,
		);
		assert_eq!(BridgeCommon::proxy_accounts(1), None);
		assert_ok!(BridgeCommon::set_proxy_accounts(Origin::root(), 1));
		assert_eq!(BridgeCommon::proxy_accounts(1), Some(0));
	});
}

#[test]
fn remove_proxy_accounts_should_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			BridgeCommon::remove_proxy_accounts(Origin::signed(42), 1),
			sp_runtime::traits::BadOrigin,
		);
		assert_eq!(BridgeCommon::proxy_accounts(1), None);
		assert_ok!(BridgeCommon::set_proxy_accounts(Origin::root(), 1));
		assert_eq!(BridgeCommon::proxy_accounts(1), Some(0));
		assert_ok!(BridgeCommon::remove_proxy_accounts(Origin::root(), 1));
		assert_eq!(BridgeCommon::proxy_accounts(1), None);
	});
}


#[test]
fn set_chain_fees_should_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			BridgeCommon::set_chain_fees(Origin::signed(42), 2, 10),
			Error::<Test>::InvalidChainId,
		);
		assert_ok!(BridgeCommon::whitelist_chain(Origin::root(), 2));
		assert_noop!(
			BridgeCommon::set_chain_fees(Origin::signed(42), 2, 10),
			Error::<Test>::InvalidProxyAccount,
		);
		assert_ok!(BridgeCommon::set_proxy_accounts(Origin::root(), 42));
		assert_eq!(BridgeCommon::chain_fees(2), None);
		assert_ok!(BridgeCommon::set_chain_fees(Origin::signed(42), 2, 10));
		assert_eq!(BridgeCommon::chain_fees(2), Some(10));
	});
}


#[test]
fn set_fees_recipient_account_should_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			BridgeCommon::set_fees_recipient_account(Origin::signed(42), 1),
			sp_runtime::traits::BadOrigin,
		);
		assert_eq!(BridgeCommon::fees_recipient_account(), None);
		assert_ok!(BridgeCommon::set_fees_recipient_account(Origin::root(), 1));
		assert_eq!(BridgeCommon::fees_recipient_account(), Some(1));
	});
}


#[test]
fn set_is_pasued_should_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			BridgeCommon::set_is_pasued(Origin::signed(42), true),
			sp_runtime::traits::BadOrigin,
		);
		assert_eq!(BridgeCommon::is_paused(), false);
		assert_ok!(BridgeCommon::set_is_pasued(Origin::root(), true));
		assert_eq!(BridgeCommon::is_paused(), true);
		assert_ok!(BridgeCommon::set_is_pasued(Origin::root(), false));
		assert_eq!(BridgeCommon::is_paused(), false);
	});
}
