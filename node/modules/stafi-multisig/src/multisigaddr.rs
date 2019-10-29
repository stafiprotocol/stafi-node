extern crate srml_support as support;
extern crate srml_system as system;

use support::{decl_module, decl_storage, decl_event, dispatch::Result, dispatch::Vec};
use system::ensure_root;

use stafi_primitives::{ChainType, MultisigAddr};

pub trait Trait: system::Trait {
	type Event: From<Event> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as MultiSigAddrsModule {
		pub MultisigAddrList get(multisig_addr): Vec<MultisigAddr>;		
	}
}


decl_module! {
	
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		fn deposit_event() = default;

		pub fn set_addr(origin, chain_type: ChainType, addr: Vec<u8>) -> Result {
			
			ensure_root(origin)?;

			let addr = MultisigAddr {
				chain_type: chain_type,
				multisig_addr: addr,
			};

			let mut list = MultisigAddrList::get();
			list.push(addr.clone());
			MultisigAddrList::put(list);

			Self::deposit_event(Event::AddrStored(addr));

			Ok(())
		}
	}
}

decl_event!(
	pub enum Event {
		AddrStored(MultisigAddr),
	}
);