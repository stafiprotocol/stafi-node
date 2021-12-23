use super::mock::*;
use super::*;
use frame_support::{assert_err, assert_ok, error as frame_support_error};
use node_primitives::RSymbol;
use rtoken_balances::traits::Currency;
use sp_core::U256;

#[test]
fn show_account_id() {
    new_test_ext().execute_with(|| {
        let mut bts: Vec<u8> = Vec::new();
        bts.resize(32, 0);
        RDexMining::account_id().to_little_endian(&mut bts);
        println!("{:x?}", bts);
    });
}

#[test]
fn increase_pool_index_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(RDexMining::increase_pool_index(
            Origin::root(),
            RSymbol::RATOM
        ));
        assert_eq!(RDexMining::pool_count(RSymbol::RATOM), 1);
        assert_eq!(RDexMining::pool_count(RSymbol::RETH), 0);
    });
}

#[test]
fn increase_pool_index_should_fail() {
    new_test_ext().execute_with(|| {
        assert_err!(
            RDexMining::increase_pool_index(Origin::signed(U256::from(42)), RSymbol::RATOM),
            frame_support_error::BadOrigin
        );
        assert_eq!(RDexMining::pool_count(RSymbol::RATOM), 0);
    });
}

#[test]
fn add_pool_should_work() {
    new_test_ext().execute_with(|| {
        let symbol = RSymbol::RATOM;
        // increase pool index
        assert_ok!(RDexMining::increase_pool_index(Origin::root(), symbol));
        assert_eq!(RDexMining::pool_count(symbol), 1);

        let start_block = 2;
        let reward_per_block = 10;
        let total_reward = 200;
        let lp_locked_blocks = 10;
        let guard_impermanent_loss = true;
        // add pool
        assert_ok!(RDexMining::add_pool(
            Origin::root(),
            symbol,
            0,
            start_block,
            lp_locked_blocks,
            reward_per_block,
            total_reward,
            guard_impermanent_loss
        ));
        let stake_pools = RDexMining::stake_pools((symbol, 0)).unwrap();
        assert_eq!(stake_pools.len(), 1);
        assert_eq!(stake_pools[0].symbol, symbol);
        assert_eq!(stake_pools[0].emergency_switch, false);
        assert_eq!(stake_pools[0].total_stake_lp, 0);
        assert_eq!(stake_pools[0].start_block, start_block);
        assert_eq!(stake_pools[0].reward_per_block, reward_per_block);
        assert_eq!(stake_pools[0].total_reward, total_reward);
        assert_eq!(stake_pools[0].left_reward, total_reward);
        assert_eq!(stake_pools[0].lp_locked_blocks, lp_locked_blocks);
        assert_eq!(stake_pools[0].last_reward_block, 0);
        assert_eq!(stake_pools[0].reward_per_share, 0);
        assert_eq!(
            stake_pools[0].guard_impermanent_loss,
            guard_impermanent_loss
        );
    });
}

#[test]
fn add_pool_should_fail() {
    new_test_ext().execute_with(|| {
        let symbol = RSymbol::RATOM;
        // increase pool index
        assert_ok!(RDexMining::increase_pool_index(Origin::root(), symbol));
        assert_eq!(RDexMining::pool_count(symbol), 1);

        let start_block = 2;
        let reward_per_block = 10;
        let total_reward = 200;
        let lp_locked_blocks = 10;
        let guard_impermanent_loss = true;
        // add pool
        assert_err!(
            RDexMining::add_pool(
                Origin::signed(U256::from(42)),
                symbol,
                1,
                start_block,
                lp_locked_blocks,
                reward_per_block,
                total_reward,
                guard_impermanent_loss
            ),
            frame_support_error::BadOrigin
        );

        assert_err!(
            RDexMining::add_pool(
                Origin::root(),
                symbol,
                1,
                start_block,
                lp_locked_blocks,
                reward_per_block,
                total_reward,
                guard_impermanent_loss
            ),
            Error::<Test>::StakePoolNotExist
        );
    });
}

#[test]
fn rm_pool_should_work() {
    new_test_ext().execute_with(|| {
        let symbol = RSymbol::RATOM;
        // increase pool index
        assert_ok!(RDexMining::increase_pool_index(Origin::root(), symbol));
        assert_eq!(RDexMining::pool_count(symbol), 1);

        let start_block = 2;
        let reward_per_block = 10;
        let total_reward = 200;
        let lp_locked_blocks = 10;
        let guard_impermanent_loss = true;
        // add pool
        assert_ok!(RDexMining::add_pool(
            Origin::root(),
            symbol,
            0,
            start_block,
            lp_locked_blocks,
            reward_per_block,
            total_reward,
            guard_impermanent_loss
        ));
        // rm pool
        assert_ok!(RDexMining::rm_pool(Origin::root(), symbol, 0, 0));
        let stake_pools = RDexMining::stake_pools((symbol, 0)).unwrap();
        assert_eq!(stake_pools.len(), 0);
    });
}

#[test]
fn rm_pool_should_fail() {
    new_test_ext().execute_with(|| {
        let symbol = RSymbol::RATOM;
        // increase pool index
        assert_ok!(RDexMining::increase_pool_index(Origin::root(), symbol));
        assert_eq!(RDexMining::pool_count(symbol), 1);

        let start_block = 2;
        let reward_per_block = 10;
        let total_reward = 200;
        let lp_locked_blocks = 10;
        let guard_impermanent_loss = true;
        // add pool
        assert_ok!(RDexMining::add_pool(
            Origin::root(),
            symbol,
            0,
            start_block,
            lp_locked_blocks,
            reward_per_block,
            total_reward,
            guard_impermanent_loss
        ));
        // rm pool
        assert_err!(
            RDexMining::rm_pool(Origin::root(), symbol, 1, 0),
            Error::<Test>::StakePoolNotExist
        );
        assert_err!(
            RDexMining::rm_pool(Origin::root(), symbol, 0, 1),
            Error::<Test>::GradeIndexOverflow
        );
    });
}

#[test]
fn deposit_should_work() {
    new_test_ext().execute_with(|| {
        let symbol = RSymbol::RATOM;
        assert_ok!(RBalances::mint(&U256::from(42), symbol, 100));
        assert_ok!(RDexSwap::create_pool(
            Origin::root(),
            U256::from(42),
            symbol,
            20,
            10
        ));

        // add liquidity
        assert_ok!(RBalances::mint(&U256::from(1), symbol, 100));
        assert_ok!(RDexSwap::add_liquidity(
            Origin::signed(U256::from(1)),
            symbol,
            20,
            10
        ));
        assert_eq!(LpBalances::free_balance(&U256::from(1), symbol), 10);

        // increase pool index
        assert_ok!(RDexMining::increase_pool_index(Origin::root(), symbol));
        assert_eq!(RDexMining::pool_count(symbol), 1);

        let start_block = 2;
        let reward_per_block = 10;
        let total_reward = 200;
        let lp_locked_blocks = 10;
        let guard_impermanent_loss = true;
        // add pool
        assert_ok!(RDexMining::add_pool(
            Origin::root(),
            symbol,
            0,
            start_block,
            lp_locked_blocks,
            reward_per_block,
            total_reward,
            guard_impermanent_loss
        ));

        // deposit
        assert_ok!(RDexMining::deposit(
            Origin::signed(U256::from(1)),
            symbol,
            0,
            0,
            1
        ));

        assert_eq!(LpBalances::free_balance(&U256::from(1), symbol), 9);
        assert_eq!(
            LpBalances::free_balance(&RDexMining::account_id(), symbol),
            1
        );

        let stake_user = RDexMining::stake_users((symbol, 0, &U256::from(1), 0)).unwrap();
        assert_eq!(stake_user.lp_amount, 1);
        assert_eq!(stake_user.reward_debt, 0);
        assert_eq!(stake_user.reserved_lp_reward, 0);
        assert_eq!(stake_user.total_fis_value, 1);
        assert_eq!(stake_user.total_rtoken_value, 2);
        assert_eq!(stake_user.deposit_height, 0);
        assert_eq!(stake_user.grade_index, 0);
        assert_eq!(stake_user.claimed_reward, 0);

        let stake_pools = RDexMining::stake_pools((symbol, 0)).unwrap();
        assert_eq!(stake_pools.len(), 1);
        assert_eq!(stake_pools[0].symbol, symbol);
        assert_eq!(stake_pools[0].emergency_switch, false);
        assert_eq!(stake_pools[0].total_stake_lp, 1);
        assert_eq!(stake_pools[0].start_block, start_block);
        assert_eq!(stake_pools[0].reward_per_block, reward_per_block);
        assert_eq!(stake_pools[0].total_reward, total_reward);
        assert_eq!(stake_pools[0].left_reward, total_reward);
        assert_eq!(stake_pools[0].lp_locked_blocks, lp_locked_blocks);
        assert_eq!(stake_pools[0].last_reward_block, 0);
        assert_eq!(stake_pools[0].reward_per_share, 0);
        assert_eq!(
            stake_pools[0].guard_impermanent_loss,
            guard_impermanent_loss
        );
    });
}

#[test]
fn deposit_should_fail() {
    new_test_ext().execute_with(|| {
        let symbol = RSymbol::RATOM;
        assert_ok!(RBalances::mint(&U256::from(42), symbol, 100));
        assert_ok!(RDexSwap::create_pool(
            Origin::root(),
            U256::from(42),
            symbol,
            20,
            10
        ));

        // add liquidity
        assert_ok!(RBalances::mint(&U256::from(1), symbol, 100));
        assert_ok!(RDexSwap::add_liquidity(
            Origin::signed(U256::from(1)),
            symbol,
            20,
            10
        ));
        assert_eq!(LpBalances::free_balance(&U256::from(1), symbol), 10);

        // increase pool index
        assert_ok!(RDexMining::increase_pool_index(Origin::root(), symbol));
        assert_eq!(RDexMining::pool_count(symbol), 1);

        let start_block = 2;
        let reward_per_block = 10;
        let total_reward = 200;
        let lp_locked_blocks = 10;
        let guard_impermanent_loss = true;
        // add pool
        assert_ok!(RDexMining::add_pool(
            Origin::root(),
            symbol,
            0,
            start_block,
            lp_locked_blocks,
            reward_per_block,
            total_reward,
            guard_impermanent_loss
        ));

        // deposit
        assert_err!(
            RDexMining::deposit(Origin::signed(U256::from(1)), symbol, 1, 0, 1),
            Error::<Test>::StakePoolNotExist
        );
        assert_err!(
            RDexMining::deposit(Origin::signed(U256::from(1)), symbol, 0, 1, 1),
            Error::<Test>::GradeIndexOverflow
        );
        assert_err!(
            RDexMining::deposit(Origin::signed(U256::from(1)), symbol, 0, 0, 0),
            Error::<Test>::AmountZero
        );
        assert_err!(
            RDexMining::deposit(Origin::signed(U256::from(1)), symbol, 0, 0, 11),
            Error::<Test>::LpBalanceNotEnough
        );
    });
}
