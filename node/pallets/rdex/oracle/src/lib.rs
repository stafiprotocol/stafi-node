#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
};
use sp_std::prelude::*;

use frame_system::{self as system, ensure_signed};
use node_primitives::RSymbol;
use rdex_monitors as monitors;

pub trait Trait: system::Trait + monitors::Trait {
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
        pub EraVersion get(fn era_version): u32 = 0;
        /// rsymbol=>price
        pub CurrrentRTokenPrice get(fn current_rtoken_price):
            map hasher(blake2_128_concat) RSymbol => u128;

        /// rsymbol=>(era_version,era)=>price
        pub HisRTokenPrice get(fn his_rtoken_price):
            double_map hasher(blake2_128_concat) RSymbol, hasher(blake2_128_concat) (u32, u32) => u128;

        pub CurrentFisPrice get(fn current_fis_price): u128;

        /// (era_version,era)=>price
        pub HisFisPrice get(fn his_fis_price):
            map hasher(blake2_128_concat) (u32,u32) => u128;

        /// rsymbol=>rate
        pub CurrentFisPerRToken get(fn current_fis_per_rtoken):
            map hasher(blake2_128_concat) RSymbol => u128;

        /// rsymbol=>(era_version,era)=>rate
        pub HisFisPerRToken get(fn his_fis_per_rtoken):
            double_map hasher(blake2_128_concat) RSymbol, hasher(blake2_128_concat) (u32, u32) => u128;

        /// rsymbol=>(era_version,era)=>vec<price>
        pub RTokenPrices get(fn rtoken_prices):
            double_map hasher(blake2_128_concat) RSymbol, hasher(blake2_128_concat) (u32, u32) => Option<Vec<u128>>;

        /// (account,rsymbol,era_version,era)=>price
        pub AccountRTokenPrices get(fn account_rtoken_prices):
            map hasher(blake2_128_concat) (T::AccountId, RSymbol, u32, u32) => Option<u128>;

        /// (era_version,era)=>vec<price>
        pub FisPrices get(fn fis_prices):
            map hasher(blake2_128_concat) (u32, u32) => Option<Vec<u128>>;

        /// (account,era_version,era)=>price
        pub AccountFisPrices get(fn account_fis_prices):
            map hasher(blake2_128_concat) (T::AccountId, u32, u32) => Option<u128>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

         /// Submit rtoken price
         #[weight = 10_000_000]
         pub fn submit_rtoken_price(origin, symbol: RSymbol, era: u32, price: u128) -> DispatchResult {
             let who = ensure_signed(origin)?;
             let era_version = Self::era_version as u32;
             ensure!(monitors::Module::<T>::is_monitor(&who), monitors::Error::<T>::MustBeMonitor);
             ensure!(Self::account_rtoken_prices((&who, symbol,era_version, era)).is_none(), Error::<T>::PriceRepeated);

             let mut prices = RTokenPrices::get(symbol, (era_version,era)).unwrap_or(vec![]);
             prices.push(price);
             <RTokenPrices>::insert(symbol, (era_version, era), &prices);
             <AccountRTokenPrices<T>>::insert((&who, symbol,era_version, era), price);

             if prices.len() == monitors::MonitorThreshold::get() as usize {
                 Self::deposit_event(RawEvent::RTokenPriceEnough(symbol, era_version, era));
             }
             Self::deposit_event(RawEvent::SubmitRtokenPrice(who.clone(), symbol,era_version, era, price));
             Ok(())
         }

    }
}
