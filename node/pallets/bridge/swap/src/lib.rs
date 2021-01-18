// Copyright 2019-2020 Stafi Protocol.
// This file is part of Stafi.

// Stafi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stafi.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use bridge_common::{self as bridge, ResourceId};
use frame_support::{
    decl_error, decl_module, dispatch::DispatchResult, ensure,
    traits::{
        Currency, EnsureOrigin, Get,
        ExistenceRequirement::{AllowDeath, KeepAlive},
    },
};
use frame_system::{self as system, ensure_signed};
use sp_runtime::{traits::{Zero, Saturating}};
use sp_core::U256;
use sp_arithmetic::traits::SaturatedConversion;
use node_primitives::{ChainId, ETH_CHAIN_ID, RSymbol};
use rtoken_balances::{traits::{Currency as RCurrency}};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

pub trait Trait: system::Trait + bridge::Trait {
    /// The currency mechanism.
    type Currency: Currency<Self::AccountId>;
    /// Currency mechanism of rtoken
    type RCurrency: RCurrency<Self::AccountId>;

    /// Specifies the origin check provided by the bridge for calls that can only be called by the bridge pallet
    type BridgeOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;

    // Ids can be defined by the runtime and passed in, perhaps from blake2b_128 hashes.
    type NativeTokenId: Get<ResourceId>;
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        ServicePaused,
        InvalidChainId,
        InvalidEthereumAddress,
        InvalidChainFee,
        InvalidFeesRecipientAccount,
        InsufficientRbalance,
        RsymbolNotMapped,
        ResourceNotMapped,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        const NativeTokenId: ResourceId = T::NativeTokenId::get();

        /// Transfers some amount of the native token to some recipient on a (whitelisted) destination chain.
        #[weight = 195_000_000]
        pub fn transfer_native(origin, amount: BalanceOf<T>, recipient: Vec<u8>, dest_id: ChainId) -> DispatchResult {
            let source = ensure_signed(origin)?;

            ensure!(!<bridge::Module<T>>::check_is_paused(), Error::<T>::ServicePaused);

            ensure!(<bridge::Module<T>>::chain_whitelisted(dest_id), Error::<T>::InvalidChainId);

            if dest_id == ETH_CHAIN_ID {
                Self::check_eth_recipient(recipient.clone())?;
            }

            let chain_fees = <bridge::Module<T>>::get_chain_fees(dest_id)
                .ok_or_else(|| Error::<T>::InvalidChainFee)?;
            let fees: BalanceOf<T> = chain_fees.saturated_into();

            let fees_recipient_account = <bridge::Module<T>>::get_fees_recipient_account()
                .ok_or_else(|| Error::<T>::InvalidFeesRecipientAccount)?;

            let total_amount = amount.saturating_add(fees);

            let bridge_id = <bridge::Module<T>>::account_id();
            T::Currency::transfer(&source, &bridge_id, total_amount.into(), AllowDeath)?;

            if fees > Zero::zero() {
                T::Currency::transfer(&bridge_id, &fees_recipient_account, fees.into(), KeepAlive)?;
            }

            let resource_id = T::NativeTokenId::get();
            <bridge::Module<T>>::transfer_fungible(source, dest_id, resource_id, recipient, U256::from(amount.saturated_into()))
        }

        /// Allows the bridge to swap native token back
        #[weight = 195_000_000]
        pub fn transfer_native_back(origin, recipient: T::AccountId, amount: BalanceOf<T>, _resource_id: ResourceId) -> DispatchResult {
            let bridge_id = T::BridgeOrigin::ensure_origin(origin)?;
            T::Currency::transfer(&bridge_id, &recipient, amount, KeepAlive)?;

            Ok(())
        }
        
        /// Transfers some amount of the native token to some recipient on a (whitelisted) destination chain.
        #[weight = 195_000_000]
        pub fn transfer_rtoken(origin, symbol: RSymbol, amount: u128, recipient: Vec<u8>, dest_id: ChainId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(!<bridge::Module<T>>::check_is_paused(), Error::<T>::ServicePaused);
            ensure!(<bridge::Module<T>>::chain_whitelisted(dest_id), Error::<T>::InvalidChainId);

            if dest_id == ETH_CHAIN_ID {
                Self::check_eth_recipient(recipient.clone())?;
            }

            let chain_fees = <bridge::Module<T>>::get_chain_fees(dest_id)
                .ok_or_else(|| Error::<T>::InvalidChainFee)?;
            let fees: BalanceOf<T> = chain_fees.saturated_into();

            let fees_recipient_account = <bridge::Module<T>>::get_fees_recipient_account()
                .ok_or_else(|| Error::<T>::InvalidFeesRecipientAccount)?;

            let op_resource = <bridge::Module<T>>::rsymbol_resource(&symbol);
            ensure!(op_resource.is_some(), Error::<T>::RsymbolNotMapped);
            let resource = op_resource.unwrap();

            let new_rbalance = T::RCurrency::free_balance(&who, symbol).checked_sub(amount)
            .ok_or(Error::<T>::InsufficientRbalance)?;
            T::RCurrency::ensure_can_withdraw(&who, symbol, amount, new_rbalance)?;

            if fees > Zero::zero() {
                T::Currency::transfer(&who, &fees_recipient_account, fees.into(), KeepAlive)?;
            }
            let bridge_id = <bridge::Module<T>>::account_id();
            T::RCurrency::transfer(&who, &bridge_id, symbol, amount)?;

            <bridge::Module<T>>::transfer_fungible(who, dest_id, resource, recipient, U256::from(amount))
        }

        /// Allows the bridge to swap rtoken back
        #[weight = 195_000_000]
        pub fn transfer_rtoken_back(origin, recipient: T::AccountId, amount: u128, resource_id: ResourceId) -> DispatchResult {
            let bridge_id = T::BridgeOrigin::ensure_origin(origin)?;
            let op_sym = <bridge::Module<T>>::resource_rsymbol(&resource_id);
            ensure!(op_sym.is_some(), Error::<T>::ResourceNotMapped);
            let sym = op_sym.unwrap();
            T::RCurrency::transfer(&bridge_id, &recipient, sym, amount)?;
            Ok(())
        }
    }
}


impl<T: Trait> Module<T> {
    pub fn check_eth_recipient(recipient: Vec<u8>) -> DispatchResult {
        ensure!(recipient.len() == 20, Error::<T>::InvalidEthereumAddress);

        Ok(())
    }
}
