//#![cfg_attr(not(feature = "std"), no_std)]

extern crate sr_std as rstd;
extern crate srml_support as support;
extern crate srml_system as system;
extern crate srml_babe as babe;
extern crate srml_timestamp as timestamp;

use support::{decl_module, decl_storage, decl_event};
use rstd::prelude::*;
use rstd::vec::Vec;
use rstd::result::Result;
use system::{ensure_none, ensure_signed, ensure_root};
use stafi_primitives::{VerifiedData, VerifyStatus, TxHashType, BabeIdType, HostData, XtzStakeData, Balance};
use codec::Decode;
use sr_primitives::traits::{Member, SaturatedConversion};

use app_crypto::{KeyTypeId, RuntimeAppPublic};
use system::offchain::SubmitSignedTransaction;

pub mod tezos;

/// only for debug
fn debug(msg: &str) {
	// let msg = format!("\x1b[34m{}", msg);
	sr_io::print_utf8(msg.as_bytes());
}

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"orin");
pub const TEZOS_BLOCK_CONFIRMED: u8 = 3;
pub const TEZOS_BLOCK_DURATION: u64 = 60000;
pub const TEZOS_RPC_HOST: &'static [u8] = b"https://rpc.tezrpc.me";

pub mod sr25519 {
	mod app_sr25519 {
		use app_crypto::{app_crypto, sr25519};
		app_crypto!(sr25519, super::super::KEY_TYPE);

		impl From<Signature> for sr_primitives::AnySignature {
			fn from(sig: Signature) -> Self {
				sr25519::Signature::from(sig).into()
			}
		}
	}

	/// An oracle signature using sr25519 as its crypto.
	// pub type AuthoritySignature = app_sr25519::Signature;

	/// An oracle identifier using sr25519 as its crypto.
	pub type AuthorityId = app_sr25519::Public;
}

pub trait Trait: system::Trait + babe::Trait + timestamp::Trait{
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	/// A dispatchable call type.
	type Call: From<Call<Self>>;
	/// A transaction submitter.
	//type SubmitTransaction: SubmitUnsignedTransaction<Self, <Self as Trait>::Call>;
	type SubmitTransaction: SubmitSignedTransaction<Self, <Self as Trait>::Call>;
	/// The local keytype
	type KeyType: RuntimeAppPublic + From<Self::AccountId> + Into<Self::AccountId> + Clone;
}

decl_storage! {
	trait Store for Module<T: Trait> as TezosWorker {
		pub Verified get(verified): map TxHashType => (i8, u64);
		VerifiedBak get(verified_bak): Vec<(TxHashType, i8, u64)>;
		pub NodeResponse get(node_response): linked_map TxHashType => Vec<VerifiedData>;
		RpcHost get(rpc_host): Vec<HostData>;
		BlocksConfirmed get(blocks_confirmed): u8;
		BlockDuration get(block_duration): u64;
		pub SendRequest get(send_request): map (TxHashType, BabeIdType) => u64;

		//for test
		StakeData get(stake_data): Vec<XtzStakeData<T::AccountId, T::Hash, Balance>>;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn set_stake_data(origin, sd: XtzStakeData<T::AccountId, T::Hash, Balance>) {
			let _who = ensure_signed(origin)?;

			let mut data: Vec<XtzStakeData<T::AccountId, T::Hash, Balance>> = Vec::new();
			data.push(sd);
			<StakeData<T>>::put(data);
		}

		fn set_node_response(origin, txhash: TxHashType, babe_id: BabeIdType, v_data: VerifiedData) {
			let _who = ensure_signed(origin)?;

			let mut vd: Vec<VerifiedData> = NodeResponse::get(&txhash).into_iter().filter(|x| x.babe_id != babe_id).collect();
			vd.push(v_data);
			NodeResponse::insert(txhash, vd);
		}

		fn add_rpc_host(origin, host:Vec<u8>) {
			let _who = ensure_signed(origin)?;

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

		fn on_finalize() {
		    //return;

			let mut response_list: Vec<(TxHashType, Vec<VerifiedData>)> = Vec::new();
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

				let new_status = get_new_status(v.clone(), &mut ts);
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

		fn offchain_worker(now: T::BlockNumber) {
			debug("in offchain worker");
			Self::offchain(now);
            debug("end of offchain worker");
			/*if let Some(key) = Self::authority_id() {
                debug("sign...");

                //let call = Call::add_rpc_host([0x33,0x34].to_vec());
		        //let _ = T::SubmitTransaction::sign_and_submit(call, key.clone().into());
            }*/

		}
	}
}

impl<T: Trait> Module<T> {
    fn get_babe_num() -> usize {
        return <babe::Module<T>>::authorities().len();
    }

    fn get_now() -> u64 {
        return <timestamp::Module<T>>::now().saturated_into::<u64>();
    }

	fn offchain(now: T::BlockNumber) {
        let bn = <system::Module<T>>::block_number().saturated_into::<u64>();

        let host;
        if Self::rpc_host().len() == 0 {
            host = TEZOS_RPC_HOST.to_vec();
        } else {
            host = Self::rpc_host()[(bn % (Self::rpc_host().len() as u64)) as usize].host.clone();
        }

        let mut blocks_confirmed = Self::blocks_confirmed();
        if blocks_confirmed == 0 {
            blocks_confirmed = TEZOS_BLOCK_CONFIRMED;
        }

        let mut block_duration = Self::block_duration();
        if block_duration == 0 {
            block_duration = TEZOS_BLOCK_DURATION;
        }

        let key = Self::authority_id();
        if let None = key  {
            debug("no authority_id");
            return
        }

        let babe_num = Self::get_babe_num();

        let babe_id: Vec<u8> = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".as_bytes().to_vec();

        //for xsd in Self::stake_data() {
        for i in 0..1 {
            let blockhash: Vec<u8> = "BKsxzJMXPxxJWRZcsgWG8AAegXNp2uUuUmMr8gzQcoEiGnNeCA6".as_bytes().to_vec();
            let txhash: Vec<u8> = "onv7i9LSacMXjhTdpgzmY4q6PxiZ18TZPq7KrRBRUVX7XJicSDi".as_bytes().to_vec();
            //let blockhash = xsd.block_hash;
            //let txhash = xsd.tx_hash;

            let ts = SendRequest::get((txhash.clone(), babe_id.clone()));
            if ts > 0 {
                debug("already send request");
                continue;
            }

            let from = "tz1SYq214SCBy9naR6cvycQsYcUGpBqQAE8d".as_bytes().to_vec();
            let to = "tz1S4MTpEV356QcuzkjQUdyZdAy36gPwPWXa".as_bytes().to_vec();
            let amount = 710391;
            //let from = xsd.stake_account;
            //let to = xsd.multi_sig_address;
            //let amount = xsd.stake_amount as u128;

            let enum_status = VerifyStatus::create(Verified::get(txhash.clone()).0);
            if enum_status == VerifyStatus::Error {
                debug("error in reading verificaiton status.");
                continue;
            }

            if enum_status == VerifyStatus::Confirmed || enum_status == VerifyStatus::NotFoundTx || enum_status == VerifyStatus::Rollback || enum_status == VerifyStatus::NotFoundBlock || enum_status == VerifyStatus::TxNotMatch {
                //sr_io::print_utf8(&format!("{:}'s status is {:}.", txhash, enum_status as i8).into_bytes());
                debug("tx status set");
                continue;
            }

            let last_timestamp = Verified::get(txhash.clone()).1;
            let now_millis = Self::get_now();
            if enum_status == VerifyStatus::Verified && last_timestamp + blocks_confirmed as u64 * block_duration > now_millis {
                debug("status is Verified and last_timestamp + blocks_confirmed * block_duration > now_millis.");
                continue;
            }

            let nodes_response = NodeResponse::get(txhash.clone());
            let mut node_status_set = false;
            for node_response in nodes_response {
                if babe_id == node_response.babe_id {
                    let status = VerifyStatus::create(node_response.status);
                    if status != VerifyStatus::UnVerified {
                        node_status_set = true;
                    } /*else if status == VerifyStatus::WaitForReply {
                        //TODO: how long time passed ?
                        node_status_set = true;
                    } */
                    break;
                }
            }

            if node_status_set {
                debug("node_status_set");
                continue;
            }

            //
            debug("run here");
            sr_io::print_utf8(&txhash.clone());
            sr_io::print_utf8(&babe_id.clone());
            sr_io::print_num(Self::get_now());
            SendRequest::insert((txhash.clone(), babe_id.clone()), Self::get_now());

            let mut new_status = VerifyStatus::Verified;
            let mut level:i64 = 0;
            let mut cur_level:i64 = 0;
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
                if enum_status == VerifyStatus::Verified && cur_level > 0 && cur_level - level > 0 && (cur_level - level) as u8 >= blocks_confirmed  {
                    new_status = VerifyStatus::Rollback;
                }
            }

            debug("run here1");

            let verified_data = VerifiedData {
                tx_hash: txhash.clone(),
                timestamp: Self::get_now(),
                status: new_status as i8,
                babe_id: babe_id.clone(),
                babe_num: babe_num as u8,
            };


            debug("sign...");

            let call = Call::set_node_response(txhash.clone(), babe_id.clone(), verified_data);
            let _ = T::SubmitTransaction::sign_and_submit(call, key.unwrap().clone().into());

            //SendRequest::remove((txhash, babe_id.clone()));

            break;
        }

	}

	pub fn remove_verified(txhash: TxHashType) {
        if Verified::exists(&txhash) {
            Verified::remove(&txhash);
            NodeResponse::remove(&txhash);
        }
	}

	fn authority_id() -> Option<T::AccountId> {
		let local_keys = T::KeyType::all().iter().map(
			|i| (*i).clone().into()
		).collect::<Vec<T::AccountId>>();

		if local_keys.len() > 0 {
			Some(local_keys[0].clone())
		} else {
			None
		}
	}
}

fn get_new_status(vd: Vec<VerifiedData>, ts: &mut u64) -> VerifyStatus {
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

    if verified_counter >= (babe_num + 2)/2 {
        new_status = VerifyStatus::Verified;
    } else if confirmed_counter >= (babe_num + 2)/2 {
        new_status = VerifyStatus::Confirmed;
    } else if notfoundtx_counter >= (babe_num + 2)/2 {
        new_status = VerifyStatus::NotFoundTx;
    } else if rollback_counter >= (babe_num + 2)/2 {
        new_status = VerifyStatus::Rollback;
    } else if notfoundblock_counter >= (babe_num + 2)/2 {
        new_status = VerifyStatus::NotFoundBlock;
    } else if notresponse_counter >= (babe_num + 2)/2 {
        new_status = VerifyStatus::NotResponse;
    } else if txnotmatch_counter >= (babe_num + 2)/2 {
        new_status = VerifyStatus::TxNotMatch;
    }

    return new_status;
}

decl_event!(
    pub enum Event<T>
    where
        <T as system::Trait>::AccountId,
    {
        SetAuthority(AccountId),
    }
);