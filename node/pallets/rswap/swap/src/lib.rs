#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{Currency, ExistenceRequirement::KeepAlive},
};
use sp_std::prelude::*;

use frame_system::{self as system, ensure_root, ensure_signed};
use node_primitives::{Balance, RSymbol};
use rtoken_balances::traits::Currency as RCurrency;
use sp_arithmetic::helpers_128bit::multiply_by_rational;
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
}

pub mod models;
pub use models::*;

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
        RTokenAmountNotEnough,
        PoolBalanceNotEnough,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RSwapSwap {
        pub SwapPools get(fn swap_pools): map hasher(blake2_128_concat) RSymbol => Option<SwapPool>;
        pub SwapLiquidityProviders get(fn swap_liquidity_providers): map hasher(blake2_128_concat) (T::AccountId, RSymbol) => Option<SwapLiquidityProvider<T::AccountId>>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;
        /// swap rtoken for fis
        #[weight = 10_000_000_000]
        pub fn swap(origin, symbol: RSymbol, in_amount: u128, min_out_amount: u128, in_is_fis: bool) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut pool = Self::swap_pools(symbol).ok_or(Error::<T>::PoolNotExist)?;

            ensure!(in_amount > u128::MIN || min_out_amount > u128::MIN, Error::<T>::AmountAllZero);
            let result = Self::cal_swap_result(pool.fis_balance, pool.rtoken_balance, in_amount,in_is_fis);
            if in_is_fis {
                ensure!(result < pool.rtoken_balance, Error::<T>::PoolBalanceNotEnough);
            } else {
                ensure!(T::RCurrency::free_balance(&who, symbol) >= in_amount, Error::<T>::RTokenAmountNotEnough);
                ensure!(result < pool.fis_balance, Error::<T>::PoolBalanceNotEnough);
            }

            //update state
            if in_is_fis {
                T::Currency::transfer(&who, &Self::account_id(), in_amount.saturated_into(), KeepAlive)?;
                T::RCurrency::transfer(&Self::account_id(), &who, symbol, result)?;

                pool.fis_balance = pool.fis_balance.saturating_add(in_amount);
                pool.rtoken_balance = pool.rtoken_balance.saturating_sub(result);
            } else {
                T::RCurrency::transfer( &who, &Self::account_id(),symbol, in_amount)?;
                T::Currency::transfer(&who, &Self::account_id(), result.saturated_into(), KeepAlive)?;

                pool.rtoken_balance = pool.rtoken_balance.saturating_add(in_amount);
                pool.fis_balance = pool.fis_balance.saturating_sub(result);
            }

            <SwapPools>::insert(symbol, pool);
            Ok(())
        }

        /// create pool
        #[weight = 10_000_000_000]
        pub fn create_pool(origin, symbol: RSymbol, rtoken_amount: u128, fis_amount: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let now_block = system::Module::<T>::block_number().saturated_into::<u64>();

            ensure!(Self::swap_pools(symbol).is_none(), Error::<T>::PoolAlreadyExist);
            ensure!(fis_amount > u128::MIN && rtoken_amount > u128::MIN, Error::<T>::AmountZero);
            ensure!(T::RCurrency::free_balance(&who, symbol) >= rtoken_amount, Error::<T>::RTokenAmountNotEnough);

            let (pool_unit, lp_unit) = Self::cal_pool_unit(0, 0, 0, fis_amount, rtoken_amount);

            // transfer token to moudle account
            T::Currency::transfer(&who, &Self::account_id(), fis_amount.saturated_into(), KeepAlive)?;
            T::RCurrency::transfer(&who, &Self::account_id(), symbol, rtoken_amount)?;

            let pool = SwapPool {
                rtoken: symbol,
                fis_balance: fis_amount,
                rtoken_balance: rtoken_amount,
                total_unit: pool_unit,
            };
            <SwapPools>::insert(symbol, pool);

            let lp = SwapLiquidityProvider {
                account: who.clone(),
                rtoken: symbol,
                unit: lp_unit,
                last_add_height: now_block,
                last_remove_height: 0,
                fis_add_value: fis_amount,
                rtoken_add_value: rtoken_amount,
            };
            <SwapLiquidityProviders<T>>::insert((who, symbol), lp);

            Ok(())
        }

        /// add liquidity
        #[weight = 10_000_000_000]
        pub fn add_liquidity(origin, symbol: RSymbol, rtoken_amount: u128, fis_amount: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut pool = Self::swap_pools(symbol).ok_or(Error::<T>::PoolNotExist)?;
            let now_block = system::Module::<T>::block_number().saturated_into::<u64>();

            ensure!(fis_amount > u128::MIN || rtoken_amount > u128::MIN, Error::<T>::AmountAllZero);
            ensure!(T::RCurrency::free_balance(&who, symbol) >= rtoken_amount, Error::<T>::RTokenAmountNotEnough);

            let (pool_unit, lp_unit) = Self::cal_pool_unit(pool.total_unit, pool.fis_balance, pool.rtoken_balance, fis_amount, rtoken_amount);

            // transfer token to moudle account
            if fis_amount > 0 {
                T::Currency::transfer(&who, &Self::account_id(), fis_amount.saturated_into(), KeepAlive)?;
            }
            if rtoken_amount > 0 {
                T::RCurrency::transfer(&who, &Self::account_id(), symbol, rtoken_amount)?;
            }

            pool.total_unit = pool_unit;
            pool.fis_balance =  pool.fis_balance.saturating_add(fis_amount);
            pool.rtoken_balance = pool.rtoken_balance.saturating_add(rtoken_amount);
            <SwapPools>::insert(symbol, pool);

            let mut lp = Self::swap_liquidity_providers((who.clone(), symbol)).unwrap_or(
                SwapLiquidityProvider{
                account: who.clone(),
                rtoken: symbol,
                unit: 0,
                last_add_height: 0,
                last_remove_height: 0,
                fis_add_value: 0,
                rtoken_add_value: 0,});
            lp.unit = lp_unit;
            lp.last_add_height = now_block;
            lp.fis_add_value = lp.fis_add_value.saturating_add(fis_amount);
            lp.rtoken_add_value = lp.rtoken_add_value.saturating_add(rtoken_amount);
            <SwapLiquidityProviders<T>>::insert((who, symbol), lp);

            Ok(())
        }

        /// remove liduidity
        #[weight = 10_000_000_000]
        pub fn remove_liquidity(origin, symbol: RSymbol, unit: u128, asymmetry: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;
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
    // units = ((P (r F + R f))/(2 R F))*slidAdjustment
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

        let F = fis_balance;
        let R = rtoken_balance;
        let f = fis_amount;
        let r = rtoken_amount;
        let P = old_pool_unit;

        let slip_adj_denominator = f.saturating_add(F).saturating_mul(r.saturating_add(R));
        let abs: u128;
        if F.saturating_mul(r) > f.saturating_mul(R) {
            abs = F.saturating_mul(r).saturating_sub(f.saturating_mul(R));
        } else {
            abs = f.saturating_mul(R).saturating_sub(F.saturating_mul(r));
        }

        let numerator = F.saturating_mul(r).saturating_add(f.saturating_mul(R));
        let raw_unit = multiply_by_rational(P, numerator, R.saturating_mul(F).saturating_mul(2))
            .unwrap_or(u128::MIN) as u128;
        if raw_unit == 0 {
            return (0, 0);
        }
        let adj =
            multiply_by_rational(raw_unit, abs, slip_adj_denominator).unwrap_or(u128::MIN) as u128;
        let add_unit = raw_unit.saturating_sub(adj);
        let total_unit = add_unit.saturating_add(add_unit);
        (total_unit, add_unit)
    }

    //y = (x * X * Y) / (x + X)^2
    pub fn cal_swap_result(
        fis_balance: u128,
        rtoken_balance: u128,
        in_amount: u128,
        in_is_fis: bool,
    ) -> u128 {
        if fis_balance == 0 || rtoken_balance == 0 || in_amount == 0 {
            return 0;
        }
        let mut x = in_amount;
        let mut X = rtoken_balance;
        let mut Y = fis_balance;
        if in_is_fis {
            X = fis_balance;
            Y = rtoken_balance;
        }
        let t = x.saturating_add(X);
        let denominator = t.saturating_mul(t);
        let y =
            multiply_by_rational(x, X.saturating_mul(Y), denominator).unwrap_or(u128::MIN) as u128;
        return y;
    }
}
