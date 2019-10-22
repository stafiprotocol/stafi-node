use stafi_primitives::AccountId;
use grandpa_primitives::{AuthorityId as GrandpaId};
use babe_primitives::{AuthorityId as BabeId};
use im_online::sr25519::{AuthorityId as ImOnlineId};
use primitives::crypto::UncheckedInto;
use hex_literal::hex;

pub fn get_vals() -> Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId)> {
	return vec![(
		// 5FWEqXvKfvf3tGgVZJdpU6YHoUpnYWsK5AWdrGJGLAXPpZXg
		hex!["982e1ff9bc6e2c1b04b2d66a04f28aff2f767d330aefc43ffecc7d912b29727f"].unchecked_into(),
		// 5EWkRypfBwuMkHQ3PUscMWWb1qtqR8vARhwAVyqu28zjFXH4
		hex!["6c55836f9fe08b30b57c0d97e029d7cb66787267f9c99ee24e15b956bff88476"].unchecked_into(),
		// 5C8cybYi1aucBjvYZQH9XpJ99FuEFG5ikKoeuthq55MEvUVf
		hex!["02fc8d5cdd2632564a9f3402968a98011407f179587a1bf20a6fb87786070837"].unchecked_into(),
		// 5DRQWvBoKmDa85HAFukQtRYv7dHnBTVRt2iBQKBUCiNNzqwX
		hex!["3c0603752105ef97a9fd693a74d6692382f599474db260c37cabbbacaa26f612"].unchecked_into(),
		// 5FCPzDDdgWDAXFz6ue7WFyumFd1ZMXUdvoVvDhKN3sUziLrC
		hex!["8a92820a446409b940b4da97ad0e27fc1ce6be4ba7e32a1f3997f8b08ad10e3a"].unchecked_into(),
	
	), (
		// 5GCTf7sn5KCY8jVRPXsHmQUTUXz7bCgFACWtPAe7LKsWqqrq
		hex!["b6db1938cd6c9148ef9e48960e86b3eb76796b1f07d7afd32de7dd559b39fd63"].unchecked_into(),
		// 5GgwHTLTGs7i7mWz7snNAZuTir3gLuvJjzg89N9bq5ZU7f71
		hex!["cc92f8066c5c80a8b61c59bf5a50384367cfb121bb77134a9a6e374977a42a45"].unchecked_into(),
		// 5E9zm4oe7s5Uc6XLLnRfEZiL6f9yyjTu8LFRXApyu4Gq7wXE
		hex!["5c81988afcd078e7f43ada10fcbd49de7d5695f1af8459a6b3d8e5726b5b6bdb"].unchecked_into(),
		// 5FPCCNHaLdpCPAjtzgtALxNmiHCo36xEqRCBXP7LFVXjAsLA
		hex!["92ce84721e0101d5d7267b4ae6d64d5624d59fefe585183ba699a3d82d659164"].unchecked_into(),
		// 5Fc1JMU1XZUzQv3AQ5pSDosP3k7sBUF7oBbRz5KxzEzqE72e
		hex!["9c940696dc3a1c41338cf075ad47d0a12d1f7d4f616b54b52511f657b0cd9b18"].unchecked_into(),

	), (
		// 5CDSNzRxQANpnzFCXrRg7uGJnMe1gz4AM6n4f8GdufZt4gjY
		hex!["06a91b943ac7624d1b8018922f2f4db7b2d42ad5e720787f4f338fbc54c81c6e"].unchecked_into(),
		// 5G1R2bASPmgpRyKZaoX31ukAQH7aAthCUbiXE4Rj4ePbX24s
		hex!["ae6e8bdb333819bbee4b7a4c783ceed2ab55fa33da3eb6c77635a21d67201017"].unchecked_into(),
		// 5H8ZwmSifeEr5mBYSgfDNcFjQxQmyujS1n9oS6jEL1Ejnc5F
		hex!["e01f83aedfb211d9321813007a220879ae70b085b6f5150b4be0967c1670ee2d"].unchecked_into(),
		// 5E7S6Cnn4PsAm8CV4cZi96ZfXCQro3hM3nsnUGwsnsaGFFzx
		hex!["5a8d20be4acc691068282aab5827d237a7522cf2bec9edc02591606918325c1e"].unchecked_into(),
		// 5HeNMuD8beMi9JAw7TNiuxesWKLeQshjYCzn7DhxzvbveSbF
		hex!["f6d9e1b32662705fe3a08991d58e4c02f4e1eca24a1869a212e12d91567b261d"].unchecked_into(),
	)];
}

pub fn get_more_endowed() -> Vec<AccountId> {
	return vec![
		// 5HEMcNK5EfLwY2Q91QbDB9yeZyxfk9X3uDjgSCYYbEXGdoaE
		hex!["e489771ea3c4f10cb28698d21d5382ce3c5a673f47bade2b7325718701ad4b0c"].unchecked_into(),
		// 5EvBsWCMdDfzsEHX9GK4SZuXrBVrzckDvKrmBvpA2S1ryhFf
		hex!["7e35cbeea9f986613567088dbdb56da124f6511e339a44fd53a127b7653cff34"].unchecked_into(),
		// 5EJqvEFQLQ45a1scevWGcLFKYhLLSLLWUk7AL4FjHa9Z42QR
		hex!["63410a24555c6f0c5ba8f0d27f85740dca150d9a0d67e3fa8502d5d9e6a4fafe"].unchecked_into(),
		// 5C7XeNv7p7RaV7WrpssxNkjchvUsWkNL2SQDZS4z8EcxigjS
		hex!["02275c22710ffea0e562a503684ebed07c68aabb84425aa3efce611374501253"].unchecked_into(),
		// 5GExJMV6mmGmEjxVyp5wvgiycMeTzEkB4kyw2FtEWvYkr9fE
		hex!["b8c201beead1491e3f3eec3a4cf32d006ea232da45c1573aff0d3b4a3f3a4049"].unchecked_into(),
	];
}

pub fn get_root_key() -> AccountId {
	// 5GrybpgHW1bpg6WSnK8yrqApAEqsjYpfaUuhiY5nHzeAvLJJ
	return hex!["d43b38b84b60b06e7f1a00d892dcff67ea69dc1dc2f837fdb6a27344b63c9279"].unchecked_into();
}

