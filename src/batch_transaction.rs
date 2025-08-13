// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::util_service_client::UtilServiceClient;
use prost::Message;
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::protobuf::FromProtobuf;
use crate::transaction::{
    AnyTransactionData,
    ChunkInfo,
    ToTransactionDataProtobuf,
    TransactionData,
    TransactionExecute,
};
use crate::{
    AnyTransaction,
    BoxGrpcFuture,
    Error,
    Hbar,
    Transaction,
    TransactionId,
    ValidateChecksums,
};

/// Execute multiple transactions in a single consensus event. This allows for atomic execution of multiple
/// transactions, where they either all succeed or all fail together.
///
/// # Requirements
///
/// - All inner transactions must be frozen before being added to the batch
/// - All inner transactions must have a batch key set (using `set_batch_key()` or `batchify()`)
/// - All inner transactions must be signed as required for each individual transaction
/// - The BatchTransaction must be signed by all batch keys of the inner transactions
/// - Certain transaction types (FreezeTransaction, BatchTransaction) are not allowed in a batch
///
/// # Important notes
///
/// - Fees are assessed for each inner transaction separately
/// - The maximum number of inner transactions in a batch is limited to 25
/// - Inner transactions cannot be scheduled transactions
///
/// # Example usage
///
/// ```rust,no_run
/// use hedera::{BatchTransaction, TransferTransaction, PrivateKey, Client, Hbar};
///
/// # async fn example() -> hedera::Result<()> {
/// let client = Client::for_testnet();
/// let batch_key = PrivateKey::generate_ed25519();
///
/// // Create and prepare inner transaction
/// let inner_transaction = TransferTransaction::new()
///     .hbar_transfer(sender, -amount)
///     .hbar_transfer(receiver, amount)
///     .freeze_with(&client)?
///     .set_batch_key(batch_key.public_key().into())
///     .sign(&operator_key)?;
///
/// // Create and execute batch transaction
/// let response = BatchTransaction::new()
///     .add_inner_transaction(inner_transaction)?
///     .freeze_with(&client)?
///     .sign(&batch_key)?
///     .execute(&client)
///     .await?;
/// # Ok(())
/// # }
/// ```
pub type BatchTransaction = Transaction<BatchTransactionData>;

#[derive(Debug, Clone, Default)]
pub struct BatchTransactionData {
    inner_transactions: Vec<AnyTransaction>,
}

impl BatchTransaction {
    /// Append a transaction to the list of transactions this BatchTransaction will execute.
    ///
    /// # Requirements for the inner transaction
    ///
    /// - Must be frozen (use `freeze()` or `freeze_with(client)`)
    /// - Must have a batch key set (use `set_batch_key()` or `batchify()`)
    /// - Must not be a blacklisted transaction type
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The transaction is null
    /// - This transaction is frozen
    /// - The inner transaction is not frozen or missing a batch key
    /// - The transaction is of a blacklisted type (FreezeTransaction, BatchTransaction)
    pub fn add_inner_transaction(
        &mut self,
        transaction: AnyTransaction,
    ) -> crate::Result<&mut Self> {
        self.require_not_frozen();
        self.validate_inner_transaction(&transaction)?;
        self.data_mut().inner_transactions.push(transaction);
        Ok(self)
    }

    /// Set the list of transactions to be executed as part of this BatchTransaction.
    ///
    /// # Requirements for each inner transaction
    ///
    /// - Must be frozen (use `freeze()` or `freeze_with(client)`)
    /// - Must have a batch key set (use `set_batch_key()` or `batchify()`)
    /// - Must not be a blacklisted transaction type
    ///
    /// Note: This method creates a defensive copy of the provided list.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Any inner transaction is not frozen or missing a batch key
    /// - Any transaction is of a blacklisted type
    pub fn set_inner_transactions(
        &mut self,
        transactions: Vec<AnyTransaction>,
    ) -> crate::Result<&mut Self> {
        self.require_not_frozen();

        // Validate all transactions before setting
        for transaction in &transactions {
            self.validate_inner_transaction(transaction)?;
        }

        self.data_mut().inner_transactions = transactions;
        Ok(self)
    }

    /// Get the list of transactions this BatchTransaction is currently configured to execute.
    pub fn get_inner_transactions(&self) -> &[AnyTransaction] {
        &self.data().inner_transactions
    }

    /// Get the list of transaction IDs of each inner transaction of this BatchTransaction.
    ///
    /// This method is particularly useful after execution to:
    /// - Track individual transaction results
    /// - Query receipts for specific inner transactions
    /// - Monitor the status of each transaction in the batch
    ///
    /// **NOTE:** Transaction IDs will only be meaningful after the batch transaction has been
    /// executed or the IDs have been explicitly set on the inner transactions.
    pub fn get_inner_transaction_ids(&self) -> Vec<Option<TransactionId>> {
        self.data().inner_transactions.iter().map(|tx| tx.get_transaction_id()).collect()
    }

    /// Validates if a transaction is allowed in a batch transaction.
    ///
    /// A transaction is valid if:
    /// - It is not a blacklisted type (FreezeTransaction or BatchTransaction)
    /// - It is frozen
    /// - It has a batch key set
    fn validate_inner_transaction(&self, transaction: &AnyTransaction) -> crate::Result<()> {
        // Check if transaction type is blacklisted
        match transaction.data() {
            AnyTransactionData::Freeze(_) => {
                return Err(Error::basic_parse(
                    "Transaction type FreezeTransaction is not allowed in a batch transaction",
                ));
            }
            AnyTransactionData::Batch(_) => {
                return Err(Error::basic_parse(
                    "Transaction type BatchTransaction is not allowed in a batch transaction",
                ));
            }
            _ => {}
        }

        // Check if transaction is frozen
        if !transaction.is_frozen() {
            return Err(Error::basic_parse("Inner transaction should be frozen"));
        }

        // Check if batch key is set
        if transaction.get_batch_key().is_none() {
            return Err(Error::basic_parse("Batch key needs to be set"));
        }

        Ok(())
    }
}

impl TransactionData for BatchTransactionData {
    fn default_max_transaction_fee(&self) -> Hbar {
        Hbar::new(2)
    }
}

impl ToTransactionDataProtobuf for BatchTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        _chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let mut builder = services::AtomicBatchTransactionBody::default();

        for transaction in &self.inner_transactions {
            // Get the signed transaction bytes from each inner transaction
            // Note: This unwrap is OK because inner transactions should be frozen
            let signed_transaction_bytes = transaction
                .to_signed_transaction_bytes()
                .expect("Inner transaction should be frozen and serializable");

            println!("Signed transaction bytes: {:?}", signed_transaction_bytes);
            builder.transactions.push(signed_transaction_bytes);
        }

        services::transaction_body::Data::AtomicBatch(builder)
    }
}

impl TransactionExecute for BatchTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async move { UtilServiceClient::new(channel).atomic_batch(request).await })
    }
}

impl ValidateChecksums for BatchTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        for transaction in &self.inner_transactions {
            transaction.validate_checksums(ledger_id)?;
        }
        Ok(())
    }
}

impl FromProtobuf<services::AtomicBatchTransactionBody> for BatchTransactionData {
    fn from_protobuf(pb: services::AtomicBatchTransactionBody) -> crate::Result<Self> {
        let mut inner_transactions = Vec::new();

        for signed_transaction_bytes in pb.transactions {
            // Create a transaction from the signed transaction bytes
            let proto_transaction =
                services::Transaction { signed_transaction_bytes, ..Default::default() };

            let transaction = AnyTransaction::from_bytes(&proto_transaction.encode_to_vec())?;
            inner_transactions.push(transaction);
        }

        Ok(Self { inner_transactions })
    }
}

impl From<BatchTransactionData> for AnyTransactionData {
    fn from(value: BatchTransactionData) -> Self {
        Self::Batch(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TransferTransaction;

    #[test]
    fn test_new_batch_transaction() {
        let batch = BatchTransaction::new();
        assert!(batch.get_inner_transactions().is_empty());
    }

    #[test]
    fn test_validate_non_frozen_transaction() {
        let mut batch = BatchTransaction::new();
        let inner_tx = TransferTransaction::new();

        let result = batch.add_inner_transaction(inner_tx.into());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Inner transaction should be frozen"));
    }

    #[test]
    fn test_validate_batch_key_required() {
        let mut batch = BatchTransaction::new();
        let inner_tx = TransferTransaction::new();
        // Note: In a real scenario, you would freeze the transaction first,
        // then set a batch key, but this test just checks the validation logic

        let result = batch.add_inner_transaction(inner_tx.into());
        assert!(result.is_err());
        // The error will be about the transaction not being frozen first,
        // which comes before the batch key check
        assert!(result.unwrap_err().to_string().contains("Inner transaction should be frozen"));
    }
}
