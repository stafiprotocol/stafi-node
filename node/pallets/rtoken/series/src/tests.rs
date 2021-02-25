use sp_core::sr25519::{Pair as Sr25519Pair, Public, Signature};
use sp_core::{Pair as  TraitPair};
use hex_literal::hex;

#[test]
fn sr25519_verify_should_work() {
    let pair = <Sr25519Pair as TraitPair>::from_seed(&hex!(
        "d0f12a951dff55564a1563d7d88018f55c8e7c6596e0e55af410e4902904e14c"
    ));
    let public = pair.public();
    assert_eq!(
        public,
        Public::from_raw(hex!(
            "ced7a8ebf15d00260fd329f5df6efe789193366bf71cf0c7edf31902ca466743"
        ))
    );
    let message = hex!("128facc37e039a47fe12e791deaa147217e276f7a879cd0c064e17419fe24bff");
    let signature = pair.sign(&message[..]);
    let Signature(bytes) = signature;
    assert!(<Sr25519Pair as TraitPair>::verify(&signature, &message[..], &public));
    println!("{:?}", hex::encode(bytes.to_vec()));
    // fe8836c2a97190bc23cdb0ba11357b63eb07d9912a8bd35c07bc488a1ab89a5652631122884c3650b402ca5a3eaab06dfa35f9d7f76479469f645692e2683a84
}