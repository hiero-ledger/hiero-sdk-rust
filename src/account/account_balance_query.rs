// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::crypto_service_client::CryptoServiceClient;
use services::crypto_get_account_balance_query::BalanceSource;
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::query::{
    AnyQueryData,
    Query,
    QueryExecute,
    ToQueryProtobuf,
};
use crate::{
    AccountBalance,
    AccountId,
    BoxGrpcFuture,
    ContractId,
    Error,
    ToProtobuf,
    ValidateChecksums,
};

/// Get the balance of a cryptocurrency account.
///
/// This returns only the balance, so it is a smaller reply
/// than [`AccountInfoQuery`][crate::AccountInfoQuery],
/// which returns the balance plus additional information.
pub type AccountBalanceQuery = Query<AccountBalanceQueryData>;

#[derive(Clone, Debug)]
pub struct AccountBalanceQueryData {
    source: AccountBalanceSource,
}

impl Default for AccountBalanceQueryData {
    fn default() -> Self {
        Self { source: AccountBalanceSource::AccountId(AccountId::from(0)) }
    }
}

impl From<AccountBalanceQueryData> for AnyQueryData {
    #[inline]
    fn from(data: AccountBalanceQueryData) -> Self {
        Self::AccountBalance(data)
    }
}

#[derive(Clone, Debug)]
enum AccountBalanceSource {
    AccountId(AccountId),
    ContractId(ContractId),
}

impl AccountBalanceQuery {
    /// Get the account ID for which information is requested.
    #[must_use]
    pub fn get_account_id(&self) -> Option<AccountId> {
        match self.data.source {
            AccountBalanceSource::AccountId(it) => Some(it),
            AccountBalanceSource::ContractId(_) => None,
        }
    }

    /// Sets the account ID for which information is requested.
    ///
    /// This is mutually exclusive with [`contract_id`](Self::contract_id).
    pub fn account_id(&mut self, id: AccountId) -> &mut Self {
        self.data.source = AccountBalanceSource::AccountId(id);
        self
    }

    /// Get the contract ID for which information is requested.
    #[must_use]
    pub fn get_contract_id(&self) -> Option<ContractId> {
        match self.data.source {
            AccountBalanceSource::ContractId(it) => Some(it),
            AccountBalanceSource::AccountId(_) => None,
        }
    }

    /// Sets the contract ID for which information is requested.
    ///
    /// This is mutually exclusive with [`account_id`](Self::account_id).
    pub fn contract_id(&mut self, id: ContractId) -> &mut Self {
        self.data.source = AccountBalanceSource::ContractId(id);
        self
    }
}

impl ToQueryProtobuf for AccountBalanceQueryData {
    fn to_query_protobuf(&self, header: services::QueryHeader) -> services::Query {
        let source = Some(&self.source).as_ref().map(|source| match source {
            AccountBalanceSource::AccountId(id) => BalanceSource::AccountId(id.to_protobuf()),
            AccountBalanceSource::ContractId(id) => BalanceSource::ContractId(id.to_protobuf()),
        });

        services::Query {
            query: Some(services::query::Query::CryptogetAccountBalance(
                services::CryptoGetAccountBalanceQuery {
                    balance_source: source,
                    header: Some(header),
                },
            )),
        }
    }
}

impl QueryExecute for AccountBalanceQueryData {
    type Response = AccountBalance;

    fn is_payment_required(&self) -> bool {
        false
    }

    fn execute(
        &self,
        channel: Channel,
        request: services::Query,
    ) -> BoxGrpcFuture<'_, services::Response> {
        Box::pin(async { CryptoServiceClient::new(channel).crypto_get_balance(request).await })
    }
}

impl ValidateChecksums for AccountBalanceQueryData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        match self.source {
            AccountBalanceSource::AccountId(account_id) => account_id.validate_checksums(ledger_id),
            AccountBalanceSource::ContractId(contract_id) => {
                contract_id.validate_checksums(ledger_id)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::query::ToQueryProtobuf;
    use crate::{
        AccountBalanceQuery,
        AccountId,
        ContractId,
    };

    #[test]
    fn serialize_with_account_id() {
        expect![[r#"
            Query {
                query: Some(
                    CryptogetAccountBalance(
                        CryptoGetAccountBalanceQuery {
                            header: Some(
                                QueryHeader {
                                    payment: None,
                                    response_type: AnswerOnly,
                                },
                            ),
                            balance_source: Some(
                                AccountId(
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
                            ),
                        },
                    ),
                ),
            }
        "#]]
        .assert_debug_eq(
            &AccountBalanceQuery::new()
                .account_id(crate::AccountId::new(0, 0, 5005))
                .data
                .to_query_protobuf(Default::default()),
        );
    }

    #[test]
    fn serialize_with_contract_id() {
        expect![[r#"
            Query {
                query: Some(
                    CryptogetAccountBalance(
                        CryptoGetAccountBalanceQuery {
                            header: Some(
                                QueryHeader {
                                    payment: None,
                                    response_type: AnswerOnly,
                                },
                            ),
                            balance_source: Some(
                                ContractId(
                                    ContractId {
                                        shard_num: 0,
                                        realm_num: 0,
                                        contract: Some(
                                            ContractNum(
                                                5005,
                                            ),
                                        ),
                                    },
                                ),
                            ),
                        },
                    ),
                ),
            }
        "#]]
        .assert_debug_eq(
            &AccountBalanceQuery::new()
                .contract_id(crate::ContractId::new(0, 0, 5005))
                .data
                .to_query_protobuf(Default::default()),
        );
    }

    #[test]
    fn get_set_account_id() {
        let mut query = AccountBalanceQuery::new();
        query.account_id(AccountId::new(0, 0, 5005));

        assert_eq!(query.get_account_id(), Some(AccountId::new(0, 0, 5005)));
    }

    #[test]
    fn get_set_contract_id() {
        let mut query = AccountBalanceQuery::new();
        query.contract_id(ContractId::new(0, 0, 5005));

        assert_eq!(query.get_contract_id(), Some(ContractId::new(0, 0, 5005)));
    }
}
