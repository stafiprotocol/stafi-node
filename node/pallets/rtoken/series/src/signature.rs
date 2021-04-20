use sp_std::{prelude::*, convert::TryFrom};
use codec::{Encode, Decode};
use sp_runtime::RuntimeDebug;
use sp_core::sr25519::{Public, Signature};
use node_primitives::{RSymbol, ChainType, Sr25519AppCrypto, Ed25519AppCrypto, EcdsaAppCrypto};
use frame_system::offchain::AppCrypto;

pub fn verify_signature(symbol: RSymbol, pubkey: &Vec<u8>, signature: &Vec<u8>, message: &Vec<u8>) -> SigVerifyResult {
    match symbol.chain_type() {
        ChainType::Substrate => super::substrate_verify(&pubkey, &signature, &message),
        ChainType::Tendermint => super::tendermint_verify(&pubkey, &signature, &message),
    }
}

pub fn verify_recipient(symbol: RSymbol, recipient: &Vec<u8>) -> bool {
    match symbol.chain_type() {
        ChainType::Substrate => {
            let re_public = <Public as TryFrom<_>>::try_from(&recipient[..]);
            if re_public.is_err() {
                return false;
            }
            return true;
        },
        ChainType::Tendermint => {
            return recipient.len() == 20;
        },
    }
}

/// signature verify result
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum SigVerifyResult {
    /// invalid pubkey
    InvalidPubkey,
    /// Fail
    Fail,
    /// pass
    Pass,
}

pub fn substrate_verify(pubkey: &Vec<u8>, signature: &Vec<u8>, message: &Vec<u8>) -> SigVerifyResult {
    let re_public = <Public as TryFrom<_>>::try_from(&pubkey[..]);
    if re_public.is_err() {
        return SigVerifyResult::InvalidPubkey;
    }

    let public = re_public.unwrap();
    let sig = Signature::from_slice(&signature);

    let mut vrf_result = <Sr25519AppCrypto as AppCrypto<_,_>>::verify(&message, public.into(), sig.clone().into());
    if vrf_result {
        return SigVerifyResult::Pass;
    }

    vrf_result = <Ed25519AppCrypto as AppCrypto<_,_>>::verify(&message, public.into(), sig.clone().into());
    if vrf_result {
        return SigVerifyResult::Pass;
    }

    vrf_result = <EcdsaAppCrypto as AppCrypto<_,_>>::verify(&message, public.into(), sig.into());
    if vrf_result {
        return SigVerifyResult::Pass;
    }

    SigVerifyResult::Fail
}

pub fn tendermint_verify(pubkey: &Vec<u8>, _signature: &Vec<u8>, _message: &Vec<u8>) -> SigVerifyResult {
    if !super::check_tendermint_pubkey(&pubkey) {
        return SigVerifyResult::InvalidPubkey;
    }
    
    if true {
        return SigVerifyResult::Pass;
    }

    SigVerifyResult::Fail
}

pub fn check_tendermint_pubkey(pubkey: &Vec<u8>) -> bool {
    return pubkey.len() == 33;
}
