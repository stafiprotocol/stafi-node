#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use sp_std::prelude::*;

use frame_system::{self as system, ensure_signed, ensure_root};
use node_primitives::RSymbol;
use rdex_requestor as requestor;
use rdex_token_price as token_price;

pub trait Trait: system::Trait + requestor::Trait + token_price::Trait {
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
        /// price duplicated
        PriceRepeated,
        /// price is zero
        PriceZero,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RDexOracle {
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Submit rtoken price
        #[weight = 10_000_000]
        pub fn submit_rtoken_price(origin, symbol: RSymbol, era: u32, price: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let era_version = token_price::EraVersion::get() as u32;
            // check state
            ensure!(requestor::Module::<T>::is_requestor(&who), requestor::Error::<T>::MustBeRequestor);
            ensure!(token_price::AccountRTokenPrices::<T>::get((&who, symbol, era_version, era)).is_none(), Error::<T>::PriceRepeated);

            // update prices vec
            let mut prices = token_price::RTokenPrices::get(symbol, (era_version, era)).unwrap_or(vec![]);
            prices.push(price);
            token_price::RTokenPrices::insert(symbol, (era_version, era), &prices);
            token_price::AccountRTokenPrices::<T>::insert((&who, symbol, era_version, era), price);

            // update CurrentRTokenPrice and HisRTokenPrice
            if prices.len() == requestor::RequestorThreshold::get() as usize {
                prices.sort_by(|a, b| b.cmp(a));
                let will_use_price = prices.get(prices.len() / 2).unwrap_or(&u128::MIN);
                ensure!(*will_use_price != u128::MIN, Error::<T>::PriceZero);

                token_price::CurrrentRTokenPrice::insert(symbol, will_use_price);
                token_price::HisRTokenPrice::insert(symbol, (era_version, era), will_use_price);

                Self::deposit_event(RawEvent::RTokenPriceEnough(symbol, era_version, era, *will_use_price));
             }

            Self::deposit_event(RawEvent::SubmitRtokenPrice(who.clone(), symbol, era_version, era, price));
            Ok(())
        }
        /// Submit fis price
        #[weight = 10_000_000]
        pub fn submit_fis_price(origin, era: u32, price: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let era_version = token_price::EraVersion::get() as u32;
            // check state
            ensure!(requestor::Module::<T>::is_requestor(&who), requestor::Error::<T>::MustBeRequestor);
            ensure!(token_price::AccountFisPrices::<T>::get((&who, era_version, era)).is_none(), Error::<T>::PriceRepeated);

            // update prices vec
            let mut prices = token_price::FisPrices::get((era_version, era)).unwrap_or(vec![]);
            prices.push(price);
            token_price::FisPrices::insert((era_version, era), &prices);
            token_price::AccountFisPrices::<T>::insert((&who, era_version, era), price);

            // update CurrentFisPrice and HisFisPrice
            if prices.len() == requestor::RequestorThreshold::get() as usize {
                prices.sort_by(|a, b| b.cmp(a));
                let will_use_price = prices.get(prices.len()/2).unwrap_or(&u128::MIN);
                ensure!(*will_use_price != u128::MIN, Error::<T>::PriceZero);

                token_price::CurrentFisPrice::put(will_use_price);
                token_price::HisFisPrice::insert((era_version, era), will_use_price);

                Self::deposit_event(RawEvent::FisPriceEnough(era_version, era, *will_use_price));
             }

            Self::deposit_event(RawEvent::SubmitFisPrice(who.clone(), era_version, era, price));
            Ok(())
        }

        /// Adds a new requestor to the requestor set.
        #[weight = 10_000]
        pub fn set_era_block_number(origin, block_number: u32) -> DispatchResult {
            ensure_root(origin)?;
            token_price::EraBlockNumber::put(block_number);
            token_price::EraVersion::mutate(|i| *i += 1);
            Self::deposit_event(RawEvent::EraBlockNumberChanged(token_price::EraVersion::get() as u32, block_number));
            Ok(())
        }
    }
}
