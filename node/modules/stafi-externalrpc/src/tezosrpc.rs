//#![cfg_attr(not(feature = "std"), no_std)]

extern crate sr_std as rstd;
extern crate srml_support as support;
extern crate srml_system as system;

use support::{decl_module, decl_storage};
use rstd::prelude::*;
use system::{ensure_none, ensure_signed};
use inherents::{RuntimeString, InherentIdentifier, ProvideInherent, MakeFatalError, InherentData};

use stafi_primitives::{XtzTransferData, VerifiedData, TxHashType};
use codec::{Encode, Decode};

pub mod tezos;
pub use tezos::INHERENT_IDENTIFIER;
#[cfg(feature = "std")]
pub use tezos::InherentDataProvider;

pub use tezos::InherentType;
pub use tezos::TXHASH_LEN;

pub const ZERO_HASH: &'static [u8] = b"000000000000000000000000000000000000000000000000000";

pub trait Trait: system::Trait { }

decl_storage! {
	trait Store for Module<T: Trait> as TezosRpc {
		XtzTransferDataVec get(xtz_transfter_data_vec): Vec<XtzTransferData<T::AccountId, T::Hash>>;
		pub Verified get(verified): map TxHashType => i8;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn set_xtz_transfer_data(origin, xtd: XtzTransferData<T::AccountId, T::Hash>) {
			let _who = ensure_signed(origin)?;
			let mut v:Vec<XtzTransferData<T::AccountId, T::Hash>> = Vec::new();
			v.push(xtd); 
			<XtzTransferDataVec<T>>::put(v);	
		}

		fn set_verified(origin, txhash: TxHashType, verified: i8) {
			let _who = ensure_none(origin)?;
			
			if txhash != ZERO_HASH.to_vec() {
				Verified::insert(txhash, verified);
			}
		}

		/*fn set_verified1(origin, txhash: TxHashType, verified: i8) {
			let _who = ensure_signed(origin)?;
			
			<Self as Store>::Verified::insert(txhash, verified);
		}*/
	}
}

impl<T: Trait> Module<T> {}

fn extract_inherent_data(data: &InherentData) -> Result<InherentType, RuntimeString> {
	//data.get_data::<InherentType>(&INHERENT_IDENTIFIER)
	//	.map_err(|_| RuntimeString::from("Invalid inherent data encoding."))?
	//	.ok_or_else(|| "Inherent data is not provided.".into())
	
	let result = data.get_data::<InherentType>(&INHERENT_IDENTIFIER).unwrap();
	
	if let Some(s) = result {
		Ok(s)
	} else {
		let mut verified_data_vec:Vec<VerifiedData> = Vec::new();
		let verified_data = VerifiedData {
			tx_hash: ZERO_HASH.to_vec()
		};
		verified_data_vec.push(verified_data);

		let data = Encode::encode(&verified_data_vec);
		Ok(data)
	}
}

impl<T: Trait> ProvideInherent for Module<T> {
	type Call = Call<T>;
	type Error = MakeFatalError<RuntimeString>;
	const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;

	fn create_inherent(data: &InherentData) -> Option<Self::Call> {
        let data1 = extract_inherent_data(data).expect("Error in extracting inherent data.");

		let verified_data_vec:Vec<VerifiedData> = Decode::decode(&mut &data1[..]).unwrap();

		let mut call:Call<T> = Call::set_verified(verified_data_vec[0].tx_hash.to_vec(), 1);
		for index in 1..verified_data_vec.len() {
			let txhash = &verified_data_vec[index].tx_hash;
			call = Call::set_verified(txhash.to_vec(), 1);
		}

		Some(call)
	}

	// TODO: Implement check_inherent.
}
