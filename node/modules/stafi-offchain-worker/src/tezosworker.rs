//#![cfg_attr(not(feature = "std"), no_std)]

extern crate sr_std as rstd;
extern crate sr_io as runtime_io;

use support::{decl_module, decl_storage, decl_event, ensure};
use rstd::prelude::*;
use rstd::vec::Vec;
//use rstd::result::Result;
use system::{ensure_signed, ensure_root, ensure_none};
use node_primitives::{OcVerifiedData, VerifyStatus, TxHashType, HostData, XtzStakeData, Balance, AuthIndex};
use sr_primitives::traits::{SaturatedConversion, StaticLookup};
use sr_primitives::transaction_validity::{
    TransactionValidity, ValidTransaction, InvalidTransaction,
    TransactionPriority, TransactionLongevity,
};

use app_crypto::{RuntimeAppPublic};
use babe_primitives::AuthorityId;
use codec::{Encode};
use log::info;

pub mod tezos;

/// only for debug
fn debug(msg: &str) {
	info!("{}", msg);
}

pub const TEZOS_BLOCK_CONFIRMED: u8 = 3;
pub const TEZOS_BLOCK_DURATION: u64 = 60000;
pub const TEZOS_RPC_HOST: &'static [u8] = b"https://rpc.tezrpc.me";
pub const OFFCHAIN_MIN_AUTH: u8 = 1;
pub const TEZOS_WAIT_DURATION: u64 = 120000;

use system::offchain::SubmitUnsignedTransaction;

pub trait Trait: system::Trait + babe::Trait + timestamp::Trait + session::Trait + stafi_staking_storage::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Call: From<Call<Self>>;
	type SubmitTransaction: SubmitUnsignedTransaction<Self, <Self as Trait>::Call>;
}

decl_storage! {
	trait Store for Module<T: Trait> as TezosWorker {
		pub Verified get(verified): map TxHashType => (i8, u64);
		VerifiedBak get(verified_bak): Vec<(TxHashType, i8, u64)>;
		pub NodeResponse get(node_response): linked_map TxHashType => Vec<OcVerifiedData<AuthorityId>>;
		RpcHost get(rpc_host): Vec<HostData>;
		BlocksConfirmed get(blocks_confirmed): Option<u8>;
		BlockDuration get(block_duration): Option<u64>;
		WaitDuration get(wait_duration): Option<u64>;
        AuthAccount get(auth_account) : Vec<T::AccountId>;
        MinAuthorityNumber get(min_authority_number): Option<u8>;

		//for test
		StakeData get(stake_data): Vec<XtzStakeData<T::AccountId, T::Hash, Balance>>;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn set_stake_data(origin, sd: XtzStakeData<T::AccountId, T::Hash, Balance>) {
			let _who = ensure_signed(origin)?;

			let mut data: Vec<XtzStakeData<T::AccountId, T::Hash, Balance>> = <StakeData<T>>::get();
			data.push(sd);
			<StakeData<T>>::put(data);
		}

		fn set_node_response(origin, txhash: TxHashType, v_data: OcVerifiedData<AuthorityId>,
		            _signature: <AuthorityId as RuntimeAppPublic>::Signature) {

			let _who = ensure_none(origin)?;

			let mut vd: Vec<OcVerifiedData<AuthorityId>> = NodeResponse::get(&txhash).into_iter().filter(|x| x.babe_id != v_data.babe_id).collect();
			vd.push(v_data);
			NodeResponse::insert(txhash, vd);
		}

        fn add_auth_account(origin, account: <T::Lookup as StaticLookup>::Source) {
            let _who = ensure_root(origin)?;

            let new = T::Lookup::lookup(account)?;
            let mut accounts: Vec<T::AccountId> = <AuthAccount<T>>::get();
            if !accounts.contains(&new) {
                accounts.push(new);
                <AuthAccount<T>>::put(accounts);
            }
        }

        fn remove_auth_account(origin, account: <T::Lookup as StaticLookup>::Source) {
            let _who = ensure_root(origin)?;

            let new = T::Lookup::lookup(account)?;
            let accounts: Vec<T::AccountId> = <AuthAccount<T>>::get().into_iter().filter(|x| *x != new).collect();
            <AuthAccount<T>>::put(accounts);
        }

        fn set_min_authority_number(origin, min: u8) {
            let _who = ensure_root(origin)?;

            ensure!(min>0, "bad min authority number");
            MinAuthorityNumber::put(min);
        }

		fn add_rpc_host(origin, host:Vec<u8>) {
			let _who = ensure_root(origin)?;

			let host_data = HostData {
				host: host,
				weight: 1,
			};

			let mut v: Vec<HostData> = RpcHost::get();
			v.push(host_data);
			RpcHost::put(v);
		}

		fn remove_rpc_host(origin, host:Vec<u8>) {
			let _who = ensure_root(origin)?;

			let v: Vec<HostData> = RpcHost::get().into_iter().filter(|x| x.host != host).collect();
			RpcHost::put(v);
		}

		fn set_blocks_confirmed(origin, blocks:u8) {
			let _who = ensure_root(origin)?;

			BlocksConfirmed::put(blocks);
		}

		fn set_block_duration(origin, duration:u64) {
			let _who = ensure_root(origin)?;

			BlockDuration::put(duration);
		}

        fn set_wait_duration(origin, duration:u64) {
			let _who = ensure_root(origin)?;

			WaitDuration::put(duration);
		}

        fn on_finalize() {
            Self::finalizer()
        }

		fn offchain_worker(now: T::BlockNumber) {
            if runtime_io::offchain::is_validator() {
                debug("in offchain worker");
				Self::offchain(now);
			} else {
			    debug("the node isn't a validator");
			}
		}
	}
}

impl<T: Trait> Module<T> {
    fn get_babe_list() -> Vec<AuthorityId> {
        <babe::Module<T>>::authorities().iter().map(|x| x.0.clone()).collect::<Vec<AuthorityId>>()
    }

    fn get_babe_num() -> u8 {
        if <AuthAccount<T>>::get().len() == 0 {
            return <babe::Module<T>>::authorities().len() as u8;
        }

        let mut count:u8 = 0;
        Self::get_babe_list().iter().for_each(|key|
            if Self::is_auth_account(&key) {
                count += 1;
            }
        );

        count
    }

    fn is_auth_account(key: &AuthorityId) -> bool {
        if <AuthAccount<T>>::get().len() == 0 {
            return true;
        }

        <AuthAccount<T>>::get().iter().any(|x| x.encode() == key.encode())
    }

    fn authority_id() -> Option<(AuthIndex, AuthorityId)> {
        let authorities = Self::get_babe_list();
        let local_keys = AuthorityId::all();

        for (authority_index, key) in authorities.into_iter().enumerate() {
            if !Self::is_auth_account(&key) {
                debug("the key is not in auth account list");
                continue;
            }

            if local_keys.contains(&key) {
                return Some((authority_index as u32, key.clone()));
            }
        }

        None
    }

    fn get_now() -> u64 {
        return <timestamp::Module<T>>::now().saturated_into::<u64>();
    }

    fn offchain(now: T::BlockNumber) {
        let bn = now.saturated_into::<u64>();

        let host;
        if Self::rpc_host().len() == 0 {
            host = TEZOS_RPC_HOST.to_vec();
        } else {
            host = Self::rpc_host()[(bn % (Self::rpc_host().len() as u64)) as usize].host.clone();
        }

        let blocks_confirmed = Self::blocks_confirmed().unwrap_or(TEZOS_BLOCK_CONFIRMED);
        let block_duration = Self::block_duration().unwrap_or(TEZOS_BLOCK_DURATION);
        let wait_duration = Self::wait_duration().unwrap_or(TEZOS_WAIT_DURATION);
        let min_authority_number = Self::min_authority_number().unwrap_or(OFFCHAIN_MIN_AUTH);

        let babe_num = Self::get_babe_num();
        if babe_num < min_authority_number {
            debug("babe num is less than minimum");
            return;
        }

        let (authority_index, authority) = match Self::authority_id() {
            Some((ai, au)) => (ai, au),
            None => {
                debug("local storage has no authority key");
                return
            },
        };

        //for test
        /*struct MyStakeData {
            block_hash: Vec<u8>,
            tx_hash: Vec<u8>,
            stake_account: Vec<u8>,
            multi_sig_address: Vec<u8>,
            stake_amount: u128,
        };

        let mut stake_data:Vec<MyStakeData> = Vec::new();
        let xsd = MyStakeData {
            block_hash: "BKsxzJMXPxxJWRZcsgWG8AAegXNp2uUuUmMr8gzQcoEiGnNeCA6".as_bytes().to_vec(),
            tx_hash: "onv7i9LSacMXjhTdpgzmY4q6PxiZ18TZPq7KrRBRUVX7XJicSDi".as_bytes().to_vec(),
            stake_account: "tz1SYq214SCBy9naR6cvycQsYcUGpBqQAE8d".as_bytes().to_vec(),
            multi_sig_address: "tz1S4MTpEV356QcuzkjQUdyZdAy36gPwPWXa".as_bytes().to_vec(),
            stake_amount: 710391,
        };
        stake_data.push(xsd);

        let xsd = MyStakeData {
            block_hash: "BL2ReFqekwvFVFNXKkjcNdW71ProJtH54zSM2nVKvmciHvRXxm8".as_bytes().to_vec(),
            tx_hash: "op8BF61b4iFWpsVmHUYAyPCvKNCcBLGeCZyY3TnQ6NNcimBerdj".as_bytes().to_vec(),
            stake_account: "tz1S4MTpEV356QcuzkjQUdyZdAy36gPwPWXa".as_bytes().to_vec(),
            multi_sig_address: "KT1F1EfPahaN35gi6aCYB93xV5W6HgVEiZuQ".as_bytes().to_vec(),
            stake_amount: 708971,
        };
        stake_data.push(xsd);

        let xsd = MyStakeData {
            block_hash: "BMYfb5aWPpgRcMVfcD2g1HRzEvS22Camsx36PYzwEdFTJGTqJCZ".as_bytes().to_vec(),
            tx_hash: "oo24BA3DCP1KSBwDXAyz3bLVhP7btAZftQ59yR4HnX1Dpw7wQfj".as_bytes().to_vec(),
            stake_account: "tz1SWEwaPZsou41ZBG5c4HCLdT2p8x6n8nHz".as_bytes().to_vec(),
            multi_sig_address: "tz1VnRTCgVJkcTg51e2f9dtfxkMRnXeLwAv5".as_bytes().to_vec(),
            stake_amount: 614074000,
        };
        stake_data.push(xsd);

        let xsd = MyStakeData {
            block_hash: "BLZr5LJhdyjtPtZh3J92Z34B1MW4hthGpQaQRMFbwyZZ9eVrfdc".as_bytes().to_vec(),
            tx_hash: "opNBVDJddK5d6ykNDPQncKaPkbfchfPMSWrRVoSFtbX4dsnLW2Y".as_bytes().to_vec(),
            stake_account: "tz1Wy5Ph1FkD1ypUjpFpsHLXLEXDAeCRkZce".as_bytes().to_vec(),
            multi_sig_address: "KT1EvmJSSvvqMBafGqr7XD5T6PbedYssD47z".as_bytes().to_vec(),
            stake_amount: 272299,
        };
        stake_data.push(xsd);

        let xsd = MyStakeData {
            block_hash: "BMRu3JGLTMPgFLwyQBKg1Wz6oQnvFtZMQ7vk6TxLnoAwznbK5Mk".as_bytes().to_vec(),
            tx_hash: "oo1hcJRer2Hq3U6pmVXEgsYWhPvrfSKqibkbxxbYWzSHkLpdg77".as_bytes().to_vec(),
            stake_account: "tz1e4N6UZzrjoxKbsJoLnxuBy6DfZu4voiTV".as_bytes().to_vec(),
            multi_sig_address: "tz1YSGFfMeFBLaBati1AeWkMtDsrpjrkzvPx".as_bytes().to_vec(),
            stake_amount: 225000000,
        };
        stake_data.push(xsd);*/

        for xsd in <stafi_staking_storage::Module<T>>::xtz_transfer_init_data_records() {
        //for xsd in Self::stake_data() {
        //for xsd in stake_data {
            let blockhash = xsd.block_hash;
            let txhash = xsd.tx_hash;
            let from = xsd.stake_account;
            let to = xsd.multi_sig_address;
            let amount = xsd.stake_amount as u128;

            let enum_status = VerifyStatus::create(Verified::get(txhash.clone()).0);
            if enum_status == VerifyStatus::Error {
                info!("error in reading verificaiton status {:}.", core::str::from_utf8(&txhash).unwrap());
                continue;
            }

            if enum_status == VerifyStatus::Confirmed || enum_status == VerifyStatus::NotFoundTx || enum_status == VerifyStatus::Rollback || enum_status == VerifyStatus::NotFoundBlock || enum_status == VerifyStatus::TxNotMatch {
                info!("the status of tx {:} has set to {:}", core::str::from_utf8(&txhash).unwrap(), enum_status as i8);
                continue;
            }

            let last_timestamp = Verified::get(txhash.clone()).1;
            if enum_status == VerifyStatus::Verified && last_timestamp + blocks_confirmed as u64 * block_duration > Self::get_now() {
                info!("status of {:} is Verified and last_timestamp + blocks_confirmed * block_duration > now_millis.", core::str::from_utf8(&txhash).unwrap());
                continue;
            }

            let nodes_response = NodeResponse::get(txhash.clone());
            let mut node_status_set = false;
            for node_response in nodes_response {
                if authority.clone() == node_response.babe_id {
                    let status = VerifyStatus::create(node_response.status);
                    if status != VerifyStatus::UnVerified {
                        node_status_set = true;
                    }
                    break;
                }
            }

            if node_status_set {
                info!("the status of tx {:} for node {:?} has set", core::str::from_utf8(&txhash).unwrap(), &authority);
                continue;
            }

            let mut key:Vec<u8> = txhash.clone();
            key.extend(&authority.encode());
            let v = tezos::get_value(&key).unwrap_or((0 as u64).to_be_bytes().to_vec());
            let ts = tezos::vec8_to_u64(v);
            //runtime_io::print_num(ts);
            if ts > 0 {
                info!("the node {:?} has already send request [tx:{:}]", &authority, core::str::from_utf8(&txhash).unwrap());
                if Self::get_now() - ts > wait_duration {
                    tezos::set_value(&key, &(0 as u64).to_be_bytes());
                }

                continue;
            }

            tezos::set_value(&key, &Self::get_now().to_be_bytes());

            let mut new_status = VerifyStatus::Verified;
            let mut level: i64 = 0;
            let mut cur_level: i64 = 0;
            let status = tezos::request_tezos(host.clone(), blockhash, txhash.clone(), from, to, amount, &mut level);
            if status == VerifyStatus::TxOk || status == VerifyStatus::NotFoundTx {
                let _ = tezos::request_tezos(host.clone(), "head".as_bytes().to_vec(), "".as_bytes().to_vec(), "".as_bytes().to_vec(), "".as_bytes().to_vec(), 0, &mut cur_level);
            }

            if status == VerifyStatus::TxOk {
                if cur_level > 0 && cur_level - level > 0 && (cur_level - level) as u8 >= blocks_confirmed {
                    new_status = VerifyStatus::Confirmed;
                }
            } else {
                new_status = status;
            }

            if new_status == VerifyStatus::NotFoundTx {
                if enum_status == VerifyStatus::Verified && cur_level > 0 && cur_level - level > 0 && (cur_level - level) as u8 >= blocks_confirmed {
                    new_status = VerifyStatus::Rollback;
                }
            }

            let verified_data = OcVerifiedData {
                tx_hash: txhash.clone(),
                timestamp: Self::get_now(),
                status: new_status as i8,
                babe_id: authority.clone(),
                babe_num,
                authority_index,
            };

            info!("the node {:?} set response for {:}", &authority, core::str::from_utf8(&txhash).unwrap());

            let signature = match authority.clone().sign(&verified_data.encode()) {
                Some(sig) => sig,
                None => {
                    info!("{:?} signature error", &authority);
                    break;
                }
            };

            let call = Call::set_node_response(txhash.clone(), verified_data, signature);
            let ret = T::SubmitTransaction::submit_unsigned(call);
            match ret {
                Ok(_) => info!("{:?} submit ok", &authority),
                Err(_) => info!("{:?} submit failed", &authority),
            }

            tezos::set_value(&key, &(0 as u64).to_be_bytes());

            break;
        }
    }
}

#[allow(deprecated)]
impl<T: Trait> support::unsigned::ValidateUnsigned for Module<T> {
    type Call = Call<T>;

    fn validate_unsigned(call: &Self::Call) -> TransactionValidity {
        if let Call::set_node_response(txhash, verified, signature) = call {

            if *txhash != verified.tx_hash {
                return InvalidTransaction::BadProof.into();
            }

            let key = &verified.babe_id;
            let signature_valid = verified.using_encoded(|encoded_verified| {
                key.verify(&encoded_verified, &signature)
            });
            if !signature_valid {
                return InvalidTransaction::BadProof.into();
            }

            Ok(ValidTransaction {
                priority: TransactionPriority::max_value(),
                requires: vec![],
                provides: vec![txhash.to_vec()],
                longevity: TransactionLongevity::max_value(),
                propagate: true,
            })
        } else {
            InvalidTransaction::Call.into()
        }

    }
}
impl<T: Trait> Module<T> {
    fn finalizer() {
        //let bn = <system::Module<T>>::block_number().saturated_into::<u64>();
        //runtime_io::print_num(bn);

        let mut response_list: Vec<(TxHashType, Vec<OcVerifiedData<AuthorityId>>)> = Vec::new();
        for (k, v) in NodeResponse::enumerate() {
            let txhash = k;
            let status = VerifyStatus::create(Verified::get(txhash.clone()).0);
            if status != VerifyStatus::Confirmed && status != VerifyStatus::NotFoundTx && status != VerifyStatus::Rollback && status != VerifyStatus::NotFoundBlock {
                response_list.push((txhash, v));
                if response_list.len() > 100 {
                    break
                }
            }
        }

        for (k, v) in response_list {
            let mut ts = 0;
            let txhash = k;

            let new_status = Self::get_new_status(v.clone(), &mut ts);
            if new_status != VerifyStatus::UnVerified {
                let status = new_status as i8;
                let (s, t) = Verified::get(txhash.clone());
                if s != status && t != ts {
                    Verified::insert(txhash.clone(), (status, ts));
                    let mut vb = VerifiedBak::get();
                    vb.push((txhash, status, ts));
                    VerifiedBak::put(vb);
                }
            }
        }
    }

    fn get_new_status(vd: Vec<OcVerifiedData<AuthorityId>>, ts: &mut u64) -> VerifyStatus {
        let mut verified_counter = 0;
        let mut confirmed_counter = 0;
        let mut notfoundtx_counter = 0;
        let mut rollback_counter = 0;
        let mut notfoundblock_counter = 0;
        let mut notresponse_counter = 0;
        let mut txnotmatch_counter = 0;

        let mut babe_num = 0;
        for v in vd {
            *ts = v.timestamp;
            babe_num = v.babe_num;

            match VerifyStatus::create(v.status) {
                VerifyStatus::Verified => verified_counter = verified_counter + 1,
                VerifyStatus::Confirmed => confirmed_counter = confirmed_counter + 1,
                VerifyStatus::NotFoundTx => notfoundtx_counter = notfoundtx_counter + 1,
                VerifyStatus::Rollback => rollback_counter = rollback_counter + 1,
                VerifyStatus::NotFoundBlock => notfoundblock_counter = notfoundblock_counter + 1,
                VerifyStatus::NotResponse => notresponse_counter = notresponse_counter + 1,
                VerifyStatus::TxNotMatch => txnotmatch_counter = txnotmatch_counter + 1,
                _ => (),
            }
        }

        let mut new_status = VerifyStatus::UnVerified;

        if verified_counter >= (babe_num + 2) / 2 {
            new_status = VerifyStatus::Verified;
        } else if confirmed_counter >= (babe_num + 2) / 2 {
            new_status = VerifyStatus::Confirmed;
        } else if notfoundtx_counter >= (babe_num + 2) / 2 {
            new_status = VerifyStatus::NotFoundTx;
        } else if rollback_counter >= (babe_num + 2) / 2 {
            new_status = VerifyStatus::Rollback;
        } else if notfoundblock_counter >= (babe_num + 2) / 2 {
            new_status = VerifyStatus::NotFoundBlock;
        } else if notresponse_counter >= (babe_num + 2) / 2 {
            new_status = VerifyStatus::NotResponse;
        } else if txnotmatch_counter >= (babe_num + 2) / 2 {
            new_status = VerifyStatus::TxNotMatch;
        }

        return new_status;
    }

    pub fn remove_verified(txhash: TxHashType) {
        if Verified::exists(&txhash) {
            Verified::remove(&txhash);
            NodeResponse::remove(&txhash);
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        <T as system::Trait>::AccountId,
    {
        SetAuthority(AccountId),
    }
);