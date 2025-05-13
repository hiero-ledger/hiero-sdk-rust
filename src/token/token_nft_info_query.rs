// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::token_service_client::TokenServiceClient;
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::query::{
    AnyQueryData,
    Query,
    QueryExecute,
    ToQueryProtobuf,
};
use crate::{
    BoxGrpcFuture,
    Error,
    NftId,
    ToProtobuf,
    TokenNftInfo,
    ValidateChecksums,
};

/// Gets info on an NFT for a given `TokenID` and serial number.
pub type TokenNftInfoQuery = Query<TokenNftInfoQueryData>;

#[derive(Clone, Default, Debug)]
pub struct TokenNftInfoQueryData {
    /// The ID of the NFT
    nft_id: Option<NftId>,
}

impl From<TokenNftInfoQueryData> for AnyQueryData {
    #[inline]
    fn from(data: TokenNftInfoQueryData) -> Self {
        Self::TokenNftInfo(data)
    }
}

impl TokenNftInfoQuery {
    /// Returns the ID of the NFT for which information is requested.
    #[must_use]
    pub fn get_nft_id(&self) -> Option<NftId> {
        self.data.nft_id
    }

    /// Sets the ID of the NFT for which information is requested.
    pub fn nft_id(&mut self, nft_id: impl Into<NftId>) -> &mut Self {
        self.data.nft_id = Some(nft_id.into());
        self
    }
}

impl ToQueryProtobuf for TokenNftInfoQueryData {
    fn to_query_protobuf(&self, header: services::QueryHeader) -> services::Query {
        let nft_id = self.nft_id.to_protobuf();

        services::Query {
            query: Some(services::query::Query::TokenGetNftInfo(services::TokenGetNftInfoQuery {
                header: Some(header),
                nft_id,
            })),
        }
    }
}

impl QueryExecute for TokenNftInfoQueryData {
    type Response = TokenNftInfo;

    fn execute(
        &self,
        channel: Channel,
        request: services::Query,
    ) -> BoxGrpcFuture<'_, services::Response> {
        Box::pin(async { TokenServiceClient::new(channel).get_token_nft_info(request).await })
    }
}

impl ValidateChecksums for TokenNftInfoQueryData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.nft_id.validate_checksums(ledger_id)
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::query::ToQueryProtobuf;
    use crate::{
        Hbar,
        TokenId,
        TokenNftInfoQuery,
    };

    #[test]
    fn serialize() {
        expect![[r#"
            Query {
                query: Some(
                    TokenGetNftInfo(
                        TokenGetNftInfoQuery {
                            header: Some(
                                QueryHeader {
                                    payment: None,
                                    response_type: AnswerOnly,
                                },
                            ),
                            nft_id: Some(
                                NftId {
                                    token_id: Some(
                                        TokenId {
                                            shard_num: 0,
                                            realm_num: 0,
                                            token_num: 5005,
                                        },
                                    ),
                                    serial_number: 101,
                                },
                            ),
                        },
                    ),
                ),
            }
        "#]]
        .assert_debug_eq(
            &TokenNftInfoQuery::new()
                .nft_id(TokenId::new(0, 0, 5005).nft(101))
                .max_payment_amount(Hbar::from_tinybars(100_000))
                .data
                .to_query_protobuf(Default::default()),
        )
    }

    #[test]
    fn properties() {
        let mut query = TokenNftInfoQuery::new();
        query
            .nft_id(TokenId::new(0, 0, 5005).nft(101))
            .max_payment_amount(Hbar::from_tinybars(100_000));

        assert_eq!(query.get_nft_id().unwrap().to_string(), "0.0.5005/101");
    }
}
