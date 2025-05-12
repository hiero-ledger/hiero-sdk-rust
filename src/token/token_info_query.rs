// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::token_service_client::TokenServiceClient;
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::query::{
    AnyQueryData,
    QueryExecute,
    ToQueryProtobuf,
};
use crate::token::token_info::TokenInfo;
use crate::{
    BoxGrpcFuture,
    Error,
    Query,
    ToProtobuf,
    TokenId,
    ValidateChecksums,
};

/// Gets information about Token instance.
///
pub type TokenInfoQuery = Query<TokenInfoQueryData>;

#[derive(Default, Clone, Debug)]
pub struct TokenInfoQueryData {
    token_id: Option<TokenId>,
}

impl From<TokenInfoQueryData> for AnyQueryData {
    #[inline]
    fn from(data: TokenInfoQueryData) -> Self {
        Self::TokenInfo(data)
    }
}

impl TokenInfoQuery {
    /// Returns the token ID for which information is requested.
    #[must_use]
    pub fn get_token_id(&self) -> Option<TokenId> {
        self.data.token_id
    }

    /// Sets the token ID for which information is requested.
    pub fn token_id(&mut self, id: impl Into<TokenId>) -> &mut Self {
        self.data.token_id = Some(id.into());
        self
    }
}

impl ToQueryProtobuf for TokenInfoQueryData {
    fn to_query_protobuf(&self, header: services::QueryHeader) -> services::Query {
        let token_id = self.token_id.to_protobuf();

        services::Query {
            query: Some(services::query::Query::TokenGetInfo(services::TokenGetInfoQuery {
                header: Some(header),
                token: token_id,
            })),
        }
    }
}

impl QueryExecute for TokenInfoQueryData {
    type Response = TokenInfo;

    fn execute(
        &self,
        channel: Channel,
        request: services::Query,
    ) -> BoxGrpcFuture<'_, services::Response> {
        Box::pin(async { TokenServiceClient::new(channel).get_token_info(request).await })
    }
}

impl ValidateChecksums for TokenInfoQueryData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.token_id.validate_checksums(ledger_id)
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::query::ToQueryProtobuf;
    use crate::{
        Hbar,
        TokenId,
        TokenInfoQuery,
    };

    #[test]
    fn serialize() {
        expect![[r#"
            Query {
                query: Some(
                    TokenGetInfo(
                        TokenGetInfoQuery {
                            header: Some(
                                QueryHeader {
                                    payment: None,
                                    response_type: AnswerOnly,
                                },
                            ),
                            token: Some(
                                TokenId {
                                    shard_num: 0,
                                    realm_num: 0,
                                    token_num: 5005,
                                },
                            ),
                        },
                    ),
                ),
            }
        "#]]
        .assert_debug_eq(
            &TokenInfoQuery::new()
                .token_id(TokenId::new(0, 0, 5005))
                .max_payment_amount(Hbar::from_tinybars(100_000))
                .data
                .to_query_protobuf(Default::default()),
        )
    }

    #[test]
    fn get_set_token_id() {
        let mut query = TokenInfoQuery::new();
        query.token_id(TokenId::new(0, 0, 5005));

        assert_eq!(query.get_token_id(), Some(TokenId::new(0, 0, 5005)));
    }
}
