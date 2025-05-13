// SPDX-License-Identifier: Apache-2.0

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::num::{
    NonZeroU64,
    NonZeroUsize,
};
use std::sync::atomic::{
    AtomicBool,
    AtomicU64,
    Ordering,
};
use std::time::Duration;

pub(crate) use network::{
    Network,
    NetworkData,
};
pub(crate) use operator::Operator;
use parking_lot::RwLock;
use tokio::sync::watch;
use triomphe::Arc;

use self::network::managed::ManagedNetwork;
use self::network::mirror::MirrorNetwork;
pub(crate) use self::network::mirror::MirrorNetworkData;
use crate::ping_query::PingQuery;
use crate::signer::AnySigner;
use crate::{
    AccountId,
    ArcSwapOption,
    Error,
    Hbar,
    LedgerId,
    NodeAddressBook,
    NodeAddressBookQuery,
    PrivateKey,
    PublicKey,
};

#[cfg(feature = "serde")]
mod config;

mod network;
mod operator;

#[derive(Copy, Clone)]
pub(crate) struct ClientBackoff {
    pub(crate) max_backoff: Duration,
    // min backoff.
    pub(crate) initial_backoff: Duration,
    pub(crate) max_attempts: usize,
    pub(crate) request_timeout: Option<Duration>,
    pub(crate) grpc_timeout: Option<Duration>,
}

impl Default for ClientBackoff {
    fn default() -> Self {
        Self {
            max_backoff: Duration::from_millis(backoff::default::MAX_INTERVAL_MILLIS),
            initial_backoff: Duration::from_millis(backoff::default::INITIAL_INTERVAL_MILLIS),
            max_attempts: 10,
            request_timeout: None,
            grpc_timeout: None,
        }
    }
}

// yes, client is complicated enough for this, even if it's only internal.
struct ClientBuilder {
    network: ManagedNetwork,
    operator: Option<Operator>,
    max_transaction_fee: Option<NonZeroU64>,
    max_query_payment: Option<NonZeroU64>,
    ledger_id: Option<LedgerId>,
    auto_validate_checksums: bool,
    regenerate_transaction_ids: bool,
    update_network: bool,
    backoff: ClientBackoff,
}

impl ClientBuilder {
    #[must_use]
    fn new(network: ManagedNetwork) -> Self {
        Self {
            network,
            operator: None,
            max_transaction_fee: None,
            max_query_payment: None,
            ledger_id: None,
            auto_validate_checksums: false,
            regenerate_transaction_ids: true,
            update_network: true,
            backoff: ClientBackoff::default(),
        }
    }

    fn disable_network_updating(self) -> Self {
        Self { update_network: false, ..self }
    }

    fn ledger_id(self, ledger_id: Option<LedgerId>) -> Self {
        Self { ledger_id, ..self }
    }

    fn build(self) -> Client {
        let Self {
            network,
            operator,
            max_transaction_fee,
            max_query_payment,
            ledger_id,
            auto_validate_checksums,
            regenerate_transaction_ids,
            update_network,
            backoff,
        } = self;

        let network_update_tx = match update_network {
            true => network::managed::spawn_network_update(
                network.clone(),
                Some(Duration::from_secs(24 * 60 * 60)),
            ),
            // yeah, we just drop the rx.
            false => watch::channel(None).0,
        };

        Client(Arc::new(ClientInner {
            network,
            operator: ArcSwapOption::new(operator.map(Arc::new)),
            max_transaction_fee_tinybar: AtomicU64::new(
                max_transaction_fee.map_or(0, NonZeroU64::get),
            ),
            max_query_payment_tinybar: AtomicU64::new(max_query_payment.map_or(0, NonZeroU64::get)),
            ledger_id: ArcSwapOption::new(ledger_id.map(Arc::new)),
            auto_validate_checksums: AtomicBool::new(auto_validate_checksums),
            regenerate_transaction_ids: AtomicBool::new(regenerate_transaction_ids),
            network_update_tx,
            backoff: RwLock::new(backoff),
        }))
    }
}

struct ClientInner {
    network: ManagedNetwork,
    operator: ArcSwapOption<Operator>,
    max_transaction_fee_tinybar: AtomicU64,
    max_query_payment_tinybar: AtomicU64,
    ledger_id: ArcSwapOption<LedgerId>,
    auto_validate_checksums: AtomicBool,
    regenerate_transaction_ids: AtomicBool,
    network_update_tx: watch::Sender<Option<Duration>>,
    backoff: RwLock<ClientBackoff>,
}

/// Managed client for use on the Hiero network.
#[derive(Clone)]
pub struct Client(Arc<ClientInner>);

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // todo: put anything important here.
        f.debug_struct("Client").finish_non_exhaustive()
    }
}

impl Client {
    #[cfg(feature = "serde")]
    fn from_config_data(config: config::ClientConfig) -> crate::Result<Self> {
        let config::ClientConfig { operator, network, mirror_network } = config;

        // fixme: check to ensure net and mirror net are the same when they're a network name (no other SDK actually checks this though)
        let client = match network {
            config::Either::Left(network) => Client::for_network(network)?,
            config::Either::Right(it) => match it {
                config::NetworkName::Mainnet => Client::for_mainnet(),
                config::NetworkName::Testnet => Client::for_testnet(),
                config::NetworkName::Previewnet => Client::for_previewnet(),
            },
        };

        let mirror_network = mirror_network.map(|mirror_network| match mirror_network {
            config::Either::Left(mirror_network) => {
                MirrorNetwork::from_addresses(mirror_network.into_iter().map(Cow::Owned).collect())
            }
            config::Either::Right(it) => match it {
                config::NetworkName::Mainnet => MirrorNetwork::mainnet(),
                config::NetworkName::Testnet => MirrorNetwork::testnet(),
                config::NetworkName::Previewnet => MirrorNetwork::previewnet(),
            },
        });

        if let Some(operator) = operator {
            client.0.operator.store(Some(Arc::new(operator)));
        }

        if let Some(mirror_network) = mirror_network {
            client.set_mirror_network(mirror_network.load().addresses());
        }

        Ok(client)
    }

    /// Create a client from the given json config.
    ///
    /// # Errors
    /// - [`Error::BasicParse`] if an error occurs parsing the configuration.
    #[cfg(feature = "serde")]
    pub fn from_config(json: &str) -> crate::Result<Self> {
        let config = serde_json::from_str::<config::ClientConfigInner>(json)
            .map_err(crate::Error::basic_parse)?
            .into();

        Self::from_config_data(config)
    }

    /// Returns the addresses for the configured mirror network.
    ///
    /// Unless _explicitly_ set, the return value isn't guaranteed to be anything in particular in order to allow future changes without breaking semver.
    /// However, when a function such as `for_testnet` is used, _some_ valid value will be returned.
    ///
    /// Current return values (reminder that these are semver exempt)
    ///
    /// - mainnet: `["mainnet-public.mirrornode.hedera.com:443"]`
    /// - testnet: `["testnet.mirrornode.hedera.com:443"]`
    /// - previewnet: `["previewnet.mirrornode.hedera.com:443"]`
    ///
    /// # Examples
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() {
    /// use hedera::Client;
    ///
    /// let client = Client::for_testnet();
    ///
    /// // note: This isn't *guaranteed* in a semver sense, but this is the current result.
    /// let expected = Vec::from(["testnet.mirrornode.hedera.com:443".to_owned()]);
    /// assert_eq!(expected, client.mirror_network());
    ///
    /// # }
    /// ```
    #[must_use]
    pub fn mirror_network(&self) -> Vec<String> {
        self.mirrornet().load().addresses().collect()
    }

    /// Sets the addresses to use for the mirror network.
    ///
    /// This is mostly useful if you used [`Self::for_network`] and need to set a mirror network.
    pub fn set_mirror_network<I: IntoIterator<Item = String>>(&self, addresses: I) {
        self.mirrornet().store(
            MirrorNetworkData::from_addresses(addresses.into_iter().map(Cow::Owned).collect())
                .into(),
        );
    }

    /// Construct a client with the given nodes configured.
    ///
    /// Note that this disables network auto-updating.
    ///
    /// # Errors
    /// - [`Error::BasicParse`] if an error occurs parsing the configuration.
    // allowed for API compatibility.
    #[allow(clippy::needless_pass_by_value)]
    pub fn for_network(network: HashMap<String, AccountId>) -> crate::Result<Self> {
        let network =
            ManagedNetwork::new(Network::from_addresses(&network)?, MirrorNetwork::default());

        Ok(ClientBuilder::new(network).disable_network_updating().build())
    }

    /// Construct a client from a select mirror network
    pub async fn for_mirror_network(mirror_networks: Vec<String>) -> crate::Result<Self> {
        let network_addresses: HashMap<String, AccountId> = HashMap::new();
        let network = ManagedNetwork::new(
            Network::from_addresses(&network_addresses)?,
            MirrorNetwork::from_addresses(mirror_networks.into_iter().map(Cow::Owned).collect()),
        );

        let client = ClientBuilder::new(network).build();
        let address_book = NodeAddressBookQuery::default().execute(&client).await?;

        client.set_network_from_address_book(address_book);

        Ok(client)
    }

    /// Construct a Hiero client pre-configured for mainnet access.
    #[must_use]
    pub fn for_mainnet() -> Self {
        ClientBuilder::new(ManagedNetwork::mainnet()).ledger_id(Some(LedgerId::mainnet())).build()
    }

    /// Construct a Hiero client pre-configured for testnet access.
    #[must_use]
    pub fn for_testnet() -> Self {
        ClientBuilder::new(ManagedNetwork::testnet()).ledger_id(Some(LedgerId::testnet())).build()
    }

    /// Construct a Hiero client pre-configured for previewnet access.
    #[must_use]
    pub fn for_previewnet() -> Self {
        ClientBuilder::new(ManagedNetwork::previewnet())
            .ledger_id(Some(LedgerId::previewnet()))
            .build()
    }

    /// Updates the network to use the given address book.
    ///
    /// Note: This is only really useful if you used `for_network`, because the network can auto-update.
    ///
    /// If network auto-updating is enabled this will eventually be overridden.
    // allowed for API compatibility.
    #[allow(clippy::needless_pass_by_value)]
    pub fn set_network_from_address_book(&self, address_book: NodeAddressBook) {
        self.net().update_from_address_book(&address_book);
    }

    /// Updates the network to use the given addresses.
    ///
    /// Note: This is only really useful if you used `for_network`, because the network can auto-update.
    ///
    /// If network auto-updating is enabled this will eventually be overridden.
    ///
    /// Tend to prefer [`set_network_from_address_book`](Self::set_network_from_address_book) where possible.
    ///
    /// # Errors
    /// [`Error::BasicParse`](crate::Error::BasicParse) If any node address is unparsable.
    // allowed for API compatibility.
    #[allow(clippy::needless_pass_by_value)]
    pub fn set_network(&self, network: HashMap<String, AccountId>) -> crate::Result<()> {
        self.net().update_from_addresses(&network)?;

        Ok(())
    }

    /// Returns the nodes associated with this client.
    #[must_use]
    pub fn network(&self) -> HashMap<String, AccountId> {
        self.net().0.load().addresses()
    }

    /// Returns the max number of times a node can be retried before removing it from the network.
    pub fn max_node_attempts(&self) -> Option<NonZeroUsize> {
        self.net().0.load().max_node_attempts()
    }

    /// Set the max number of times a node can return a bad gRPC status before we remove it from the list.
    pub fn set_max_node_attempts(&self, attempts: usize) {
        self.net().0.load().set_max_node_attempts(NonZeroUsize::new(attempts))
    }

    /// Returns the max backoff interval for network nodes if gRPC response fail.    
    pub fn max_node_backoff(&self) -> Duration {
        self.net().0.load().max_backoff()
    }

    /// Sets max backoff interval for network nodes
    pub fn set_max_node_backoff(&self, max_node_backoff: Duration) {
        self.net().0.load().set_max_backoff(max_node_backoff)
    }

    /// Returns the initial backoff interval for network nodes if gRPC response fail.    
    pub fn min_node_backoff(&self) -> Duration {
        self.net().0.load().min_backoff()
    }

    /// Sets initial backoff interval for network nodes
    pub fn set_min_node_backoff(&self, min_node_backoff: Duration) {
        self.net().0.load().set_min_backoff(min_node_backoff)
    }

    /// Construct a hedera client pre-configured for access to the given network.
    ///
    /// Currently supported network names are `"mainnet"`, `"testnet"`, and `"previewnet"`.
    ///
    /// # Errors
    /// - [`Error::BasicParse`] if the network name is not a supported network name.
    pub fn for_name(name: &str) -> crate::Result<Self> {
        match name {
            "mainnet" => Ok(Self::for_mainnet()),
            "testnet" => Ok(Self::for_testnet()),
            "previewnet" => Ok(Self::for_previewnet()),
            "localhost" => {
                let mut network: HashMap<String, AccountId> = HashMap::new();
                network.insert("127.0.0.1:50211".to_string(), AccountId::new(0, 0, 3));

                let client = Client::for_network(network).unwrap();
                client.set_mirror_network(["127.0.0.1:5600".to_string()]);
                Ok(client)
            }
            _ => Err(Error::basic_parse(format!("Unknown network name {name}"))),
        }
    }

    // optimized function to avoid allocations/pointer chasing.
    // this shouldn't be exposed because it exposes repr.
    pub(crate) fn ledger_id_internal(&self) -> arc_swap::Guard<Option<Arc<LedgerId>>> {
        self.0.ledger_id.load()
    }

    /// Sets the ledger ID for the Client's network.
    pub fn set_ledger_id(&self, ledger_id: Option<LedgerId>) {
        self.0.ledger_id.store(ledger_id.map(Arc::new));
    }

    /// Returns true if checksums should be automatically validated.
    #[must_use]
    pub fn auto_validate_checksums(&self) -> bool {
        self.0.auto_validate_checksums.load(Ordering::Relaxed)
    }

    /// Enable or disable automatic entity ID checksum validation.
    pub fn set_auto_validate_checksums(&self, value: bool) {
        self.0.auto_validate_checksums.store(value, Ordering::Relaxed);
    }

    /// Returns true if transaction IDs should be automatically regenerated.
    ///
    /// This is `true` by default.
    #[must_use]
    pub fn default_regenerate_transaction_id(&self) -> bool {
        self.0.regenerate_transaction_ids.load(Ordering::Relaxed)
    }

    /// Enable or disable transaction ID regeneration.
    pub fn set_default_regenerate_transaction_id(&self, value: bool) {
        self.0.regenerate_transaction_ids.store(value, Ordering::Relaxed);
    }

    /// Sets the account that will, by default, be paying for transactions and queries built with
    /// this client.
    ///
    /// The operator account ID is used to generate the default transaction ID for all transactions
    /// executed with this client.
    ///
    /// The operator private key is used to sign all transactions executed by this client.
    pub fn set_operator(&self, id: AccountId, key: PrivateKey) {
        self.0
            .operator
            .store(Some(Arc::new(Operator { account_id: id, signer: AnySigner::PrivateKey(key) })));
    }

    /// Sets the account that will, by default, be paying for transactions and queries built with
    /// this client.
    ///
    /// The operator account ID is used to generate the default transaction ID for all transactions
    /// executed with this client.
    ///
    /// The operator signer is used to sign all transactions executed by this client.
    pub fn set_operator_with<F: Fn(&[u8]) -> Vec<u8> + Send + Sync + 'static>(
        &self,
        id: AccountId,
        public_key: PublicKey,
        f: F,
    ) {
        self.0.operator.store(Some(Arc::new(Operator {
            account_id: id,
            signer: AnySigner::arbitrary(Box::new(public_key), f),
        })));
    }

    /// Gets a reference to the configured network.
    pub(crate) fn net(&self) -> &Network {
        &self.0.network.primary
    }

    /// Gets a reference to the configured mirror network.
    pub(crate) fn mirrornet(&self) -> &MirrorNetwork {
        &self.0.network.mirror
    }

    /// Sets the maximum transaction fee to be used when no explicit max transaction fee is set.
    ///
    /// Note: Setting `amount` to zero is "unlimited"
    /// # Panics
    /// - if amount is negative
    pub fn set_default_max_transaction_fee(&self, amount: Hbar) {
        assert!(amount >= Hbar::ZERO);
        self.0.max_transaction_fee_tinybar.store(amount.to_tinybars() as u64, Ordering::Relaxed);
    }

    /// Gets the maximum transaction fee the paying account is willing to pay.
    #[must_use]
    pub fn default_max_transaction_fee(&self) -> Option<Hbar> {
        let val = self.0.max_transaction_fee_tinybar.load(Ordering::Relaxed);

        (val > 0).then(|| Hbar::from_tinybars(val as i64))
    }

    /// Gets the maximum query fee the paying account is willing to pay.
    #[must_use]
    pub fn default_max_query_payment(&self) -> Option<Hbar> {
        let val = self.0.max_query_payment_tinybar.load(Ordering::Relaxed);

        (val > 0).then(|| Hbar::from_tinybars(val as i64))
    }

    /// Sets the maximum query payment to be used when no explicit max query payment is set.
    ///
    /// Note: Setting `amount` to zero is "unlimited"
    /// # Panics
    /// - if amount is negative
    pub fn set_default_max_query_payment(&self, amount: Hbar) {
        assert!(amount >= Hbar::ZERO);
        self.0.max_query_payment_tinybar.store(amount.to_tinybars() as u64, Ordering::Relaxed);
    }

    /// Returns the maximum amount of time that will be spent on a request.
    #[must_use]
    pub fn request_timeout(&self) -> Option<Duration> {
        self.backoff().request_timeout
    }

    /// Sets the maximum amount of time that will be spent on a request.
    pub fn set_request_timeout(&self, timeout: Option<Duration>) {
        self.0.backoff.write().request_timeout = timeout;
    }

    /// Returns the maximum number of attempts for a request.
    #[must_use]
    pub fn max_attempts(&self) -> usize {
        self.backoff().max_attempts
    }

    /// Sets the maximum number of attempts for a request.
    pub fn set_max_attempts(&self, max_attempts: usize) {
        self.0.backoff.write().max_attempts = max_attempts;
    }

    /// The initial backoff for a request being executed.
    #[doc(alias = "initial_backoff")]
    #[must_use]
    pub fn min_backoff(&self) -> Duration {
        self.backoff().initial_backoff
    }

    /// Sets the initial backoff for a request being executed.
    #[doc(alias = "set_initial_backoff")]
    pub fn set_min_backoff(&self, max_backoff: Duration) {
        self.0.backoff.write().max_backoff = max_backoff;
    }

    /// Returns the maximum amount of time a request will wait between attempts.
    #[must_use]
    pub fn max_backoff(&self) -> Duration {
        self.backoff().max_backoff
    }

    /// Sets the maximum amount of time a request will wait between attempts.
    pub fn set_max_backoff(&self, max_backoff: Duration) {
        self.0.backoff.write().max_backoff = max_backoff;
    }

    #[must_use]
    pub(crate) fn backoff(&self) -> ClientBackoff {
        *self.0.backoff.read()
    }

    // keep this internal (repr)
    pub(crate) fn load_operator(&self) -> arc_swap::Guard<Option<Arc<Operator>>> {
        self.0.operator.load()
    }

    // keep this internal (repr)
    pub(crate) fn full_load_operator(&self) -> Option<Arc<Operator>> {
        self.0.operator.load_full()
    }

    /// Send a ping to the given node.
    pub async fn ping(&self, node_account_id: AccountId) -> crate::Result<()> {
        PingQuery::new(node_account_id).execute(self, None).await
    }

    /// Send a ping to the given node, canceling the ping after `timeout` has elapsed.
    pub async fn ping_with_timeout(
        &self,
        node_account_id: AccountId,
        timeout: Duration,
    ) -> crate::Result<()> {
        PingQuery::new(node_account_id).execute(self, Some(timeout)).await
    }

    /// Send a ping to all nodes.
    pub async fn ping_all(&self) -> crate::Result<()> {
        futures_util::future::try_join_all(
            self.net().0.load().node_ids().iter().map(|it| self.ping(*it)),
        )
        .await?;

        Ok(())
    }

    /// Send a ping to all nodes, canceling the ping after `timeout` has elapsed.
    pub async fn ping_all_with_timeout(&self, timeout: Duration) -> crate::Result<()> {
        futures_util::future::try_join_all(
            self.net().0.load().node_ids().iter().map(|it| self.ping_with_timeout(*it, timeout)),
        )
        .await?;

        Ok(())
    }

    /// Returns the frequency at which the network will update (if it will update at all).
    #[must_use = "this function has no side-effects"]
    pub fn network_update_period(&self) -> Option<Duration> {
        *self.0.network_update_tx.borrow()
    }

    /// Sets the frequency at which the network will update.
    ///
    /// Note that network updates will not affect any in-flight requests.
    pub fn set_network_update_period(&self, period: Option<Duration>) {
        self.0.network_update_tx.send_if_modified(|place| {
            let changed = *place == period;
            if changed {
                *place = period;
            }

            changed
        });
    }

    /// Returns the Account ID for the operator.
    #[must_use]
    pub fn get_operator_account_id(&self) -> Option<AccountId> {
        self.load_operator().as_deref().map(|it| it.account_id)
    }

    /// Returns the `PublicKey` for the current operator.
    #[must_use]
    pub fn get_operator_public_key(&self) -> Option<PublicKey> {
        self.load_operator().as_deref().map(|it| it.signer.public_key())
    }
}
