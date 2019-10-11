//#![cfg_attr(not(feature = "std"), no_std)]

extern crate sr_std as rstd;
extern crate srml_support as support;
extern crate srml_system as system;

use support::{decl_module, decl_storage};
use rstd::prelude::*;
use system::{ensure_none};
use inherents::{RuntimeString, InherentIdentifier, ProvideInherent, MakeFatalError, InherentData};

pub type InherentType = Vec<u8>;

pub mod irisnet;
pub use irisnet::INHERENT_IDENTIFIER;
#[cfg(feature = "std")]
pub use irisnet::InherentDataProvider;

pub trait Trait: system::Trait { }

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn set_result(origin, result: Vec<u8>) {
			let _who = ensure_none(origin)?;
			
			<Self as Store>::RpcResult::put(result);
		}
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as IrisnetRpc {
		pub RpcResult get(rpc_result): Vec<u8>;
	}
}

impl<T: Trait> Module<T> {}

fn extract_inherent_data(data: &InherentData) -> Result<InherentType, RuntimeString> {
	data.get_data::<InherentType>(&INHERENT_IDENTIFIER)
		.map_err(|_| RuntimeString::from("Invalid inherent data encoding."))?
		.ok_or_else(|| "Inherent data is not provided.".into())
}

impl<T: Trait> ProvideInherent for Module<T> {
	type Call = Call<T>;
	type Error = MakeFatalError<RuntimeString>;
	const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;

	fn create_inherent(data: &InherentData) -> Option<Self::Call> {
        let data1 = extract_inherent_data(data).expect("Error in extracting inherent data.");
		Some(Call::set_result(data1.into()))
	}

	// TODO: Implement check_inherent.
}
