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
use super::mock::{*, Call};
use frame_support::{assert_ok, assert_noop, assert_err};
use node_primitives::{ETH_CHAIN_ID, RSymbol};
use sp_runtime::traits::BadOrigin;

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

#[test]
fn transfer_rtoken_should_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			BridgeSwap::transfer_rtoken(Origin::signed(42), RSymbol::RFIS, 100, vec![11, 21], ETH_CHAIN_ID),
			Error::<Test>::InvalidChainId,
		);

		assert_ok!(BridgeCommon::set_is_pasued(Origin::root(), true));
		assert_noop!(
			BridgeSwap::transfer_rtoken(Origin::signed(42), RSymbol::RFIS, 100, vec![11, 21], ETH_CHAIN_ID),
			Error::<Test>::ServicePaused,
		);
		assert_ok!(BridgeCommon::set_is_pasued(Origin::root(), false));

		assert_ok!(BridgeCommon::whitelist_chain(Origin::root(), ETH_CHAIN_ID));
		assert_noop!(
			BridgeSwap::transfer_rtoken(Origin::signed(42), RSymbol::RFIS, 100, vec![11, 21], ETH_CHAIN_ID),
			Error::<Test>::InvalidEthereumAddress,
		);

		let eth_address = vec![11, 21, 31, 43, 88, 120, 43, 54, 55, 99, 54, 98, 23, 24, 54, 64, 29, 94, 26, 75];
		assert_noop!(
			BridgeSwap::transfer_rtoken(Origin::signed(42), RSymbol::RFIS, 100, eth_address.clone(), ETH_CHAIN_ID),
			Error::<Test>::InvalidChainFee,
		);

		assert_ok!(BridgeCommon::set_proxy_accounts(Origin::root(), 40));
		let chain_fees = 10;
		assert_ok!(BridgeCommon::set_chain_fees(Origin::signed(40), ETH_CHAIN_ID, chain_fees));
		assert_noop!(
			BridgeSwap::transfer_rtoken(Origin::signed(42), RSymbol::RFIS, 100, eth_address.clone(), ETH_CHAIN_ID),
			Error::<Test>::InvalidFeesRecipientAccount,
		);

		let recipient_account = 2;
		assert_ok!(BridgeCommon::set_fees_recipient_account(Origin::root(), recipient_account));
		assert_noop!(
			BridgeSwap::transfer_rtoken(Origin::signed(42), RSymbol::RFIS, 100, eth_address.clone(), ETH_CHAIN_ID),
			Error::<Test>::RsymbolNotMapped,
        );

        let rid: ResourceId = [1; 32];
        let sym: RSymbol = RSymbol::RFIS;
        assert_ok!(BridgeCommon::map_resource_and_rsymbol(Origin::root(), rid, sym));

		assert_noop!(
			BridgeSwap::transfer_rtoken(Origin::signed(42), RSymbol::RFIS, 100, eth_address.clone(), ETH_CHAIN_ID),
			Error::<Test>::InsufficientRbalance,
        );
        assert_ok!(RBalances::mint(&(42 as u64), sym, 100));
        
        assert_noop!(
			BridgeSwap::transfer_rtoken(Origin::signed(42), RSymbol::RFIS, 100, eth_address.clone(), ETH_CHAIN_ID),
			pallet_balances::Error::<Test, _>::InsufficientBalance,
        );
        assert_ok!(Balances::transfer(Origin::signed(1), 42, 20));

        assert_ok!(BridgeSwap::transfer_rtoken(Origin::signed(42), RSymbol::RFIS, 100, eth_address.clone(), ETH_CHAIN_ID));

		assert_eq!(RBalances::free_balance(&BridgeCommon::account_id(), RSymbol::RFIS), 100);
		assert_eq!(Balances::free_balance(&recipient_account), chain_fees);
	});
}

#[test]
fn transfer_native_back_should_work() {
    new_test_ext().execute_with(|| {
        let recipient = RELAYER_A;
        let bridge_id: u64 = BridgeCommon::account_id();
        let rid: ResourceId = [1; 32];

		assert_noop!(
			BridgeSwap::transfer_native_back(Origin::signed(1), recipient, 100, rid),
			BadOrigin,
		);

		assert_ok!(Balances::transfer(Origin::signed(1), bridge_id, 100));

        // transfer_native_back
        assert_ok!(BridgeSwap::transfer_native_back(Origin::signed(bridge_id), recipient, 100, rid));
    })
}

fn make_transfer_proposal(to: u64, amount: u64) -> Call {
    let rid: ResourceId = [1; 32];
    Call::BridgeSwap(crate::Call::transfer_native_back(to, amount.into(), rid))
}

#[test]
fn transfer_native_back_proposal() {
    new_test_ext().execute_with(|| {
        let prop_id = 1;
        let src_id = 2;
        let rid: ResourceId = [1; 32];
        let resource = b"BridgeSwap.transfer_native_back".to_vec();
		let proposal = make_transfer_proposal(RELAYER_A, 10);
		
		assert_ok!(Balances::transfer(Origin::signed(1), BridgeCommon::account_id(), 100));

        assert_ok!(BridgeCommon::set_threshold(Origin::root(), TEST_THRESHOLD));
        assert_ok!(BridgeCommon::add_relayer(Origin::root(), RELAYER_A));
        assert_ok!(BridgeCommon::add_relayer(Origin::root(), RELAYER_B));
        assert_ok!(BridgeCommon::add_relayer(Origin::root(), RELAYER_C));
        assert_ok!(BridgeCommon::whitelist_chain(Origin::root(), src_id));
        assert_ok!(BridgeCommon::add_resource(Origin::root(), rid, resource));

        // Create proposal (& vote)
        assert_ok!(BridgeCommon::acknowledge_proposal(
            Origin::signed(RELAYER_A),
            prop_id,
            src_id,
            rid,
            Box::new(proposal.clone())
        ));
        let prop = BridgeCommon::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
        let expected = bridge::ProposalVotes {
            voted: vec![RELAYER_A],
            status: bridge_common::ProposalStatus::Active,
            expiry: ProposalLifetime::get() as u64,
        };
        assert_eq!(prop, expected);

        // Third relayer votes in favour
        assert_ok!(BridgeCommon::acknowledge_proposal(
            Origin::signed(RELAYER_C),
            prop_id,
            src_id,
            rid,
            Box::new(proposal.clone())
        ));
        let prop = BridgeCommon::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
        let expected = bridge::ProposalVotes {
            voted: vec![RELAYER_A, RELAYER_C],
            status: bridge::ProposalStatus::Executed,
            expiry: ProposalLifetime::get() as u64,
        };
        assert_eq!(prop, expected);

        assert_eq!(Balances::free_balance(RELAYER_A), 10);
        assert_eq!(Balances::free_balance(BridgeCommon::account_id()), 90);
    })
}

fn make_transfer_rtoken_proposal(to: u64, amount: u128) -> Call {
    let rid: ResourceId = [1; 32];
    Call::BridgeSwap(crate::Call::transfer_rtoken_back(to, amount, rid))
}

#[test]
fn transfer_rtoken_back_proposal() {
    new_test_ext().execute_with(|| {
        let prop_id = 1;
        let src_id = 2;
        let rid: ResourceId = [1; 32];
        let resource = b"BridgeSwap.transfer_rtoken_back".to_vec();
		let proposal = make_transfer_rtoken_proposal(RELAYER_A, 10);
		let sym: RSymbol = RSymbol::RFIS;
		
		let ac = BridgeCommon::account_id();
		assert_ok!(RBalances::mint(&ac, sym, 100));

        assert_ok!(BridgeCommon::set_threshold(Origin::root(), TEST_THRESHOLD));
        assert_ok!(BridgeCommon::add_relayer(Origin::root(), RELAYER_A));
        assert_ok!(BridgeCommon::add_relayer(Origin::root(), RELAYER_B));
        assert_ok!(BridgeCommon::add_relayer(Origin::root(), RELAYER_C));
        assert_ok!(BridgeCommon::whitelist_chain(Origin::root(), src_id));
        assert_ok!(BridgeCommon::add_resource(Origin::root(), rid, resource));

        // Create proposal (& vote)
        assert_ok!(BridgeCommon::acknowledge_proposal(
            Origin::signed(RELAYER_A),
            prop_id,
            src_id,
            rid,
            Box::new(proposal.clone())
        ));
        let prop = BridgeCommon::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
        let expected = bridge::ProposalVotes {
            voted: vec![RELAYER_A],
            status: bridge_common::ProposalStatus::Active,
            expiry: ProposalLifetime::get() as u64,
        };
        assert_eq!(prop, expected);

        // Third relayer votes in favour
        assert_err!(BridgeCommon::acknowledge_proposal(
            	Origin::signed(RELAYER_C),
            	prop_id,
            	src_id,
            	rid,
            	Box::new(proposal.clone())
			),
			Error::<Test>::ResourceNotMapped
		);
		assert_ok!(BridgeCommon::map_resource_and_rsymbol(Origin::root(), rid, sym));

		// Create proposal (& vote)
        assert_ok!(BridgeCommon::acknowledge_proposal(
            Origin::signed(RELAYER_B),
            prop_id,
            src_id,
            rid,
            Box::new(proposal.clone())
        ));
        let prop = BridgeCommon::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
        let expected = bridge::ProposalVotes {
            voted: vec![RELAYER_A, RELAYER_C, RELAYER_B],
            status: bridge::ProposalStatus::Executed,
            expiry: ProposalLifetime::get() as u64,
        };
        assert_eq!(prop, expected);

        assert_eq!(RBalances::free_balance(&RELAYER_A, sym), 10);
        assert_eq!(RBalances::free_balance(&ac, sym), 90);
    })
}
