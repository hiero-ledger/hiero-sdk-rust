use hedera_proto::services;
use tonic::transport::Channel;

use super::{
    AnyTransactionData,
    ChunkInfo,
    ToTransactionDataProtobuf,
    TransactionBody,
    TransactionData,
    TransactionExecute,
};
use crate::{
    BoxGrpcFuture,
    Transaction,
    ValidateChecksums,
};

#[derive(Clone)]
pub(crate) struct CostTransactionData<D> {
    pub(crate) inner: D,
}

pub(crate) type CostTransaction<D> = Transaction<CostTransactionData<D>>;

impl<D: Clone> CostTransaction<D> {
    pub(crate) fn from_transaction(transaction: &Transaction<D>) -> Self {
        let transaction = transaction.clone();

        Self {
            body: TransactionBody {
                data: CostTransactionData { inner: transaction.body.data },
                node_account_ids: transaction.body.node_account_ids,
                transaction_valid_duration: transaction.body.transaction_valid_duration,
                max_transaction_fee: transaction.body.max_transaction_fee,
                transaction_memo: transaction.body.transaction_memo,
                transaction_id: transaction.body.transaction_id,
                operator: transaction.body.operator,
                is_frozen: transaction.body.is_frozen,
                regenerate_transaction_id: transaction.body.regenerate_transaction_id,
                custom_fee_limits: transaction.body.custom_fee_limits,
            },
            // cost transactions have no signers
            signers: Vec::new(),
            sources: transaction.sources,
        }
    }
}

impl<D: Into<AnyTransactionData>> From<CostTransactionData<D>> for AnyTransactionData {
    fn from(transaction: CostTransactionData<D>) -> Self {
        transaction.inner.into()
    }
}

impl<D: TransactionData> TransactionData for CostTransactionData<D> {
    #[doc(hidden)]
    fn for_cost_estimate(&self) -> bool {
        true
    }
}

impl<D: TransactionExecute> TransactionExecute for CostTransactionData<D> {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        self.inner.execute(channel, request)
    }
}

impl<D: ValidateChecksums> ValidateChecksums for CostTransactionData<D> {
    fn validate_checksums(&self, ledger_id: &crate::ledger_id::RefLedgerId) -> crate::Result<()> {
        self.inner.validate_checksums(ledger_id)
    }
}

impl<D: ToTransactionDataProtobuf> ToTransactionDataProtobuf for CostTransactionData<D> {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        self.inner.to_transaction_data_protobuf(chunk_info)
    }
}
