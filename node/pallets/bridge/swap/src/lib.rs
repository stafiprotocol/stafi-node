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
use frame_support::traits::{Currency, ExistenceRequirement::AllowDeath, ExistenceRequirement::KeepAlive, Get};
use frame_support::{decl_error, decl_module, dispatch::DispatchResult, ensure};
use frame_system::{self as system, ensure_signed};
use sp_runtime::{traits::{Zero, Saturating}};
use sp_core::U256;
use sp_arithmetic::traits::SaturatedConversion;
use node_primitives::{ChainId, ETH_CHAIN_ID};

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

pub trait Trait: system::Trait + bridge::Trait {
    /// The currency mechanism.
    type Currency: Currency<Self::AccountId>;

    // Ids can be defined by the runtime and passed in, perhaps from blake2b_128 hashes.
    type NativeTokenId: Get<ResourceId>;
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        ServicePaused,
        InvalidChainId,
        InvalidEthereumAddress,
        InvalidChainFee,
        InvalidFeesRecipientAccount
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
    }
}


impl<T: Trait> Module<T> {
    pub fn check_eth_recipient(recipient: Vec<u8>) -> DispatchResult {
        ensure!(recipient.len() == 20, Error::<T>::InvalidEthereumAddress);

        Ok(())
    }
}
