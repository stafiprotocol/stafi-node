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
    let message = hex!("26db25c52b007221331a844e5335e59874e45b03e81c3d76ff007377c2c17965");
    let signature = pair.sign(&message[..]);
    let Signature(bytes) = signature;
    assert!(<Sr25519Pair as TraitPair>::verify(&signature, &message[..], &public));
    println!("{:?}", hex::encode(bytes.to_vec()));
    // 94986e713df3303e9f6e7e04b764bac73ab4cc57752e5bd9b2f238ffdc8d4b4ddb68d9095cf0cc6b33ba90a6b3c716631b1d6b4504c5b03e496cf354d348a887
}