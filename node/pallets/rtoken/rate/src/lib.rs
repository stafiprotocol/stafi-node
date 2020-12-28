// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use frame_support::{decl_event, decl_module, decl_storage};
use frame_system::{self as system};
use pallet_staking::EraIndex;
use sp_arithmetic::{helpers_128bit::multiply_by_rational};
use node_primitives::{RSymbol};

pub type RateType = u64;
pub const RATEBASE: RateType = 1_000_000_000_000;

pub trait Trait: system::Trait {
    type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;
}

decl_event! {
    pub enum Event {
        RateSet(RateType),
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RTokenRate {
        /// rate of symbol & eraIndex
        pub EraRate get(fn era_rate):
            double_map hasher(blake2_128_concat) RSymbol, hasher(blake2_128_concat) EraIndex => Option<RateType>;
        
        /// current rate
        pub Rate get(fn rate): map hasher(blake2_128_concat) RSymbol => Option<RateType>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;
    }
}

impl<T: Trait> Module<T> {
    pub fn set_rate(symbol: RSymbol, total: u128, rtotal: u128) -> RateType {
        if total == 0 || rtotal == 0 {
            <Rate>::insert(symbol, RATEBASE);
            Self::deposit_event(Event::RateSet(RATEBASE));
            return RATEBASE;
        }

        let new_rate = multiply_by_rational(total, RATEBASE.into(), rtotal).unwrap_or(RATEBASE.into()) as RateType;
        let op_rate = <Rate>::get(symbol);
        if op_rate.is_none() || op_rate.unwrap() != new_rate {
            <Rate>::insert(symbol, new_rate);
            Self::deposit_event(Event::RateSet(RATEBASE));
        }

        new_rate
    }

    pub fn token_to_rtoken(symbol: RSymbol, balance: u128) -> u128 {
        let op_rate = Rate::get(symbol);
        if op_rate.is_none() || op_rate.unwrap() == 0 {
            return balance;
        }

        multiply_by_rational(balance, RATEBASE.into(), op_rate.unwrap().into()).unwrap_or(balance)
    }

    pub fn rtoken_to_token(symbol: RSymbol, rbalance: u128) -> u128 {
        let op_rate = Rate::get(symbol);
        if op_rate.is_none() {
            return 0;
        }

        multiply_by_rational(rbalance, op_rate.unwrap().into(), RATEBASE.into()).unwrap_or(0)
    }
}



