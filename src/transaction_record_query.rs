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
    FromProtobuf,
    Query,
    Status,
    ToProtobuf,
    TransactionId,
    TransactionRecord,
    ValidateChecksums,
};

/// Get the record of a transaction, given its transaction ID.
///
pub type TransactionRecordQuery = Query<TransactionRecordQueryData>;

#[derive(Default, Clone, Debug)]
pub struct TransactionRecordQueryData {
    transaction_id: Option<TransactionId>,
    include_children: bool,
    include_duplicates: bool,
    validate_status: bool,
}

impl From<TransactionRecordQueryData> for AnyQueryData {
    #[inline]
    fn from(data: TransactionRecordQueryData) -> Self {
        Self::TransactionRecord(data)
    }
}

impl TransactionRecordQuery {
    /// Get the ID of the transaction for which the record is being requested.
    #[must_use]
    pub fn get_transaction_id(&self) -> Option<TransactionId> {
        self.data.transaction_id
    }

    /// Sets the ID of the transaction for which the record is being requested.
    pub fn transaction_id(&mut self, transaction_id: TransactionId) -> &mut Self {
        self.data.transaction_id = Some(transaction_id);
        self
    }

    /// Whether the response should include the records of any child transactions spawned by the
    /// top-level transaction with the given transaction.
    #[must_use]
    pub fn get_include_children(&self) -> bool {
        self.data.include_children
    }

    /// Whether the response should include the records of any child transactions spawned by the
    /// top-level transaction with the given transaction.
    pub fn include_children(&mut self, include: bool) -> &mut Self {
        self.data.include_children = include;
        self
    }

    /// Whether records of processing duplicate transactions should be returned.
    #[must_use]
    pub fn get_include_duplicates(&self) -> bool {
        self.data.include_duplicates
    }

    /// Whether records of processing duplicate transactions should be returned.
    pub fn include_duplicates(&mut self, include: bool) -> &mut Self {
        self.data.include_duplicates = include;
        self
    }

    /// Whether records of processing duplicate transactions should be returned.
    #[must_use]
    pub fn get_validate_status(&self) -> bool {
        self.data.validate_status
    }

    /// Whether the record status should be validated.
    pub fn validate_status(&mut self, validate: bool) -> &mut Self {
        self.data.validate_status = validate;
        self
    }
}

impl ToQueryProtobuf for TransactionRecordQueryData {
    fn to_query_protobuf(&self, header: services::QueryHeader) -> services::Query {
        let transaction_id = self.transaction_id.to_protobuf();

        services::Query {
            query: Some(services::query::Query::TransactionGetRecord(
                services::TransactionGetRecordQuery {
                    header: Some(header),
                    transaction_id,
                    include_child_records: self.include_children,
                    include_duplicates: self.include_duplicates,
                },
            )),
        }
    }
}

impl QueryExecute for TransactionRecordQueryData {
    type Response = TransactionRecord;

    fn transaction_id(&self) -> Option<TransactionId> {
        self.transaction_id
    }

    fn execute(
        &self,
        channel: Channel,
        request: services::Query,
    ) -> BoxGrpcFuture<'_, services::Response> {
        Box::pin(async { CryptoServiceClient::new(channel).get_tx_record_by_tx_id(request).await })
    }

    fn should_retry_pre_check(&self, status: Status) -> bool {
        matches!(status, Status::ReceiptNotFound | Status::RecordNotFound)
    }

    fn make_response(&self, response: Response) -> crate::Result<Self::Response> {
        let record = TransactionRecord::from_protobuf(response)?;

        if self.validate_status && record.receipt.status != Status::Success {
            return Err(Error::ReceiptStatus {
                transaction_id: self.transaction_id.map(Box::new),
                status: record.receipt.status,
            });
        }

        Ok(record)
    }
}

impl ValidateChecksums for TransactionRecordQueryData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.transaction_id.validate_checksums(ledger_id)
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::query::ToQueryProtobuf;
    use crate::transaction::test_helpers::TEST_TX_ID;
    use crate::TransactionRecordQuery;

    #[test]
    fn serialize() {
        expect![[r#"
            Query {
                query: Some(
                    TransactionGetRecord(
                        TransactionGetRecordQuery {
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
                            include_duplicates: true,
                            include_child_records: true,
                        },
                    ),
                ),
            }
        "#]]
        .assert_debug_eq(
            &TransactionRecordQuery::new()
                .transaction_id(TEST_TX_ID)
                .include_children(true)
                .include_duplicates(true)
                .data
                .to_query_protobuf(Default::default()),
        )
    }

    #[test]
    fn get_set_transaction_id() {
        let mut query = TransactionRecordQuery::new();
        query.transaction_id(TEST_TX_ID);

        assert_eq!(query.get_transaction_id(), Some(TEST_TX_ID));
    }

    // default is false for all of these, so setting it to `true` is the "interesting" state.
    #[test]
    fn get_set_include_children() {
        let mut query = TransactionRecordQuery::new();
        query.include_children(true);

        assert_eq!(query.get_include_children(), true);
    }

    #[test]
    fn get_set_include_duplicates() {
        let mut query = TransactionRecordQuery::new();
        query.include_duplicates(true);

        assert_eq!(query.get_include_duplicates(), true);
    }

    #[test]
    fn get_set_validate_status() {
        let mut query = TransactionRecordQuery::new();
        query.validate_status(true);

        assert_eq!(query.get_validate_status(), true);
    }
}
