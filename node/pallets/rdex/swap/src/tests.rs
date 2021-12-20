use super::mock::{Call, *};
use super::*;
use frame_support::{assert_err, assert_noop, assert_ok};

pub struct CalPoolUnitData {
    pub old_pool_unit: u128,
    pub fis_balance: u128,
    pub rtoken_balance: u128,
    pub fis_amount: u128,
    pub rtoken_amount: u128,
    pub expect_add_unit: u128,
    pub expect_new_pool_unit: u128,
}

#[test]
fn cal_pool_unit_should_work() {
    new_test_ext().execute_with(|| {
        let test_datas = vec![
            CalPoolUnitData {
                old_pool_unit: 0,
                fis_balance: 0,
                rtoken_balance: 0,
                fis_amount: 803080648314941877218,
                rtoken_amount: 442072129,
                expect_add_unit: 803080648314941877218,
                expect_new_pool_unit: 803080648314941877218,
            },
            CalPoolUnitData {
                old_pool_unit: 803080648314941877218,
                fis_balance: 803080648314941877218,
                rtoken_balance: 442072129,
                fis_amount: 803080648314941877218,
                rtoken_amount: 442072129,
                expect_add_unit: 803080648314941877218,
                expect_new_pool_unit: 803080648314941877218 * 2,
            },
            CalPoolUnitData {
                old_pool_unit: 803080648314941877218,
                fis_balance: 803080648314941877218,
                rtoken_balance: 442072129,
                fis_amount: 803080648314941877218 * 2,
                rtoken_amount: 442072129 * 2,
                expect_add_unit: 803080648314941877218 * 2,
                expect_new_pool_unit: 803080648314941877218 * 3,
            },
            CalPoolUnitData {
                old_pool_unit: 1,
                fis_balance: 1,
                rtoken_balance: 1,
                fis_amount: u128::max_value(),
                rtoken_amount: u128::max_value(),
                expect_add_unit: u128::max_value(),
                expect_new_pool_unit: u128::max_value(),
            },
            CalPoolUnitData {
                old_pool_unit: 2,
                fis_balance: 2,
                rtoken_balance: 2,
                fis_amount: 3,
                rtoken_amount: 3,
                expect_add_unit: 3,
                expect_new_pool_unit: 5,
            },
            CalPoolUnitData {
                old_pool_unit: 1,
                fis_balance: 3,
                rtoken_balance: 3,
                fis_amount: 4,
                rtoken_amount: 4,
                expect_add_unit: 1,
                expect_new_pool_unit: 2,
            },
        ];

        for data in test_datas {
            let (new_total_unit, add_unit) = RDexSwap::cal_pool_unit(
                data.old_pool_unit,
                data.fis_balance,
                data.rtoken_balance,
                data.fis_amount,
                data.rtoken_amount,
            );
            assert_eq!(add_unit, data.expect_add_unit);
            assert_eq!(new_total_unit, data.expect_new_pool_unit);
        }
    });
}

pub struct CalSwapResultData {
    pub fis_balance: u128,
    pub rtoken_balance: u128,
    pub input_amount: u128,
    pub input_is_fis: bool,
    pub expect_result: u128,
    pub expect_fee: u128,
}

#[test]
fn cal_swap_result_should_work() {
    new_test_ext().execute_with(|| {
        let test_datas = vec![
            CalSwapResultData {
                fis_balance: 0,
                rtoken_balance: 803080648314941877218,
                input_amount: 803080648314941877218,
                input_is_fis: true,
                expect_result: 0,
                expect_fee: 0,
            },
            CalSwapResultData {
                fis_balance: 803080648314941877218,
                rtoken_balance: 0,
                input_amount: 803080648314941877218,
                input_is_fis: true,
                expect_result: 0,
                expect_fee: 0,
            },
            CalSwapResultData {
                fis_balance: 803080648314941877218,
                rtoken_balance: 803080648314941877218,
                input_amount: 0,
                input_is_fis: true,
                expect_result: 0,
                expect_fee: 0,
            },
            CalSwapResultData {
                fis_balance: 1000000000000000000000000000,
                rtoken_balance: 1000000000000000000000000000,
                input_amount: 100000000000000000000000000,
                input_is_fis: true,
                expect_result: 82644628099173553719008264,
                expect_fee: 8264462809917355371900826,
            },
            CalSwapResultData {
                fis_balance: 100,
                rtoken_balance: 1000,
                input_amount: 10,
                input_is_fis: true,
                expect_result: 82,
                expect_fee: 8,
            },
            CalSwapResultData {
                fis_balance: 1000,
                rtoken_balance: 100,
                input_amount: 10,
                input_is_fis: false,
                expect_result: 82,
                expect_fee: 8,
            },
        ];

        for data in test_datas {
            let (result, fee) = RDexSwap::cal_swap_result(
                data.fis_balance,
                data.rtoken_balance,
                data.input_amount,
                data.input_is_fis,
            );
            assert_eq!(result, data.expect_result);
            assert_eq!(fee, data.expect_fee);
        }
    });
}

pub struct CalRemoveResultData {
    pub pool_unit: u128,
    pub rm_unit: u128,
    pub swap_unit: u128,
    pub fis_balance: u128,
    pub rtoken_balance: u128,
    pub input_is_fis: bool,
    pub expect_rm_fis_amount: u128,
    pub expect_rm_rtoken_amount: u128,
    pub expect_swap_amount: u128,
}

#[test]
fn cal_remove_result_should_work() {
    new_test_ext().execute_with(|| {
        let test_datas = vec![
            CalRemoveResultData {
                pool_unit: 0,
                rm_unit: 1,
                swap_unit: 0,
                fis_balance: 0,
                rtoken_balance: 0,
                input_is_fis: true,
                expect_rm_fis_amount: 0,
                expect_rm_rtoken_amount: 0,
                expect_swap_amount: 0,
            },
            CalRemoveResultData {
                pool_unit: 1,
                rm_unit: 0,
                swap_unit: 0,
                fis_balance: 1,
                rtoken_balance: 1,
                input_is_fis: true,
                expect_rm_fis_amount: 0,
                expect_rm_rtoken_amount: 0,
                expect_swap_amount: 0,
            },
            CalRemoveResultData {
                pool_unit: 10,
                rm_unit: 1,
                swap_unit: 0,
                fis_balance: 20,
                rtoken_balance: 20,
                input_is_fis: true,
                expect_rm_fis_amount: 2,
                expect_rm_rtoken_amount: 2,
                expect_swap_amount: 0,
            },
            CalRemoveResultData {
                pool_unit: 10,
                rm_unit: 1,
                swap_unit: 1,
                fis_balance: 20,
                rtoken_balance: 20,
                input_is_fis: true,
                expect_rm_fis_amount: 2,
                expect_rm_rtoken_amount: 2,
                expect_swap_amount: 2,
            },
            CalRemoveResultData {
                pool_unit: u128::max_value(),
                rm_unit: u128::max_value(),
                swap_unit: 1,
                fis_balance: u128::max_value(),
                rtoken_balance: u128::max_value(),
                input_is_fis: true,
                expect_rm_fis_amount: u128::max_value(),
                expect_rm_rtoken_amount: u128::max_value(),
                expect_swap_amount: 1,
            },
        ];

        for data in test_datas {
            let (rm_fis_amount, rm_rtoken_amount, swap_amount) = RDexSwap::cal_remove_result(
                data.pool_unit,
                data.rm_unit,
                data.swap_unit,
                data.fis_balance,
                data.rtoken_balance,
                data.input_is_fis,
            );
            assert_eq!(rm_fis_amount, data.expect_rm_fis_amount);
            assert_eq!(rm_rtoken_amount, data.expect_rm_rtoken_amount);
            assert_eq!(swap_amount, data.expect_swap_amount);
        }
    });
}

pub struct SafeToU128Data {
    pub number: U512,
    pub expect_u128: u128,
}
#[test]
fn safe_to_u128_should_work() {
    new_test_ext().execute_with(|| {
        let test_datas = vec![
            SafeToU128Data {
                number: U512::from(0),
                expect_u128: 0,
            },
            SafeToU128Data {
                number: U512::from(1),
                expect_u128: 1,
            },
            SafeToU128Data {
                number: U512::from(u128::max_value()),
                expect_u128: u128::max_value(),
            },
            SafeToU128Data {
                number: U512::from(u128::max_value()) + 1,
                expect_u128: u128::max_value(),
            },
        ];
        for data in test_datas {
            let (u128_value) = RDexSwap::safe_to_u128(data.number);
            assert_eq!(u128_value, data.expect_u128);
        }
    });
}
