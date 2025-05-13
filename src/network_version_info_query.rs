// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::network_service_client::NetworkServiceClient;
use tonic::transport::Channel;

use crate::entity_id::ValidateChecksums;
use crate::query::{
    AnyQueryData,
    QueryExecute,
    ToQueryProtobuf,
};
use crate::{
    BoxGrpcFuture,
    Error,
    NetworkVersionInfo,
    Query,
};

/// Get information about the versions of protobuf and hedera.
pub type NetworkVersionInfoQuery = Query<NetworkVersionInfoQueryData>;

#[derive(Default, Clone, Debug)]
#[non_exhaustive]
pub struct NetworkVersionInfoQueryData {}

impl From<NetworkVersionInfoQueryData> for AnyQueryData {
    #[inline]
    fn from(data: NetworkVersionInfoQueryData) -> Self {
        Self::NetworkVersionInfo(data)
    }
}

impl ToQueryProtobuf for NetworkVersionInfoQueryData {
    fn to_query_protobuf(&self, header: services::QueryHeader) -> services::Query {
        services::Query {
            query: Some(services::query::Query::NetworkGetVersionInfo(
                services::NetworkGetVersionInfoQuery { header: Some(header) },
            )),
        }
    }
}

impl QueryExecute for NetworkVersionInfoQueryData {
    type Response = NetworkVersionInfo;

    fn execute(
        &self,
        channel: Channel,
        request: services::Query,
    ) -> BoxGrpcFuture<'_, services::Response> {
        Box::pin(async { NetworkServiceClient::new(channel).get_version_info(request).await })
    }
}

impl ValidateChecksums for NetworkVersionInfoQueryData {
    fn validate_checksums(&self, _ledger_id: &crate::ledger_id::RefLedgerId) -> Result<(), Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::query::ToQueryProtobuf;
    use crate::{
        Hbar,
        NetworkVersionInfoQuery,
    };

    #[test]
    fn serialize() {
        expect![[r#"
            Query {
                query: Some(
                    NetworkGetVersionInfo(
                        NetworkGetVersionInfoQuery {
                            header: Some(
                                QueryHeader {
                                    payment: None,
                                    response_type: AnswerOnly,
                                },
                            ),
                        },
                    ),
                ),
            }
        "#]]
        .assert_debug_eq(
            &NetworkVersionInfoQuery::new()
                .max_payment_amount(Hbar::from_tinybars(100_000))
                .data
                .to_query_protobuf(Default::default()),
        )
    }
}
