#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage};
use sp_std::prelude::*;

use frame_system::{self as system};
use node_primitives::RSymbol;

pub const DEFAULT_ERA_BLOCK_NUMBER: u32 = 20;

pub trait Trait: system::Trait {}

decl_storage! {
    trait Store for Module<T: Trait> as RDexTokenPrice {
        pub EraVersion get(fn era_version): u32 = 0;
        pub EraBlockNumber get(fn era_block_number): u32 = DEFAULT_ERA_BLOCK_NUMBER;
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
    }
}
