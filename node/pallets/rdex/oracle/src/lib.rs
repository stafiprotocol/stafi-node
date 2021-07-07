#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use sp_std::prelude::*;

use frame_system::{self as system, ensure_signed};
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
        /// rsymbol, era_version, era
        RTokenPriceEnough(RSymbol,u32,u32),
        SubmitRtokenPrice(AccountId,RSymbol,u32,u32,u128),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        PriceRepeated,
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
             
             ensure!(requestor::Module::<T>::is_requestor(&who), requestor::Error::<T>::MustBeRequestor);
             ensure!(token_price::AccountRTokenPrices::<T>::get((&who, symbol,era_version, era)).is_none(), Error::<T>::PriceRepeated);

             let mut prices = token_price::RTokenPrices::get(symbol, (era_version,era)).unwrap_or(vec![]);
             prices.push(price);
             token_price::RTokenPrices::insert(symbol, (era_version, era), &prices);
             token_price::AccountRTokenPrices::<T>::insert((&who, symbol,era_version, era), price);

             if prices.len() == requestor::RequestorThreshold::get() as usize {
                 Self::deposit_event(RawEvent::RTokenPriceEnough(symbol, era_version, era));
             }
             Self::deposit_event(RawEvent::SubmitRtokenPrice(who.clone(), symbol,era_version, era, price));
             Ok(())
         }

    }
}
