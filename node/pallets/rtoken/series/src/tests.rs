use sp_core::sr25519::{Pair as Sr25519Pair, Public, Signature};
use sp_core::{Pair as TraitPair};
use hex_literal::hex;

#[test]
fn sr25519_verify_should_work() {
    let pair = <Sr25519Pair as TraitPair>::from_seed(&hex!(
        "7caadbb21966e5e268945005488a3c9be64a37b949d4691a367e477ed9e57995"
    ));
    let public = pair.public();
    assert_eq!(
        public,
        Public::from_raw(hex!(
            "26db25c52b007221331a844e5335e59874e45b03e81c3d76ff007377c2c17965"
        ))
    );
    let message = hex!("38c5fb2d5b2e404291aba53921854875c1341b01c97af71f5f353743b7aadfce");
    let signature = pair.sign(&message[..]);
    let Signature(bytes) = signature;
    assert!(<Sr25519Pair as TraitPair>::verify(&signature, &message[..], &public));
    println!("{:?}", hex::encode(bytes.to_vec()));
    // 3a09e37ac5172f45061d8ee03a34be1e677bb5efd8538fe3f78d0085bf67342013b8de01f99e939926a8bfa8c6728bcff18539c6632cd58ea68a8c17e28de584
}