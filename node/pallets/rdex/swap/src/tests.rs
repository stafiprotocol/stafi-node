
#[test]
fn cal_pool_unit_should_work() {
    let old_pool_unit: u128 = 0;
    let fis_balance: u128 = 0;
    let rtoken_balance: u128 = 0;
    let fis_amount: u128 = 1_000_000_000_000;
    let rtoken_amount: u128 = 0;
    let (new_total_unit, add_unit) = cal_pool_unit(
        old_pool_unit,
        fis_balance,
        rtoken_balance,
        fis_amount,
        rtoken_amount,
    );
    assert_eq!(new_total_unit, fis_amount);
    assert_eq!(add_unit, fis_amount);
}
