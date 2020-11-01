// Copyright 2019-2020 Stafi Protocol.
// This file is part of Stafi.

// Stafi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stafi.  If not, see <http://www.gnu.org/licenses/>.

//! Stafi chain configurations.

use sc_chain_spec::ChainSpecExtension;
use sp_core::{Pair, Public, sr25519};
use serde::{Serialize, Deserialize};
use node_runtime::{
	AuthorityDiscoveryConfig, BabeConfig, BalancesConfig, CouncilConfig,
	DemocracyConfig,GrandpaConfig, ImOnlineConfig, SessionConfig, SessionKeys, StakerStatus,
	StakingConfig, ElectionsConfig, IndicesConfig, SudoConfig, SystemConfig,
	TechnicalCommitteeConfig, VestingConfig, wasm_binary_unwrap,
};
use node_runtime::Block;
use node_runtime::constants::currency::*;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use grandpa_primitives::{AuthorityId as GrandpaId};
use sp_consensus_babe::{AuthorityId as BabeId};
use pallet_im_online::sr25519::{AuthorityId as ImOnlineId};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_runtime::{Perbill, traits::{Verify, IdentifyAccount}};

pub use node_primitives::{AccountId, Balance, Signature, BlockNumber};
pub use node_runtime::GenesisConfig;

use std::fs::File;
use std::io::Read;
use std::path::Path;
use hex::FromHex;

type AccountPublic = <Signature as Verify>::Signer;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

const DEFAULT_PROTOCOL_ID: &str = "fis";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Stafi core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client_api::ForkBlocks<Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<Block>,
}

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::GenericChainSpec<
	GenesisConfig,
	Extensions,
>;

#[derive(Serialize, Deserialize)]
struct Allocation {
    balances: Vec<(String, String)>,
    vesting: Vec<(String, BlockNumber, BlockNumber, String)>,
}

fn get_drop_sitara_allocation() -> serde_json::Result<Allocation>{
	let path = Path::new("node/cli/res/drop-sitara.json");
	let mut file = File::open(&path).unwrap();
	let mut data = String::new();
	file.read_to_string(&mut data).unwrap();
	let a: Allocation = serde_json::from_str(&data)?;
	return Ok(a);
}

fn get_drop_mainnet_allocation() -> serde_json::Result<Allocation>{
	let path = Path::new("node/cli/res/drop-mainnet.json");
	let mut file = File::open(&path).unwrap();
	let mut data = String::new();
	file.read_to_string(&mut data).unwrap();
	let a: Allocation = serde_json::from_str(&data)?;
	return Ok(a);
}

fn properties() -> Option<sc_service::Properties> {
	let properties_json = r#"
		{
			"ss58Format": 20,
			"tokenDecimals": 12,
			"tokenSymbol": "FIS"
		}"#;
	serde_json::from_str(properties_json).unwrap()
}

/// Mainnet
pub fn stafi_mainnet_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/mainnet.json")[..])
}

/// Public testnet
pub fn stafi_testnet_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/testnet.json")[..])
}

/// Sitara testnet
pub fn stafi_sitara_testnet_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/sitara-testnet.json")[..])
}

fn session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
	SessionKeys { grandpa, babe, im_online, authority_discovery }
}

fn stafi_testnet_config_genesis() -> GenesisConfig {
	const INITIAL_STASH_STAKED: Balance = 1_000 * FIS;
	genesis(
		crate::testnet_fixtures::get_initial_authorities(),
		crate::testnet_fixtures::get_root_key(),
		crate::testnet_fixtures::get_balances(),
		crate::testnet_fixtures::get_vestings(),
		INITIAL_STASH_STAKED
	)
}

fn stafi_sitara_testnet_config_genesis() -> GenesisConfig {
	const INITIAL_STASH_STAKED: Balance = 1_000 * FIS;

	let allocation = get_drop_sitara_allocation().unwrap();
	let balances = allocation.balances.iter().map(|b| {
		let balance = b.1.to_string().parse::<Balance>().unwrap();
		return (
			<[u8; 32]>::from_hex(b.0.clone()).unwrap().into(),
			balance,
		);
	})
	.filter(|b| b.1 > 0)
	.collect();

	let vesting = allocation.vesting.iter().map(|v| {
		let vesting_balance = v.3.to_string().parse::<Balance>().unwrap();
		return (
			<[u8; 32]>::from_hex(v.0.clone()).unwrap().into(),
			v.1,
			v.2,
			vesting_balance,
		);
	})
	.collect();

	genesis(
		crate::testnet_fixtures::get_initial_authorities(),
		crate::testnet_fixtures::get_root_key(),
		balances,
		vesting,
		INITIAL_STASH_STAKED
	)
}

fn stafi_mainnet_config_genesis() -> GenesisConfig {
	const INITIAL_STASH_STAKED: Balance = 300_000 * FIS;

	let allocation = get_drop_mainnet_allocation().unwrap();
	let balances = allocation.balances.iter().map(|b| {
		let balance = b.1.to_string().parse::<Balance>().unwrap();
		return (
			<[u8; 32]>::from_hex(b.0.clone()).unwrap().into(),
			balance,
		);
	})
	.filter(|b| b.1 > 0)
	.collect();

	let vesting = allocation.vesting.iter().map(|v| {
		let vesting_balance = v.3.to_string().parse::<Balance>().unwrap();
		return (
			<[u8; 32]>::from_hex(v.0.clone()).unwrap().into(),
			v.1,
			v.2,
			vesting_balance,
		);
	})
	.collect();

	genesis(
		crate::mainnet_fixtures::get_initial_authorities(),
		crate::mainnet_fixtures::get_root_key(),
		balances,
		vesting,
		INITIAL_STASH_STAKED
	)
}

/// Seiya testnet config.
pub fn stafi_public_testnet_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Stafi Testnet Seiya",
		"stafi_public_testnet",
		ChainType::Live,
		stafi_testnet_config_genesis,
		crate::testnet_fixtures::get_bootnodes(),
		Some(TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
			.expect("Staging telemetry url is valid; qed")),
		Some(DEFAULT_PROTOCOL_ID),
		properties(),
		Default::default(),
	)
}

/// Sitara testnet config.
pub fn stafi_incentive_testnet_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Stafi Testnet Sitara2.0",
		"stafi_sitara2.0",
		ChainType::Live,
		stafi_sitara_testnet_config_genesis,
		crate::testnet_fixtures::get_sitara_bootnodes(),
		Some(TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
			.expect("Staging telemetry url is valid; qed")),
		Some(DEFAULT_PROTOCOL_ID),
		properties(),
		Default::default(),
	)
}

/// Mainnet config.
pub fn stafi_mainnet_spec_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Stafi",
		"stafi_mainnet",
		ChainType::Live,
		stafi_mainnet_config_genesis,
		crate::mainnet_fixtures::get_bootnodes(),
		Some(TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
			.expect("Mainnet telemetry url is valid; qed")),
		Some(DEFAULT_PROTOCOL_ID),
		properties(),
		Default::default(),
	)
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(seed: &str) -> (
	AccountId,
	AccountId,
	GrandpaId,
	BabeId,
	ImOnlineId,
	AuthorityDiscoveryId,
) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

/// Helper function to create GenesisConfig for testing
pub fn testnet_genesis(
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)>,
	root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
	_enable_println: bool,
) -> GenesisConfig {
	let endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Charlie"),
			get_account_id_from_seed::<sr25519::Public>("Dave"),
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
			get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
			get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
		]
	});
	// let num_endowed_accounts = endowed_accounts.len();

	const ENDOWMENT: Balance = 1_000_000 * FIS;
	const STASH: Balance = 100 * FIS;

	GenesisConfig {
		frame_system: Some(SystemConfig {
			code: wasm_binary_unwrap().to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_balances: Some(BalancesConfig {
			balances: endowed_accounts.iter().cloned()
				.map(|k| (k, ENDOWMENT))
				.chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH)))
				.collect(),
		}),
		pallet_indices: Some(IndicesConfig {
			indices: vec![],
		}),
		pallet_session: Some(SessionConfig {
			keys: initial_authorities.iter().map(|x| {
				(x.0.clone(), x.0.clone(), session_keys(
					x.2.clone(),
					x.3.clone(),
					x.4.clone(),
					x.5.clone(),
				))
			}).collect::<Vec<_>>(),
		}),
		pallet_staking: Some(StakingConfig {
			validator_count: initial_authorities.len() as u32 * 2,
			minimum_validator_count: initial_authorities.len() as u32,
			stakers: initial_authorities.iter().map(|x| {
				(x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator)
			}).collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			.. Default::default()
		}),
		pallet_democracy: Some(DemocracyConfig::default()),
		// pallet_elections_phragmen: Some(ElectionsConfig {
		// 	members: endowed_accounts.iter()
		// 				.take((num_endowed_accounts + 1) / 2)
		// 				.cloned()
		// 				.map(|member| (member, STASH))
		// 				.collect(),
		// }),
		pallet_elections_phragmen: Some(ElectionsConfig::default()),
		pallet_collective_Instance1: Some(CouncilConfig::default()),
		pallet_collective_Instance2: Some(TechnicalCommitteeConfig::default()),
		// pallet_collective_Instance2: Some(TechnicalCommitteeConfig {
		// 	members: endowed_accounts.iter()
		// 				.take((num_endowed_accounts + 1) / 2)
		// 				.cloned()
		// 				.collect(),
		// 	phantom: Default::default(),
		// }),
		pallet_sudo: Some(SudoConfig {
			key: root_key,
		}),
		pallet_babe: Some(BabeConfig {
			authorities: vec![],
		}),
		pallet_im_online: Some(ImOnlineConfig {
			keys: vec![],
		}),
		pallet_authority_discovery: Some(AuthorityDiscoveryConfig {
			keys: vec![],
		}),
		pallet_grandpa: Some(GrandpaConfig {
			authorities: vec![],
		}),
		pallet_membership_Instance1: Some(Default::default()),
		pallet_treasury: Some(Default::default()),
		pallet_vesting: Some(Default::default()),
	}
}

fn development_config_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![
			authority_keys_from_seed("Alice"),
		],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
		true,
	)
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Development",
		"dev",
		ChainType::Development,
		development_config_genesis,
		vec![],
		None,
		Some(DEFAULT_PROTOCOL_ID),
		properties(),
		Default::default(),
	)
}

fn local_testnet_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![
			authority_keys_from_seed("Alice"),
			authority_keys_from_seed("Bob"),
		],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
		false,
	)
}

/// Local testnet config (multivalidator Alice + Bob)
pub fn local_testnet_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Local Testnet",
		"local_testnet",
		ChainType::Local,
		local_testnet_genesis,
		vec![],
		None,
		Some(DEFAULT_PROTOCOL_ID),
		properties(),
		Default::default(),
	)
}

/// Helper function to create GenesisConfig
pub fn genesis(
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)>,
	root_key: AccountId,
	balances: Vec<(AccountId, Balance)>,
	vesting: Vec<(AccountId, BlockNumber, BlockNumber, Balance)>,
	initial_stash_staked: Balance
) -> GenesisConfig {

	GenesisConfig {
		frame_system: Some(SystemConfig {
			code: wasm_binary_unwrap().to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_balances: Some(BalancesConfig {
			balances: balances,
		}),
		pallet_indices: Some(IndicesConfig {
			indices: vec![],
		}),
		pallet_session: Some(SessionConfig {
			keys: initial_authorities.iter().map(|x| {
				(x.0.clone(), x.0.clone(), session_keys(
					x.2.clone(),
					x.3.clone(),
					x.4.clone(),
					x.5.clone(),
				))
			}).collect::<Vec<_>>(),
		}),
		pallet_staking: Some(StakingConfig {
			validator_count: 6,
			minimum_validator_count: 3,
			stakers: initial_authorities.iter().map(|x| {
				(x.0.clone(), x.1.clone(), initial_stash_staked, StakerStatus::Validator)
			}).collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			.. Default::default()
		}),
		pallet_democracy: Some(DemocracyConfig::default()),
		pallet_elections_phragmen: Some(ElectionsConfig::default()),
		pallet_collective_Instance1: Some(CouncilConfig::default()),
		pallet_collective_Instance2: Some(TechnicalCommitteeConfig::default()),
		pallet_sudo: Some(SudoConfig {
			key: root_key,
		}),
		pallet_babe: Some(BabeConfig {
			authorities: vec![],
		}),
		pallet_im_online: Some(ImOnlineConfig {
			keys: vec![],
		}),
		pallet_authority_discovery: Some(AuthorityDiscoveryConfig {
			keys: vec![],
		}),
		pallet_grandpa: Some(GrandpaConfig {
			authorities: vec![],
		}),
		pallet_membership_Instance1: Some(Default::default()),
		pallet_treasury: Some(Default::default()),
		pallet_vesting: Some(VestingConfig {
			vesting: vesting,
		}),
	}
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use crate::service::{new_full_base, new_light_base, NewFullBase};
	use sc_service_test;
	use sp_runtime::BuildStorage;

	fn local_testnet_genesis_instant_single() -> GenesisConfig {
		testnet_genesis(
			vec![
				authority_keys_from_seed("Alice"),
			],
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			None,
			false,
		)
	}

	/// Local testnet config (single validator - Alice)
	pub fn integration_test_config_with_single_authority() -> ChainSpec {
		ChainSpec::from_genesis(
			"Integration Test",
			"test",
			ChainType::Development,
			local_testnet_genesis_instant_single,
			vec![],
			None,
			None,
			None,
			Default::default(),
		)
	}

	/// Local testnet config (multivalidator Alice + Bob)
	pub fn integration_test_config_with_two_authorities() -> ChainSpec {
		ChainSpec::from_genesis(
			"Integration Test",
			"test",
			ChainType::Development,
			local_testnet_genesis,
			vec![],
			None,
			None,
			None,
			Default::default(),
		)
	}

	#[test]
	#[ignore]
	fn test_connectivity() {
		sc_service_test::connectivity(
			integration_test_config_with_two_authorities(),
			|config| {
				let NewFullBase { task_manager, client, network, transaction_pool, .. }
					= new_full_base(config,|_, _| ())?;
				Ok(sc_service_test::TestNetComponents::new(task_manager, client, network, transaction_pool))
			},
			|config| {
				let (keep_alive, _, client, network, transaction_pool) = new_light_base(config)?;
				Ok(sc_service_test::TestNetComponents::new(keep_alive, client, network, transaction_pool))
			}
		);
	}

	#[test]
	fn test_create_development_chain_spec() {
		development_config().build_storage().unwrap();
	}

	#[test]
	fn test_create_local_testnet_chain_spec() {
		local_testnet_config().build_storage().unwrap();
	}

	#[test]
	fn test_staging_test_net_chain_spec() {
		stafi_public_testnet_config().build_storage().unwrap();
	}
}
