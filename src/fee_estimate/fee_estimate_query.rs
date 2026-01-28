// SPDX-License-Identifier: Apache-2.0
use std::time::Duration;

use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_core::Stream;
use futures_util::TryStreamExt;
use hedera_proto::mirror;
use mirror::network_service_client::NetworkServiceClient;
use tonic::transport::Channel;
use tonic::Response;

use crate::entity_id::ValidateChecksums;
use crate::ledger_id::RefLedgerId;
use crate::mirror_query::{
    AnyMirrorQueryData,
    AnyMirrorQueryMessage,
    MirrorRequest,
};
use crate::protobuf::FromProtobuf;
use crate::{
    AnyMirrorQueryResponse,
    AnyTransaction,
    Error,
    FeeEstimateMode,
    FeeEstimateResponse,
    MirrorQuery,
};

/// Query to estimate fees for a transaction without submitting it to the network.
pub type FeeEstimateQuery = MirrorQuery<FeeEstimateQueryData>;

#[derive(Debug, Clone)]
pub struct FeeEstimateQueryData {
    /// The estimation mode (defaults to State).
    mode: FeeEstimateMode,
    /// The transaction to estimate fees for.
    transaction: Option<AnyTransaction>,
}

impl Default for FeeEstimateQueryData {
    fn default() -> Self {
        Self { mode: FeeEstimateMode::State, transaction: None }
    }
}

impl FeeEstimateQueryData {
    fn map_stream<'a, S>(stream: S) -> impl Stream<Item = crate::Result<FeeEstimateResponse>>
    where
        S: Stream<Item = crate::Result<mirror::FeeEstimateResponse>> + Send + 'a,
    {
        stream.and_then(|it| std::future::ready(FeeEstimateResponse::from_protobuf(it)))
    }
}

impl FeeEstimateQuery {
    /// Set the estimation mode (optional, defaults to State).
    pub fn set_mode(&mut self, mode: FeeEstimateMode) -> &mut Self {
        self.data.mode = mode;
        self
    }

    /// Get the current estimation mode.
    pub fn get_mode(&self) -> FeeEstimateMode {
        self.data.mode
    }

    /// Set the transaction to estimate (required).
    pub fn set_transaction<D>(&mut self, transaction: crate::Transaction<D>) -> &mut Self
    where
        crate::Transaction<D>: Into<AnyTransaction>,
    {
        self.data.transaction = Some(transaction.into());
        self
    }

    /// Get the current transaction.
    pub fn get_transaction(&self) -> Option<&AnyTransaction> {
        self.data.transaction.as_ref()
    }
}

impl From<FeeEstimateQueryData> for AnyMirrorQueryData {
    fn from(data: FeeEstimateQueryData) -> Self {
        Self::FeeEstimate(data)
    }
}

impl MirrorRequest for FeeEstimateQueryData {
    type GrpcItem = mirror::FeeEstimateResponse;

    type ConnectStream = tonic::Streaming<Self::GrpcItem>;

    type Item = FeeEstimateResponse;

    type Response = FeeEstimateResponse;

    type Context = ();

    type ItemStream<'a> = BoxStream<'a, crate::Result<FeeEstimateResponse>>;

    fn connect(
        &self,
        _context: &Self::Context,
        channel: Channel,
    ) -> BoxFuture<'_, tonic::Result<Self::ConnectStream>> {
        Box::pin(async {
            let transaction = self.transaction.as_ref().ok_or_else(|| {
                tonic::Status::invalid_argument("FeeEstimateQuery requires a transaction")
            })?;

            // Transaction must be frozen before we can call make_sources
            if !transaction.is_frozen() {
                return Err(tonic::Status::failed_precondition(
                    "Transaction must be frozen before calling FeeEstimateQuery. Call freeze_with(client) on the transaction first.",
                ));
            }

            // Get all transaction chunks
            let sources = transaction.make_sources().map_err(|e| {
                tonic::Status::internal(format!("Failed to get transaction sources: {}", e))
            })?;
            let transaction_list = sources.transactions().to_vec();

            if transaction_list.is_empty() {
                return Err(tonic::Status::invalid_argument("Transaction has no chunks"));
            }

            // For now, we'll request estimate for the first chunk
            // TODO: Handle multiple chunks by making multiple requests and aggregating
            let proto_transaction = transaction_list.first().unwrap();

            let request = mirror::FeeEstimateQuery {
                mode: self.mode.value() as i32,
                transaction: Some(proto_transaction.clone()),
            };

            NetworkServiceClient::new(channel)
                .get_fee_estimate(request)
                .await
                .map(Response::into_inner)
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
        // FeeEstimateResponse is a single response, not a stream of items
        // So we just take the first item
        Box::pin(async move {
            let mut stream = std::pin::pin!(Self::map_stream(stream));
            stream
                .try_next()
                .await?
                .ok_or_else(|| Error::basic_parse("FeeEstimateQuery returned no response"))
        })
    }

    fn update_context(_context: &mut Self::Context, _item: &Self::GrpcItem) {}
}

impl From<FeeEstimateResponse> for AnyMirrorQueryMessage {
    fn from(value: FeeEstimateResponse) -> Self {
        Self::FeeEstimate(value)
    }
}

impl From<FeeEstimateResponse> for AnyMirrorQueryResponse {
    fn from(value: FeeEstimateResponse) -> Self {
        Self::FeeEstimate(value)
    }
}

impl FeeEstimateQuery {
    pub(crate) async fn execute_mirrornet(
        &self,
        channel: Channel,
        timeout: Option<Duration>,
    ) -> crate::Result<FeeEstimateResponse> {
        let timeout = timeout.unwrap_or_else(|| {
            std::time::Duration::from_millis(backoff::default::MAX_ELAPSED_TIME_MILLIS)
        });

        FeeEstimateQueryData::try_collect(crate::mirror_query::subscribe(
            channel,
            timeout,
            self.data.clone(),
        ))
        .await
    }
}

impl ValidateChecksums for FeeEstimateQuery {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        if let Some(transaction) = &self.data.transaction {
            transaction.validate_checksums(ledger_id)?;
        }
        Ok(())
    }
}
