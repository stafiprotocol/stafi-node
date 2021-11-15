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
use sp_runtime::traits::SaturatedConversion;
pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// currency of rtoken
    type RCurrency: RCurrency<Self::AccountId>;
    /// The currency mechanism.
    type Currency: Currency<Self::AccountId>;
}

pub mod models;
pub use models::*;
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
        pub fn swap_rtoken_for_fis(origin, symbol: RSymbol, rtoken_amount: u128, min_out_amount: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Ok(())
        }

        /// swap fis for rtoken
        #[weight = 10_000_000_000]
        pub fn swap_fis_for_rtoken(origin, symbol: RSymbol, fis_amount: u128, min_out_amount: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Ok(())
        }

        /// create pool
        #[weight = 10_000_000_000]
        pub fn create_pool(origin, symbol: RSymbol, rtoken_amount: u128, fis_amount: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;
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

impl<T: Trait> Module<T> {}
