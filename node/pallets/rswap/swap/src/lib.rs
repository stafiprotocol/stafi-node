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
        PoolAlreadyExist,
        RTokenAmountNotEnough,
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
    // f = fis asset added;
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
        (1, 1)
    }
}
