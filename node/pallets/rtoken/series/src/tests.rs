use sp_core::sr25519::{Pair as Sr25519Pair, Public, Signature};
use sp_core::{Pair as TraitPair};
use hex_literal::hex;

#[test]
fn sr25519_verify_should_work() {
    let pair = <Sr25519Pair as TraitPair>::from_seed(&hex!(
        "5cb9063bc28a07f7444dbb29b6fc15b802b141a4e7dd5c932f2981be69e127e7"
    ));
    let public = pair.public();
    assert_eq!(
        public,
        Public::from_raw(hex!(
            "9c4189297ad2140c85861f64656d1d1318994599130d98b75ff094176d2ca31e"
        ))
    );
    let message = hex!("46094babab45e2fd5265b2aab02c9342fe0ca1a5aacd455d3bc8a2ac99dce8fd");
    let signature = pair.sign(&message[..]);
    let Signature(bytes) = signature;
    assert!(<Sr25519Pair as TraitPair>::verify(&signature, &message[..], &public));
    println!("{:?}", hex::encode(bytes.to_vec()));
    // d25232bef5133781d97d5f8bf6c3ef9c51be5cf0f2f325b8b38d7fb7f1b7d178776fa8348de900fb4697547dbaad5e8c99095c38186addadb4a515169525e787
}