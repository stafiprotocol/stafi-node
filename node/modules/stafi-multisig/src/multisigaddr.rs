extern crate frame_support as support;
extern crate frame_system as system;

use support::{decl_module, decl_storage, decl_event, dispatch::Result, dispatch::Vec};
use system::ensure_root;

use node_primitives::{ChainType, MultisigAddr};

pub trait Trait: system::Trait {
	type Event: From<Event> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as MultisigAddress {
		pub MultisigAddrList get(multisig_addr_list): map ChainType => Vec<MultisigAddr>;		
	}
	add_extra_genesis {
		config(addrs): Vec<MultisigAddr>;
		build(|config| Module::<T>::initialize_multisig_addrs(config.addrs.clone()))
	}
}


decl_module! {
	
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		fn deposit_event() = default;

		pub fn set_addr(origin, chain_type: ChainType, addr: Vec<u8>) -> Result {
			
			ensure_root(origin)?;

			let multisig_addr = MultisigAddr {
				chain_type: chain_type,
				multisig_addr: addr,
			};

			let mut list = Self::multisig_addr_list(chain_type);
			list.push(multisig_addr.clone());
			MultisigAddrList::insert(chain_type, list);

			Self::deposit_event(Event::AddrStored(multisig_addr));

			Ok(())
		}
	}
}

decl_event!(
	pub enum Event {
		AddrStored(MultisigAddr),
	}
);


impl<T: Trait> Module<T> {

	fn initialize_multisig_addrs(multisig_addrs: Vec<MultisigAddr>) {
		for multisig_addr in multisig_addrs {
			let mut list = Self::multisig_addr_list(multisig_addr.chain_type);
			list.push(multisig_addr.clone());
			MultisigAddrList::insert(multisig_addr.chain_type, list.clone());
		}	
	}

	pub fn check_multisig_address(chain_type: ChainType, multisig_address: Vec<u8>) -> bool {
		let list = Self::multisig_addr_list(chain_type);
		list.into_iter().find(|addr| multisig_address == addr.multisig_addr).is_some()
    }
}
