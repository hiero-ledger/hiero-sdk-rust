// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::crypto_service_client::CryptoServiceClient;
use tonic::transport::Channel;

use crate::account::AccountInfo;
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
    Query,
    ToProtobuf,
    ValidateChecksums,
};

/// Get all the information about an account, including the balance.
///
/// This does not get the list of account records.
///
pub type AccountInfoQuery = Query<AccountInfoQueryData>;

#[derive(Default, Clone, Debug)]
pub struct AccountInfoQueryData {
    account_id: Option<AccountId>,
}

impl From<AccountInfoQueryData> for AnyQueryData {
    #[inline]
    fn from(data: AccountInfoQueryData) -> Self {
        Self::AccountInfo(data)
    }
}

impl AccountInfoQuery {
    /// Gets the account ID for which information is requested.
    #[must_use]
    pub fn get_account_id(&self) -> Option<AccountId> {
        self.data.account_id
    }

    /// Sets the account ID for which information is requested.
    pub fn account_id(&mut self, id: AccountId) -> &mut Self {
        self.data.account_id = Some(id);
        self
    }
}

impl ToQueryProtobuf for AccountInfoQueryData {
    fn to_query_protobuf(&self, header: services::QueryHeader) -> services::Query {
        let account_id = self.account_id.to_protobuf();

        services::Query {
            query: Some(services::query::Query::CryptoGetInfo(services::CryptoGetInfoQuery {
                account_id,
                header: Some(header),
            })),
        }
    }
}

impl QueryExecute for AccountInfoQueryData {
    type Response = AccountInfo;

    fn execute(
        &self,
        channel: Channel,
        request: services::Query,
    ) -> BoxGrpcFuture<'_, services::Response> {
        Box::pin(async { CryptoServiceClient::new(channel).get_account_info(request).await })
    }
}

impl ValidateChecksums for AccountInfoQueryData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.account_id.validate_checksums(ledger_id)
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::query::ToQueryProtobuf;
    use crate::{
        AccountId,
        AccountInfoQuery,
        Hbar,
    };

    #[test]
    fn serialize() {
        expect![[r#"
            Query {
                query: Some(
                    CryptoGetInfo(
                        CryptoGetInfoQuery {
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
            &AccountInfoQuery::new()
                .account_id(AccountId {
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
        let mut query = AccountInfoQuery::new();
        query.account_id(AccountId::new(0, 0, 5005));

        assert_eq!(query.get_account_id(), Some(AccountId::new(0, 0, 5005)));
    }
}
