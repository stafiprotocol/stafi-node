use sp_std::{prelude::*, convert::TryFrom};
use codec::{Encode, Decode};
use sp_runtime::RuntimeDebug;
use sp_core::sr25519::{Public, Signature};
use node_primitives::{RSymbol, ChainType, report::ReporterAppCrypto};
use frame_system::offchain::AppCrypto;

pub fn verify_signature(symbol: RSymbol, pubkey: &Vec<u8>, signature: &Vec<u8>, txhash: &Vec<u8>) -> SigVerifyResult {
    match symbol.chain_type() {
        ChainType::Substrate => super::sr25519_verify(&pubkey, &signature, &txhash),
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

pub fn sr25519_verify(pubkey: &Vec<u8>, signature: &Vec<u8>, txhash: &Vec<u8>) -> SigVerifyResult {
    let re_public = <Public as TryFrom<_>>::try_from(&pubkey[..]);
    if re_public.is_err() {
        return SigVerifyResult::InvalidPubkey;
    }
    let public = re_public.unwrap();
    let sig = Signature::from_slice(&signature);
    let vrf_result = <ReporterAppCrypto as AppCrypto<_,_>>::verify(&txhash, public.into(), sig.into());
    if vrf_result {
        return SigVerifyResult::Pass;
    }

    SigVerifyResult::Fail
}