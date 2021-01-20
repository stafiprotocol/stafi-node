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
use sp_runtime::traits::BadOrigin;
use node_primitives::{RSymbol};
use sp_io::hashing::blake2_128;

#[test]
fn new_resource_id() {
    let rfis_resource_id = derive_resource_id(1, &blake2_128(b"RFIS"));
    println!("{:?}", hex::encode(rfis_resource_id));
    //result 000000000000000000000000000000df7e6fee39d3ace035c108833854667701
}

#[test]
fn derive_ids() {
    let chain = 1;
    let id = [
        0x21, 0x60, 0x5f, 0x71, 0x84, 0x5f, 0x37, 0x2a, 0x9e, 0xd8, 0x42, 0x53, 0xd2, 0xd0, 0x24,
        0xb7, 0xb1, 0x09, 0x99, 0xf4,
    ];
    let r_id = derive_resource_id(chain, &id);
    let expected = [
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x21, 0x60, 0x5f, 0x71, 0x84, 0x5f,
        0x37, 0x2a, 0x9e, 0xd8, 0x42, 0x53, 0xd2, 0xd0, 0x24, 0xb7, 0xb1, 0x09, 0x99, 0xf4, chain,
    ];
    assert_eq!(r_id, expected);
}

#[test]
fn derive_ids1() {
    let chain = 1;
    let id = [];
    let r_id = derive_resource_id(chain, &id);
    let expected = [
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, chain,
    ];
    assert_eq!(r_id, expected);
}

#[test]
fn whitelist_chain_should_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			BridgeCommon::whitelist_chain(Origin::signed(42), 2),
			BadOrigin,
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
fn remove_whitelist_chain_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(BridgeCommon::whitelist_chain(Origin::root(), 2));
		assert_eq!(BridgeCommon::chains(2), Some(0));
		assert_noop!(
			BridgeCommon::remove_whitelist_chain(Origin::signed(42), 1),
			BadOrigin,
        );
        assert_noop!(
			BridgeCommon::remove_whitelist_chain(Origin::root(), 1),
			Error::<Test>::ChainNotWhitelisted,
        );

        assert_ok!(BridgeCommon::remove_whitelist_chain(Origin::root(), 2));
        assert_eq!(BridgeCommon::chains(2), None);
	});
}


#[test]
fn set_get_threshold() {
    new_test_ext().execute_with(|| {
        assert_eq!(<RelayerThreshold>::get(), 1);

        assert_ok!(BridgeCommon::set_threshold(Origin::root(), TEST_THRESHOLD));
		assert_eq!(<RelayerThreshold>::get(), TEST_THRESHOLD);

        assert_noop!(
            BridgeCommon::set_threshold(Origin::signed(42), 5),
            BadOrigin
        );

        assert_noop!(
            BridgeCommon::set_threshold(Origin::root(), 0),
            Error::<Test>::InvalidThreshold
        );
    })
}

#[test]
fn setup_resources() {
    new_test_ext().execute_with(|| {
        let id: ResourceId = [1; 32];
        let method = "Pallet.do_something".as_bytes().to_vec();
        let method2 = "Pallet.do_somethingElse".as_bytes().to_vec();

        assert_ok!(BridgeCommon::add_resource(Origin::root(), id, method.clone()));
        assert_eq!(BridgeCommon::resources(id), Some(method));

        assert_noop!(
            BridgeCommon::add_resource(Origin::signed(42), id, method2.clone()),
            BadOrigin
        );

        assert_noop!(
            BridgeCommon::remove_resource(Origin::signed(42), id),
            BadOrigin
        );

        assert_ok!(BridgeCommon::remove_resource(Origin::root(), id));
        assert_eq!(BridgeCommon::resources(id), None);
    })
}

#[test]
fn add_remove_relayer() {
    new_test_ext().execute_with(|| {
        assert_ok!(BridgeCommon::set_threshold(Origin::root(), TEST_THRESHOLD));
        assert_eq!(BridgeCommon::relayer_count(), 0);

        assert_ok!(BridgeCommon::add_relayer(Origin::root(), RELAYER_A));
        
        assert_noop!(
            BridgeCommon::add_relayer(Origin::signed(42), RELAYER_B),
            BadOrigin
        );
        assert_ok!(BridgeCommon::add_relayer(Origin::root(), RELAYER_B));
        assert_ok!(BridgeCommon::add_relayer(Origin::root(), RELAYER_C));
        assert_eq!(BridgeCommon::relayer_count(), 3);
        assert_noop!(
            BridgeCommon::add_relayer(Origin::root(), RELAYER_C),
            Error::<Test>::RelayerAlreadyExists
        );

        // Already exists
        assert_noop!(
            BridgeCommon::add_relayer(Origin::root(), RELAYER_A),
            Error::<Test>::RelayerAlreadyExists
        );

        // Confirm removal
        assert_noop!(
            BridgeCommon::remove_relayer(Origin::signed(42), RELAYER_B),
            BadOrigin
        );
        assert_ok!(BridgeCommon::remove_relayer(Origin::root(), RELAYER_B));
        assert_eq!(BridgeCommon::relayer_count(), 2);
        assert_noop!(
            BridgeCommon::remove_relayer(Origin::root(), RELAYER_B),
            Error::<Test>::RelayerInvalid
        );
        assert_eq!(BridgeCommon::relayer_count(), 2);
    })
}

#[test]
fn map_resource_to_rsymbol_should_work() {
    new_test_ext().execute_with(|| {
        let rid: ResourceId = [1; 32];
        let sym: RSymbol = RSymbol::RFIS;

        assert_noop!(
			BridgeCommon::map_resource_and_rsymbol(Origin::signed(42), rid, sym),
			sp_runtime::traits::BadOrigin,
        );
        assert_ok!(BridgeCommon::map_resource_and_rsymbol(Origin::root(), rid, sym));
        assert_eq!(BridgeCommon::resource_rsymbol(rid), Some(sym));
        assert_eq!(BridgeCommon::rsymbol_resource(sym), Some(rid));
        
        assert_noop!(
			BridgeCommon::unmap_resource_and_rsymbol(Origin::signed(42), rid, sym),
			sp_runtime::traits::BadOrigin,
        );
        assert_ok!(BridgeCommon::unmap_resource_and_rsymbol(Origin::root(), rid, sym));
        assert_eq!(BridgeCommon::resource_rsymbol(rid).is_none(), true);
        assert_eq!(BridgeCommon::rsymbol_resource(sym).is_none(), true);
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

fn make_proposal(r: Vec<u8>) -> mock::Call {
    Call::System(system::Call::remark(r))
}

#[test]
fn create_sucessful_proposal() {
    let src_id = 2;
    let r_id = derive_resource_id(src_id, b"remark");

    new_test_ext_initialized(src_id, r_id, b"System.remark".to_vec()).execute_with(|| {
        let prop_id = 1;
		let proposal = make_proposal(vec![10]);

        // Create proposal (& vote)
        assert_ok!(BridgeCommon::acknowledge_proposal(
            Origin::signed(RELAYER_A),
            prop_id,
            src_id,
            r_id,
            Box::new(proposal.clone())
        ));
        let prop = BridgeCommon::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
        let expected = ProposalVotes {
            voted: vec![RELAYER_A],
            status: ProposalStatus::Active,
            expiry: ProposalLifetime::get() as u64,
        };
        assert_eq!(prop, expected);

        // Third relayer votes in favour
        assert_ok!(BridgeCommon::acknowledge_proposal(
            Origin::signed(RELAYER_C),
            prop_id,
            src_id,
            r_id,
            Box::new(proposal.clone())
        ));
        let prop = BridgeCommon::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
        let expected = ProposalVotes {
            voted: vec![RELAYER_A, RELAYER_C],
            status: ProposalStatus::Executed,
            expiry: ProposalLifetime::get() as u64,
        };
        assert_eq!(prop, expected);
    })
}

#[test]
fn proposal_expires_should_work() {
    let src_id = 2;
    let r_id = derive_resource_id(src_id, b"remark");

    new_test_ext_initialized(src_id, r_id, b"System.remark".to_vec()).execute_with(|| {
        let prop_id = 1;
        let proposal = make_proposal(vec![10]);

        // Create proposal (& vote)
        assert_ok!(BridgeCommon::acknowledge_proposal(
            Origin::signed(RELAYER_A),
            prop_id,
            src_id,
            r_id,
            Box::new(proposal.clone())
        ));
        let prop = BridgeCommon::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
        let expected = ProposalVotes {
            voted: vec![RELAYER_A],
            status: ProposalStatus::Active,
            expiry: ProposalLifetime::get() as u64,
        };
        assert_eq!(prop, expected);

        // Increment enough blocks such that now == expiry
		System::set_block_number((ProposalLifetime::get() + 1) as u64);
		let now = System::block_number();
		let votes = BridgeCommon::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
		assert_eq!(votes.is_expired(now), true);

        // Attempt to submit a vote should fail
        assert_err!(
            BridgeCommon::acknowledge_proposal(
                Origin::signed(RELAYER_B),
                prop_id,
                src_id,
                r_id,
                Box::new(proposal.clone())
            ),
            Error::<Test>::ProposalExpired
        );

        // Proposal state should changed
        let prop = BridgeCommon::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
        let expected = ProposalVotes {
            voted: vec![RELAYER_A],
            status: ProposalStatus::Expired,
            expiry: ProposalLifetime::get() as u64,
        };
        assert_eq!(prop, expected);
    })
}

// fn last_event() -> TestEvent {
// 	system::Module::<Test>::events().pop().map(|e| e.event).expect("Event expected")
// }