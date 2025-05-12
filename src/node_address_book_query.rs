// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_core::Stream;
use futures_util::{
    TryFutureExt,
    TryStreamExt,
};
use hedera_proto::{
    mirror,
    services,
};
use mirror::network_service_client::NetworkServiceClient;
use tonic::transport::Channel;
use tonic::Response;

use crate::mirror_query::{
    AnyMirrorQueryData,
    AnyMirrorQueryMessage,
    MirrorRequest,
};
use crate::protobuf::FromProtobuf;
use crate::{
    AnyMirrorQueryResponse,
    FileId,
    MirrorQuery,
    NodeAddress,
    NodeAddressBook,
    ToProtobuf,
};

// TODO: validate checksums after PR is merged

/// Query for an address book and return its nodes.
/// The nodes are returned in ascending order by node ID.
pub type NodeAddressBookQuery = MirrorQuery<NodeAddressBookQueryData>;

#[derive(Debug, Clone)]
pub struct NodeAddressBookQueryData {
    /// The ID of the address book file on the network.
    /// Can either be `0.0.101` or `0.0.102`. Defaults to `0.0.102`.
    file_id: FileId,

    /// The maximum number of node addresses to receive.
    /// Defaults to _all_.
    limit: u32,
}

impl NodeAddressBookQueryData {
    fn map_stream<'a, S>(stream: S) -> impl Stream<Item = crate::Result<NodeAddress>>
    where
        S: Stream<Item = crate::Result<services::NodeAddress>> + Send + 'a,
    {
        stream.and_then(|it| std::future::ready(NodeAddress::from_protobuf(it)))
    }
}

impl Default for NodeAddressBookQueryData {
    fn default() -> Self {
        Self { file_id: FileId::ADDRESS_BOOK, limit: 0 }
    }
}

impl NodeAddressBookQuery {
    /// Returns the file ID of the address book file on the network.
    #[must_use]
    pub fn get_file_id(&self) -> FileId {
        self.data.file_id
    }

    /// Sets the ID of the address book file on the network.
    /// Can either be `0.0.101` or `0.0.102`. Defaults to `0.0.102`.
    pub fn file_id(&mut self, id: impl Into<FileId>) -> &mut Self {
        self.data.file_id = id.into();
        self
    }

    /// Returns the configured limit of node addresses to receive.
    #[must_use]
    pub fn get_limit(&self) -> u32 {
        self.data.limit
    }

    /// Sets the maximum number of node addresses to receive.
    /// Defaults to _all_.
    pub fn limit(&mut self, limit: u32) -> &mut Self {
        self.data.limit = limit;
        self
    }
}

impl From<NodeAddressBookQueryData> for AnyMirrorQueryData {
    fn from(data: NodeAddressBookQueryData) -> Self {
        Self::NodeAddressBook(data)
    }
}

impl MirrorRequest for NodeAddressBookQueryData {
    type GrpcItem = services::NodeAddress;

    type ConnectStream = tonic::Streaming<Self::GrpcItem>;

    type Item = NodeAddress;

    type Context = ();

    type Response = NodeAddressBook;

    type ItemStream<'a> = BoxStream<'a, crate::Result<NodeAddress>>;

    fn connect(
        &self,
        _context: &Self::Context,
        channel: Channel,
    ) -> BoxFuture<'_, tonic::Result<Self::ConnectStream>> {
        Box::pin(async {
            let file_id = self.file_id.to_protobuf();
            let request =
                mirror::AddressBookQuery { file_id: Some(file_id), limit: self.limit as i32 };

            NetworkServiceClient::new(channel).get_nodes(request).await.map(Response::into_inner)
        })
    }

    fn make_item_stream<'a, S>(stream: S) -> Self::ItemStream<'a>
    where
        S: Stream<Item = crate::Result<Self::GrpcItem>> + Send + 'a,
    {
        Box::pin(Self::map_stream(stream))
    }

    fn try_collect<'a, S>(stream: S) -> BoxFuture<'a, crate::Result<Self::Response>>
    where
        S: Stream<Item = crate::Result<Self::GrpcItem>> + Send + 'a,
    {
        // this doesn't reuse the work in `make_item_stream`
        Box::pin(
            Self::map_stream(stream)
                .try_collect()
                .map_ok(|addresses| NodeAddressBook { node_addresses: addresses }),
        )
    }

    fn update_context(_context: &mut Self::Context, _item: &Self::GrpcItem) {}
}

impl From<NodeAddress> for AnyMirrorQueryMessage {
    fn from(value: NodeAddress) -> Self {
        Self::NodeAddressBook(value)
    }
}

impl From<NodeAddressBook> for AnyMirrorQueryResponse {
    fn from(value: NodeAddressBook) -> Self {
        Self::NodeAddressBook(value)
    }
}

impl NodeAddressBookQuery {
    pub(crate) async fn execute_mirrornet(
        &self,
        channel: Channel,
        timeout: Option<Duration>,
    ) -> crate::Result<NodeAddressBook> {
        let timeout = timeout.unwrap_or_else(|| {
            std::time::Duration::from_millis(backoff::default::MAX_ELAPSED_TIME_MILLIS)
        });

        NodeAddressBookQueryData::try_collect(crate::mirror_query::subscribe(
            channel,
            timeout,
            self.data.clone(),
        ))
        .await
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        FileId,
        NodeAddressBookQuery,
    };

    #[test]
    fn get_set_file_id() {
        let mut query = NodeAddressBookQuery::new();
        query.file_id(FileId::new(0, 0, 1111));

        assert_eq!(query.get_file_id(), FileId::new(0, 0, 1111));
    }

    #[test]
    fn get_set_limit() {
        let mut query = NodeAddressBookQuery::new();
        query.limit(231);

        assert_eq!(query.get_limit(), 231);
    }
}
