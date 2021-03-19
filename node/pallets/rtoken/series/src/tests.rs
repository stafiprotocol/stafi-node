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
    let message = hex!("af04970ad0aa3880c685a417b29a35662d8a16d481696f6f3e7ce53a0b91e92f");
    let signature = pair.sign(&message[..]);
    let Signature(bytes) = signature;
    assert!(<Sr25519Pair as TraitPair>::verify(&signature, &message[..], &public));
    println!("{:?}", hex::encode(bytes.to_vec()));
    // 00140a44a87fb5eadac23db1436e10956049fd2b6bab1c4de31f9bf15b318e029c708d2d41c9b6921cbb9cdd5dd5ce79751b386685bf911c14d15c9c2d0d6088
}