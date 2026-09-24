// SPDX-License-Identifier: Apache-2.0

use hiero_sdk_proto::services;
use services::crypto_get_account_balance_query::BalanceSource;

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
    Channel,
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
///
/// The network no longer serves this query: constructing it logs a warning, and executing it
/// (or fetching its cost) fails with [`Error::AccountBalanceQueryDeprecated`] without sending
/// a request.
#[deprecated(
    note = "Deprecated: AccountBalanceQuery is no longer supported. Use MirrorNodeAccountBalanceQuery or the mirror node REST API (GET /api/v1/accounts/{id}) to retrieve account balances."
)]
pub type AccountBalanceQuery = Query<AccountBalanceQueryData>;

#[derive(Clone, Debug)]
pub struct AccountBalanceQueryData {
    source: AccountBalanceSource,
}

impl Default for AccountBalanceQueryData {
    fn default() -> Self {
        // every `AccountBalanceQuery::new()` / `default()` comes through here.
        log::warn!("{}", Error::AccountBalanceQueryDeprecated);

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

#[allow(deprecated)]
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

    fn check_supported(&self) -> crate::Result<()> {
        Err(Error::AccountBalanceQueryDeprecated)
    }

    fn execute(
        &self,
        _channel: Channel,
        _request: services::Query,
    ) -> BoxGrpcFuture<'_, services::Response> {
        // unreachable: `check_supported` stops execution before any request is made.
        Box::pin(async {
            Err(tonic::Status::unimplemented(Error::AccountBalanceQueryDeprecated.to_string()))
        })
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
#[allow(deprecated)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::future::Future;
    use std::io::ErrorKind;
    use std::net::TcpListener;
    use std::time::Duration;

    use expect_test::expect;

    use super::AccountBalanceQueryData;
    use crate::query::{
        AnyQueryData,
        QueryExecute,
        ToQueryProtobuf,
    };
    use crate::{
        AccountBalanceQuery,
        AccountId,
        AccountInfoQuery,
        Client,
        ContractId,
        Error,
        Hbar,
        TransactionId,
        TransactionReceiptQuery,
    };

    const MESSAGE: &str = "Deprecated: AccountBalanceQuery is no longer supported. Use MirrorNodeAccountBalanceQuery or the mirror node REST API (GET /api/v1/accounts/{id}) to retrieve account balances.";

    thread_local! {
        static WARNINGS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    }

    /// Records `warn` records of the current thread, so parallel tests don't interfere.
    struct CaptureWarnings;

    impl log::Log for CaptureWarnings {
        fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
            metadata.level() <= log::Level::Warn
        }

        fn log(&self, record: &log::Record<'_>) {
            if record.level() == log::Level::Warn {
                WARNINGS.with(|it| it.borrow_mut().push(record.args().to_string()));
            }
        }

        fn flush(&self) {}
    }

    fn capture_warnings(f: impl FnOnce()) -> Vec<String> {
        static LOGGER: CaptureWarnings = CaptureWarnings;

        // First logger wins; nothing else in the lib tests installs one.
        let _ = log::set_logger(&LOGGER);
        log::set_max_level(log::LevelFilter::Warn);

        WARNINGS.with(|it| it.borrow_mut().clear());
        f();
        WARNINGS.with(RefCell::take)
    }

    async fn assert_fails_fast<T: std::fmt::Debug>(fut: impl Future<Output = crate::Result<T>>) {
        let err = tokio::time::timeout(Duration::from_secs(5), fut)
            .await
            .expect("must fail without waiting on the network")
            .unwrap_err();

        assert!(matches!(err, Error::AccountBalanceQueryDeprecated), "{err:?}");
        assert_eq!(err.to_string(), MESSAGE);
    }

    #[test]
    fn every_construction_warns() {
        let warnings = capture_warnings(|| {
            let _ = AccountBalanceQuery::new();
            let _ = AccountBalanceQuery::default();
            let _ = AccountInfoQuery::new();
        });

        assert_eq!(warnings, [MESSAGE, MESSAGE]);
    }

    #[test]
    fn error_message_is_canonical() {
        assert_eq!(Error::AccountBalanceQueryDeprecated.to_string(), MESSAGE);
    }

    #[tokio::test]
    async fn execute_and_get_cost_fail_without_network() {
        // a node that accepts TCP but is never served: any connection attempt stays in the backlog.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();

        let client = Client::for_network(HashMap::from([(
            listener.local_addr().unwrap().to_string(),
            AccountId::new(0, 0, 3),
        )]))
        .unwrap();

        let mut query = AccountBalanceQuery::new();
        query.account_id(AccountId::new(0, 0, 5005));

        assert_fails_fast(query.execute(&client)).await;
        assert_fails_fast(query.execute_with_timeout(&client, Duration::from_secs(1))).await;
        // `get_cost` used to return `Ok(0)` for this free query without a request.
        assert_fails_fast(query.get_cost(&client)).await;
        assert_fails_fast(query.get_cost_with_timeout(&client, Duration::from_secs(1))).await;

        assert!(matches!(
            AnyQueryData::from(AccountBalanceQueryData::default()).check_supported(),
            Err(Error::AccountBalanceQueryDeprecated)
        ));

        // other free queries keep their early `Ok(0)` cost.
        let cost = TransactionReceiptQuery::new()
            .transaction_id(TransactionId::generate(AccountId::new(0, 0, 5005)))
            .get_cost(&client)
            .await
            .unwrap();
        assert_eq!(cost, Hbar::ZERO);

        // no TCP connection was ever attempted.
        assert_eq!(listener.accept().unwrap_err().kind(), ErrorKind::WouldBlock);
    }

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
