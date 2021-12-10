#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{Currency, ExistenceRequirement::KeepAlive},
};
use sp_std::prelude::*;

use frame_system::{self as system, ensure_root, ensure_signed};
use node_primitives::RSymbol;
use rdex_balances::traits::Currency as LpCurrency;
use rtoken_balances::traits::Currency as RCurrency;
use sp_runtime::{
    traits::{AccountIdConversion, SaturatedConversion},
    ModuleId,
};
pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// currency of rtoken
    type RCurrency: RCurrency<Self::AccountId>;
    /// The currency mechanism.
    type Currency: Currency<Self::AccountId>;
    /// currency of lp
    type LpCurrency: LpCurrency<Self::AccountId>;
}

pub mod models;
pub use models::*;
use sp_core::U512;

const MODULE_ID: ModuleId = ModuleId(*b"rtk/swap");

decl_event! {
    pub enum Event<T> where
        AccountId = <T as system::Trait>::AccountId
    {
        /// swap rtoken to native: account, symbol, rtoken amount, fis amount, fee
        SwapRTokenForFis(AccountId, RSymbol, u128, u128, u128),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        AmountZero,
        AmountAllZero,
        PoolAlreadyExist,
        PoolNotExist,
        LiquidityProviderNotExist,
        RTokenAmountNotEnough,
        PoolBalanceNotEnough,
        UnitAmountImproper,
        NoGuardPool,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RSwapSwap {
        /// swap pools
        pub SwapPools get(fn swap_pools): map hasher(blake2_128_concat) RSymbol => Option<SwapPool>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;
        /// swap rtoken for fis
        #[weight = 10_000_000_000]
        pub fn swap(origin, symbol: RSymbol, in_amount: u128, min_out_amount: u128, input_is_fis: bool) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut pool = Self::swap_pools(symbol).ok_or(Error::<T>::PoolNotExist)?;

            ensure!(in_amount > 0 || min_out_amount > 0, Error::<T>::AmountAllZero);
            let result = Self::cal_swap_result(pool.fis_balance, pool.rtoken_balance, in_amount, input_is_fis);

            if input_is_fis {
                ensure!(result < pool.rtoken_balance, Error::<T>::PoolBalanceNotEnough);

                //transfer
                T::Currency::transfer(&who, &Self::account_id(), in_amount.saturated_into(), KeepAlive)?;
                T::RCurrency::transfer(&Self::account_id(), &who, symbol, result)?;

                //update pool
                pool.fis_balance = pool.fis_balance.saturating_add(in_amount);
                pool.rtoken_balance = pool.rtoken_balance.saturating_sub(result);
            } else {
                ensure!(T::RCurrency::free_balance(&who, symbol) >= in_amount, Error::<T>::RTokenAmountNotEnough);
                ensure!(result < pool.fis_balance, Error::<T>::PoolBalanceNotEnough);

                //transfer
                T::Currency::transfer(&Self::account_id(), &who, result.saturated_into(), KeepAlive)?;
                T::RCurrency::transfer(&who, &Self::account_id(), symbol, in_amount)?;

                //update pool
                pool.rtoken_balance = pool.rtoken_balance.saturating_add(in_amount);
                pool.fis_balance = pool.fis_balance.saturating_sub(result);
            }
            //update pool storage
            <SwapPools>::insert(symbol, pool);
            Ok(())
        }

        /// create pool
        #[weight = 10_000_000_000]
        pub fn create_pool(origin, symbol: RSymbol, rtoken_amount: u128, fis_amount: u128) -> DispatchResult {
            ensure_root(origin.clone())?;
            let who = ensure_signed(origin)?;
            ensure!(Self::swap_pools(symbol).is_none(), Error::<T>::PoolAlreadyExist);
            ensure!(fis_amount > 0 && rtoken_amount > 0, Error::<T>::AmountZero);
            ensure!(T::RCurrency::free_balance(&who, symbol) >= rtoken_amount, Error::<T>::RTokenAmountNotEnough);

            let (pool_unit, lp_unit) = Self::cal_pool_unit(0, 0, 0, fis_amount, rtoken_amount);

            //create pool/lp
            let pool = SwapPool {
                symbol: symbol,
                fis_balance: fis_amount,
                rtoken_balance: rtoken_amount,
                total_unit: pool_unit,
            };

            // transfer token to module account
            T::Currency::transfer(&who, &Self::account_id(), fis_amount.saturated_into(), KeepAlive)?;
            T::RCurrency::transfer(&who, &Self::account_id(), symbol, rtoken_amount)?;

            // update pool/lp
            T::LpCurrency::mint(&who, symbol, lp_unit)?;
            <SwapPools>::insert(symbol, pool);

            Ok(())
        }

        /// add liquidity
        #[weight = 10_000_000_000]
        pub fn add_liquidity(origin, symbol: RSymbol, rtoken_amount: u128, fis_amount: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut pool = Self::swap_pools(symbol).ok_or(Error::<T>::PoolNotExist)?;

            ensure!(fis_amount > 0 || rtoken_amount > 0, Error::<T>::AmountAllZero);
            ensure!(T::RCurrency::free_balance(&who, symbol) >= rtoken_amount, Error::<T>::RTokenAmountNotEnough);

            let (new_total_pool_unit, new_lp_unit) = Self::cal_pool_unit(pool.total_unit, pool.fis_balance, pool.rtoken_balance, fis_amount, rtoken_amount);

            // transfer token to moudle account
            if fis_amount > 0 {
                T::Currency::transfer(&who, &Self::account_id(), fis_amount.saturated_into(), KeepAlive)?;
            }
            if rtoken_amount > 0 {
                T::RCurrency::transfer(&who, &Self::account_id(), symbol, rtoken_amount)?;
            }

            //update pool
            pool.total_unit = new_total_pool_unit;
            pool.fis_balance =  pool.fis_balance.saturating_add(fis_amount);
            pool.rtoken_balance = pool.rtoken_balance.saturating_add(rtoken_amount);

            //update pool/lp storage
            T::LpCurrency::mint(&who, symbol, new_lp_unit)?;
            <SwapPools>::insert(symbol, pool);

            Ok(())
        }

        /// remove liduidity
        #[weight = 10_000_000_000]
        pub fn remove_liquidity(origin, symbol: RSymbol, rm_unit: u128, swap_unit: u128, input_is_fis: bool) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut pool = Self::swap_pools(symbol).ok_or(Error::<T>::PoolNotExist)?;
            let lp_unit = T::RCurrency::free_balance(&who, symbol);

            ensure!(rm_unit > 0 && rm_unit <= lp_unit && rm_unit >= swap_unit, Error::<T>::UnitAmountImproper);

            let (mut rm_fis_amount, mut rm_rtoken_amount, swap_in_amount) = Self::cal_remove_result(pool.total_unit, rm_unit, swap_unit, pool.fis_balance, pool.rtoken_balance, input_is_fis);

            //update pool/lp
            pool.total_unit = pool.total_unit.saturating_sub(rm_unit);
            pool.fis_balance =  pool.fis_balance.saturating_sub(rm_fis_amount);
            pool.rtoken_balance = pool.rtoken_balance.saturating_sub(rm_rtoken_amount);
            if swap_in_amount > 0 {
                let swap_result = Self::cal_swap_result(pool.fis_balance, pool.rtoken_balance, swap_in_amount, input_is_fis);
                if input_is_fis {
                    pool.fis_balance = pool.fis_balance.saturating_add(swap_in_amount);
                    pool.rtoken_balance = pool.rtoken_balance.saturating_sub(swap_result);

                    rm_fis_amount = rm_fis_amount.saturating_sub(swap_in_amount);
                    rm_rtoken_amount = rm_rtoken_amount.saturating_add(swap_result);
                } else {
                    pool.rtoken_balance = pool.rtoken_balance.saturating_add(swap_in_amount);
                    pool.fis_balance = pool.fis_balance.saturating_sub(swap_result);
                    rm_rtoken_amount = rm_rtoken_amount.saturating_add(swap_in_amount);

                    rm_rtoken_amount = rm_rtoken_amount.saturating_sub(swap_in_amount);
                    rm_fis_amount = rm_fis_amount.saturating_add(swap_result);
                }
            }

            // burn unit
            T::LpCurrency::burn(&who, symbol, rm_unit)?;

            // transfer token to user
            if rm_fis_amount > 0 {
                T::Currency::transfer(&Self::account_id(), &who, rm_fis_amount.saturated_into(), KeepAlive)?;
            }
            if rm_rtoken_amount > 0 {
                T::RCurrency::transfer(&Self::account_id(), &who, symbol, rm_rtoken_amount)?;
            }

            // update pool
            <SwapPools>::insert(symbol, pool);

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    /// Provides an AccountId for the pallet.
    pub fn account_id() -> T::AccountId {
        MODULE_ID.into_account()
    }

    // F = fis Balance (before)
    // R = rToken Balance (before)
    // f = fis added;
    // r = rToken added
    // P = existing Pool Units
    // slipAdjustment = (1 - ABS((F r - f R)/((f + F) (r + R))))
    // units = ((P (r F + R f))/(2 R F))*slipAdjustment
    pub fn cal_pool_unit(
        old_pool_unit: u128,
        fis_balance: u128,
        rtoken_balance: u128,
        fis_amount: u128,
        rtoken_amount: u128,
    ) -> (u128, u128) {
        if fis_amount == 0 && rtoken_amount == 0 {
            return (0, 0);
        }
        if fis_balance.saturating_add(fis_amount) == 0 {
            return (0, 0);
        }
        if rtoken_balance.saturating_add(rtoken_amount) == 0 {
            return (0, 0);
        }
        if fis_balance == 0 || rtoken_balance == 0 {
            return (fis_amount, fis_amount);
        }

        let p_capital = U512::from(old_pool_unit);
        let f_capital = U512::from(fis_balance);
        let r_capital = U512::from(rtoken_balance);
        let f = U512::from(fis_amount);
        let r = U512::from(rtoken_amount);

        let slip_adj_denominator = f
            .saturating_add(f_capital)
            .saturating_mul(r.saturating_add(r_capital));

        let abs: U512;
        if f_capital.saturating_mul(r) > f.saturating_mul(r_capital) {
            abs = f_capital
                .saturating_mul(r)
                .saturating_sub(f.saturating_mul(r_capital));
        } else {
            abs = f
                .saturating_mul(r_capital)
                .saturating_sub(f_capital.saturating_mul(r));
        }

        let numerator = f_capital
            .saturating_mul(r)
            .saturating_add(f.saturating_mul(r_capital));
        let raw_unit = p_capital
            .saturating_mul(numerator)
            .checked_div(
                r_capital
                    .saturating_mul(f_capital)
                    .saturating_mul(U512::from(2)),
            )
            .unwrap_or(U512::zero());
        if raw_unit.is_zero() {
            return (0, 0);
        }
        let adj_unit = raw_unit
            .saturating_mul(abs)
            .checked_div(slip_adj_denominator)
            .unwrap_or(U512::zero());
        let add_unit = raw_unit.saturating_sub(adj_unit);
        let total_unit = p_capital.saturating_add(add_unit);

        (total_unit.as_u128(), add_unit.as_u128())
    }

    //y = (x * X * Y) / (x + X)^2
    pub fn cal_swap_result(
        fis_balance: u128,
        rtoken_balance: u128,
        in_amount: u128,
        input_is_fis: bool,
    ) -> u128 {
        if fis_balance == 0 || rtoken_balance == 0 || in_amount == 0 {
            return 0;
        }
        let x = U512::from(in_amount);
        let mut x_capital = U512::from(rtoken_balance);
        let mut y_capital = U512::from(fis_balance);
        if input_is_fis {
            x_capital = U512::from(fis_balance);
            y_capital = U512::from(rtoken_balance);
        }
        let t = x.saturating_add(x_capital);
        let denominator = t.saturating_mul(t);
        let y = x
            .saturating_mul(x_capital)
            .saturating_mul(y_capital)
            .checked_div(denominator)
            .unwrap_or(U512::zero());

        y.as_u128()
    }

    pub fn cal_remove_result(
        pool_unit: u128,
        rm_unit: u128,
        swap_unit: u128,
        fis_balance: u128,
        rtoken_balance: u128,
        input_is_fis: bool,
    ) -> (u128, u128, u128) {
        if pool_unit == 0 || rm_unit == 0 {
            return (0, 0, 0);
        }
        let use_pool_unit = U512::from(pool_unit);
        let use_fis_balance = U512::from(fis_balance);
        let use_rtoken_balance = U512::from(rtoken_balance);
        let mut use_rm_unit = U512::from(rm_unit);
        let mut use_swap_unit = U512::from(swap_unit);
        if rm_unit > pool_unit {
            use_rm_unit = U512::from(pool_unit);
        }
        if swap_unit > rm_unit {
            use_swap_unit = U512::from(rm_unit);
        }

        let fis_amount = use_rm_unit
            .saturating_mul(use_fis_balance)
            .checked_div(use_pool_unit)
            .unwrap_or(U512::zero());
        let rtoken_amount = use_rm_unit
            .saturating_mul(use_rtoken_balance)
            .checked_div(use_pool_unit)
            .unwrap_or(U512::zero());

        let swap_amount: U512;
        if input_is_fis {
            swap_amount = use_swap_unit
                .saturating_mul(use_fis_balance)
                .checked_div(use_pool_unit)
                .unwrap_or(U512::zero());
        } else {
            swap_amount = use_swap_unit
                .saturating_mul(use_rtoken_balance)
                .checked_div(use_pool_unit)
                .unwrap_or(U512::zero());
        }

        (
            fis_amount.as_u128(),
            rtoken_amount.as_u128(),
            swap_amount.as_u128(),
        )
    }

    pub fn cal_share_amount(pool_unit: u128, share_unit: u128, amount: u128) -> u128 {
        if pool_unit == 0 || share_unit == 0 || amount == 0 {
            return 0;
        }
        let use_pool_unit = U512::from(pool_unit);
        let use_share_unit = U512::from(share_unit);
        let use_amount = U512::from(amount);
        let share_amount = use_amount
            .saturating_mul(use_share_unit)
            .checked_div(use_pool_unit)
            .unwrap_or(U512::zero());

        share_amount.as_u128()
    }
}
