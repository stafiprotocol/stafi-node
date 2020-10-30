// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use codec::{Encode, Decode};
use frame_support::{
    Parameter, decl_error, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{EnsureOrigin, Get},
};

use frame_system::{self as system, ensure_signed, ensure_root};
use sp_core::U256;
use sp_runtime::traits::{AccountIdConversion};
use sp_runtime::{ModuleId};
use pallet_staking::EraIndex;
pub use self::imbalances::{PositiveImbalance, NegativeImbalance};
// use sp_arithmetic::{helpers_128bit::multiply_by_rational}

// const MODULE_ID: ModuleId = ModuleId(*b"rFIS/pot");

pub trait Trait: system::Trait {
    // type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// The balance of current token
    type RBalance: Parameter;
    /// token
    type RToken: Parameter;
    /// rate
    type Rate: Default;
}

#[derive(Encode, Decode)]
pub struct RtokenData<RBalance> {
    /// total amount of origin token
    pub total_origin: RBalance,
    /// total amount of rtoken
    pub total_rtoken: RBalance,
}

// impl<Balance> RtokenData<Balance> {
//     pub fn stake_mint_amount(&self, value: Balance) -> Balance {
//         if self.total_rtoken == 0 {
//             return value
//         } else {
//             return multiply_by_rational(value, total_origin, total_rtoken);
//         }
//     }
// }

// decl_event! {
//     pub enum Event<T> where
//         AccountId = <T as system::Trait>::AccountId
//     {
//         /// liquidity stake record
//         LiquidityStake(AccountId, U256),
//     }
// }

// decl_error! {
//     pub enum Error for Module<T: Trait> {
//     }
// }

decl_storage! {
    trait Store for Module<T: Trait> as RtokenCommon {
        /// info of eraIndex & rtoken
        pub RtokenInfo get(fn total_origin_token): 
            double_map hasher(twox_64_concat) T::RToken, hasher(blake2_128_concat) EraIndex => Option<RtokenData<T::RBalance>>;
        /// rate of eraIndex & rtoken
        pub RateInfo get(fn total_origin_token): 
            double_map hasher(twox_64_concat) T::RToken, hasher(blake2_128_concat) EraIndex => Option<T::Rate>;


    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    }
}



