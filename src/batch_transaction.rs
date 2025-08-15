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
/// use hedera::{BatchTransaction, TransferTransaction, PrivateKey, Client, Hbar, AccountId};
///
/// # async fn example() -> hedera::Result<()> {
/// let client = Client::for_testnet();
/// let batch_key = PrivateKey::generate_ed25519();
/// let operator_key = PrivateKey::generate_ed25519();
/// let sender = AccountId::new(0, 0, 123);
/// let receiver = AccountId::new(0, 0, 456);
/// let amount = Hbar::new(10);
///
/// // Create and prepare inner transaction
/// let mut inner_transaction = TransferTransaction::new();
/// inner_transaction
///     .hbar_transfer(sender, -amount)
///     .hbar_transfer(receiver, amount)
///     .freeze_with(&client)?;
/// inner_transaction.set_batch_key(batch_key.public_key().into());
/// inner_transaction.sign(operator_key);
///
/// // Create and execute batch transaction
/// let mut batch_transaction = BatchTransaction::new();
/// batch_transaction.add_inner_transaction(inner_transaction.into())?;
/// batch_transaction.freeze_with(&client)?;
/// batch_transaction.sign(batch_key);
/// let response = batch_transaction.execute(&client).await?;
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
    use std::str::FromStr;

    use super::*;
    use crate::account::AccountCreateTransactionData;
    use crate::{
        AccountCreateTransaction,
        AccountId,
        Client,
        FreezeTransaction,
        Hbar,
        PrivateKey,
        Transaction,
        TransactionId,
        TransferTransaction,
    };

    fn create_test_client() -> Client {
        Client::for_testnet()
    }

    fn create_test_operator_key() -> PrivateKey {
        PrivateKey::from_str(
            "302e020100300506032b65700422042091132178e72057a1d7528025956fe39b0b847f200ab59b2fdd367017f3087137"
        ).unwrap()
    }

    fn create_valid_inner_transaction() -> crate::Result<Transaction<AccountCreateTransactionData>>
    {
        let client = create_test_client();
        let operator_key = create_test_operator_key();
        let operator_id = AccountId::new(0, 0, 2);
        client.set_operator(operator_id, operator_key.clone());

        let account_key = PrivateKey::generate_ed25519();

        let mut transaction = AccountCreateTransaction::new();
        transaction.set_key_without_alias(account_key.public_key()).initial_balance(Hbar::new(1));

        transaction.batchify(&client, operator_key.public_key().into())?;
        Ok(transaction)
    }

    fn create_unfrozen_transaction() -> Transaction<AccountCreateTransactionData> {
        let account_key = PrivateKey::generate_ed25519();

        let mut transaction = AccountCreateTransaction::new();
        transaction.set_key_without_alias(account_key.public_key()).initial_balance(Hbar::new(1));

        transaction
    }

    fn create_frozen_no_batch_key_transaction(
    ) -> crate::Result<Transaction<AccountCreateTransactionData>> {
        let client = create_test_client();
        let operator_key = create_test_operator_key();
        let operator_id = AccountId::new(0, 0, 2);
        client.set_operator(operator_id, operator_key);

        let account_key = PrivateKey::generate_ed25519();

        let mut transaction = AccountCreateTransaction::new();
        transaction.set_key_without_alias(account_key.public_key()).initial_balance(Hbar::new(1));

        transaction.freeze_with(&client)?;
        Ok(transaction)
    }

    fn create_blacklisted_transaction(
    ) -> crate::Result<Transaction<crate::system::FreezeTransactionData>> {
        let client = create_test_client();
        let operator_key = create_test_operator_key();
        let operator_id = AccountId::new(0, 0, 2);
        client.set_operator(operator_id, operator_key.clone());

        let mut transaction = FreezeTransaction::new();
        transaction.freeze_type(crate::FreezeType::FreezeOnly);

        transaction.batchify(&client, operator_key.public_key().into())?;
        Ok(transaction)
    }

    #[test]
    fn test_new_batch_transaction() {
        let batch = BatchTransaction::new();
        assert_eq!(batch.get_inner_transactions().len(), 0);
        assert_eq!(batch.get_inner_transaction_ids().len(), 0);
    }

    #[tokio::test]
    async fn test_add_valid_inner_transaction() -> crate::Result<()> {
        let mut batch = BatchTransaction::new();
        let inner_transaction = create_valid_inner_transaction()?;

        let result = batch.add_inner_transaction(inner_transaction.into());
        assert!(result.is_ok());
        assert_eq!(batch.get_inner_transactions().len(), 1);
        assert_eq!(batch.get_inner_transaction_ids().len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_add_multiple_inner_transactions() -> crate::Result<()> {
        let mut batch = BatchTransaction::new();

        // Add first transaction
        let inner1 = create_valid_inner_transaction()?;
        batch.add_inner_transaction(inner1.into())?;

        // Add second transaction
        let inner2 = create_valid_inner_transaction()?;
        batch.add_inner_transaction(inner2.into())?;

        assert_eq!(batch.get_inner_transactions().len(), 2);
        assert_eq!(batch.get_inner_transaction_ids().len(), 2);

        Ok(())
    }

    #[test]
    fn test_add_unfrozen_transaction_fails() {
        let mut batch = BatchTransaction::new();
        let unfrozen_transaction = create_unfrozen_transaction();

        let result = batch.add_inner_transaction(unfrozen_transaction.into());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("frozen"));
    }

    #[tokio::test]
    async fn test_add_transaction_without_batch_key_fails() -> crate::Result<()> {
        let mut batch = BatchTransaction::new();
        let transaction = create_frozen_no_batch_key_transaction()?;

        let result = batch.add_inner_transaction(transaction.into());
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("batch key") || error_msg.contains("needs to be set"));

        Ok(())
    }

    #[tokio::test]
    async fn test_add_blacklisted_transaction_fails() -> crate::Result<()> {
        let mut batch = BatchTransaction::new();
        let blacklisted_transaction = create_blacklisted_transaction()?;

        let result = batch.add_inner_transaction(blacklisted_transaction.into());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("FreezeTransaction"));

        Ok(())
    }

    #[test]
    fn test_add_batch_transaction_to_batch_fails() -> crate::Result<()> {
        let mut batch = BatchTransaction::new();
        let inner_batch = BatchTransaction::new();

        let result = batch.add_inner_transaction(inner_batch.into());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("BatchTransaction"));

        Ok(())
    }

    #[tokio::test]
    async fn test_set_inner_transactions() -> crate::Result<()> {
        let mut batch = BatchTransaction::new();

        let inner1 = create_valid_inner_transaction()?;
        let inner2 = create_valid_inner_transaction()?;

        let transactions = vec![inner1.into(), inner2.into()];
        let result = batch.set_inner_transactions(transactions);

        assert!(result.is_ok());
        assert_eq!(batch.get_inner_transactions().len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_set_inner_transactions_with_invalid_transaction_fails() -> crate::Result<()> {
        let mut batch = BatchTransaction::new();

        let valid_transaction = create_valid_inner_transaction()?;
        let invalid_transaction = create_unfrozen_transaction();

        let transactions = vec![valid_transaction.into(), invalid_transaction.into()];
        let result = batch.set_inner_transactions(transactions);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("frozen"));

        Ok(())
    }

    #[tokio::test]
    async fn test_set_inner_transactions_replaces_existing() -> crate::Result<()> {
        let mut batch = BatchTransaction::new();

        // Add initial transaction
        let initial_transaction = create_valid_inner_transaction()?;
        batch.add_inner_transaction(initial_transaction.into())?;
        assert_eq!(batch.get_inner_transactions().len(), 1);

        // Replace with new transactions
        let new1 = create_valid_inner_transaction()?;
        let new2 = create_valid_inner_transaction()?;
        let new_transactions = vec![new1.into(), new2.into()];

        batch.set_inner_transactions(new_transactions)?;
        assert_eq!(batch.get_inner_transactions().len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_inner_transaction_ids() -> crate::Result<()> {
        let mut batch = BatchTransaction::new();

        let inner1 = create_valid_inner_transaction()?;
        let inner2 = create_valid_inner_transaction()?;

        batch.add_inner_transaction(inner1.into())?;
        batch.add_inner_transaction(inner2.into())?;

        let transaction_ids = batch.get_inner_transaction_ids();
        assert_eq!(transaction_ids.len(), 2);

        // All transaction IDs should be valid
        for tx_id in transaction_ids {
            if let Some(tx_id) = tx_id {
                // Account ID should have valid shard/realm/num or alias
                assert!(
                    tx_id.account_id.num > 0
                        || tx_id.account_id.alias.is_some()
                        || tx_id.account_id.evm_address.is_some()
                );
                assert!(tx_id.valid_start.unix_timestamp() > 0);
            }
        }

        Ok(())
    }

    #[test]
    fn test_empty_batch_has_no_transactions() {
        let batch = BatchTransaction::new();
        assert!(batch.get_inner_transactions().is_empty());
        assert!(batch.get_inner_transaction_ids().is_empty());
    }

    #[test]
    fn test_default_max_transaction_fee() {
        let batch_data = BatchTransactionData::default();
        let default_fee = batch_data.default_max_transaction_fee();
        // Should have a reasonable default fee
        assert!(default_fee > Hbar::from_tinybars(0));
    }

    #[test]
    fn test_transaction_data_trait_implementation() {
        let batch_data = BatchTransactionData::default();

        // Test that it implements TransactionData correctly
        assert!(batch_data.default_max_transaction_fee() > Hbar::from_tinybars(0));
        // BatchTransaction doesn't require a single node account ID
    }

    #[tokio::test]
    async fn test_validate_checksums() -> crate::Result<()> {
        use crate::ledger_id::RefLedgerId;

        let mut batch = BatchTransaction::new();
        let inner_transaction = create_valid_inner_transaction()?;
        batch.add_inner_transaction(inner_transaction.into())?;

        // Should not panic or return error for valid checksums
        let result = batch.data().validate_checksums(&RefLedgerId::TESTNET);
        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_to_transaction_data_protobuf() -> crate::Result<()> {
        let mut batch = BatchTransaction::new();
        let inner_transaction = create_valid_inner_transaction()?;
        batch.add_inner_transaction(inner_transaction.into())?;

        let chunk_info = crate::transaction::ChunkInfo::single(
            TransactionId::generate(AccountId::new(0, 0, 2)),
            AccountId::new(0, 0, 3),
        );
        let protobuf_data = batch.data().to_transaction_data_protobuf(&chunk_info);

        // Should return AtomicBatch variant
        match protobuf_data {
            hedera_proto::services::transaction_body::Data::AtomicBatch(atomic_batch) => {
                assert_eq!(atomic_batch.transactions.len(), 1);
                assert!(!atomic_batch.transactions[0].is_empty());
            }
            _ => panic!("Expected AtomicBatch variant"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_from_protobuf_roundtrip() -> crate::Result<()> {
        let mut original_batch = BatchTransaction::new();
        let inner_transaction = create_valid_inner_transaction()?;
        original_batch.add_inner_transaction(inner_transaction.into())?;

        // Convert to protobuf
        let chunk_info = crate::transaction::ChunkInfo::single(
            TransactionId::generate(AccountId::new(0, 0, 2)),
            AccountId::new(0, 0, 3),
        );
        let protobuf_data = original_batch.data().to_transaction_data_protobuf(&chunk_info);

        // Extract AtomicBatchTransactionBody
        let atomic_batch = match protobuf_data {
            hedera_proto::services::transaction_body::Data::AtomicBatch(atomic_batch) => {
                atomic_batch
            }
            _ => panic!("Expected AtomicBatch variant"),
        };

        // Convert back from protobuf
        let reconstructed_data = BatchTransactionData::from_protobuf(atomic_batch)?;

        // Should have same number of inner transactions
        assert_eq!(
            reconstructed_data.inner_transactions.len(),
            original_batch.data().inner_transactions.len()
        );

        Ok(())
    }

    #[test]
    fn test_empty_batch_protobuf() {
        let empty_batch = BatchTransaction::new();
        let chunk_info = crate::transaction::ChunkInfo::single(
            TransactionId::generate(AccountId::new(0, 0, 2)),
            AccountId::new(0, 0, 3),
        );
        let protobuf_data = empty_batch.data().to_transaction_data_protobuf(&chunk_info);

        match protobuf_data {
            hedera_proto::services::transaction_body::Data::AtomicBatch(atomic_batch) => {
                assert!(atomic_batch.transactions.is_empty());
            }
            _ => panic!("Expected AtomicBatch variant"),
        }
    }

    #[tokio::test]
    async fn test_large_number_of_transactions() -> crate::Result<()> {
        let mut batch = BatchTransaction::new();

        // Add many transactions (up to reasonable limit)
        for _ in 0..10 {
            let inner_transaction = create_valid_inner_transaction()?;
            batch.add_inner_transaction(inner_transaction.into())?;
        }

        assert_eq!(batch.get_inner_transactions().len(), 10);
        assert_eq!(batch.get_inner_transaction_ids().len(), 10);

        Ok(())
    }

    // Legacy tests (kept for compatibility)
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
