use sp_core::sr25519::{Pair as Sr25519Pair, Public, Signature};
use sp_core::{Pair as TraitPair};
use hex_literal::hex;
use super::signature::{SigVerifyResult, ethereum_verify};

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

#[test]
fn ethereum_verify_should_work() {
    let msg = hex!("1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8");
    let sig = hex!["82dbd11468a4fe72682e656a03bcb5817f4470b9e41a25ed0e0a50f7fdb22c380070999361924984f66fb5d7772049539207c73d836f4578579638df3513ae6700"].to_vec();
    let signer = hex!["Bca9567A9e8D5F6F58C419d32aF6190F74C880e6"].to_vec();

    let result = ethereum_verify(&signer, &sig, &msg);

    assert_eq!(result, SigVerifyResult::Pass);
}

#[test]
fn ethereum_verify_should_not_work() {
    let msg = hex!("1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8");
    let sig = hex!["82dbd11468a4fe72682e656a03bcb5817f4470b9e41a25ed0e0a50f7fdb22c380070999361924984f66fb5d7772049539207c73d836f4578579638df3513ae6700"].to_vec();
    let signer = hex!["aca9567A9e8D5F6F58C419d32aF6190F74C880e6"].to_vec();

    let result = ethereum_verify(&signer, &sig, &msg);

    assert_eq!(result, SigVerifyResult::Fail);
}

