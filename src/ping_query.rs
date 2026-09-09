use std::time::Duration;

use hiero_sdk_proto::services;
use hiero_sdk_proto::services::crypto_service_client::CryptoServiceClient;

use crate::entity_id::ValidateChecksums;
use crate::execute::{
    execute,
    Execute,
};
use crate::protobuf::ToProtobuf;
use crate::query::response_header;
use crate::{
    AccountId,
    Client,
};

/// Account number of the treasury system account, present on every network from genesis.
const TREASURY_ACCOUNT_NUM: u64 = 2;

/// Internal "query" to ping a specific node.
///
/// The probe is a `CryptoService/getAccountInfo` query for the treasury account
/// (`<shard>.<realm>.2`) sent with `ResponseType = COST_ANSWER`: the node answers
/// with the query fee without executing the query, so nothing is charged and no
/// operator is required. A cost response means the node is reachable; a gRPC-level
/// failure is recorded against the node like any other failed request.
///
/// This is *here* so that it can change implementation at will.
pub(crate) struct PingQuery {
    node_account_id: AccountId,
}

impl PingQuery {
    pub(crate) fn new(node_account_id: AccountId) -> Self {
        Self { node_account_id }
    }

    pub(crate) async fn execute(
        &self,
        client: &Client,
        timeout: Option<Duration>,
    ) -> crate::Result<()> {
        execute(client, self, timeout).await
    }
}

impl ValidateChecksums for PingQuery {
    fn validate_checksums(
        &self,
        ledger_id: &crate::ledger_id::RefLedgerId,
    ) -> Result<(), crate::Error> {
        self.node_account_id.validate_checksums(ledger_id)
    }
}

impl Execute for PingQuery {
    type GrpcRequest = services::Query;

    type GrpcResponse = services::Response;

    type Context = ();

    type Response = ();

    fn node_account_ids(&self) -> Option<&[AccountId]> {
        Some(std::slice::from_ref(&self.node_account_id))
    }

    fn transaction_id(&self) -> Option<crate::TransactionId> {
        None
    }

    fn operator_account_id(&self) -> Option<&AccountId> {
        None
    }

    fn requires_transaction_id(&self) -> bool {
        false
    }

    fn make_request(
        &self,
        _transaction_id: Option<&crate::TransactionId>,
        node_account_id: AccountId,
    ) -> crate::Result<(Self::GrpcRequest, Self::Context)> {
        const HEADER: services::QueryHeader = services::QueryHeader {
            payment: None,
            response_type: services::ResponseType::CostAnswer as i32,
        };

        debug_assert_eq!(node_account_id, self.node_account_id);

        // System accounts live in the same shard and realm as the network's nodes.
        let treasury_account_id = AccountId::new(
            self.node_account_id.shard,
            self.node_account_id.realm,
            TREASURY_ACCOUNT_NUM,
        );

        let query = services::Query {
            query: Some(services::query::Query::CryptoGetInfo(services::CryptoGetInfoQuery {
                account_id: Some(treasury_account_id.to_protobuf()),
                header: Some(HEADER),
            })),
        };

        Ok((query, ()))
    }

    fn execute(
        &self,
        channel: crate::Channel,
        request: Self::GrpcRequest,
    ) -> crate::BoxGrpcFuture<Self::GrpcResponse> {
        Box::pin(async { CryptoServiceClient::new(channel).get_account_info(request).await })
    }

    fn make_response(
        &self,
        _response: Self::GrpcResponse,
        _context: Self::Context,
        _node_account_id: AccountId,
        _transaction_id: Option<&crate::TransactionId>,
    ) -> crate::Result<Self::Response> {
        Ok(())
    }

    fn make_error_pre_check(
        &self,
        status: hiero_sdk_proto::services::ResponseCodeEnum,
        _transaction_id: Option<&crate::TransactionId>,
        _response: Self::GrpcResponse,
    ) -> crate::Error {
        crate::Error::QueryNoPaymentPreCheckStatus { status }
    }

    fn response_pre_check_status(response: &Self::GrpcResponse) -> crate::Result<i32> {
        Ok(response_header(&response.response)?.node_transaction_precheck_code)
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hiero_sdk_proto::services;

    use super::PingQuery;
    use crate::execute::Execute;
    use crate::protobuf::ToProtobuf;
    use crate::AccountId;

    #[test]
    fn probe_is_a_cost_answer_get_account_info_for_the_treasury() {
        let node_account_id = AccountId::new(0, 0, 3);

        let (query, ()) =
            PingQuery::new(node_account_id).make_request(None, node_account_id).unwrap();

        expect![[r#"
            Query {
                query: Some(
                    CryptoGetInfo(
                        CryptoGetInfoQuery {
                            header: Some(
                                QueryHeader {
                                    payment: None,
                                    response_type: CostAnswer,
                                },
                            ),
                            account_id: Some(
                                AccountId {
                                    shard_num: 0,
                                    realm_num: 0,
                                    account: Some(
                                        AccountNum(
                                            2,
                                        ),
                                    ),
                                },
                            ),
                        },
                    ),
                ),
            }
        "#]]
        .assert_debug_eq(&query);
    }

    #[test]
    fn probe_follows_the_shard_and_realm_of_the_node() {
        let node_account_id = AccountId::new(1, 2, 3);

        let (query, ()) =
            PingQuery::new(node_account_id).make_request(None, node_account_id).unwrap();

        let Some(services::query::Query::CryptoGetInfo(info)) = query.query else {
            panic!("expected a `CryptoGetInfo` query, got {:?}", query.query);
        };

        assert_eq!(info.account_id, Some(AccountId::new(1, 2, 2).to_protobuf()));
    }
}
