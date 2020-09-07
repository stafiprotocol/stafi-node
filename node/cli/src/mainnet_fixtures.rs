// Copyright 2019-2020 Stafi Protocol.
// This file is part of Stafi.

// Stafi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stafi.  If not, see <http://www.gnu.org/licenses/>.

//! Mainnet fixtures

use grandpa_primitives::{AuthorityId as GrandpaId};
use sp_consensus_babe::{AuthorityId as BabeId};
use pallet_im_online::sr25519::{AuthorityId as ImOnlineId};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_core::crypto::UncheckedInto;
use hex_literal::hex;
use node_primitives::{AccountId};
use sc_network::{config::MultiaddrWithPeerId};

/// Mainnet bootnodes
pub fn get_bootnodes() -> Vec<MultiaddrWithPeerId> {
	return vec![
		"/dns/p2p.node-0.stafi.io/tcp/30333/p2p/12D3KooWSQnaHog7sezJ7qYubhsyuCuGyb61DGXaDo4HVf5bAmfu".parse().unwrap(),
		"/dns/p2p.node-1.stafi.io/tcp/30333/p2p/12D3KooWFLnUHeuFqLDr5jxJFXdyt6tovUA3C7PYKNfTBr5dpkaa".parse().unwrap(),
		"/dns/p2p.node-2.stafi.io/tcp/30333/p2p/12D3KooWK5H2wCL1vtRpgLEPfxAQsREjhzJu3tNXPg7Uohu6EpDb".parse().unwrap(),
		"/dns/p2p.node-3.stafi.io/tcp/30333/p2p/12D3KooWBvrETZtMRPE94EiUsTtYvEe9gHrBvD6EoCtHaJ5VB2qh".parse().unwrap(),
		"/dns/p2p.node-4.stafi.io/tcp/30333/p2p/12D3KooW9qmEqjNxnuhhQMM1XW3BMUWSzmXynuVyr4sQktFB87ud".parse().unwrap(),
		"/dns/p2p.node-5.stafi.io/tcp/30333/p2p/12D3KooWPqJXX3scW76MYSjuEgn6Rb4NhyMJFtz9g4X3jgVuzJFm".parse().unwrap(),
	];
}

/// Mainnet initial authorities
pub fn get_initial_authorities() -> Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)> {
	return vec![(
		// 344cYqUXrBovzoFYe7EyWRQyyW1SdvnzezSpjtSR4YCtonPz
		hex!["92c12f73962397512c4b054852dc45173499be614b7fa44fb472823a9ed4903f"].into(),
		// 35da1YSTDZ2wz7grDaQPL8SW9DPpnLeE3w6UpGMZAogoxv8R
		hex!["d81ff7e1068a40ae52f0074bc940ab6fd68e2fa8335421f4cae289da64db3c5c"].into(),
		hex!["39e2d5867649a13c4223236e56c05ac4edac9e10cafc0090491c87959ed15bab"].unchecked_into(),
		hex!["28069e7be58b718f24b1f931ce653f70c8c54eeb0d8be6816210c5643488c151"].unchecked_into(),
		hex!["2a6b9741b7434a290cd5303c17909c15c2630196ad520f7ec557ec505475147a"].unchecked_into(),
		hex!["1e7517d38f065634db0850681ea7f90dcbcc41265f57f642745a4f54c26d3633"].unchecked_into(),
	),(
		// 34NuqL8dRHk3KCqxRDkDgMeTSimovh6EF6BCMEieranQ6DeJ
		hex!["a0b5c97bfd3a04fe40af53be040a57bf8841697752ef3d88cc1c55d2568cf409"].into(),
		// 35We9q6DJJ9GpFAi7GzmG9bA3uTaWssAvK4vKJZqxntariKw
		hex!["d2d73256226ce1e27bfc69a51d7883eb6caae35e44ccc62b567804af2fdaa010"].into(),
		hex!["9484c608f4c65aae0edd17689f25a4563c55adea125ee537e5138ffd0e4f5ffd"].unchecked_into(),
		hex!["e4cbe36de8161f8aa4444215362d0887eac22d0eb49c7e76974954ccda080e34"].unchecked_into(),
		hex!["d67cbad1478265b87f7283609158f49fd2001af14bb2b90354dd0249916c694c"].unchecked_into(),
		hex!["ac0105b90c860b3a69488aa0721f5a4779161dac9e0e7f35a24a64280747c27b"].unchecked_into(),
	),(
		// 31f9GbtBBM384jKjDvqAEr83yPVVjnUKfWrBqcMS9PWqeuvf
		hex!["28623de774f97fb2a23ab3f7795d200ed2e8305ab42e31576e2bc83c1f86cf03"].into(),
		// 35Yr6JefKSGxgVfqk1GFDGSGi8HWuocvrfGztyebU9K3nsQQ
		hex!["d485e0e27fe70cf09ba41300c77f8e06a7cfa06b44287058247bc49ed2ba6268"].into(),
		hex!["2c57b510fcc18b1463c531fcbb159f7622ee1344e1d1d71c9bde9a1bf67d2d2b"].unchecked_into(),
		hex!["ac412e4417b2d59ed90012a35d164054f44152e294018008ed91c61f838a6255"].unchecked_into(),
		hex!["cc3d3f07aba16412b1d6c36f83bd864631bc3c32c1d81ce384060e37e590405b"].unchecked_into(),
		hex!["66b6d7057643d1d80a668ec48a14a7e0bae5654730348da3f62f1911e6bbe328"].unchecked_into(),
	),(
		// 329uiEjVDR6dA52Ygt58kUMGtktUCQmnXsXLM269XuZHivvW
		hex!["3e52b7dc4c5f8054544687a585cd86711e0c6b4a9b8d2824676d7381a1caf07f"].into(),
		// 2zvsb8Su8o5KpkTvG1UQCezupgaaUfyg2WEFPqDru3TnHVri
		hex!["0825263a54046ae2cf33f417e68bc2428f4bf274c413cff4161ccb6635d84f43"].into(),
		hex!["4c7c64e687d8b7221c910c37acdacaa853f6e4654d5c057becdc52624bd526ae"].unchecked_into(),
		hex!["16362c472e4d61655485de73beef5e3f1a1ed988d1d3dd3c201767676c618678"].unchecked_into(),
		hex!["acd62aeeefb10a7d5ce6f04644edd277cfdaa9813f2b6c66eecfffd129516d6b"].unchecked_into(),
		hex!["7e7b5506c9d507be3558f1eb6d49cd6ebbf4785f126d10fb0378246e08f4d335"].unchecked_into(),
	),(
		// 35E5JNRfaJv77hXfNJL78hhrCXhnVi8CqP3gHprooXS6m9yb
		hex!["c634ad8cbac42362747a07091b118018242b29b1c1aa9f77b575f4477882f101"].into(),
		// 34X9apeUm75WM6waMztsbZzmmnhe7m9MKosxEm6NU4GW2Ft7
		hex!["a6fe09530be30e608eee8ccdd23a0445d3b42271fc82b0b8ac6658404cde1b38"].into(),
		hex!["42ecd642f0fed514be241d9d8769023d114ea3b1401deba81cf82da7557ea0c0"].unchecked_into(),
		hex!["e2bcbfd3342007f4b703f0dd0195ba801ce138b387e16325d88c73efb8215854"].unchecked_into(),
		hex!["da0361d06795e8e4eb57f416eb7766edcdee9a201925c17e960b4fdabeef5c57"].unchecked_into(),
		hex!["f48250f1458393d3b4bf51c74fbfabf9567fd1499cbe3e57438e8abed9cb047f"].unchecked_into(),
	),(
		// 36Qh56g4jVLNpYmrmYQ1Bgt61NQTsi7yKtTmDid9MqSwTrFc
		hex!["fa89c82dddf8319f01883bfd2975efe9d9428cf905b93bfbcf9a1e037c5caf53"].into(),
		// 31yCZcxeGj1ud38Zuyjrex3HiakcBqGHvCUTfAnSh5xss5U5
		hex!["3627bee90e5fb5843a565b93b2ecdef77ba2f4046f16bd8d65b692d323f22b35"].into(),
		hex!["69b7da81638e477b1ab592a0038b187dae3919d2999ed0a95e3e88a97c202973"].unchecked_into(),
		hex!["7cfbadf2aa8e6401d8f57a2aa851af90bf357c9971058eb407201b8f30e37406"].unchecked_into(),
		hex!["88871a702c45d4130760d848dbc849640dc967bdef64c041ec5eea6670becc13"].unchecked_into(),
		hex!["14a1e4e11fc0d985240cb8fd33de48357059086ef18283fe8ea47e247807eb44"].unchecked_into(),
	)];
}

/// Mainnet root key
pub fn get_root_key() -> AccountId {
	return hex!["e8b9fdb324ba6c0388a9d7e0ca0450a6a41aa7795e7428785ea2a329c58fde32"].into();
}
