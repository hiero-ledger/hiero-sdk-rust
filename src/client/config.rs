// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::str::FromStr;

use crate::signer::AnySigner;
use crate::{
    AccountId,
    PrivateKey,
};

struct FromStrProxy<T>(T);

impl<'de, T: FromStr> serde::Deserialize<'de> for FromStrProxy<T>
where
    T::Err: std::fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        String::deserialize(deserializer)
            .and_then(|it| it.parse().map_err(D::Error::custom))
            .map(Self)
    }
}

#[derive(serde_derive::Deserialize)]
pub(super) struct Operator {
    account_id: FromStrProxy<AccountId>,
    private_key: FromStrProxy<PrivateKey>,
}

impl From<Operator> for super::Operator {
    fn from(value: Operator) -> Self {
        Self { account_id: value.account_id.0, signer: AnySigner::PrivateKey(value.private_key.0) }
    }
}

#[derive(serde_derive::Deserialize)]
#[serde(untagged)]
pub(super) enum Either<L, R> {
    Left(L),
    Right(R),
}

#[derive(serde_derive::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum NetworkName {
    Mainnet,
    Testnet,
    Previewnet,
}

#[derive(serde_derive::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ClientConfigInner {
    operator: Option<Operator>,
    network: Either<HashMap<String, FromStrProxy<AccountId>>, NetworkName>,
    mirror_network: Option<Either<Vec<String>, NetworkName>>,
}

impl From<ClientConfigInner> for ClientConfig {
    fn from(value: ClientConfigInner) -> Self {
        Self {
            operator: value.operator.map(Into::into),
            network: match value.network {
                Either::Left(it) => Either::Left(it.into_iter().map(|(k, v)| (k, v.0)).collect()),
                Either::Right(it) => Either::Right(it),
            },
            mirror_network: value.mirror_network,
        }
    }
}

pub(super) struct ClientConfig {
    pub(super) operator: Option<super::Operator>,
    pub(super) network: Either<HashMap<String, AccountId>, NetworkName>,
    pub(super) mirror_network: Option<Either<Vec<String>, NetworkName>>,
}
