// SPDX-License-Identifier: Apache-2.0

use std::borrow::Cow;
use std::collections::HashMap;

use hedera_proto::services;
use prost::Message;
use tonic::transport::Channel;

use super::chunked::ChunkInfo;
use super::source::SourceChunk;
use super::{
    ChunkData,
    TransactionSources,
};
use crate::execute::Execute;
use crate::ledger_id::RefLedgerId;
use crate::transaction::any::AnyTransactionData;
use crate::transaction::protobuf::ToTransactionDataProtobuf;
use crate::transaction::DEFAULT_TRANSACTION_VALID_DURATION;
use crate::{
    AccountId,
    BoxGrpcFuture,
    Client,
    Error,
    Hbar,
    PublicKey,
    ToProtobuf,
    Transaction,
    TransactionHash,
    TransactionId,
    TransactionResponse,
    ValidateChecksums,
};

#[derive(Debug)]
pub(super) struct SignaturePair {
    signature: Vec<u8>,
    public: PublicKey,
}

impl SignaturePair {
    pub fn into_protobuf(self) -> services::SignaturePair {
        let signature = match self.public.kind() {
            crate::key::KeyKind::Ed25519 => {
                services::signature_pair::Signature::Ed25519(self.signature)
            }
            crate::key::KeyKind::Ecdsa => {
                services::signature_pair::Signature::EcdsaSecp256k1(self.signature)
            }
        };
        services::SignaturePair {
            signature: Some(signature),
            // TODO: is there any way to utilize the _prefix_ nature of this field?
            pub_key_prefix: self.public.to_bytes_raw(),
        }
    }
}

impl From<(PublicKey, Vec<u8>)> for SignaturePair {
    fn from((public, signature): (PublicKey, Vec<u8>)) -> Self {
        Self { signature, public }
    }
}

impl<D> Transaction<D>
where
    D: TransactionData + ToTransactionDataProtobuf,
{
    pub(crate) fn make_request_inner(
        &self,
        chunk_info: &ChunkInfo,
    ) -> (services::Transaction, TransactionHash) {
        assert!(self.is_frozen());

        let transaction_body = self.to_transaction_body_protobuf(chunk_info);

        let body_bytes = transaction_body.encode_to_vec();

        let mut signatures = Vec::with_capacity(1 + self.signers.len());

        if let Some(operator) = &self.body.operator {
            let operator_signature = operator.sign(&body_bytes);

            signatures.push(SignaturePair::from(operator_signature).into_protobuf());
        }

        for signer in &self.signers {
            let public_key = signer.public_key().to_bytes();
            if !signatures.iter().any(|it| public_key.starts_with(&it.pub_key_prefix)) {
                let signature = signer.sign(&body_bytes);
                signatures.push(SignaturePair::from(signature).into_protobuf());
            }
        }

        let signed_transaction = services::SignedTransaction {
            body_bytes,
            sig_map: Some(services::SignatureMap { sig_pair: signatures }),
        };

        let signed_transaction_bytes = signed_transaction.encode_to_vec();

        let transaction_hash = TransactionHash::new(&signed_transaction_bytes);

        let transaction =
            services::Transaction { signed_transaction_bytes, ..services::Transaction::default() };

        (transaction, transaction_hash)
    }
}

/// Pre-execute associated fields for transaction data.
pub trait TransactionData: Clone + Into<AnyTransactionData> {
    /// Whether this transaction is intended to be executed to return a cost estimate.
    #[doc(hidden)]
    fn for_cost_estimate(&self) -> bool {
        false
    }

    /// Returns the maximum allowed transaction fee if none is specified.
    ///
    /// Specifically, this default will be used in the following case:
    /// - The transaction itself (direct user input) has no `max_transaction_fee` specified, AND
    /// - The [`Client`](crate::Client) has no `max_transaction_fee` specified.
    fn default_max_transaction_fee(&self) -> Hbar {
        Hbar::new(2)
    }

    /// Returns the chunk data for this transaction if this is a chunked transaction.
    fn maybe_chunk_data(&self) -> Option<&ChunkData> {
        None
    }

    /// Returns `true` if `self` is a chunked transaction *and* it should wait for receipts between each chunk.
    fn wait_for_receipt(&self) -> bool {
        false
    }
}

pub trait TransactionExecute:
    ToTransactionDataProtobuf + TransactionData + ValidateChecksums
{
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse>;
}

impl<D> Execute for Transaction<D>
where
    D: TransactionExecute,
{
    type GrpcRequest = services::Transaction;

    type GrpcResponse = services::TransactionResponse;

    type Context = TransactionHash;

    type Response = TransactionResponse;

    fn node_account_ids(&self) -> Option<&[AccountId]> {
        self.body.node_account_ids.as_deref()
    }

    fn transaction_id(&self) -> Option<TransactionId> {
        self.body.transaction_id
    }

    fn requires_transaction_id(&self) -> bool {
        true
    }

    fn operator_account_id(&self) -> Option<&AccountId> {
        self.body.operator.as_deref().map(|it| &it.account_id)
    }

    fn regenerate_transaction_id(&self) -> Option<bool> {
        self.body.regenerate_transaction_id
    }

    fn make_request(
        &self,
        transaction_id: Option<&TransactionId>,
        node_account_id: AccountId,
    ) -> crate::Result<(Self::GrpcRequest, Self::Context)> {
        assert!(self.is_frozen());

        Ok(self.make_request_inner(&ChunkInfo::single(
            *transaction_id.ok_or(Error::NoPayerAccountOrTransactionId)?,
            node_account_id,
        )))
    }

    fn execute(
        &self,
        channel: Channel,
        request: Self::GrpcRequest,
    ) -> BoxGrpcFuture<'_, Self::GrpcResponse> {
        self.body.data.execute(channel, request)
    }

    fn make_response(
        &self,
        _response: Self::GrpcResponse,
        transaction_hash: Self::Context,
        node_account_id: AccountId,
        transaction_id: Option<&TransactionId>,
    ) -> crate::Result<Self::Response> {
        Ok(TransactionResponse {
            node_account_id,
            transaction_id: *transaction_id.unwrap(),
            transaction_hash,
            validate_status: true,
        })
    }

    fn make_error_pre_check(
        &self,
        status: crate::Status,
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

/// Marker trait for transactions that support Chunking.
pub trait TransactionExecuteChunked: TransactionExecute {}

impl<D: ValidateChecksums> ValidateChecksums for Transaction<D> {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        if let Some(node_account_ids) = &self.body.node_account_ids {
            for node_account_id in node_account_ids {
                node_account_id.validate_checksums(ledger_id)?;
            }
        }
        self.body.transaction_id.validate_checksums(ledger_id)?;
        self.body.data.validate_checksums(ledger_id)
    }
}

impl<D> Transaction<D>
where
    D: TransactionData + ToTransactionDataProtobuf,
{
    #[allow(deprecated)]
    fn to_transaction_body_protobuf(&self, chunk_info: &ChunkInfo) -> services::TransactionBody {
        assert!(self.is_frozen());
        let data = self.body.data.to_transaction_data_protobuf(chunk_info);

        let transaction_fee = if self.body.data.for_cost_estimate() {
            0
        } else {
            self.body
                .max_transaction_fee
                .unwrap_or_else(|| self.body.data.default_max_transaction_fee())
                .to_tinybars() as u64
        };

        services::TransactionBody {
            data: Some(data),
            transaction_id: Some(chunk_info.current_transaction_id.to_protobuf()),
            transaction_valid_duration: Some(
                self.body
                    .transaction_valid_duration
                    .unwrap_or(DEFAULT_TRANSACTION_VALID_DURATION)
                    .into(),
            ),
            memo: self.body.transaction_memo.clone(),
            node_account_id: Some(chunk_info.node_account_id.to_protobuf()),
            generate_record: false,
            transaction_fee,
            max_custom_fees: vec![],
        }
    }
}

// fixme: find a better name.
pub(crate) struct SourceTransaction<'a, D> {
    inner: &'a Transaction<D>,
    sources: Cow<'a, TransactionSources>,
}

impl<'a, D> SourceTransaction<'a, D> {
    pub(crate) fn new(transaction: &'a Transaction<D>, sources: &'a TransactionSources) -> Self {
        // fixme: be way more lazy.
        let sources = sources.sign_with(&transaction.signers);

        Self { inner: transaction, sources }
    }

    pub(crate) async fn execute(
        &self,
        client: &Client,
        timeout: Option<std::time::Duration>,
    ) -> crate::Result<TransactionResponse>
    where
        D: TransactionExecute,
    {
        Ok(self.execute_all(client, timeout).await?.swap_remove(0))
    }

    pub(crate) async fn execute_all(
        &self,
        client: &Client,
        timeout_per_chunk: Option<std::time::Duration>,
    ) -> crate::Result<Vec<TransactionResponse>>
    where
        D: TransactionExecute,
    {
        let mut responses = Vec::with_capacity(self.sources.chunks_len());
        for chunk in self.sources.chunks() {
            let response = crate::execute::execute(
                client,
                &SourceTransactionExecuteView::new(self.inner, chunk),
                timeout_per_chunk,
            )
            .await?;

            if self.inner.data().wait_for_receipt() {
                response.get_receipt(client).await?;
            }

            responses.push(response);
        }

        Ok(responses)
    }
}

// fixme: better name.
struct SourceTransactionExecuteView<'a, D> {
    transaction: &'a Transaction<D>,
    chunk: SourceChunk<'a>,
    indecies_by_node_id: HashMap<AccountId, usize>,
}

impl<'a, D> SourceTransactionExecuteView<'a, D> {
    fn new(transaction: &'a Transaction<D>, chunk: SourceChunk<'a>) -> Self {
        let indecies_by_node_id =
            chunk.node_ids().iter().copied().enumerate().map(|it| (it.1, it.0)).collect();
        Self { transaction, chunk, indecies_by_node_id }
    }
}

impl<'a, D: ValidateChecksums> ValidateChecksums for SourceTransactionExecuteView<'a, D> {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.transaction.validate_checksums(ledger_id)
    }
}

impl<'a, D: TransactionExecute> Execute for SourceTransactionExecuteView<'a, D> {
    type GrpcRequest = <Transaction<D> as Execute>::GrpcRequest;

    type GrpcResponse = <Transaction<D> as Execute>::GrpcResponse;

    type Context = <Transaction<D> as Execute>::Context;

    type Response = <Transaction<D> as Execute>::Response;

    fn node_account_ids(&self) -> Option<&[AccountId]> {
        Some(self.chunk.node_ids())
    }

    fn transaction_id(&self) -> Option<TransactionId> {
        Some(self.chunk.transaction_id())
    }

    fn requires_transaction_id(&self) -> bool {
        true
    }

    fn operator_account_id(&self) -> Option<&AccountId> {
        None
    }

    fn regenerate_transaction_id(&self) -> Option<bool> {
        Some(false)
    }

    fn make_request(
        &self,
        transaction_id: Option<&TransactionId>,
        node_account_id: AccountId,
    ) -> crate::Result<(Self::GrpcRequest, Self::Context)> {
        debug_assert_eq!(transaction_id, self.transaction_id().as_ref());

        let index = *self.indecies_by_node_id.get(&node_account_id).unwrap();
        Ok((self.chunk.transactions()[index].clone(), self.chunk.transaction_hashes()[index]))
    }

    fn execute(
        &self,
        channel: Channel,
        request: Self::GrpcRequest,
    ) -> BoxGrpcFuture<Self::GrpcResponse> {
        self.transaction.execute(channel, request)
    }

    fn make_response(
        &self,
        response: Self::GrpcResponse,
        context: Self::Context,
        node_account_id: AccountId,
        transaction_id: Option<&TransactionId>,
    ) -> crate::Result<Self::Response> {
        self.transaction.make_response(response, context, node_account_id, transaction_id)
    }

    fn make_error_pre_check(
        &self,
        status: crate::Status,
        transaction_id: Option<&TransactionId>,
        response: Self::GrpcResponse,
    ) -> crate::Error {
        self.transaction.make_error_pre_check(status, transaction_id, response)
    }

    fn response_pre_check_status(response: &Self::GrpcResponse) -> crate::Result<i32> {
        Transaction::<D>::response_pre_check_status(response)
    }
}
