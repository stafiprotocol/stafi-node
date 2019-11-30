#![cfg_attr(not(feature = "std"), no_std)]

extern crate sr_primitives as runtime_primitives;

use support::{decl_module, decl_storage, decl_event, dispatch::Result, Parameter, dispatch::Vec};
use system::ensure_signed;
use parity_codec::{Codec, Encode, Decode};
use sr_primitives::traits::MaybeSerialize;
use node_primitives::{Symbol}; 

pub mod bondtoken;

pub type SymbolString = &'static [u8];
pub type DescString = SymbolString;

pub trait Trait: balances::Trait {
	const STAFI_SYMBOL: SymbolString;
    const STAFI_TOKEN_DESC: DescString;
	type TokenBalance: Parameter + Codec + Default + Copy + MaybeSerialize + From<Self::BlockNumber>;
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

pub type TokenDesc = Vec<u8>;
pub type Precision = u16;

#[derive(PartialEq, Eq, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive( Debug))]
pub struct Token {
    symbol: Symbol,
    token_desc: TokenDesc,
    precision: Precision,
}


impl Token {
    pub fn new(symbol: Symbol, token_desc: TokenDesc, precision: Precision) -> Self {
        Token {
            symbol,
            token_desc,
            precision,
        }
    }

    pub fn symbol(&self) -> Symbol {
        self.symbol.clone()
    }

    pub fn precision(&self) -> Precision {
        self.precision
    }

    pub fn token_desc(&self) -> TokenDesc {
        self.token_desc.clone()
    }

    pub fn set_token_desc(&mut self, desc: &TokenDesc) {
        self.token_desc = desc.clone();
    }

}

decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		pub TokenInfo get(token_info): map Symbol => Token;

        pub TotalFreeToken get(total_free_token): map Symbol => T::TokenBalance;

        pub FreeToken get(token_free_balance): map (T::AccountId, Symbol) => T::TokenBalance;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		pub fn register_token(
            origin,
            symbol: Symbol,
            token_desc: TokenDesc,
            precision: Precision
        ) -> Result {
			let who = ensure_signed(origin)?;
            let token = Token{
				symbol : symbol,
            	token_desc : token_desc,
            	precision : precision,
			};
            <TokenInfo>::insert(token.symbol(), token.clone());
			Self::deposit_event(RawEvent::TokenInfoStored(token, who));
            Ok(())
        }

		pub fn set_free_token(
            origin, 
            who: T::AccountId, 
            sym: Symbol, 
            free: T::TokenBalance
            ) -> Result {
            let from = ensure_signed(origin)?;
			let key = (who.clone(), sym.clone());
            FreeToken::<T>::insert(key, free);
			Self::deposit_event(RawEvent::FreeTokenStored(sym.clone(), from));
            Ok(())
        }
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		SomethingStored(u32, AccountId),
		SomeValueStored(u32, AccountId),
		TokenInfoStored(Token, AccountId),
		FreeTokenStored(Symbol, AccountId),
	}
);