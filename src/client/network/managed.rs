use std::collections::HashMap;
use std::time::Duration;

use rand::Rng;
use tokio::sync::watch;
use triomphe::Arc;

use super::mirror::MirrorNetwork;
use super::Network;
use crate::client::config::EndpointConfig;
use crate::{
    AccountId,
    NodeAddressBookQuery,
};

pub(crate) enum ManagedNetworkBuilder {
    Addresses(HashMap<String, AccountId>),
    Network(ManagedNetwork),
    Mainnet,
    Previewnet,
    Testnet,
}

impl ManagedNetworkBuilder {
    pub(crate) fn build(self, endpoint_config: EndpointConfig) -> crate::Result<ManagedNetwork> {
        let managed = match self {
            Self::Addresses(addresses) => {
                let network = Network::from_addresses(endpoint_config, &addresses)?;

                ManagedNetwork::new(network, MirrorNetwork::default())
            }
            Self::Network(network) => network,
            Self::Mainnet => ManagedNetwork::mainnet(endpoint_config),
            Self::Previewnet => ManagedNetwork::previewnet(endpoint_config),
            Self::Testnet => ManagedNetwork::testnet(endpoint_config),
        };

        Ok(managed)
    }
}

#[derive(Clone)]
pub(crate) struct ManagedNetwork(Arc<ManagedNetworkInner>);

impl ManagedNetwork {
    /// The time to wait before updating the network for the first time.
    const NETWORK_FIRST_UPDATE_DELAY: Duration = Duration::from_secs(10);

    pub(crate) fn new(
        primary: Network,
        mirror: MirrorNetwork,
        // first_update_delay: Duration,
    ) -> Self {
        Self(Arc::new(ManagedNetworkInner { primary, mirror }))
    }

    pub(crate) fn mainnet(endpoint_config: EndpointConfig) -> Self {
        Self::new(Network::mainnet(endpoint_config), MirrorNetwork::mainnet())
    }

    pub(crate) fn testnet(endpoint_config: EndpointConfig) -> Self {
        Self::new(Network::testnet(endpoint_config), MirrorNetwork::testnet())
    }

    pub(crate) fn previewnet(endpoint_config: EndpointConfig) -> Self {
        Self::new(Network::previewnet(endpoint_config), MirrorNetwork::previewnet())
    }
}

impl std::ops::Deref for ManagedNetwork {
    type Target = ManagedNetworkInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) struct ManagedNetworkInner {
    /// The network made up of Consensus nodes, IE, the "real"? network.
    pub(crate) primary: Network,
    //
    pub(crate) mirror: MirrorNetwork,
}

pub(crate) fn spawn_network_update(
    network: ManagedNetwork,
    initial_update_interval: Option<Duration>,
) -> watch::Sender<Option<Duration>> {
    let (tx, rx) = watch::channel(initial_update_interval);

    // note: this 100% dies if there's no runtime.
    tokio::task::spawn(update_network(network, rx));

    tx
}

// note: This keeps the `ManagedNetwork` alive (has a strong reference),
// however when network updates are no longer needed the sender can be dropped,
// which will eventually lead to this function returning and the strong count being decremented.
async fn update_network(
    network: ManagedNetwork,
    mut update_interval_rx: watch::Receiver<Option<Duration>>,
) {
    tokio::time::sleep(ManagedNetwork::NETWORK_FIRST_UPDATE_DELAY).await;

    'outer: loop {
        // log::debug!("updating network");
        let start = tokio::time::Instant::now();

        // note: ideally we'd have a `select!` on the channel closing, but, we can't
        // since there's no `async fn closed()`, and honestly, I'm not 100% certain these futures are cancel safe.
        match NodeAddressBookQuery::new()
            .execute_mirrornet(network.mirror.load().channel(), None)
            .await
        {
            Ok(it) => network.primary.update_from_address_book(&it),
            Err(e) => {
                log::warn!("{e:?}");
            }
        }

        // precompued jitter to theoretically avoid a thundering herd problem (in practice this probably won't matter much)
        let jitter = rand::thread_rng().gen_range(0..100);

        // some slightly complicated logic to make sure we
        // 1. Wait until the `update_interval` has elapsed
        // 2. Don't update the network when updating is disabled (`update_interval` == None)
        // 3. Wait the minimal amount of time if the update interval is changed while we're waiting
        //    (say it's been 23 hours out of 24, and then it's changed to a 12 hour interval, we'd want to update *now*)
        'wait: loop {
            // note that `wait_for` will *always* check the current value, even if it's been seen.
            // However, `closed` takes priority over `seen`, which is fine (we'd just want to return rather than wait).
            let update_interval = match update_interval_rx.wait_for(Option::is_some).await {
                // the value is `Some` so this unwrap is okay (although unfortunate)
                Ok(it) => it.unwrap(),
                Err(e) => {
                    log::debug!("client network update shutdown: {e}");
                    return;
                }
            };

            tokio::select! {
                // We very specifically want to use a `sleep_until` here because it means we don't wait at all if the time is in the past
                // and this can be called multiple times per `'outer` loop which means we don't want to wait the sum of all times.
                _ = tokio::time::sleep_until(start + update_interval + Duration::from_millis(jitter)) => {
                    continue 'outer
                }

                // it's fine to not do anything at all with the result here, if it's `Err` we'll pick it up on the next `'wait` loop (the channel will never unclose),
                // if it isn't, well, we'll also pick it up on the next `'wait` loop (it doesn't matter if the value changes again, even to closed).
                _ = update_interval_rx.changed() => continue 'wait,
            }
        }
    }
}
