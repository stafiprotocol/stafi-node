#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use sp_std::prelude::*;

use frame_system::{self as system, ensure_root, ensure_signed};
use node_primitives::RSymbol;
use rdex_token_price as token_price;

pub trait Trait: system::Trait + token_price::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
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
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RDexSwap {
        /// swap total switch
        pub SwapTotalClosed get(fn swap_total_closed): bool = true;
        pub SwapRTokenClosed get(fn swap_rtoken_closed): map hasher(blake2_128_concat)  RSymbol => bool;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Submit rtoken price
        #[weight = 10_000_000]
        pub fn swap_rtoken_to_fis(origin, symbol: RSymbol, amount: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let era_version = token_price::EraVersion::get() as u32;
            let fis_price = token_price::CurrentFisPrice::get() as u128;
            let rtoken_price = token_price::CurrentRTokenPrice::get(symbol) as u128;
            // check
            ensure!(!Self::swap_total_closed(), Error::<T>::SwapTotalClosed);
            ensure!(!Self::swap_rtoken_closed(symbol), Error::<T>::SwapRtokenClosed);
            ensure!(amount != u128::MIN, Error::<T>::ParamsErr);
            ensure!(fis_price != u128::MIN && rtoken_price != u128::MIN, Error::<T>::PriceZero);


            Ok(())
        }


        /// turn on/off swap switch
        #[weight = 1_000_000]
        fn swap_total_switch(origin) -> DispatchResult {
            ensure_root(origin)?;
            let state = Self::swap_total_closed();
            SwapTotalClosed::put(!state);
            Ok(())
        }

        /// turn on/off swap rtoken switch
        #[weight = 1_000_000]
        fn swap_rtoken_switch(origin, symbol: RSymbol) -> DispatchResult {
            ensure_root(origin)?;
            let state = Self::swap_rtoken_closed(symbol);
            SwapRTokenClosed::insert(symbol, !state);
            Ok(())
        }
    }
}
