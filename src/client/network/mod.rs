// SPDX-License-Identifier: Apache-2.0

pub(super) mod managed;
pub(super) mod mirror;

use std::borrow::Cow;
use std::collections::{
    BTreeSet,
    HashMap,
};
use std::fmt;
use std::net::Ipv4Addr;
use std::num::NonZeroUsize;
use std::str::FromStr;
use std::time::{
    Duration,
    Instant,
};

use backoff::backoff::Backoff;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use rand::thread_rng;
use tonic::transport::{
    Channel,
    Endpoint,
};
use triomphe::Arc;

use crate::{
    AccountId,
    ArcSwap,
    Error,
    NodeAddressBook,
};

pub(crate) const MAINNET: &[(u64, &[&str])] = &[
    (3, &["13.124.142.126", "15.164.44.66", "15.165.118.251", "34.239.82.6", "35.237.200.180"]),
    (4, &["3.130.52.236", "35.186.191.247"]),
    (5, &["3.18.18.254", "23.111.186.250", "35.192.2.25", "74.50.117.35", "107.155.64.98"]),
    (6, &["13.52.108.243", "13.71.90.154", "35.199.161.108", "104.211.205.124"]),
    (7, &["3.114.54.4", "35.203.82.240"]),
    (8, &["35.183.66.150", "35.236.5.219"]),
    (9, &["35.181.158.250", "35.197.192.225"]),
    (10, &["3.248.27.48", "35.242.233.154", "177.154.62.234"]),
    (11, &["13.53.119.185", "35.240.118.96"]),
    (12, &["35.177.162.180", "35.204.86.32", "170.187.184.238"]),
    (13, &["34.215.192.104", "35.234.132.107"]),
    (14, &["35.236.2.27", "52.8.21.141"]),
    (15, &["3.121.238.26", "35.228.11.53"]),
    (16, &["18.157.223.230", "34.91.181.183"]),
    (17, &["18.232.251.19", "34.86.212.247"]),
    (18, &["141.94.175.187"]),
    (19, &["13.244.166.210", "13.246.51.42", "18.168.4.59", "34.89.87.138"]),
    (20, &["34.82.78.255", "52.39.162.216"]),
    (21, &["13.36.123.209", "34.76.140.109"]),
    (22, &["34.64.141.166", "52.78.202.34"]),
    (23, &["3.18.91.176", "35.232.244.145", "69.167.169.208"]),
    (24, &["18.135.7.211", "34.89.103.38"]),
    (25, &["13.232.240.207", "34.93.112.7"]),
    (26, &["13.228.103.14", "34.87.150.174"]),
    (27, &["13.56.4.96", "34.125.200.96"]),
    (28, &["18.139.47.5", "35.198.220.75"]),
    (29, &["34.142.71.129", "54.74.60.120", "80.85.70.197"]),
    (30, &["34.201.177.212", "35.234.249.150"]),
    (31, &["3.77.94.254", "34.107.78.179"]),
];

pub(crate) const TESTNET: &[(u64, &[&str])] = &[
    (3, &["0.testnet.hedera.com", "34.94.106.61", "50.18.132.211"]),
    (4, &["1.testnet.hedera.com", "35.237.119.55", "3.212.6.13"]),
    (5, &["2.testnet.hedera.com", "35.245.27.193", "52.20.18.86"]),
    (6, &["3.testnet.hedera.com", "34.83.112.116", "54.70.192.33"]),
    (7, &["4.testnet.hedera.com", "34.94.160.4", "54.176.199.109"]),
    (8, &["5.testnet.hedera.com", "34.106.102.218", "35.155.49.147"]),
    (9, &["6.testnet.hedera.com", "34.133.197.230", "52.14.252.207"]),
];

pub(crate) const PREVIEWNET: &[(u64, &[&str])] = &[
    (3, &["0.previewnet.hedera.com", "35.231.208.148", "3.211.248.172", "40.121.64.48"]),
    (4, &["1.previewnet.hedera.com", "35.199.15.177", "3.133.213.146", "40.70.11.202"]),
    (5, &["2.previewnet.hedera.com", "35.225.201.195", "52.15.105.130", "104.43.248.63"]),
    (6, &["3.previewnet.hedera.com", "35.247.109.135", "54.241.38.1", "13.88.22.47"]),
    (7, &["4.previewnet.hedera.com", "35.235.65.51", "54.177.51.127", "13.64.170.40"]),
    (8, &["5.previewnet.hedera.com", "34.106.247.65", "35.83.89.171", "13.78.232.192"]),
    (9, &["6.previewnet.hedera.com", "34.125.23.49", "50.18.17.93", "20.150.136.89"]),
];

#[derive(Default)]
pub(crate) struct Network(pub(crate) ArcSwap<NetworkData>);

impl Network {
    pub(super) fn mainnet() -> Self {
        NetworkData::from_static(MAINNET).into()
    }

    pub(super) fn testnet() -> Self {
        NetworkData::from_static(TESTNET).into()
    }

    pub(super) fn previewnet() -> Self {
        NetworkData::from_static(PREVIEWNET).into()
    }

    pub(super) fn from_addresses(addresses: &HashMap<String, AccountId>) -> crate::Result<Self> {
        Ok(NetworkData::from_addresses(addresses)?.into())
    }

    fn try_rcu<T: Into<Arc<NetworkData>>, E, F: FnMut(&Arc<NetworkData>) -> Result<T, E>>(
        &self,
        mut f: F,
    ) -> Result<Arc<NetworkData>, E> {
        // note: we can't use the `arc_swap` rcu function because we return a result
        let mut cur = self.0.load();
        loop {
            let new = f(&cur)?.into();
            let prev = self.0.compare_and_swap(&*cur, new);
            let swapped = Arc::ptr_eq(&*cur, &*prev);
            if swapped {
                return Ok(arc_swap::Guard::into_inner(cur));
            }

            cur = prev;
        }
    }

    fn rcu<T: Into<Arc<NetworkData>>, F: FnMut(&Arc<NetworkData>) -> T>(
        &self,
        mut f: F,
    ) -> Arc<NetworkData> {
        match self.try_rcu(|it| -> Result<T, std::convert::Infallible> { Ok(f(it)) }) {
            Ok(it) => it,
            Err(e) => match e {},
        }
    }

    pub(crate) fn update_from_addresses(
        &self,
        addresses: &HashMap<String, AccountId>,
    ) -> crate::Result<()> {
        self.try_rcu(|old| old.with_addresses(addresses))?;

        Ok(())
    }

    pub(crate) fn update_from_address_book(&self, address_book: &NodeAddressBook) {
        // todo: skip the updating whem `map` is the same and `connections` is the same.
        self.rcu(|old| NetworkData::with_address_book(old, address_book));
    }
}

impl From<NetworkData> for Network {
    fn from(value: NetworkData) -> Self {
        Self(ArcSwap::new(Arc::new(value)))
    }
}

// note: `Default` here is mostly only useful so that we don't need to implement `from_addresses` twice, notably this doesn't allocate.
#[derive(Default)]
pub(crate) struct NetworkData {
    map: HashMap<AccountId, usize>,
    node_ids: Box<[AccountId]>,
    backoff: RwLock<NodeBackoff>,
    // Health stuff has to be in an Arc because it needs to stick around even if the map changes.
    health: Box<[Arc<parking_lot::RwLock<NodeHealth>>]>,
    connections: Box<[NodeConnection]>,
}

impl NetworkData {
    pub(crate) fn from_addresses(addresses: &HashMap<String, AccountId>) -> crate::Result<Self> {
        Self::default().with_addresses(addresses)
    }

    pub(crate) fn from_static(network: &'static [(u64, &'static [&'static str])]) -> Self {
        let mut map = HashMap::with_capacity(network.len());
        let mut node_ids = Vec::with_capacity(network.len());
        let mut connections = Vec::with_capacity(network.len());
        let mut health = Vec::with_capacity(network.len());

        for (i, (num, address)) in network.iter().copied().enumerate() {
            let node_account_id = AccountId::from(num);

            map.insert(node_account_id, i);
            node_ids.push(node_account_id);
            health.push(Arc::default());
            connections.push(NodeConnection::new_static(address));
        }

        Self {
            map,
            node_ids: node_ids.into_boxed_slice(),
            health: health.into_boxed_slice(),
            connections: connections.into_boxed_slice(),
            backoff: NodeBackoff::default().into(),
        }
    }

    fn with_address_book(old: &Self, address_book: &NodeAddressBook) -> Self {
        let address_book = &address_book.node_addresses;

        let mut map = HashMap::with_capacity(address_book.len());
        let mut node_ids = Vec::with_capacity(address_book.len());
        let mut connections = Vec::with_capacity(address_book.len());
        let mut health = Vec::with_capacity(address_book.len());

        for (i, address) in address_book.iter().enumerate() {
            let new: BTreeSet<_> = address
                .service_endpoints
                .iter()
                .filter(|it| it.port() == NodeConnection::PLAINTEXT_PORT)
                .map(|it| (*it.ip()).into())
                .collect();

            // if the node is the exact same we want to reuse everything (namely the connections and `healthy`).
            // if the node has different routes then we still want to reuse `healthy` but replace the channel with a new channel.
            // if the node just flat out doesn't exist in `old`, we want to add the new node.
            // and, last but not least, if the node doesn't exist in `new` we want to get rid of it.
            let upsert = match old.map.get(&address.node_account_id) {
                Some(&account) => {
                    let connection =
                        match old.connections[account].addresses.symmetric_difference(&new).count()
                        {
                            0 => old.connections[account].clone(),
                            _ => NodeConnection { addresses: new, channel: OnceCell::new() },
                        };

                    (old.health[account].clone(), connection)
                }
                None => {
                    (Arc::default(), NodeConnection { addresses: new, channel: OnceCell::new() })
                }
            };

            map.insert(address.node_account_id, i);
            node_ids.push(address.node_account_id);
            health.push(upsert.0);
            connections.push(upsert.1);
        }

        Self {
            map,
            node_ids: node_ids.into_boxed_slice(),
            health: health.into_boxed_slice(),
            connections: connections.into_boxed_slice(),
            backoff: NodeBackoff::default().into(),
        }
    }

    fn with_addresses(&self, addresses: &HashMap<String, AccountId>) -> crate::Result<Self> {
        use std::collections::hash_map::Entry;
        let mut map: HashMap<AccountId, usize> = HashMap::new();
        let mut node_ids = Vec::new();
        let mut connections: Vec<NodeConnection> = Vec::new();
        let mut health = Vec::new();

        for (address, node) in addresses {
            let next_index = node_ids.len();

            let address = address.parse()?;

            match map.entry(*node) {
                Entry::Occupied(entry) => {
                    connections[*entry.get()].addresses.insert(address);
                }
                Entry::Vacant(entry) => {
                    entry.insert(next_index);
                    node_ids.push(*node);
                    // fixme: keep the channel around more.
                    connections.push(NodeConnection {
                        addresses: BTreeSet::from([address]),
                        channel: OnceCell::new(),
                    });

                    health.push(match self.map.get(node) {
                        Some(it) => self.health[*it].clone(),
                        None => Arc::default(),
                    });
                }
            };
        }

        Ok(Self {
            map,
            node_ids: node_ids.into_boxed_slice(),
            health: health.into_boxed_slice(),
            connections: connections.into_boxed_slice(),
            backoff: NodeBackoff::default().into(),
        })
    }

    pub(crate) fn node_ids(&self) -> &[AccountId] {
        &self.node_ids
    }

    pub(crate) fn node_indexes_for_ids(&self, ids: &[AccountId]) -> crate::Result<Vec<usize>> {
        let mut indexes = Vec::new();
        for id in ids {
            indexes.push(
                self.map
                    .get(id)
                    .copied()
                    .ok_or_else(|| Error::NodeAccountUnknown(Box::new(*id)))?,
            );
        }

        Ok(indexes)
    }

    // Sets the max attempts that an unhealthy node can retry
    pub(crate) fn set_max_node_attempts(&self, max_attempts: Option<NonZeroUsize>) {
        self.backoff.write().max_attempts = max_attempts
    }

    // Returns the max attempts that an unhealthy node can retry
    pub(crate) fn max_node_attempts(&self) -> Option<NonZeroUsize> {
        self.backoff.read().max_attempts
    }

    // Sets the max backoff for a node.
    pub(crate) fn set_max_backoff(&self, max_backoff: Duration) {
        self.backoff.write().max_backoff = max_backoff
    }

    // Return the initial backoff for a node.
    #[must_use]
    pub(crate) fn max_backoff(&self) -> Duration {
        self.backoff.read().max_backoff
    }

    // Sets the initial backoff for a request being executed.
    pub(crate) fn set_min_backoff(&self, min_backoff: Duration) {
        self.backoff.write().min_backoff = min_backoff
    }

    // Return the initial backoff for a request being executed.
    #[must_use]
    pub(crate) fn min_backoff(&self) -> Duration {
        self.backoff.read().min_backoff
    }

    pub(crate) fn mark_node_unhealthy(&self, node_index: usize) {
        let now = Instant::now();

        self.health[node_index].write().mark_unhealthy(*self.backoff.read(), now);
    }

    pub(crate) fn mark_node_healthy(&self, node_index: usize) {
        self.health[node_index].write().mark_healthy(Instant::now());
    }

    pub(crate) fn is_node_healthy(&self, node_index: usize, now: Instant) -> bool {
        // a healthy node has a healthiness before now.

        self.health[node_index].read().is_healthy(now)
    }

    pub(crate) fn node_recently_pinged(&self, node_index: usize, now: Instant) -> bool {
        self.health[node_index].read().recently_pinged(now)
    }

    pub(crate) fn healthy_node_indexes(&self, time: Instant) -> impl Iterator<Item = usize> + '_ {
        (0..self.node_ids.len()).filter(move |index| self.is_node_healthy(*index, time))
    }

    pub(crate) fn healthy_node_ids(&self) -> impl Iterator<Item = AccountId> + '_ {
        self.healthy_node_indexes(Instant::now()).map(|it| self.node_ids[it])
    }
    pub(crate) fn random_node_ids(&self) -> Vec<AccountId> {
        let mut node_ids: Vec<_> = self.healthy_node_ids().collect();
        // self.remove_dead_nodes();

        if node_ids.is_empty() {
            log::warn!("No healthy nodes, randomly picking some unhealthy ones");
            // hack, slowpath, don't care perf, fix this better later tho.
            node_ids = self.node_ids.to_vec();
        }

        let node_sample_amount = (node_ids.len() + 2) / 3;

        let node_id_indecies =
            rand::seq::index::sample(&mut thread_rng(), node_ids.len(), node_sample_amount);

        node_id_indecies.into_iter().map(|index| node_ids[index]).collect()
    }

    pub(crate) fn channel(&self, index: usize) -> (AccountId, Channel) {
        let id = self.node_ids[index];

        let channel = self.connections[index].channel();

        (id, channel)
    }

    pub(crate) fn addresses(&self) -> HashMap<String, AccountId> {
        self.map
            .iter()
            .flat_map(|(&account, &index)| {
                self.connections[index].addresses.iter().map(move |it| (it.to_string(), account))
            })
            .collect()
    }
}

#[derive(Default)]
enum NodeHealth {
    /// The node has never been used, so we don't know anything about it.
    ///
    /// However, we'll vaguely consider it healthy (`is_healthy` returns `true`).
    #[default]
    Unused,

    /// When we used or pinged the node we got some kind of error with it (like a BUSY response).
    ///
    /// Repeated errors cause the backoff to increase.
    ///
    /// Once we've reached `healthyAt` the node is *semantically* in the ``unused`` state,
    /// other than retaining the backoff until a `healthy` request happens.
    Unhealthy { backoff: NodeBackoff, healthy_at: Instant, attempts: usize },

    /// When we last used the node the node acted as normal, so, we get to treat it as a healthy node for 15 minutes.
    Healthy { used_at: Instant },
}

#[derive(Copy, Clone)]
pub(crate) struct NodeBackoff {
    pub(crate) current_interval: Duration,
    pub(crate) max_backoff: Duration,
    pub(crate) min_backoff: Duration,
    pub(crate) max_attempts: Option<NonZeroUsize>,
}

impl Default for NodeBackoff {
    fn default() -> Self {
        Self {
            current_interval: Duration::from_millis(250),
            max_backoff: Duration::from_secs(60 * 60),
            min_backoff: Duration::from_millis(250),
            max_attempts: NonZeroUsize::new(10),
        }
    }
}

impl NodeHealth {
    fn backoff(&self, backoff_config: NodeBackoff) -> (backoff::ExponentialBackoff, usize) {
        // If node is already labeled Unhealthy, preserve backoff and attempt amount
        // For new Unhealthy nodes, apply config and start attempt count at 0
        let (node_backoff, attempts) = match self {
            Self::Unhealthy { backoff, healthy_at: _, attempts } => (*backoff, attempts),
            _ => (backoff_config, &0),
        };

        (
            backoff::ExponentialBackoff {
                current_interval: node_backoff.current_interval,
                initial_interval: node_backoff.min_backoff,
                max_elapsed_time: None,
                max_interval: node_backoff.max_backoff,
                ..Default::default()
            },
            *attempts + 1,
        )
    }

    pub(crate) fn mark_unhealthy(&mut self, backoff_config: NodeBackoff, now: Instant) {
        let (mut backoff, unhealthy_node_attempts) = self.backoff(backoff_config);

        // Remove node if max_attempts has been reached and max_attempts is not 0
        if backoff_config
            .max_attempts
            .map_or(false, |max_attempts| unhealthy_node_attempts > max_attempts.get())
        {
            log::debug!("Node has reached the max amount of retries, removing from network")
        }

        // Generates the next current_interval with a random duration
        let next_backoff = backoff.next_backoff().expect("`max_elapsed_time` is hardwired to None");

        let healthy_at = now + next_backoff;

        *self = Self::Unhealthy {
            backoff: NodeBackoff {
                current_interval: next_backoff,
                max_backoff: backoff.max_interval,
                min_backoff: backoff.initial_interval,
                max_attempts: backoff_config.max_attempts,
            },
            healthy_at,
            attempts: unhealthy_node_attempts,
        };
    }

    pub(crate) fn mark_healthy(&mut self, now: Instant) {
        *self = Self::Healthy { used_at: now };
    }

    pub(crate) fn is_healthy(&self, now: Instant) -> bool {
        // a healthy node has a healthiness before now.
        match self {
            Self::Unhealthy { backoff: _, healthy_at, attempts: _ } => healthy_at < &now,
            _ => true,
        }
    }

    pub(crate) fn recently_pinged(&self, now: Instant) -> bool {
        match self {
            // when used at was less than 15 minutes ago we consider ourselves "pinged", otherwise we're basically `.unused`.
            Self::Healthy { used_at } => now < *used_at + Duration::from_secs(15 * 60),
            // likewise an unhealthy node (healthyAt > now) has been "pinged" (although we don't want to use it probably we at least *have* gotten *something* from it)
            Self::Unhealthy { backoff: _, healthy_at, attempts: _ } => now < *healthy_at,

            // an unused node is by definition not pinged.
            Self::Unused => false,
        }
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
struct HostAndPort {
    host: Cow<'static, str>,
    port: u16,
}

impl HostAndPort {
    const fn from_static(host: &'static str) -> Self {
        Self { host: Cow::Borrowed(host), port: NodeConnection::PLAINTEXT_PORT }
    }
}

impl FromStr for HostAndPort {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (host, port) = s.split_once(':').ok_or_else(|| Error::basic_parse("Invalid uri"))?;

        Ok(Self {
            host: Cow::Owned(host.to_owned()),
            port: port.parse().map_err(Error::basic_parse)?,
        })
    }
}

impl fmt::Display for HostAndPort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.host, self.port)
    }
}

impl From<Ipv4Addr> for HostAndPort {
    fn from(value: Ipv4Addr) -> Self {
        Self { host: Cow::Owned(value.to_string()), port: NodeConnection::PLAINTEXT_PORT }
    }
}

#[derive(Clone)]
struct NodeConnection {
    addresses: BTreeSet<HostAndPort>,
    channel: OnceCell<Channel>,
}

impl NodeConnection {
    const PLAINTEXT_PORT: u16 = 50211;

    fn new_static(addresses: &[&'static str]) -> NodeConnection {
        Self {
            addresses: addresses.iter().copied().map(HostAndPort::from_static).collect(),
            channel: OnceCell::default(),
        }
    }

    pub(crate) fn channel(&self) -> Channel {
        let channel = self
            .channel
            .get_or_init(|| {
                let addresses = self.addresses.iter().map(|it| {
                    Endpoint::from_shared(format!("tcp://{it}"))
                        .unwrap()
                        .keep_alive_timeout(Duration::from_secs(10))
                        .keep_alive_while_idle(true)
                        .tcp_keepalive(Some(Duration::from_secs(10)))
                        .connect_timeout(Duration::from_secs(10))
                });

                Channel::balance_list(addresses)
            })
            .clone();

        channel
    }
}
