// SPDX-License-Identifier: Apache-2.0

use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::TryStreamExt;

use super::subscribe::MirrorQueryExecute;
use crate::topic::TopicMessageQueryData;
use crate::{
    MirrorQuery,
    NodeAddress,
    NodeAddressBookQueryData,
    TopicMessage,
};

/// Represents any possible query to the mirror network.
pub type AnyMirrorQuery = MirrorQuery<AnyMirrorQueryData>;

#[derive(Debug, Clone)]
pub enum AnyMirrorQueryData {
    NodeAddressBook(NodeAddressBookQueryData),
    TopicMessage(TopicMessageQueryData),
}

#[derive(Debug, Clone)]
pub enum AnyMirrorQueryMessage {
    NodeAddressBook(NodeAddress),
    TopicMessage(TopicMessage),
}

/// Represents the response of any possible query to the mirror network.
pub enum AnyMirrorQueryResponse {
    /// Response for `AnyMirrorQuery::NodeAddressBook`.
    NodeAddressBook(<NodeAddressBookQueryData as MirrorQueryExecute>::Response),
    /// Response for `AnyMirrorQuery::TopicMessage`.
    TopicMessage(<TopicMessageQueryData as MirrorQueryExecute>::Response),
}

impl MirrorQueryExecute for AnyMirrorQueryData {
    type Item = AnyMirrorQueryMessage;

    type Response = AnyMirrorQueryResponse;

    type ItemStream<'a>
        = BoxStream<'a, crate::Result<Self::Item>>
    where
        Self: 'a;

    fn subscribe_with_optional_timeout<'a>(
        &self,
        params: &crate::mirror_query::MirrorQueryCommon,
        client: &'a crate::Client,
        timeout: Option<std::time::Duration>,
    ) -> Self::ItemStream<'a>
    where
        Self: 'a,
    {
        match self {
            AnyMirrorQueryData::NodeAddressBook(it) => Box::pin(
                it.subscribe_with_optional_timeout(params, client, timeout)
                    .map_ok(Self::Item::from),
            ),
            AnyMirrorQueryData::TopicMessage(it) => Box::pin(
                it.subscribe_with_optional_timeout(params, client, timeout)
                    .map_ok(Self::Item::from),
            ),
        }
    }

    fn execute_with_optional_timeout<'a>(
        &'a self,
        params: &'a super::MirrorQueryCommon,
        client: &'a crate::Client,
        timeout: Option<std::time::Duration>,
    ) -> BoxFuture<'a, crate::Result<Self::Response>> {
        match self {
            AnyMirrorQueryData::NodeAddressBook(it) => Box::pin(async move {
                it.execute_with_optional_timeout(params, client, timeout)
                    .await
                    .map(Self::Response::from)
            }),
            AnyMirrorQueryData::TopicMessage(it) => Box::pin(async move {
                it.execute_with_optional_timeout(params, client, timeout)
                    .await
                    .map(Self::Response::from)
            }),
        }
    }
}
