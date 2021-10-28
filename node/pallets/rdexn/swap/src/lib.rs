#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{Currency, ExistenceRequirement::{KeepAlive}},
};
use sp_std::prelude::*;

use frame_system::{self as system, ensure_root, ensure_signed};
use node_primitives::{Balance, RSymbol};
use rtoken_balances::traits::Currency as RCurrency;
use general_signature::verify_recipient;
use rtoken_rate as RTokenRate;
use rdexn_payers as RDexnPayers;
use sp_arithmetic::helpers_128bit::multiply_by_rational;
use sp_runtime::traits::{SaturatedConversion};
pub trait Trait: system::Trait + RTokenRate::Trait + RDexnPayers::Trait{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// currency of rtoken
    type RCurrency: RCurrency<Self::AccountId>;
    /// The currency mechanism.
    type Currency: Currency<Self::AccountId>;
}

pub mod models;
pub use models::*;
pub const RATEBASE: u128 = 1_000_000_000_000;
decl_event! {
    pub enum Event<T> where
        AccountId = <T as system::Trait>::AccountId
    {
        /// swap rtoken to native: account, symbol, trans block,fee amount, rtoken amount, out amount, rtoken rate, swap rate
        SwapRTokenToNative(AccountId, Vec<u8>, RSymbol, u64, Balance, u128, u128, u64, u128),
        /// report with block: account, symbol, deal block
        ReportTransResultWithBlock(AccountId, RSymbol, u64),
        /// report with index: account, symbol, deal block
        ReportTransResultWithIndex(AccountId, RSymbol, u64, u32),
        /// report with index: account, symbol, deal block
        ReportTransResultWithIndexBlockEnd(AccountId, RSymbol, u64),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// price is zero
        PriceZero,
        /// params err
        ParamsErr,
        /// receiver invalid
        ReceiverInvalid,
        /// swap total switch is closed
        SwapTotalClosed,
        /// swap rtoken switch is closed
        SwapRtokenClosed,
        /// no fund address
        NoFundAddress,
        /// rtoken amount not enough
        RTokenAmountNotEnough,
        /// rtoken rate failed
        RTokenRateFailed,
        /// swap rate failed
        SwapRateFailed,
        /// over swap limit
        OverSwapLimitPerBlock,
        /// native token of other chain not enough
        NativeTokenReserveNotEnough,
        /// out amount less than min out amount
        LessThanMinOutAmount,
        /// overs flow
        OverFlow,
        /// voter repeat
        VoterRepeat,
        /// get trans info failed
        GetTransInfoFailed,
    }
}


decl_storage! {
    trait Store for Module<T: Trait> as RDexnSwap {
        /// swap total switch, default closed
        pub SwapTotalSwitch get(fn swap_total_switch): bool = false;
        /// swap rtoken switch, default open
        pub SwapRTokenSwitch get(fn swap_rtoken_switch): map hasher(blake2_128_concat)  RSymbol => bool = true;
        /// fund address
        pub FundAddress get(fn fund_address): Option<T::AccountId>;
        /// swap fee of rtokens
        pub SwapFees get(fn swap_fees): map hasher(blake2_128_concat) RSymbol => Balance = 1500000000000;
        /// swap rate that admin can set 
        pub SwapRates get(fn swap_rates): map hasher(blake2_128_concat) (RSymbol, u8) => Option<SwapRate>;
        // trans info
        pub TransInfos get(fn trans_infos): map hasher(blake2_128_concat) (RSymbol, u64) => Option<Vec<SwapTransactionInfo<T::AccountId>>>;
        /// latest deal block number
        pub LatestDealBlock get(fn latest_deal_block): map hasher(blake2_128_concat) RSymbol => u64;
        /// swap number limit per block
        pub SwapLimitPerBlock get(fn swap_limit_per_block): u32 = 200;
        /// other chain native token reserve
        pub NativeTokenReserves get(fn native_token_reserves): map hasher(blake2_128_concat) RSymbol => u128;
        /// vote info
        pub VoteInfos get(fn vote_infos): map hasher(blake2_128_concat) (RSymbol, u64) => Option<Vec<T::AccountId>>;
        /// vote info with index
        pub VoteInfosWithIndex get(fn vote_infos_with_index): map hasher(blake2_128_concat) (RSymbol, u64, u32) => Option<Vec<T::AccountId>>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;
        /// swap rtoken for native token
        #[weight = 10_000_000_000]
        pub fn swap_rtoken_for_native_token(origin, receiver: Vec<u8>, symbol: RSymbol, rtoken_amount: u128, min_out_amount: u128, grade: u8) -> DispatchResult {
            let who = ensure_signed(origin)?;   
            let now_block = system::Module::<T>::block_number().saturated_into::<u64>();
            let fund_addr = Self::fund_address().ok_or(Error::<T>::NoFundAddress)?; 
            let rtoken_rate = RTokenRate::Rate::get(symbol).ok_or(Error::<T>::RTokenRateFailed)?;
            let fee_amount = Self::swap_fees(symbol);
            let swap_rate = Self::swap_rates((symbol, grade)).ok_or(Error::<T>::SwapRateFailed)?;
            let trans_block = now_block.checked_add(swap_rate.lock_number).ok_or(Error::<T>::OverFlow)?;
            let out_reserve = Self::native_token_reserves(symbol);

            
            ensure!(Self::swap_total_switch(), Error::<T>::SwapTotalClosed);
            ensure!(Self::swap_rtoken_switch(symbol), Error::<T>::SwapRtokenClosed);
            ensure!(rtoken_amount > u128::MIN, Error::<T>::ParamsErr);
            ensure!(min_out_amount > u128::MIN, Error::<T>::ParamsErr);
            ensure!(rtoken_rate > 0, Error::<T>::RTokenRateFailed);
            ensure!(swap_rate.rate > 0, Error::<T>::SwapRateFailed);
            ensure!(T::RCurrency::free_balance(&who, symbol) >= rtoken_amount, Error::<T>::RTokenAmountNotEnough);

            // check receiver
            match verify_recipient(symbol, &receiver) {
                false => Err(Error::<T>::ReceiverInvalid)?,
                _ => (),
            }

            // check limit per block
            let mut trans_block_trans_info = Self::trans_infos((symbol, trans_block)).unwrap_or(vec![]);
            ensure!(trans_block_trans_info.len() < Self::swap_limit_per_block() as usize, Error::<T>::OverSwapLimitPerBlock);
            
            // check min out amount and reserve amount
            let temp_out_amount = RTokenRate::Module::<T>::rtoken_to_token(symbol, rtoken_amount);
            let out_amount = multiply_by_rational(temp_out_amount, swap_rate.rate, RATEBASE.into()).unwrap_or(u128::MIN) as u128;

            ensure!(out_amount >= min_out_amount, Error::<T>::LessThanMinOutAmount);
            ensure!(out_amount < out_reserve, Error::<T>::NativeTokenReserveNotEnough);

            //update state
            if fee_amount > 0 {
                T::Currency::transfer(&who, &fund_addr, fee_amount.saturated_into(), KeepAlive)?;
            }
            T::RCurrency::transfer(&who, &fund_addr, symbol, rtoken_amount)?;
            trans_block_trans_info.push(SwapTransactionInfo{account: who.clone(), receiver: receiver.clone(), value: out_amount, is_deal: false});
            <TransInfos<T>>::insert((symbol, trans_block), trans_block_trans_info);
            NativeTokenReserves::insert(symbol, out_reserve.saturating_sub(out_amount));
            Self::deposit_event(RawEvent::SwapRTokenToNative(who.clone(), receiver.clone(), symbol, trans_block, fee_amount, rtoken_amount, out_amount, rtoken_rate, swap_rate.rate));
            Ok(())
        }


        /// report transfer result with block
        #[weight = 100_000]
        pub fn report_transfer_result_with_block(origin, symbol: RSymbol, block: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // check
            ensure!(RDexnPayers::Module::<T>::is_payer(symbol, &who), RDexnPayers::Error::<T>::MustBePayer);
            let mut trans_block_trans_info = Self::trans_infos((symbol, block)).unwrap_or(vec![]);
            
            let mut vote_infos = Self::vote_infos((symbol, block)).unwrap_or(vec![]);
            ensure!(!vote_infos.contains(&who), Error::<T>::VoterRepeat);

            vote_infos.push(who.clone());
            <VoteInfos<T>>::insert((symbol, block), &vote_infos);

            if vote_infos.len() == RDexnPayers::PayerThreshold::get(symbol) as usize {
                LatestDealBlock::insert(symbol, block);
                for trans_info in trans_block_trans_info.iter_mut() {
                    trans_info.is_deal = true;
                }
                <TransInfos<T>>::insert((symbol, block), trans_block_trans_info);
                Self::deposit_event(RawEvent::ReportTransResultWithBlock(who.clone(), symbol, block));
            }
            Ok(())
        }

        /// report transfer result with index
        #[weight = 100_000]
        pub fn report_transfer_result_with_index(origin, symbol: RSymbol, block: u64, index: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // check
            ensure!(RDexnPayers::Module::<T>::is_payer(symbol, &who), RDexnPayers::Error::<T>::MustBePayer);
            let mut trans_block_trans_info = Self::trans_infos((symbol, block)).unwrap_or(vec![]);
            ensure!(trans_block_trans_info.len() > index as usize, Error::<T>::ParamsErr);
            
            let mut vote_infos = Self::vote_infos_with_index((symbol, block, index)).unwrap_or(vec![]);
            ensure!(!vote_infos.contains(&who), Error::<T>::VoterRepeat);

            vote_infos.push(who.clone());
            <VoteInfosWithIndex<T>>::insert((symbol, block, index), &vote_infos);

            if vote_infos.len() == RDexnPayers::PayerThreshold::get(symbol) as usize {
                let trans_info = trans_block_trans_info.get_mut(index as usize).ok_or(Error::<T>::GetTransInfoFailed)?;
                trans_info.is_deal = true;
                <TransInfos<T>>::insert((symbol, block), trans_block_trans_info.clone());
                Self::deposit_event(RawEvent::ReportTransResultWithIndex(who.clone(), symbol, block, index));
                let mut block_deal_ok = true;
                for trans in trans_block_trans_info.iter() {
                    if !trans.is_deal {
                        block_deal_ok = false;
                        break;
                    }
                }
                if block_deal_ok {
                    LatestDealBlock::insert(symbol, block);
                    Self::deposit_event(RawEvent::ReportTransResultWithIndexBlockEnd(who.clone(), symbol, block));
                }
            }
            Ok(())
        }

        /// turn on/off swap total switch, default closed
        #[weight = 100_000]
        fn toggle_swap_total_switch(origin) -> DispatchResult {
            ensure_root(origin)?;
            let state = Self::swap_total_switch();
            SwapTotalSwitch::put(!state);
            Ok(())
        }

        /// turn on/off swap rtoken switch, default opened
        #[weight = 100_000]
        fn toggle_swap_rtoken_switch(origin, symbol: RSymbol) -> DispatchResult {
            ensure_root(origin)?;
            let state = Self::swap_rtoken_switch(symbol);
            SwapRTokenSwitch::insert(symbol, !state);
            Ok(())
        }

        /// set fund address
        #[weight = 100_000]
        fn set_fund_address(origin, address: T::AccountId) -> DispatchResult {
            ensure_root(origin)?;
            <FundAddress<T>>::put(address);
            Ok(())
        }

        /// set native reserve
        #[weight = 100_000]
        fn set_native_token_reserve(origin, symbol: RSymbol, reserve: u128) -> DispatchResult {
            ensure_root(origin)?;
            NativeTokenReserves::insert(symbol, reserve);
            Ok(())
        }

        /// set swap fee
        #[weight = 100_000]
        fn set_swap_fee(origin, symbol: RSymbol, fee: Balance) -> DispatchResult {
            ensure_root(origin)?;
            SwapFees::insert(symbol, fee);
            Ok(())
        }

        /// set swap rate
        #[weight = 100_000]
        fn set_swap_rate(origin, symbol: RSymbol, grade: u8, lock_number: u64, rate: u128) -> DispatchResult {
            ensure_root(origin)?;
            SwapRates::insert((symbol, grade), SwapRate{lock_number, rate});
            Ok(())
        }

        #[weight = 100_000]
        fn set_swap_limit_per_block(origin, limit: u32) -> DispatchResult {
            ensure_root(origin)?;
            SwapLimitPerBlock::put(limit);
            Ok(())
        }

        #[weight = 100_000]
        fn set_latest_deal_block(origin, symbol: RSymbol, block: u64) -> DispatchResult {
            ensure_root(origin)?;
            LatestDealBlock::insert(symbol, block);
            let mut trans_block_trans_info = Self::trans_infos((symbol, block)).unwrap_or(vec![]);
            for trans_info in trans_block_trans_info.iter_mut() {
                trans_info.is_deal = true;
            }
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {}