use std::cmp;
use std::num::NonZeroUsize;

use hedera_proto::services;
use tonic::transport::Channel;

use super::{
    TransactionData,
    TransactionExecute,
};
use crate::entity_id::ValidateChecksums;
use crate::execute::Execute;
use crate::ledger_id::RefLedgerId;
use crate::{
    AccountId,
    BoxGrpcFuture,
    Error,
    Hbar,
    Transaction,
    TransactionHash,
    TransactionId,
    TransactionResponse,
};

/// Per transaction chunk data (you'd add this to any chunked transaction)
#[derive(Clone, Debug, PartialEq)]
pub struct ChunkData {
    pub(crate) max_chunks: usize,
    pub(crate) chunk_size: NonZeroUsize,
    pub(crate) data: Vec<u8>,
}

impl Default for ChunkData {
    fn default() -> Self {
        Self {
            max_chunks: Self::DEFAULT_MAX_CHUNKS,
            chunk_size: Self::DEFAULT_CHUNK_SIZE,
            data: Vec::new(),
        }
    }
}

impl ChunkData {
    const DEFAULT_MAX_CHUNKS: usize = 20;
    // safety: 1024 is not zero.
    // note: Use `NonZeroUsize::new().unwrap()` once that's const stable.
    const DEFAULT_CHUNK_SIZE: NonZeroUsize = match NonZeroUsize::new(1024) {
        Some(it) => it,
        None => unreachable!(),
    };

    pub(crate) fn used_chunks(&self) -> usize {
        if self.data.is_empty() {
            return 1;
        }

        // div ceil algorithm, fun fact: the intrinsic `div_ceil` can't get rid of the panic (it's unstable anyway)
        (self.data.len() + self.chunk_size.get()) / self.chunk_size
    }

    pub(crate) fn message_chunk(&self, chunk_info: &ChunkInfo) -> &[u8] {
        debug_assert!(chunk_info.current < self.used_chunks());

        let start = self.chunk_size.get() * chunk_info.current;
        let end = cmp::min(self.chunk_size.get() * (chunk_info.current + 1), self.data.len());

        &self.data[start..end]
    }

    pub(crate) fn max_message_len(&self) -> usize {
        self.max_chunks * self.chunk_size.get()
    }
}

pub struct ChunkInfo {
    /// Current chunk # out of [`max_chunks`](ChunkData.max_chunks) total.
    pub(crate) current: usize,

    /// How many chunks there will be.
    /// if `chunks` was a `[ChunkInfo]` this would be the length.
    pub(crate) total: usize,

    /// transaction ID for transaction 0 (if `current` is 0 this is that one).
    pub(crate) initial_transaction_id: TransactionId,

    /// transaction ID for the current transaction.
    pub(crate) current_transaction_id: TransactionId,

    /// ID for the account this transaction will be submitted to.
    pub(crate) node_account_id: Option<AccountId>,
}

impl ChunkInfo {
    #[must_use]
    pub(crate) fn assert_single_transaction(&self) -> (TransactionId, Option<AccountId>) {
        assert!(self.current == 0 && self.total == 1);
        (self.current_transaction_id, self.node_account_id)
    }

    #[must_use]
    // taking `transaction_id` by reference and then dereferencing it to copy it unconditionally... Feels weird.
    #[allow(clippy::large_types_passed_by_value)]
    pub(crate) const fn single(transaction_id: TransactionId, node_account_id: AccountId) -> Self {
        Self::initial(1, transaction_id, node_account_id)
    }

    #[must_use]
    // taking `transaction_id` by reference and then dereferencing it to copy it unconditionally... Feels weird.
    #[allow(clippy::large_types_passed_by_value)]
    pub(crate) const fn initial(
        total: usize,
        transaction_id: TransactionId,
        node_account_id: AccountId,
    ) -> Self {
        Self {
            current: 0,
            total,
            initial_transaction_id: transaction_id,
            current_transaction_id: transaction_id,
            node_account_id: Some(node_account_id),
        }
    }
}

pub(super) struct FirstChunkView<'a, D> {
    pub(super) transaction: &'a Transaction<D>,
    pub(super) total_chunks: usize,
}

impl<'a, D> Execute for FirstChunkView<'a, D>
where
    D: TransactionExecute,
{
    type GrpcRequest = services::Transaction;

    type GrpcResponse = services::TransactionResponse;

    type Context = TransactionHash;

    type Response = TransactionResponse;

    fn node_account_ids(&self) -> Option<&[AccountId]> {
        self.transaction.body.node_account_ids.as_deref()
    }

    fn transaction_id(&self) -> Option<TransactionId> {
        self.transaction.get_transaction_id()
    }

    fn requires_transaction_id(&self) -> bool {
        true
    }

    fn regenerate_transaction_id(&self) -> Option<bool> {
        self.transaction.regenerate_transaction_id()
    }

    fn operator_account_id(&self) -> Option<&AccountId> {
        self.transaction.operator_account_id()
    }

    fn make_request(
        &self,
        transaction_id: Option<&TransactionId>,
        node_account_id: AccountId,
    ) -> crate::Result<(Self::GrpcRequest, Self::Context)> {
        assert!(self.transaction.is_frozen());

        Ok(self.transaction.make_request_inner(&ChunkInfo::initial(
            self.total_chunks,
            *transaction_id.ok_or(Error::NoPayerAccountOrTransactionId)?,
            node_account_id,
        )))
    }

    fn execute(
        &self,
        channel: Channel,
        request: Self::GrpcRequest,
    ) -> BoxGrpcFuture<'_, Self::GrpcResponse> {
        self.transaction.body.data.execute(channel, request)
    }

    fn make_response(
        &self,
        _response: Self::GrpcResponse,
        context: Self::Context,
        node_account_id: AccountId,
        transaction_id: Option<&TransactionId>,
    ) -> crate::Result<Self::Response> {
        Ok(TransactionResponse {
            node_account_id,
            transaction_id: *transaction_id.unwrap(),
            transaction_hash: context,
            validate_status: true,
        })
    }

    fn make_error_pre_check(
        &self,
        status: services::ResponseCodeEnum,
        transaction_id: Option<&TransactionId>,
        response: Self::GrpcResponse,
    ) -> crate::Error {
        crate::Error::TransactionPreCheckStatus {
            status,
            cost: (response.cost != 0).then(|| Hbar::from_tinybars(response.cost as i64)),
            transaction_id: Box::new(
                *transaction_id.expect("transactions must have transaction IDs"),
            ),
        }
    }

    fn response_pre_check_status(response: &Self::GrpcResponse) -> crate::Result<i32> {
        Ok(response.node_transaction_precheck_code)
    }
}

impl<'a, D: ValidateChecksums> ValidateChecksums for FirstChunkView<'a, D> {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.transaction.validate_checksums(ledger_id)
    }
}

pub(super) struct ChunkView<'a, D> {
    pub(super) transaction: &'a Transaction<D>,
    pub(super) initial_transaction_id: TransactionId,
    pub(super) current_chunk: usize,
    pub(super) total_chunks: usize,
}

impl<'a, D> Execute for ChunkView<'a, D>
where
    D: TransactionExecute,
{
    type GrpcRequest = services::Transaction;

    type GrpcResponse = services::TransactionResponse;

    type Context = TransactionHash;

    type Response = TransactionResponse;

    fn node_account_ids(&self) -> Option<&[AccountId]> {
        self.transaction.body.node_account_ids.as_deref()
    }

    fn transaction_id(&self) -> Option<TransactionId> {
        None
    }

    fn operator_account_id(&self) -> Option<&AccountId> {
        self.transaction.operator_account_id()
    }

    fn requires_transaction_id(&self) -> bool {
        true
    }

    fn regenerate_transaction_id(&self) -> Option<bool> {
        self.transaction.regenerate_transaction_id()
    }

    fn make_request(
        &self,
        transaction_id: Option<&TransactionId>,
        node_account_id: AccountId,
    ) -> crate::Result<(Self::GrpcRequest, Self::Context)> {
        assert!(self.transaction.is_frozen());

        Ok(self.transaction.make_request_inner(&ChunkInfo {
            total: self.total_chunks,
            current: self.current_chunk,
            initial_transaction_id: self.initial_transaction_id,
            node_account_id: Some(node_account_id),
            current_transaction_id: *transaction_id.ok_or(Error::NoPayerAccountOrTransactionId)?,
        }))
    }

    fn execute(
        &self,
        channel: Channel,
        request: Self::GrpcRequest,
    ) -> BoxGrpcFuture<'_, Self::GrpcResponse> {
        self.transaction.body.data.execute(channel, request)
    }

    fn make_response(
        &self,
        _response: Self::GrpcResponse,
        context: Self::Context,
        node_account_id: AccountId,
        transaction_id: Option<&TransactionId>,
    ) -> crate::Result<Self::Response> {
        Ok(TransactionResponse {
            node_account_id,
            transaction_id: *transaction_id.unwrap(),
            transaction_hash: context,
            validate_status: true,
        })
    }

    fn make_error_pre_check(
        &self,
        status: services::ResponseCodeEnum,
        transaction_id: Option<&TransactionId>,
        response: Self::GrpcResponse,
    ) -> crate::Error {
        crate::Error::TransactionPreCheckStatus {
            status,
            cost: (response.cost != 0).then(|| Hbar::from_tinybars(response.cost as i64)),
            transaction_id: Box::new(
                *transaction_id.expect("transactions must have transaction IDs"),
            ),
        }
    }

    fn response_pre_check_status(response: &Self::GrpcResponse) -> crate::Result<i32> {
        Ok(response.node_transaction_precheck_code)
    }
}

impl<'a, D: ValidateChecksums> ValidateChecksums for ChunkView<'a, D> {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.transaction.validate_checksums(ledger_id)?;
        self.initial_transaction_id.validate_checksums(ledger_id)?;

        Ok(())
    }
}

// Note: this is completely optional to implement, just makes more methods available on `Transaction`.
pub trait ChunkedTransactionData: TransactionData {
    fn chunk_data(&self) -> &ChunkData;
    fn chunk_data_mut(&mut self) -> &mut ChunkData;
}
