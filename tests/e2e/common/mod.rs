use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;

use anyhow::Context;
use hedera::{
    AccountId,
    Client,
    PrivateKey,
};
use once_cell::sync::Lazy;

mod keys {
    pub(super) const NETWORK: &str = "TEST_NETWORK_NAME";

    pub(super) const OPERATOR_KEY: &str = "TEST_OPERATOR_KEY";

    pub(super) const OPERATOR_ID: &str = "TEST_OPERATOR_ID";

    pub(super) const RUN_NONFREE: &str = "TEST_RUN_NONFREE";
}

static CONFIG: Lazy<Config> = Lazy::new(Config::parse_env);
static DEFAULT_LOCAL_NODE_ADDRESS: &str = "127.0.0.1:50211";
static DEFAULT_LOCAL_MIRROR_NODE_ADDRESS: &str = "127.0.0.1:5600";

/// Generates a client using the active config.
///
/// This is a function rather than a `Lazy` because every executor (IE, [`#[tokio::test]`](tokio::test)) needs its own client.
fn client() -> Client {
    let config = &*CONFIG;

    let client = match &*config.network_name {
        "mainnet" => Client::for_mainnet().unwrap(),
        "testnet" => Client::for_testnet().unwrap(),
        "previewnet" => Client::for_previewnet().unwrap(),
        "localhost" => for_local_node(),
        _ => {
            // to ensure we don't spam the logs with `Error creating client: ...`,
            // we just let an arbitrary thread win and log the "error".
            static LOGS_ONCE: AtomicBool = AtomicBool::new(false);

            // note: Relaxed is probably fine, AcqRel is *definitely* fine.
            if !LOGS_ONCE.swap(true, std::sync::atomic::Ordering::AcqRel) {
                log::error!(
                    "Error creating client: {}; creating one using `testnet`",
                    &*config.network_name
                );
            }

            Client::for_testnet().unwrap()
        }
    };

    if let Some(op) = &config.operator {
        client.set_operator(op.account_id, op.private_key.clone());
    }

    client
}

fn for_local_node() -> Client {
    let mut network: HashMap<String, AccountId> = HashMap::new();
    network.insert(DEFAULT_LOCAL_NODE_ADDRESS.to_string(), AccountId::new(0, 0, 3));

    let client = Client::for_network(network).unwrap();
    client.set_mirror_network([DEFAULT_LOCAL_MIRROR_NODE_ADDRESS.to_string()]);
    client
}

#[derive(Clone)]
pub(crate) struct Operator {
    pub(crate) private_key: PrivateKey,
    pub(crate) account_id: AccountId,
}

impl Operator {
    fn try_parse_env() -> anyhow::Result<Option<Self>> {
        let key = dotenvy::var(keys::OPERATOR_KEY).ok();
        let account_id = dotenvy::var(keys::OPERATOR_ID).ok();

        // note: intentionally avoiding zip in order to log warnings
        let (key, account_id) = match (key, account_id) {
            (Some(key), Some(account_id)) => (key, account_id),

            (Some(_), None) => {
                anyhow::bail!("operator key was set but the account id was not")
            }

            (None, Some(_)) => {
                anyhow::bail!("operator account id was set but the key was not")
            }

            (None, None) => return Ok(None),
        };

        let key = key.parse().context("failed to parse operator key")?;
        let account_id = account_id.parse().context("failed to parse operator account id")?;

        Ok(Some(Self { private_key: key, account_id }))
    }

    fn parse_env() -> Option<Self> {
        match Self::try_parse_env() {
            Ok(res) => res,
            Err(e) => {
                log::warn!("error occurred while parsing operator: {e:?}; ignoring operator");

                None
            }
        }
    }
}

pub(crate) struct Config {
    /// Name of the network used
    pub(crate) network_name: Cow<'static, str>,

    /// The operator to use for non-free transactions / queries,
    /// however it is also a source of a known account / private key for things like `AccountBalanceQuery`.
    pub(crate) operator: Option<Operator>,

    /// A setting to allow tests that cost Hbar to run.
    ///
    /// If this is set and an operator is not provided, a warning will be logged and this will be forcibly disabled.
    pub(crate) run_nonfree_tests: bool,

    /// A flag signifying that local node is the current testing environment.
    pub(crate) is_local: bool,
}

/// Returns true if the provided env var is
fn env_bool(default: bool, var_name: &str) -> bool {
    let Some(var) = dotenvy::var(var_name).ok() else {
        return default;
    };

    if var.as_str() == "1" {
        return true;
    }

    if var.as_str() == "0" {
        return false;
    }

    log::warn!("expected `{var_name}` to be `1` or `0` but it was `{var}`, returning `{default}`");

    default
}

impl Config {
    fn parse_env() -> Self {
        let network_name = dotenvy::var(keys::NETWORK).ok();

        let mut is_local = false;

        // default tests to localhost
        let network_name = network_name.map_or_else(|| Cow::Borrowed("testnet"), Cow::Owned);

        if network_name == "localhost" {
            is_local = true
        }

        let operator = Operator::parse_env();

        let run_nonfree_tests = env_bool(false, keys::RUN_NONFREE);

        Self { network_name, operator, run_nonfree_tests, is_local }
    }
}

pub(crate) struct TestEnvironment {
    pub(crate) config: &'static Config,
    pub(crate) client: Client,
}

pub(crate) fn setup_global() -> TestEnvironment {
    let _ = dotenvy::dotenv();

    let _ = env_logger::builder().parse_default_env().is_test(true).try_init();

    TestEnvironment { config: &CONFIG, client: client() }
}

pub(crate) fn setup_nonfree() -> Option<TestEnvironment> {
    let env = setup_global();

    match env.config.run_nonfree_tests {
        true => Some(env),
        false => {
            log::debug!("skipping non-free test");
            None
        }
    }
}
