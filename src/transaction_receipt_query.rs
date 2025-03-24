// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::crypto_service_client::CryptoServiceClient;
use hedera_proto::services::response::Response;
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::query::{
    AnyQueryData,
    QueryExecute,
    ToQueryProtobuf,
};
use crate::{
    BoxGrpcFuture,
    Error,
    Query,
    Status,
    ToProtobuf,
    TransactionId,
    TransactionReceipt,
    ValidateChecksums,
};

/// Get the receipt of a transaction, given its transaction ID.
///
/// Once a transaction reaches consensus, then information about whether it succeeded or failed
/// will be available until the end of the receipt period.
///
pub type TransactionReceiptQuery = Query<TransactionReceiptQueryData>;

#[derive(Default, Clone, Debug)]
pub struct TransactionReceiptQueryData {
    transaction_id: Option<TransactionId>,
    include_children: bool,
    include_duplicates: bool,
    validate_status: bool,
}

impl From<TransactionReceiptQueryData> for AnyQueryData {
    #[inline]
    fn from(data: TransactionReceiptQueryData) -> Self {
        Self::TransactionReceipt(data)
    }
}

impl TransactionReceiptQuery {
    /// Get the ID of the transaction for which the receipt is being requested.
    #[must_use]
    pub fn get_transaction_id(&self) -> Option<TransactionId> {
        self.data.transaction_id
    }

    /// Sets the ID of the transaction for which the receipt is being requested.
    pub fn transaction_id(&mut self, transaction_id: TransactionId) -> &mut Self {
        self.data.transaction_id = Some(transaction_id);
        self
    }

    /// Whether the response should include the receipts of any child transactions spawned by the
    /// top-level transaction with the given transaction.
    #[must_use]
    pub fn get_include_children(&self) -> bool {
        self.data.include_children
    }

    /// Whether the response should include the receipts of any child transactions spawned by the
    /// top-level transaction with the given transaction.
    pub fn include_children(&mut self, include: bool) -> &mut Self {
        self.data.include_children = include;
        self
    }

    /// Whether receipts of processing duplicate transactions should be returned.
    #[must_use]
    pub fn get_include_duplicates(&self) -> bool {
        self.data.include_duplicates
    }

    /// Whether receipts of processing duplicate transactions should be returned.
    pub fn include_duplicates(&mut self, include: bool) -> &mut Self {
        self.data.include_duplicates = include;
        self
    }

    /// Whether the receipt status should be validated.
    #[must_use]
    pub fn get_validate_status(&self) -> bool {
        self.data.validate_status
    }

    /// Whether the receipt status should be validated.
    pub fn validate_status(&mut self, validate: bool) -> &mut Self {
        self.data.validate_status = validate;
        self
    }
}

impl ToQueryProtobuf for TransactionReceiptQueryData {
    fn to_query_protobuf(&self, header: services::QueryHeader) -> services::Query {
        let transaction_id = self.transaction_id.to_protobuf();

        services::Query {
            query: Some(services::query::Query::TransactionGetReceipt(
                services::TransactionGetReceiptQuery {
                    header: Some(header),
                    transaction_id,
                    include_child_receipts: self.include_children,
                    include_duplicates: self.include_duplicates,
                },
            )),
        }
    }
}

impl QueryExecute for TransactionReceiptQueryData {
    type Response = TransactionReceipt;

    fn is_payment_required(&self) -> bool {
        false
    }

    fn transaction_id(&self) -> Option<TransactionId> {
        self.transaction_id
    }

    fn execute(
        &self,
        channel: Channel,
        request: services::Query,
    ) -> BoxGrpcFuture<'_, services::Response> {
        Box::pin(async {
            CryptoServiceClient::new(channel).get_transaction_receipts(request).await
        })
    }

    fn should_retry_pre_check(&self, status: Status) -> bool {
        matches!(status, Status::ReceiptNotFound | Status::RecordNotFound)
    }

    fn should_retry(&self, response: &services::Response) -> bool {
        // extract the receipt status from the receipt
        // without altering or freeing the memory from the response

        let receipt_status = {
            let Some(services::response::Response::TransactionGetReceipt(r)) = &response.response
            else {
                return false;
            };

            match r.receipt.as_ref().and_then(|it| Some(Status::try_from(it.status))) {
                Some(receipt_status) => receipt_status,
                None => return false,
            }
        };

        matches!(receipt_status, Ok(Status::Unknown))
    }

    fn make_response(&self, response: Response) -> crate::Result<Self::Response> {
        let receipt =
            TransactionReceipt::from_response_protobuf(response, self.transaction_id.as_ref())?;

        if self.validate_status && receipt.status != Status::Success {
            return Err(Error::ReceiptStatus {
                transaction_id: self.transaction_id.map(Box::new),
                status: receipt.status,
            });
        }

        Ok(receipt)
    }
}

impl ValidateChecksums for TransactionReceiptQueryData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.transaction_id.validate_checksums(ledger_id)
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::query::ToQueryProtobuf;
    use crate::transaction::test_helpers::TEST_TX_ID;
    use crate::TransactionReceiptQuery;

    #[test]
    fn serialize() {
        expect![[r#"
            Query {
                query: Some(
                    TransactionGetReceipt(
                        TransactionGetReceiptQuery {
                            header: Some(
                                QueryHeader {
                                    payment: None,
                                    response_type: AnswerOnly,
                                },
                            ),
                            transaction_id: Some(
                                TransactionId {
                                    transaction_valid_start: Some(
                                        Timestamp {
                                            seconds: 1554158542,
                                            nanos: 0,
                                        },
                                    ),
                                    account_id: Some(
                                        AccountId {
                                            shard_num: 0,
                                            realm_num: 0,
                                            account: Some(
                                                AccountNum(
                                                    5006,
                                                ),
                                            ),
                                        },
                                    ),
                                    scheduled: false,
                                    nonce: 0,
                                },
                            ),
                            include_duplicates: false,
                            include_child_receipts: false,
                        },
                    ),
                ),
            }
        "#]]
        .assert_debug_eq(
            &TransactionReceiptQuery::new()
                .transaction_id(TEST_TX_ID)
                .data
                .to_query_protobuf(Default::default()),
        )
    }

    #[test]
    fn get_set_transaction_id() {
        let mut query = TransactionReceiptQuery::new();
        query.transaction_id(TEST_TX_ID);

        assert_eq!(query.get_transaction_id(), Some(TEST_TX_ID));
    }

    // default is false for all of these, so setting it to `true` is the "interesting" state.
    #[test]
    fn get_set_include_children() {
        let mut query = TransactionReceiptQuery::new();
        query.include_children(true);

        assert_eq!(query.get_include_children(), true);
    }

    #[test]
    fn get_set_include_duplicates() {
        let mut query = TransactionReceiptQuery::new();
        query.include_duplicates(true);

        assert_eq!(query.get_include_duplicates(), true);
    }

    #[test]
    fn get_set_validate_status() {
        let mut query = TransactionReceiptQuery::new();
        query.validate_status(true);

        assert_eq!(query.get_validate_status(), true);
    }
}
