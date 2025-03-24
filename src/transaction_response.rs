// SPDX-License-Identifier: Apache-2.0

use crate::{
    AccountId,
    Client,
    TransactionHash,
    TransactionId,
    TransactionReceipt,
    TransactionReceiptQuery,
    TransactionRecord,
    TransactionRecordQuery,
};

/// Response from [`Transaction::execute`][crate::Transaction::execute].
///
/// When the client sends a node a transaction of any kind, the node replies with this, which
/// simply says that the transaction passed the pre-check (so the node will submit it to
/// the network).
///
/// To learn the consensus result, the client should later obtain a
/// receipt (free), or can buy a more detailed record (not free).
///
#[derive(Debug)]
pub struct TransactionResponse {
    /// The account ID of the node that the transaction was submitted to.
    pub node_account_id: AccountId,

    /// The client-generated transaction ID of the transaction that was submitted.
    ///
    /// This can be used to lookup the transaction in an explorer.
    pub transaction_id: TransactionId,

    /// The client-generated SHA-384 hash of the transaction that was submitted.
    ///
    /// This can be used to lookup the transaction in an explorer.
    pub transaction_hash: TransactionHash,

    /// Whether the receipt/record status should be validated.
    pub validate_status: bool,
}

impl TransactionResponse {
    /// Whether the receipt/record status should be validated.
    pub fn validate_status(&mut self, validate: bool) -> &mut Self {
        self.validate_status = validate;
        self
    }

    /// Create a query that will get the receipt for this transaction.
    #[must_use]
    pub fn get_receipt_query(&self) -> TransactionReceiptQuery {
        let mut query = TransactionReceiptQuery::new();

        query.transaction_id(self.transaction_id).validate_status(self.validate_status);

        query
    }

    /// Create a query that will get the record for this transaction.
    #[must_use]
    pub fn get_record_query(&self) -> TransactionRecordQuery {
        let mut query = TransactionRecordQuery::new();

        query.transaction_id(self.transaction_id).validate_status(self.validate_status);

        query
    }

    /// Get the receipt for this transaction.
    /// Will wait for consensus.
    ///
    /// # Errors
    /// - if [`validate_status`](Self.validate_status) is `true`:
    ///   [`Error::ReceiptStatus`](crate::Error::ReceiptStatus) for a failing receipt.
    ///
    /// fixme: is that it? Surely there are more situations.
    pub async fn get_receipt(&self, client: &Client) -> crate::Result<TransactionReceipt> {
        self.get_receipt_query().execute(client).await
    }

    /// Get the receipt for this transaction.
    /// Will wait for consensus.
    ///
    /// # Errors
    /// - if [`validate_status`](Self.validate_status) is `true`:
    ///   [`Error::ReceiptStatus`](crate::Error::ReceiptStatus) for a failing receipt.
    pub async fn get_receipt_with_timeout(
        &self,
        client: &Client,
        timeout: std::time::Duration,
    ) -> crate::Result<TransactionReceipt> {
        self.get_receipt_query().execute_with_timeout(client, timeout).await
    }

    /// Get the record for this transaction.
    /// Will wait for consensus.
    ///
    /// # Errors
    /// - if [`validate_status`](Self.validate_status) is `true`:
    ///   [`Error::ReceiptStatus`](crate::Error::ReceiptStatus) for a failing receipt in the record.
    pub async fn get_record(&self, client: &Client) -> crate::Result<TransactionRecord> {
        self.get_record_query().execute(client).await
    }

    /// Get the record for this transaction.
    /// Will wait for consensus.
    ///
    /// # Errors
    /// - if [`validate_status`](Self.validate_status) is `true`:
    ///   [`Error::ReceiptStatus`](crate::Error::ReceiptStatus) for a failing receipt in the record.
    pub async fn get_record_with_timeout(
        &self,
        client: &Client,
        timeout: std::time::Duration,
    ) -> crate::Result<TransactionRecord> {
        self.get_record_query().execute_with_timeout(client, timeout).await
    }
}
