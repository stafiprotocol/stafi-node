// Copyright 2019-2020 Stafi Protocol.
// This file is part of Stafi.

// Stafi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stafi.  If not, see <http://www.gnu.org/licenses/>.

//! Testnet fixtures

use grandpa_primitives::{AuthorityId as GrandpaId};
use sp_consensus_babe::{AuthorityId as BabeId};
use pallet_im_online::sr25519::{AuthorityId as ImOnlineId};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_core::crypto::UncheckedInto;
use hex_literal::hex;
use node_primitives::{AccountId, Balance, BlockNumber};
use node_runtime::constants::currency::*;
use sc_network::{config::MultiaddrWithPeerId};

/// Testnet bootnodes
pub fn get_bootnodes() -> Vec<MultiaddrWithPeerId> {
	return vec![
		"/ip4/185.228.137.49/tcp/30333/p2p/12D3KooWSFhGrte1XYYHkktbkMjZbpyNjNbkcsxsN5nKZVrnbgL2".parse().unwrap(),
		"/ip4/46.38.241.169/tcp/30333/p2p/12D3KooWDZBFXyguH3i7m7646KxkdVc5E2EKasakTnBhaJWJgPP1".parse().unwrap(),
		"/ip4/5.45.104.102/tcp/30333/p2p/12D3KooWGHUpJq8NUsZDuWRekuBo1qNYqm3hRCTp7qp3crYqWvWK".parse().unwrap(),
	];
}

/// Sitara Testnet bootnodes
pub fn get_sitara_bootnodes() -> Vec<MultiaddrWithPeerId> {
	return vec![
		"/ip4/185.228.137.49/tcp/30334/p2p/12D3KooWDE2ZnFUzEV6kLFpMdDMpSyNnjuKq3dH4yJ97T2AYVFcc".parse().unwrap(),
		"/ip4/46.38.241.169/tcp/30334/p2p/12D3KooWFJVjCwsGKoLgkP91X5Jc2f4UU3rGauXARddhVft3HkUE".parse().unwrap(),
		"/ip4/5.45.104.102/tcp/30334/p2p/12D3KooWMiqcUFkB5XWbVc66T3eNjFmLVztKVWy8CwQ6zoUQMPeV".parse().unwrap(),
	];
}

/// Testnet initial authorities
pub fn get_initial_authorities() -> Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)> {
	return vec![(
		// 5DAM9w5tLThneVLTuv6M91Lyjz4n6bgUM3u4gCtJ8Bai85DP
		hex!["308a03fec4d787f08caf4fa7c569ad59e75c240a749a67704d35ed71c8298356"].into(),
		// 5GWodd3RvXC1MuQ3fZUNWANDbiDhyjBPKb7xaKFEqx65uSe7
		hex!["c4d8c141c908b043a875e40a4a3ec930d60dfadc5a167a1f65970eaf34cb6e51"].into(),
		// 5E6L53rkjcMWi95R8nmrCcs5aZCz8gCZVfBHVwfmZtAtgcfY
		hex!["59b59e23011dae7880d251bc66e12bb493e2590e4421d85729d53250ccebb9e1"].unchecked_into(),
		// 5HEwVWadzu8S1JMCKu3eEwWaFmuinRrdpAcVCPvPesvN1uEb
		hex!["e4fb8581b21beb477a422612dc20b53e0c8b986679b051d5cdacbcca3cabce1b"].unchecked_into(),
		// 5DJerTA9s6xgET2rvz5K21Ew6RiSZorp5JTVRLrNpJrwuaU4
		hex!["36df8ec05dfdedff6a5fdd2ca12d9f89cf21f3305df1a32bb5676cf74d21610a"].unchecked_into(),
		// 5E9WLGGz2f6jx3ToyvzJz6V7pxiUrvmzcYa6F6UQ261uP2Xo
		hex!["5c21e6638d52e1665ec71ce6a3c8aec241fdbc4525d7bfa41d19d0b41e82e877"].unchecked_into(),
	),(
		// 5FNXemt9RBaMMiZB1mR9oV59Q1DduUk7zzJSAgs6fPSkwy18
		hex!["924cc3a998e6b2293fc46281e78be2a6693437bf33fcec370a9167bc33fdf17b"].into(),
		// 5Griwoxr3GpS8YwWWp623v72aWMKbXNCBQRw9KUseoR7sCPv
		hex!["d409e31b8d45397b0b23d6955bb084ceae0c3635fd8e7fea58c46ffb389a7523"].into(),
		// 5CFbRqqR8RjZd9ThKcdD4xqn2JDmoxN1MvZNmUPbtz8ULKeS
		hex!["084e0f8dce7115060375d5e6a4f12604d5c66f51b4389ed39a4d0c7f64e4b887"].unchecked_into(),
		// 5GuvkEb9MB4dPF3wMDw9ywV1Y6VioZeVQtEDUsMRwJ452sWn
		hex!["d67b58dcda361d416a7e1b3cd2982cbc5f3bc77cba0d09aebb969454515ae452"].unchecked_into(),
		// 5Eqsp9TZkCFECmACvCYXJtjv3JBohJuJJQwfPxQetiA6Fkg4
		hex!["7aec06d72bc18a077f21cc7471c96b826713c23fc465baa04c310d10298a885b"].unchecked_into(),
		// 5GZ6huGVy3HZdCYuLkX8FyTw9naZM2RVJGjcGxw8MtZn6x8r
		hex!["c698b8a18dc57dd8c3879adf4e5655e632235704de4c507fbde9da6aa0160501"].unchecked_into(),
	),(
		// 5F3vNkv9RoYqdH9mTouMsHrESqyYK71qzQm5e4vt2MrjWnnw
		hex!["841b98d00ff0f9548c5792d53c842e1d0e741bea470e22051ab98ae85b510833"].into(),
		// 5DZjDxfkKAFi8U1aS1RzPTf5cQ6GjLjnyGvsRdEu7ZM3fYRJ
		hex!["425ef3c6c4ca93e6047569bd61ebc0df15c9b54b460ddc4f28553c6c0ff1d518"].into(),
		// 5FNyVGbK9cw4cxb7xLGMNVKAwSCe4Z1iF7BJ1Ru51JScGkwU
		hex!["92a3bc876c9621ec908114b7304d49617221c1bedb3ac47c384030193f5900b7"].unchecked_into(),
		// 5Gp5AzJKSaTzds7nTjDviU7XiSRqbXpKcqyqijCx79SThVRM
		hex!["d2043de5ae6d474be26ab248efde1fc7ce02af6f647170bc88dd100fd662f779"].unchecked_into(),
		// 5Eh6CMCV8JcQpd3E7rY3X47Gufcqe2kNzBiU7GRTVZKAPGHw
		hex!["74388096ebc0c6a259b1e431d21afb1505ed205467f4c9bd19b519e875758764"].unchecked_into(),
		// 5EHUFBrCqrgcfESmp4TroRJm2QfGZS9GAbRTmmR5LPquRgJ8
		hex!["6234d601a7a973b3cd5dac3f805e94c0d24d8b83c32381646b2404d64f733f3c"].unchecked_into(),
	)];
}

/// Testnet root key
pub fn get_root_key() -> AccountId {
	// 5CVAqeuuBdWQvH4hAVUwBr6KKvNbLw8iXLQuGJMe1UtwkyKG
	return hex!["12a8b88f69ed1960f6fb77ad7937787c34432e459e933957d6c12106e4881122"].into();
}

/// Testnet balances
pub fn get_balances() -> Vec<(AccountId, Balance)> {
	return vec![
		(hex!["308a03fec4d787f08caf4fa7c569ad59e75c240a749a67704d35ed71c8298356"].into(), 10_000_000 * FIS),
		(hex!["c4d8c141c908b043a875e40a4a3ec930d60dfadc5a167a1f65970eaf34cb6e51"].into(), 100 * FIS),
		(hex!["924cc3a998e6b2293fc46281e78be2a6693437bf33fcec370a9167bc33fdf17b"].into(), 10_000_000 * FIS),
		(hex!["d409e31b8d45397b0b23d6955bb084ceae0c3635fd8e7fea58c46ffb389a7523"].into(), 100 * FIS),
		(hex!["841b98d00ff0f9548c5792d53c842e1d0e741bea470e22051ab98ae85b510833"].into(), 10_000_000 * FIS),
		(hex!["425ef3c6c4ca93e6047569bd61ebc0df15c9b54b460ddc4f28553c6c0ff1d518"].into(), 100 * FIS),
		(hex!["12a8b88f69ed1960f6fb77ad7937787c34432e459e933957d6c12106e4881122"].into(), 10 * FIS),
		(hex!["22aa4571470fafa5dc60cbfcde7b563929ddd271c9b06a40c09479a63edde430"].into(), 10_000_000 * FIS),
		(hex!["dfcf9666b6c1c4346507b77ce0a9da67717bb03b24602245b6157e48b1152981"].into(), 30_000_000 * FIS),
		(hex!["8cba31dbd353953c5e87c68fd30326c533de480c6c832e1c8d2828e3dd11215c"].into(), 30_000_000 * FIS),
	];
}

/// Testnet vestings
pub fn get_vestings() -> Vec<(AccountId, BlockNumber, BlockNumber, Balance)>{
	return vec![
		(
			hex!["22aa4571470fafa5dc60cbfcde7b563929ddd271c9b06a40c09479a63edde430"].into(),
			3000 as BlockNumber,
			500000 as BlockNumber,
			0 as Balance
		),
	];
}
