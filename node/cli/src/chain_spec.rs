// Copyright 2018 Stafi Protocol, Inc.
// This file is part of Stafi.

// Stafi is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Stafi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stafi.  If not, see <http://www.gnu.org/licenses/>

use chain_spec::ChainSpecExtension;
use primitives::{Pair, Public, sr25519};
use serde::{Serialize, Deserialize};
use node_runtime::{
	AuthorityDiscoveryConfig, BabeConfig, BalancesConfig, ContractsConfig, CouncilConfig, DemocracyConfig,
	GrandpaConfig, ImOnlineConfig, IndicesConfig, SessionConfig, SessionKeys, StakerStatus, StakingConfig, SudoConfig,
	SystemConfig, TechnicalCommitteeConfig, WASM_BINARY, MultisigAddressConfig,
};
use node_runtime::Block;
use node_primitives::{constants::currency::*, ChainType};
use substrate_service;
use substrate_telemetry::TelemetryEndpoints;
use grandpa_primitives::{AuthorityId as GrandpaId};
use babe_primitives::{AuthorityId as BabeId};
use im_online::sr25519::{AuthorityId as ImOnlineId};
use authority_discovery_primitives::AuthorityId as AuthorityDiscoveryId;
use sr_primitives::{Perbill, traits::{Verify, IdentifyAccount}};

pub use node_primitives::{AccountId, Balance, Signature, MultisigAddr};
pub use node_runtime::GenesisConfig;
use crate::fixtures::*;

type AccountPublic = <Signature as Verify>::Signer;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

const DEFAULT_PROTOCOL_ID: &str = "fis";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: client::ForkBlocks<Block>,
}

/// Specialized `ChainSpec`.
pub type ChainSpec = substrate_service::ChainSpec<
	GenesisConfig,
	Extensions,
>;

/// Stafi generator
pub fn stafi_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../../testnets/v0.1.0/stafi.json")[..])
}

fn session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
	SessionKeys { grandpa, babe, im_online, authority_discovery }
}

/// Stafi testnet generator
pub fn stafi_testnet_config_gensis() -> GenesisConfig {
    let initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)> = get_vals();
    let root_key = get_root_key();

    let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

	testnet_genesis(
        initial_authorities,
        root_key,
        Some(endowed_accounts),
		false
    )
}

/// Stafi testnet generator
pub fn stafi_testnet_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Stafi Testnet",
        "stafi_testnet",
        stafi_testnet_config_gensis,
        vec![],
        Some(TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])),
        Some(DEFAULT_PROTOCOL_ID),
        None,
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
pub fn get_authority_keys_from_seed(seed: &str) -> (
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
    initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)>,
    root_key: AccountId,
    endowed_accounts: Option<Vec<AccountId>>,
	enable_println: bool,
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

    const ENDOWMENT: Balance = 10_000_000 * DOLLARS;
    const STASH: Balance = 100 * DOLLARS;

    GenesisConfig {
        system: Some(SystemConfig {
			code: WASM_BINARY.to_vec(),
			changes_trie_config: Default::default(),
		}),
		balances: Some(BalancesConfig {
			balances: endowed_accounts.iter().cloned()
				.map(|k| (k, ENDOWMENT))
				.chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH)))
				.collect(),
			vesting: vec![],
		}),
		indices: Some(IndicesConfig {
			ids: endowed_accounts.iter().cloned()
				.chain(initial_authorities.iter().map(|x| x.0.clone()))
				.collect::<Vec<_>>(),
		}),
		session: Some(SessionConfig {
			keys: initial_authorities.iter().map(|x| {
				(x.0.clone(), session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()))
			}).collect::<Vec<_>>(),
		}),
		staking: Some(StakingConfig {
			current_era: 0,
			validator_count: initial_authorities.len() as u32 * 2,
			minimum_validator_count: initial_authorities.len() as u32,
			stakers: initial_authorities.iter().map(|x| {
				(x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator)
			}).collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			.. Default::default()
		}),
		democracy: Some(DemocracyConfig::default()),
		collective_Instance1: Some(CouncilConfig {
			members: vec![],
			phantom: Default::default(),
		}),
		collective_Instance2: Some(TechnicalCommitteeConfig {
			members: vec![],
			phantom: Default::default(),
		}),
		contracts: Some(ContractsConfig {
			current_schedule: contracts::Schedule {
				enable_println, // this should only be enabled on development chains
				..Default::default()
			},
			gas_price: 1 * MILLICENTS,
		}),
		sudo: Some(SudoConfig {
			key: root_key,
		}),
		babe: Some(BabeConfig {
			authorities: vec![],
		}),
		im_online: Some(ImOnlineConfig {
			keys: vec![],
		}),
		authority_discovery: Some(AuthorityDiscoveryConfig {
			keys: vec![],
		}),
		grandpa: Some(GrandpaConfig {
			authorities: vec![],
		}),
		multisigAddr: Some(MultisigAddressConfig {
			addrs: [
				MultisigAddr {
					chain_type: ChainType::TEZOS,
					multisig_addr: "tz1RYze45x9NpyvYyLN6z1tHoSPv6A4LpFrf".as_bytes().to_vec()
				},
				MultisigAddr {
					chain_type: ChainType::COSMOS,
					multisig_addr: "567".as_bytes().to_vec()
				},
			].to_vec()
		}),
		membership_Instance1: Some(Default::default()),
        treasury: Some(Default::default()),
    }
}

fn development_config_genesis() -> GenesisConfig {
    testnet_genesis(
        vec![
            get_authority_keys_from_seed("Alice"),
        ],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        None,
		true
    )
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Development",
        "dev",
        development_config_genesis,
        vec![],
        None,
        None,
        None,
        Default::default(),
	)
}

fn local_testnet_genesis() -> GenesisConfig {
    testnet_genesis(
        vec![
            get_authority_keys_from_seed("Alice"),
            get_authority_keys_from_seed("Bob"),
        ],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        None,
		false
    )
}

/// Local testnet config (multivalidator Alice + Bob)
pub fn local_testnet_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Local Testnet",
        "local_testnet",
        local_testnet_genesis,
        vec![],
        None,
        None,
        None,
        Default::default(),
	)
}
