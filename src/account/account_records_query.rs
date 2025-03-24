// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::crypto_service_client::CryptoServiceClient;
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::query::{
    AnyQueryData,
    QueryExecute,
    ToQueryProtobuf,
};
use crate::{
    AccountId,
    BoxGrpcFuture,
    Error,
    FromProtobuf,
    Query,
    ToProtobuf,
    TransactionRecord,
    ValidateChecksums,
};

/// Get all the records for an account for any transfers into it and out of it,
/// that were above the threshold, during the last 25 hours.
pub type AccountRecordsQuery = Query<AccountRecordsQueryData>;

#[derive(Debug, Clone, Default)]
pub struct AccountRecordsQueryData {
    account_id: Option<AccountId>,
}

impl From<AccountRecordsQueryData> for AnyQueryData {
    #[inline]
    fn from(data: AccountRecordsQueryData) -> Self {
        Self::AccountRecords(data)
    }
}

impl AccountRecordsQuery {
    /// Gets the account ID for which the records should be retrieved.
    #[must_use]
    pub fn get_account_id(&self) -> Option<AccountId> {
        self.data.account_id
    }

    /// Sets the account ID for which the records should be retrieved.
    pub fn account_id(&mut self, id: AccountId) -> &mut Self {
        self.data.account_id = Some(id);
        self
    }
}

impl ToQueryProtobuf for AccountRecordsQueryData {
    fn to_query_protobuf(&self, header: services::QueryHeader) -> services::Query {
        let account_id = self.account_id.to_protobuf();

        services::Query {
            query: Some(services::query::Query::CryptoGetAccountRecords(
                services::CryptoGetAccountRecordsQuery { account_id, header: Some(header) },
            )),
        }
    }
}

impl QueryExecute for AccountRecordsQueryData {
    type Response = Vec<TransactionRecord>;

    fn execute(
        &self,
        channel: Channel,
        request: services::Query,
    ) -> BoxGrpcFuture<'_, services::Response> {
        Box::pin(async { CryptoServiceClient::new(channel).get_account_records(request).await })
    }
}

impl ValidateChecksums for AccountRecordsQueryData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.account_id.validate_checksums(ledger_id)
    }
}

impl FromProtobuf<services::response::Response> for Vec<TransactionRecord> {
    fn from_protobuf(pb: services::response::Response) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let response = pb_getv!(pb, CryptoGetAccountRecords, services::response::Response);

        Vec::<TransactionRecord>::from_protobuf(response.records)
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::query::ToQueryProtobuf;
    use crate::{
        AccountId,
        AccountRecordsQuery,
        Hbar,
    };

    #[test]
    fn serialize() {
        expect![[r#"
            Query {
                query: Some(
                    CryptoGetAccountRecords(
                        CryptoGetAccountRecordsQuery {
                            header: Some(
                                QueryHeader {
                                    payment: None,
                                    response_type: AnswerOnly,
                                },
                            ),
                            account_id: Some(
                                AccountId {
                                    shard_num: 0,
                                    realm_num: 0,
                                    account: Some(
                                        AccountNum(
                                            5005,
                                        ),
                                    ),
                                },
                            ),
                        },
                    ),
                ),
            }
        "#]]
        .assert_debug_eq(
            &AccountRecordsQuery::new()
                .account_id(crate::AccountId {
                    shard: 0,
                    realm: 0,
                    num: 5005,
                    alias: None,
                    evm_address: None,
                    checksum: None,
                })
                .max_payment_amount(Hbar::from_tinybars(100_000))
                .data
                .to_query_protobuf(Default::default()),
        );
    }

    #[test]
    fn get_set_account_id() {
        let mut query = AccountRecordsQuery::new();
        query.account_id(AccountId::new(0, 0, 5005));

        assert_eq!(query.get_account_id(), Some(AccountId::new(0, 0, 5005)));
    }
}
