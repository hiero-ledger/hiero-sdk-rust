// SPDX-License-Identifier: Apache-2.0

use std::borrow::Cow;
use std::ops::Deref;
use std::time::Duration;

use hyper::Uri;
use hyper_openssl::client::legacy::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use once_cell::sync::OnceCell;
use openssl::ssl::{
    SslConnector,
    SslMethod,
    SslVerifyMode,
};
use tonic::transport::{
    Channel,
    Endpoint,
};
use triomphe::Arc;

use crate::ArcSwap;

pub(crate) const MAINNET: &str = "mainnet-public.mirrornode.hedera.com:443";

pub(crate) const TESTNET: &str = "testnet.mirrornode.hedera.com:443";

pub(crate) const PREVIEWNET: &str = "previewnet.mirrornode.hedera.com:443";

#[derive(Default)]
pub(crate) struct MirrorNetwork(ArcSwap<MirrorNetworkData>);

impl Deref for MirrorNetwork {
    type Target = ArcSwap<MirrorNetworkData>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MirrorNetwork {
    pub(crate) fn mainnet() -> Self {
        Self::network(MAINNET)
    }

    pub(crate) fn testnet() -> Self {
        Self::network(TESTNET)
    }

    pub(crate) fn previewnet() -> Self {
        Self::network(PREVIEWNET)
    }

    fn network(address: &'static str) -> Self {
        Self(ArcSwap::new(Arc::new(MirrorNetworkData::from_static(&[address]))))
    }

    #[cfg(feature = "serde")]
    pub(crate) fn from_addresses(addresses: Vec<Cow<'static, str>>) -> Self {
        Self(ArcSwap::new(Arc::new(MirrorNetworkData::from_addresses(addresses))))
    }
}

#[derive(Clone, Default)]
pub(crate) struct MirrorNetworkData {
    addresses: Vec<Cow<'static, str>>,
    channel: OnceCell<Channel>,
}

impl MirrorNetworkData {
    pub(crate) fn from_addresses(addresses: Vec<Cow<'static, str>>) -> Self {
        Self { addresses, channel: OnceCell::new() }
    }

    pub(crate) fn from_static(network: &[&'static str]) -> Self {
        let addresses = network.iter().map(|&addr| Cow::Borrowed(addr)).collect();

        Self { addresses, channel: OnceCell::new() }
    }

    pub(crate) fn channel(&self) -> Channel {
        self.channel
            .get_or_init(|| {
                let endpoint = self.addresses.iter().next().unwrap();
                let uri = format!("https://{endpoint}");
                let uri_parsed = Uri::from_maybe_shared(uri).unwrap();

                // Configure OpenSSL
                let mut ssl_builder = SslConnector::builder(SslMethod::tls()).unwrap();
                ssl_builder.set_verify(SslVerifyMode::PEER);
                ssl_builder.set_alpn_protos(b"\x02h2").unwrap();

                // Create HTTPS connector with OpenSSL
                let mut http = HttpConnector::new();
                http.enforce_http(false);
                let https = HttpsConnector::with_connector(http, ssl_builder).unwrap();

                Endpoint::from_shared(uri_parsed.to_string())
                    .unwrap()
                    .connect_timeout(Duration::from_secs(10))
                    .keep_alive_timeout(Duration::from_secs(10))
                    .keep_alive_while_idle(true)
                    .tcp_keepalive(Some(Duration::from_secs(10)))
                    .connect_with_connector_lazy(https)
            })
            .clone()
    }

    pub(crate) fn addresses(&self) -> impl Iterator<Item = String> + '_ {
        self.addresses.iter().cloned().map(Cow::into_owned)
    }
}
