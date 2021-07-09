#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    traits::{Currency, ExistenceRequirement::{KeepAlive}},
};
use sp_std::prelude::*;

use frame_system::{self as system, ensure_root, ensure_signed};
use node_primitives::{RSymbol};
use rdex_token_price as token_price;
use rtoken_balances::{traits::{Currency as RCurrency}};

use sp_runtime::{
    traits::{
        SaturatedConversion,
    }
};

pub trait Trait: system::Trait + token_price::Trait{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// currency of rtoken
    type RCurrency: RCurrency<Self::AccountId>;
    /// The currency mechanism.
    type Currency: Currency<Self::AccountId>;
}

decl_event! {
    pub enum Event<T> where
        AccountId = <T as system::Trait>::AccountId
    {
        /// rtoken prices vec enough: rsymbol, era_version, era, price
        RTokenPriceEnough(RSymbol, u32, u32, u128),
        /// submit rtoken price: account, rsymbol, era_version, era, price
        SubmitRtokenPrice(AccountId, RSymbol, u32, u32, u128),
        /// fis prices vec enough: era_version, era, price
        FisPriceEnough(u32, u32, u128),
        /// submit fis price: account, era_version, era, price
        SubmitFisPrice(AccountId, u32, u32, u128),
        /// EraBlockNumberChanged: era_version, block_number
        EraBlockNumberChanged(u32, u32),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// price is zero
        PriceZero,
        /// params err
        ParamsErr,
        /// swap total switch is closed
        SwapTotalClosed,
        /// swap rtoken switch is closed
        SwapRtokenClosed,
        /// no fund address
        NoFundAddress,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RDexSwap {
        /// swap total switch, default closed
        pub SwapTotalClosed get(fn swap_total_closed): bool = true;
        /// swap rtoken switch, default open
        pub SwapRTokenClosed get(fn swap_rtoken_closed): map hasher(blake2_128_concat)  RSymbol => bool;
        /// fund address
        pub FundAddress get(fn fund_address): Option<T::AccountId>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Submit rtoken price
        #[weight = 10_000_000]
        pub fn swap_rtoken_to_fis(origin, symbol: RSymbol, rtoken_amount: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let _era_version = token_price::EraVersion::get() as u32;
            let fis_price = token_price::CurrentFisPrice::get() as u128;
            let rtoken_price = token_price::CurrentRTokenPrice::get(symbol) as u128;
            let op_fund_addr = Self::fund_address();
            ensure!(op_fund_addr.is_some(),Error::<T>::NoFundAddress);
            let fund_addr = op_fund_addr.unwrap();

            // check
            ensure!(!Self::swap_total_closed(), Error::<T>::SwapTotalClosed);
            ensure!(!Self::swap_rtoken_closed(symbol), Error::<T>::SwapRtokenClosed);
            ensure!(rtoken_amount != u128::MIN, Error::<T>::ParamsErr);
            ensure!(fis_price != u128::MIN && rtoken_price != u128::MIN, Error::<T>::PriceZero);
            let fis_amount = (rtoken_price * rtoken_amount) / fis_price;
            T::Currency::transfer(&fund_addr, &who, fis_amount.saturated_into(), KeepAlive)?;
            T::RCurrency::transfer(&who, &fund_addr, symbol, rtoken_amount)?;
            Ok(())
        }


        /// turn on/off swap total switch, default closed
        #[weight = 1_000_000]
        fn swap_total_switch(origin) -> DispatchResult {
            ensure_root(origin)?;
            let state = Self::swap_total_closed();
            SwapTotalClosed::put(!state);
            Ok(())
        }

        /// turn on/off swap rtoken switch, default opened
        #[weight = 1_000_000]
        fn swap_rtoken_switch(origin, symbol: RSymbol) -> DispatchResult {
            ensure_root(origin)?;
            let state = Self::swap_rtoken_closed(symbol);
            SwapRTokenClosed::insert(symbol, !state);
            Ok(())
        }

        /// set fund address
        #[weight = 1_000_000]
        fn set_fund_address(origin, address: T::AccountId) -> DispatchResult {
            ensure_root(origin)?;
            <FundAddress<T>>::put(address);
            Ok(())
        }

        /// init bond pool
        #[weight = 1_000_000]
        pub fn mint_rtoken(origin, symbol: RSymbol, receiver: T::AccountId, amount: u128) -> DispatchResult {
            ensure_root(origin)?;
            T::RCurrency::mint(&receiver, symbol, amount)?;
            Ok(())
        }
    }
}
