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

/// Internal "query" to ping a specific node.
///
/// This is *here* so that it can change implementation at will.
/// `PingQuery` is an `AccountBalanceQuery`-ish for now,
/// but it doesn't have to stay that way.
///
/// It's also ideally smaller/faster than any other query, by virtue of just...
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
            response_type: services::ResponseType::AnswerOnly as i32,
        };

        debug_assert_eq!(node_account_id, self.node_account_id);

        let query = services::Query {
            query: Some(services::query::Query::CryptogetAccountBalance(
                services::CryptoGetAccountBalanceQuery {
                    balance_source: Some(
                        services::crypto_get_account_balance_query::BalanceSource::AccountId(
                            self.node_account_id.to_protobuf(),
                        ),
                    ),
                    header: Some(HEADER),
                },
            )),
        };

        Ok((query, ()))
    }

    fn execute(
        &self,
        channel: tonic::transport::Channel,
        request: Self::GrpcRequest,
    ) -> crate::BoxGrpcFuture<Self::GrpcResponse> {
        Box::pin(async { CryptoServiceClient::new(channel).crypto_get_balance(request).await })
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
