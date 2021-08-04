#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{prelude::*, convert::TryFrom};
use codec::{Encode, Decode};
use sp_runtime::RuntimeDebug;
use sp_core::{
    sr25519::{Public as Sr25519Public, Signature as Sr25519Signature},
    ed25519::{Public as Ed25519Public, Signature as Ed25519Signature},
    ecdsa::{Public as EcdsaPublic, Signature as EcdsaSignature},
};
use sp_io::{hashing::keccak_256, crypto::secp256k1_ecdsa_recover};

use node_primitives::{RSymbol, ChainType, Sr25519AppCrypto, Ed25519AppCrypto, EcdsaAppCrypto};
use frame_system::offchain::AppCrypto;

pub fn verify_signature(symbol: RSymbol, pubkey: &Vec<u8>, signature: &Vec<u8>, message: &Vec<u8>) -> SigVerifyResult {
    match symbol.chain_type() {
        ChainType::Substrate => substrate_verify(&pubkey, &signature, &message),
        ChainType::Tendermint => tendermint_verify(&pubkey, &signature, &message),
        ChainType::Solana => ed25519_verify(&pubkey, &signature, &message),
        ChainType::Ethereum => {
            let msg = &keccak_256(&message[..]);
            ethereum_verify(&pubkey, &signature, msg)
        },
    }
}

pub fn verify_recipient(symbol: RSymbol, recipient: &Vec<u8>) -> bool {
    match symbol.chain_type() {
        ChainType::Substrate => {
            let re_public = <Sr25519Public as TryFrom<_>>::try_from(&recipient[..]);
            return re_public.is_ok();
        },
        ChainType::Tendermint | ChainType::Ethereum => {
            return recipient.len() == 20;
        },
        ChainType::Solana => {
            let ed_public = <Ed25519Public as TryFrom<_>>::try_from(&recipient[..]);
            return ed_public.is_ok();
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
    let sr_public = <Sr25519Public as TryFrom<_>>::try_from(&pubkey[..]);
    let ed_public = <Ed25519Public as TryFrom<_>>::try_from(&pubkey[..]);
    let ecd_public = <EcdsaPublic as TryFrom<_>>::try_from(&pubkey[..]);
    if sr_public.is_err() && ed_public.is_err() && ecd_public.is_err() {
        return SigVerifyResult::InvalidPubkey;
    }

    if sr_public.is_ok() {
        let public = sr_public.unwrap();
        let sig = Sr25519Signature::from_slice(&signature);
        let result = <Sr25519AppCrypto as AppCrypto<_,_>>::verify(&message, public.into(), sig.into());
        if result {
            return SigVerifyResult::Pass;
        }
    }

    if ed_public.is_ok() {
        let public = ed_public.unwrap();
        let sig = Ed25519Signature::from_slice(&signature);
        let result = <Ed25519AppCrypto as AppCrypto<_,_>>::verify(&message, public.into(), sig.into());
        if result {
            return SigVerifyResult::Pass;
        }
    }

    if ecd_public.is_ok() {
        let public = ecd_public.unwrap();
        let sig = EcdsaSignature::from_slice(&signature);
        let result = <EcdsaAppCrypto as AppCrypto<_,_>>::verify(&message, public.into(), sig.into());
        if result {
            return SigVerifyResult::Pass;
        }
    }

    SigVerifyResult::Fail
}

pub fn tendermint_verify(pubkey: &Vec<u8>, _signature: &Vec<u8>, _message: &Vec<u8>) -> SigVerifyResult {
    if !check_tendermint_pubkey(&pubkey) {
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

pub fn ed25519_verify(pubkey: &Vec<u8>, signature: &Vec<u8>, message: &Vec<u8>) -> SigVerifyResult {
    let ed_public = <Ed25519Public as TryFrom<_>>::try_from(&pubkey[..]);

    if ed_public.is_err() {
        return SigVerifyResult::InvalidPubkey;
    }

    let public = ed_public.unwrap();
    let sig = Ed25519Signature::from_slice(&signature);
    let result = <Ed25519AppCrypto as AppCrypto<_,_>>::verify(&message, public.into(), sig.into());
    if result {
        return SigVerifyResult::Pass;
    }

    SigVerifyResult::Fail
}

pub fn ethereum_verify(pubkey: &Vec<u8>, signature: &Vec<u8>, msg: &[u8; 32]) -> SigVerifyResult {
    if pubkey.len() != 20 {
        return SigVerifyResult::InvalidPubkey;
    }

    let mut sig = [0u8; 65];
    sig.copy_from_slice(&signature);

    let signer = eth_recover(&sig, &msg).unwrap().to_vec();
    if &signer == pubkey {
        return SigVerifyResult::Pass;
    }

    SigVerifyResult::Fail
}


pub fn eth_recover(sig: &[u8; 65], msg: &[u8; 32]) -> Option<[u8; 20]> {
    let mut res = [0u8; 20];
    res.copy_from_slice(&keccak_256(&secp256k1_ecdsa_recover(&sig, &msg).ok()?[..])[12..]);
    Some(res)
}

