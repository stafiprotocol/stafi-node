#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage};
use sp_std::prelude::*;

use frame_system::{self as system};
use node_primitives::RSymbol;

pub const DEFAULT_ERA_BLOCK_NUMBER: u32 = 20;
pub const DEFAULT_DATA_RESERVE_PERIOD: u32 = 30 * 24 * 60 * 60 / 6;

pub trait Trait: system::Trait {}

decl_storage! {
    trait Store for Module<T: Trait> as RDexTokenPrice {
        pub PeriodVersion get(fn period_version): u32 = 0;
        pub PeriodBlockNumber get(fn period_block_number): u32 = DEFAULT_ERA_BLOCK_NUMBER;
        pub DataReservePeriod get(fn data_reserve_period): u32 = DEFAULT_DATA_RESERVE_PERIOD;
        /// rsymbol=>price
        pub CurrentRTokenPrice get(fn current_rtoken_price):
            map hasher(blake2_128_concat) RSymbol => u128;

        /// rsymbol=>(period_version,period)=>price
        pub HisRTokenPrice get(fn his_rtoken_price):
            double_map hasher(blake2_128_concat) RSymbol, hasher(blake2_128_concat) (u32, u32) => u128;

        pub CurrentFisPrice get(fn current_fis_price): u128;

        /// (period_version,period)=>price
        pub HisFisPrice get(fn his_fis_price):
            map hasher(blake2_128_concat) (u32,u32) => u128;

        /// rsymbol=>(period_version,period)=>vec<price>
        pub RTokenPrices get(fn rtoken_prices):
            double_map hasher(blake2_128_concat) RSymbol, hasher(blake2_128_concat) (u32, u32) => Option<Vec<u128>>;

        /// (account,rsymbol,period_version,period)=>price
        pub AccountRTokenPrices get(fn account_rtoken_prices):
            map hasher(blake2_128_concat) (T::AccountId, RSymbol, u32, u32) => Option<u128>;

        /// (period_version,period)=>vec<price>
        pub FisPrices get(fn fis_prices):
            map hasher(blake2_128_concat) (u32, u32) => Option<Vec<u128>>;

        /// (account,period_version,period)=>price
        pub AccountFisPrices get(fn account_fis_prices):
            map hasher(blake2_128_concat) (T::AccountId, u32, u32) => Option<u128>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    }
}
