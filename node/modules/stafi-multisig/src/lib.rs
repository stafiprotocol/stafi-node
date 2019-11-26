// Copyright 2018 Stafi Protocol, Inc.
// This file is part of Stafi.

// Stafi is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Stafi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stafi.  If not, see <http://www.gnu.org/licenses/>.

//! this module is for multisig, but now this is just for genesis multisig addr, not open for public.

#![cfg_attr(not(feature = "std"), no_std)]
// for encode/decode
// Needed for deriving `Serialize` and `Deserialize` for various types.
// We only implement the serde traits for std builds - they're unneeded
// in the wasm runtime.
#[cfg(feature = "std")]
#[macro_use]
extern crate serde_derive;

// Needed for deriving `Encode` and `Decode` for `RawEvent`.
#[macro_use]
extern crate codec;
// extern crate parity_codec_derive;
// extern crate parity_scale_codec as codec;

// for substrate
// Needed for the set of mock primitives used in our tests.
#[cfg(feature = "std")]
extern crate substrate_primitives;

// for substrate runtime
// map!, vec! marco.
extern crate sr_std as rstd;
// Needed for tests (`with_externalities`).
#[cfg(feature = "std")]
extern crate sr_io as runtime_io;
extern crate sr_primitives as runtime_primitives;
extern crate substrate_primitives as primitives;

// for substrate runtime module lib
// Needed for type-safe access to storage DB.
#[macro_use]
extern crate frame_support as runtime_support;
extern crate pallet_balances as balances;
extern crate frame_system as system;


// use system::GenesisConfig as BalancesConfig;
use codec::{Codec, Decode, Encode};
use rstd::marker::PhantomData;
use rstd::prelude::*;
use rstd::result::Result as StdResult;
use runtime_primitives::traits::Hash;
use runtime_support::dispatch::Result;
use runtime_support::{traits::{Currency, ExistenceRequirement}};
use substrate_primitives::crypto::{UncheckedFrom, UncheckedInto};

use system::ensure_signed;

pub mod transaction;
pub use transaction::{Transaction, TransactionType, TransferT};

pub mod multisigaddr;

pub trait MultiSigFor<AccountId: Sized, Hash: Sized> {
    /// generate multisig addr for a accountid
    fn multi_sig_addr_for(who: &AccountId) -> AccountId;

    fn multi_sig_id_for(who: &AccountId, addr: &AccountId, data: &[u8]) -> Hash;
}

pub trait Trait: balances::Trait {
    type MultiSig: MultiSigFor<Self::AccountId, Self::Hash>;
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
        <T as system::Trait>::Hash,
        <T as balances::Trait>::Balance
    {
        /// deploy a multisig and get address, who deploy, deploy addr, owners num, required num, value
        DeployMultiSig(AccountId, AccountId, u32, u32, Balance),
        /// exec. who, addr, multisigid, type
        ExecMultiSig(AccountId, AccountId, Hash,TransactionType),
        /// confirm. who, addr, multisigid, yet_needed, index, ret
        Confirm(AccountId, AccountId, Hash, u32, u32, bool),
        /// confirm. addr, multisigid
        ConfirmFinish(AccountId, Hash),

        /// remove multisig id for a multisig addr
        RemoveMultiSigIdFor(AccountId, Hash),

        /// set deploy fee, by Root
        SetDeployFee(Balance),
        /// set exec fee, by Root
        SetExecFee(Balance),
        /// set confirm fee, by Root
        SetConfirmFee(Balance),
    }
);

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deploy(origin, owners: Vec<(T::AccountId, bool)>, required_num: u32, value: T::Balance) -> Result {
            let from = ensure_signed(origin)?;
            Self::deploy_for(&from, owners, required_num, value)
        }
        
        fn is_owner_for(origin, multi_sig_addr: T::AccountId) -> Result {
            let from = ensure_signed(origin)?;
            Self::is_owner(&from, &multi_sig_addr, false).map(|_| ())
        }

        fn transfer(origin, multi_sig_addr: T::AccountId, tx_type: TransactionType, target: T::AccountId, balance: T::Balance) -> Result {
            let data = TransferT::<T> { to: target, value: balance }.encode();
            Self::execute(origin, multi_sig_addr, tx_type, data)
        }

        fn execute(origin, multi_sig_addr: T::AccountId, tx_type: TransactionType, data: Vec<u8>) -> Result {
            let from: T::AccountId = ensure_signed(origin)?;
            Self::only_owner(&from, &multi_sig_addr, true)?;

            if let Some(req_num) = Self::required_num_for(&multi_sig_addr) {
                Self::tx_check(tx_type, data.clone())?;

                let t = Transaction::new(tx_type, data.clone());

                let multi_sig_id: T::Hash;
                if req_num == 1 {
                    // real exec
                    Self::exec_tx(&multi_sig_addr, t.clone())?;
                    multi_sig_id = Default::default();
                } else {
                    // determine multi sig id
                    multi_sig_id = T::MultiSig::multi_sig_id_for(&from, &multi_sig_addr, &data);
                    <TransactionFor<T>>::insert((multi_sig_addr.clone(), multi_sig_id), t.clone());
                    // confirm for self
                    let origin = system::RawOrigin::Signed(from.clone()).into();
                    Self::confirm(origin, multi_sig_addr.clone(), multi_sig_id)?;
                }
                Self::deposit_event(RawEvent::ExecMultiSig(
                    from.clone(),
                    multi_sig_addr.clone(),
                    multi_sig_id,
                    tx_type,
                ));
                return Ok(());
            } else {
                return Err("the multi sig addr not exist");
            }
        }

        fn confirm(origin, multi_sig_addr: T::AccountId, multi_sig_id: T::Hash) -> Result {
            let from = ensure_signed(origin)?;
            let ret = Self::only_many_owner(&from, &multi_sig_addr, multi_sig_id)?;

            if ret == true {
                let ret = Self::transaction_for((multi_sig_addr.clone(), multi_sig_id));
                if let None = ret {
                    return Err("no pending tx for this addr and id or it has finished");
                }
                let t = ret.unwrap();
                // del tx first and execute later
                Self::remove_tx_for(&multi_sig_addr, multi_sig_id);
                // real exec
                Self::exec_tx(&multi_sig_addr, t.clone())?;
                // log event
            } else {
                // log event
                Self::deposit_event(RawEvent::ConfirmFinish(
                    multi_sig_addr.clone(),
                    multi_sig_id,
                ));
            }
            return Ok(());
        }

        fn remove_multi_sig_for(origin, multi_sig_addr: T::AccountId, multi_sig_id: T::Hash) -> Result {
            let from: T::AccountId = ensure_signed(origin)?;
            Self::only_owner(&from, &multi_sig_addr, true)?;

            Self::remove_multi_sig_id(&multi_sig_addr, multi_sig_id);
            Ok(())
        }
    }
}

// struct for the status of a pending operation.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(PartialEq, Eq, Clone, Encode, Decode, Default, Copy)]
pub struct PendingState {
    yet_needed: u32,
    owners_done: u32,
    index: u32,
}

const MAX_OWNERS: u32 = 32;

decl_storage! {
    trait Store for Module<T: Trait> as MultiSig {
        /// multisig deployer for this multisig addr
        pub MultiSigOwnerFor get(multi_sig_owner_for): map T::AccountId => Option<T::AccountId>;
        /// multisig owners for this multisig addr
        pub MultiSigListOwnerFor get(multi_sig_list_owner_for): map T::AccountId => Option<Vec<(T::AccountId, bool)>>;

        /// required num for this multisig addr
        pub RequiredNumFor get(required_num_for): map T::AccountId => Option<u32>;
        /// all owners count for this multisig addr
        pub NumOwnerFor get(num_owner_for): map T::AccountId => Option<u32>;

        /// pending state list for a multisig addr, can find out the index for a pending state
        pub PendingListLenFor get(pending_list_len_for): map T::AccountId => u32;
        pub PendingListItemFor get(pending_list_item_for): map (T::AccountId, u32) => Option<T::Hash>;
        /// pending state for a multisig addr
        pub PendingStateFor get(pending_state_for): map (T::AccountId, T::Hash) => PendingState;
        /// transaction behavior for a pending state
        pub TransactionFor get(transaction_for): map (T::AccountId, T::Hash) => Option<Transaction>;

        // for deployer
        /// the deployed multisig addr for a account
        pub MultiSigListLenFor get(multi_sig_list_len_for): map T::AccountId => u32;
        pub MultiSigListItemFor get(multi_sig_list_item_for): map (T::AccountId, u32) => Option<T::AccountId>;

        // for fee
        pub DeployFee get(deploy_fee) config(): T::Balance;
        pub ExecFee get(exec_fee) config(): T::Balance;
        pub ConfirmFee get(confirm_fee) config(): T::Balance;
    }
    // TODO: implement
    add_extra_genesis {
        config(genesis_multi_sig): Vec<(T::AccountId, Vec<(T::AccountId, bool)>, u32, T::Balance)>;
        // config(balances_config): BalancesConfig<T>;
        // build(|storage: & mut (runtime_primitives::StorageOverlay, runtime_primitives::ChildrenStorageOverlay), config: &GenesisConfig<T>| {
        //     use runtime_io::{with_externalities, with_storage};
        //     use substrate_primitives::Blake2Hasher;
        //     with_storage(storage, || {
        //         let src_r = BalancesConfigCopy::create_from_src(&config.balances_config).src().build_storage().unwrap();
        //         let mut tmp_storage: runtime_io::TestExternalities<Blake2Hasher> = src_r.into();
        //         let genesis = config.genesis_multi_sig.clone();
        //         with_externalities(&mut tmp_storage, || {
        //             for (deployer, owners, required_num, value) in genesis {
        //                 if let Err(e) = <Module<T>>::deploy_for(&deployer, owners, required_num, value) {
        //                     panic!(e)
        //                 }
        //                 // <system::Module<T>>::inc_account_nonce(&deployer);
        //             }
        //         });
        //     });
        // });
    }
}

//impl trait
/// Simple MultiSigIdFor struct
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, PartialEq)]
pub struct SimpleMultiSigIdFor<T: Trait>(PhantomData<T>);

impl<T: Trait> MultiSigFor<T::AccountId, T::Hash> for SimpleMultiSigIdFor<T>
where
    T::AccountId: UncheckedFrom<T::Hash>,
{
    fn multi_sig_addr_for(who: &T::AccountId) -> T::AccountId {
        let mut buf = Vec::<u8>::new();
        buf.extend_from_slice(&who.encode());
        buf.extend_from_slice(&<system::Module<T>>::account_nonce(who).encode());
        buf.extend_from_slice(&<Module<T>>::multi_sig_list_len_for(who).encode()); // in case same nonce in genesis
        T::Hashing::hash(&buf[..]).unchecked_into()
    }

    fn multi_sig_id_for(who: &T::AccountId, addr: &T::AccountId, data: &[u8]) -> T::Hash {
        let mut buf = Vec::<u8>::new();
        buf.extend_from_slice(&who.encode());
        buf.extend_from_slice(&addr.encode());
        buf.extend_from_slice(&<system::Module<T>>::account_nonce(who).encode());
        buf.extend_from_slice(data);
        T::Hashing::hash(&buf[..])
    }
}

impl<T: Trait> Module<T> {
    // event
    /// Deposit one of this module's events.
    fn deposit_event(event: Event<T>) {
        <system::Module<T>>::deposit_event(<T as Trait>::Event::from(event).into());
    }
}

impl<T: Trait> Module<T> {
    //    fn remove_multi_sig_addr(multi_sig_addr: &T::AccountId) {
    //        <PendingStateFor<T>>::remove_prefix(multi_sig_addr.clone());
    //        <TransactionFor<T>>::remove_prefix(multi_sig_addr.clone());
    //        <MultiSigOwnerFor<T>>::remove(multi_sig_addr);
    //        <MultiSigListOwnerFor<T>>::remove(multi_sig_addr);
    //    }

    fn remove_multi_sig_id(multi_sig_addr: &T::AccountId, multi_sig_id: T::Hash) {
        Self::remove_pending_for(multi_sig_addr, multi_sig_id);
        Self::remove_tx_for(multi_sig_addr, multi_sig_id);
        // event
        Self::deposit_event(RawEvent::RemoveMultiSigIdFor(
            multi_sig_addr.clone(),
            multi_sig_id,
        ));
    }

    fn remove_pending_for(multi_sig_addr: &T::AccountId, multi_sig_id: T::Hash) {
        let pending = <PendingStateFor<T>>::take((multi_sig_addr.clone(), multi_sig_id));
        <PendingListItemFor<T>>::remove((multi_sig_addr.clone(), pending.index));
    }

    fn remove_tx_for(multi_sig_addr: &T::AccountId, multi_sig_id: T::Hash) {
        <TransactionFor<T>>::remove((multi_sig_addr.clone(), multi_sig_id));
    }

    fn is_owner(
        who: &T::AccountId,
        addr: &T::AccountId,
        required: bool,
    ) -> StdResult<u32, &'static str> {
        if let Some(list_owner) = Self::multi_sig_list_owner_for(addr) {
            for (index, (id, req)) in list_owner.iter().enumerate() {
                if id == who {
                    if required && (*req == false) {
                        return Err("it's the owner but not required owner");
                    } else {
                        return Ok(index as u32);
                    }
                }
            }
        } else {
            return Err("the multi sig addr not exist");
        }
        Err("it's not the owner")
    }

    fn confirm_and_check(
        who: &T::AccountId,
        multi_sig_addr: &T::AccountId,
        multi_sig_id: T::Hash,
    ) -> StdResult<bool, &'static str> {
        let index = Self::is_owner(who, multi_sig_addr, false)?;

        let key = (multi_sig_addr.clone(), multi_sig_id);
        if let None = Self::transaction_for(&key) {
            return Err("no pending tx for this addr and id or it has finished");
        }

        let mut pending: PendingState = Self::pending_state_for(&key);
        if pending.yet_needed == 0 {
            pending.yet_needed = Self::required_num_for(multi_sig_addr).unwrap_or_default();
            pending.owners_done = 0;

            pending.index = Self::pending_list_len_for(multi_sig_addr);
            <PendingListLenFor<T>>::insert(multi_sig_addr.clone(), pending.index + 1);
            <PendingListItemFor<T>>::insert((multi_sig_addr.clone(), pending.index), multi_sig_id);
        }

        let ret: bool;

        let index_bit = 1 << index; // not longer then index_bit's type
        if pending.owners_done & index_bit == 0 {
            if pending.yet_needed <= 1 {
                // enough confirmations
                Self::remove_pending_for(multi_sig_addr, multi_sig_id);
                ret = true;
            } else {
                pending.yet_needed -= 1;
                pending.owners_done |= index_bit;
                // update pending state
                <PendingStateFor<T>>::insert((multi_sig_addr.clone(), multi_sig_id), pending);
                ret = false;
            }
            Self::deposit_event(RawEvent::Confirm(
                who.clone(),
                multi_sig_addr.clone(),
                multi_sig_id,
                pending.yet_needed,
                pending.index,
                ret,
            ));
        } else {
            return Err("this account has confirmed for this multi sig addr and id");
        }
        Ok(ret)
    }

    // func alias
    fn only_owner(
        who: &T::AccountId,
        addr: &T::AccountId,
        required: bool,
    ) -> StdResult<u32, &'static str> {
        Self::is_owner(who, addr, required)
    }
    fn only_many_owner(
        who: &T::AccountId,
        multi_sig_addr: &T::AccountId,
        multi_sig_id: T::Hash,
    ) -> StdResult<bool, &'static str> {
        Self::confirm_and_check(who, multi_sig_addr, multi_sig_id)
    }
}

impl<T: Trait> Module<T> {
    fn deploy_for(
        account_id: &T::AccountId,
        owners: Vec<(T::AccountId, bool)>,
        required_num: u32,
        value: T::Balance,
    ) -> Result {
        let mut owners = owners;
        if let None = owners.iter().find(|(a, _)| {
            if *a == *account_id {
                return true;
            } else {
                return false;
            }
        }) {
            owners.push((account_id.clone(), true));
        }

        let owners_len = owners.len() as u32;
        if owners_len > MAX_OWNERS {
            return Err("total owners can't more than `MAX_OWNERS`");
        }

        if owners_len < required_num {
            return Err("owners count can't less than required num");
        }

        let multi_addr: T::AccountId = T::MultiSig::multi_sig_addr_for(account_id);
        let origin: T::AccountId = account_id.clone();

        <balances::Module<T> as Currency<_>>::transfer(&origin, &multi_addr, value, ExistenceRequirement::AllowDeath)?;

        // 1
        let len = Self::multi_sig_list_len_for(account_id);
        <MultiSigListItemFor<T>>::insert((account_id.clone(), len), multi_addr.clone());
        // length inc
        <MultiSigListLenFor<T>>::insert(account_id.clone(), len + 1);
        // 2
        <MultiSigOwnerFor<T>>::insert(multi_addr.clone(), account_id.clone());
        // 3
        <MultiSigListOwnerFor<T>>::insert(multi_addr.clone(), owners.clone());
        // 4
        <RequiredNumFor<T>>::insert(multi_addr.clone(), required_num);
        <NumOwnerFor<T>>::insert(multi_addr.clone(), owners_len);
        // event
        Self::deposit_event(RawEvent::DeployMultiSig(
            account_id.clone(),
            multi_addr.clone(),
            owners_len,
            required_num,
            value,
        ));
        Ok(())
    }
}

impl<T: Trait> Module<T> {
    // public call for fee
    fn set_deploy_fee(value: T::Balance) -> Result {
        <DeployFee<T>>::put(value);
        Self::deposit_event(RawEvent::SetDeployFee(value));
        Ok(())
    }
    fn set_exec_fee(value: T::Balance) -> Result {
        <ExecFee<T>>::put(value);
        Self::deposit_event(RawEvent::SetExecFee(value));
        Ok(())
    }
    fn set_confirm_fee(value: T::Balance) -> Result {
        <ConfirmFee<T>>::put(value);
        Self::deposit_event(RawEvent::SetConfirmFee(value));
        Ok(())
    }
}

impl<T: Trait> Module<T> {
    fn tx_check(tx_type: TransactionType, data: Vec<u8>) -> Result {
        match tx_type {
            TransactionType::TransferStafi => {
                if let Err(_) = TransferT::<T>::decode(&mut data.as_slice()) {
                    return Err("parse err for this tx data");
                }
                Ok(())
            }
        }
    }

    fn exec_tx(addr: &T::AccountId, tx: Transaction) -> Result {
        let data: Vec<u8> = tx.data();
        match tx.tx_type() {
            TransactionType::TransferStafi => {
                let t = TransferT::<T>::decode(&mut data.as_slice()).unwrap();
                // let origin = system::RawOrigin::Signed(addr.clone()).into();
                // let to: balances::Address<T> = balances::address::Address::Id(t.to);
                // let origin = system::RawOrigin::Signed(addr.clone()).into();
                // <balances::Module<T>>::transfer(origin, &t.to, t.value)?;
                <balances::Module<T> as Currency<_>>::transfer(&addr, &t.to, t.value, ExistenceRequirement::AllowDeath)?;
                Ok(())
            }
        }
    }
}

// #[derive(Encode, Decode)]
// pub struct BalancesConfigCopy<T: Trait>(BalancesConfig<T>);

// impl<T: Trait> BalancesConfigCopy<T> {
//     pub fn create_from_src(config: &BalancesConfig<T>) -> BalancesConfigCopy<T> {
//         BalancesConfigCopy(BalancesConfig::<T> {
//             balances: config.balances.clone(),
//             vesting: config.vesting.clone(),
//             // transaction_base_fee: config.transaction_base_fee.clone(),
//             // transaction_byte_fee: config.transaction_byte_fee.clone(),
//             // transfer_fee: config.transfer_fee.clone(),
//             // creation_fee: config.creation_fee.clone(),
//             // reclaim_rebate: config.reclaim_rebate.clone(),
//             // existential_deposit: config.existential_deposit.clone(),
//         })
//     }
//     pub fn src(self) -> BalancesConfig<T> {
//         self.0
//     }
// }
