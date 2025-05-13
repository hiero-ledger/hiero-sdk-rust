// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::token_service_client::TokenServiceClient;
use tonic::transport::Channel;

use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};
use crate::token::custom_fees::AnyCustomFee;
use crate::transaction::{
    AnyTransactionData,
    ChunkInfo,
    ToSchedulableTransactionDataProtobuf,
    ToTransactionDataProtobuf,
    TransactionData,
    TransactionExecute,
};
use crate::{
    BoxGrpcFuture,
    Error,
    TokenId,
    Transaction,
    ValidateChecksums,
};

/// At consensus, updates a token type's fee schedule to the given list of custom fees.
///
/// If the target token type has no `fee_schedule_key`, resolves to `TokenHasNoFeeScheduleKey`.
/// Otherwise this transaction must be signed to the `fee_schedule_key`, or the transaction will
/// resolve to `InvalidSignature`.
///
/// If the `custom_fees` list is empty, clears the fee schedule or resolves to
/// `CustomScheduleAlreadyHasNoFees` if the fee schedule was already empty.
pub type TokenFeeScheduleUpdateTransaction = Transaction<TokenFeeScheduleUpdateTransactionData>;

#[derive(Debug, Clone, Default)]
pub struct TokenFeeScheduleUpdateTransactionData {
    /// The token whose fee schedule is to be updated.
    token_id: Option<TokenId>,

    /// The new custom fees to be assessed during a transfer.
    custom_fees: Vec<AnyCustomFee>,
}

impl TokenFeeScheduleUpdateTransaction {
    /// Returns the ID of the token that's being updated.
    #[must_use]
    pub fn get_token_id(&self) -> Option<TokenId> {
        self.data().token_id
    }

    // note(sr): what is being updated is implicit.
    /// Sets the ID of the token that's being updated.
    pub fn token_id(&mut self, token_id: impl Into<TokenId>) -> &mut Self {
        self.data_mut().token_id = Some(token_id.into());
        self
    }

    /// Returns the new custom fees to be assessed during a transfer.
    #[must_use]
    pub fn get_custom_fees(&self) -> &[AnyCustomFee] {
        &self.data().custom_fees
    }

    /// Sets the new custom fees to be assessed during a transfer.
    pub fn custom_fees(
        &mut self,
        custom_fees: impl IntoIterator<Item = AnyCustomFee>,
    ) -> &mut Self {
        self.data_mut().custom_fees = custom_fees.into_iter().collect();
        self
    }
}

impl TransactionData for TokenFeeScheduleUpdateTransactionData {}

impl TransactionExecute for TokenFeeScheduleUpdateTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async {
            TokenServiceClient::new(channel).update_token_fee_schedule(request).await
        })
    }
}

impl ValidateChecksums for TokenFeeScheduleUpdateTransactionData {
    fn validate_checksums(&self, ledger_id: &crate::ledger_id::RefLedgerId) -> Result<(), Error> {
        // TODO: validate custom fees (they need an impl)
        self.token_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for TokenFeeScheduleUpdateTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::TokenFeeScheduleUpdate(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for TokenFeeScheduleUpdateTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::TokenFeeScheduleUpdate(self.to_protobuf())
    }
}

impl From<TokenFeeScheduleUpdateTransactionData> for AnyTransactionData {
    fn from(transaction: TokenFeeScheduleUpdateTransactionData) -> Self {
        Self::TokenFeeScheduleUpdate(transaction)
    }
}

impl FromProtobuf<services::TokenFeeScheduleUpdateTransactionBody>
    for TokenFeeScheduleUpdateTransactionData
{
    fn from_protobuf(pb: services::TokenFeeScheduleUpdateTransactionBody) -> crate::Result<Self> {
        Ok(Self {
            token_id: Option::from_protobuf(pb.token_id)?,
            custom_fees: Vec::from_protobuf(pb.custom_fees)?,
        })
    }
}

impl ToProtobuf for TokenFeeScheduleUpdateTransactionData {
    type Protobuf = services::TokenFeeScheduleUpdateTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::TokenFeeScheduleUpdateTransactionBody {
            token_id: self.token_id.to_protobuf(),
            custom_fees: self.custom_fees.to_protobuf(),
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hedera_proto::services;

    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::token::TokenFeeScheduleUpdateTransactionData;
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
    };
    use crate::{
        AnyCustomFee,
        AnyTransaction,
        FixedFee,
        FractionalFee,
        TokenFeeScheduleUpdateTransaction,
        TokenId,
    };

    const TOKEN_ID: TokenId = TokenId::new(0, 0, 8798);

    fn custom_fees() -> [AnyCustomFee; 2] {
        [
            FixedFee {
                fee: crate::FixedFeeData {
                    amount: 10,
                    denominating_token_id: Some(TokenId::new(0, 0, 483902)),
                },
                fee_collector_account_id: Some("4322".parse().unwrap()),
                all_collectors_are_exempt: false,
            }
            .into(),
            FractionalFee {
                fee: crate::FractionalFeeData {
                    denominator: 7,
                    numerator: 3,
                    minimum_amount: 3,
                    maximum_amount: 100,
                    assessment_method: crate::FeeAssessmentMethod::Exclusive,
                },
                fee_collector_account_id: Some("389042".parse().unwrap()),
                all_collectors_are_exempt: false,
            }
            .into(),
        ]
    }

    fn make_transaction() -> TokenFeeScheduleUpdateTransaction {
        let mut tx = TokenFeeScheduleUpdateTransaction::new_for_tests();

        tx.token_id(TOKEN_ID).custom_fees(custom_fees()).freeze().unwrap();

        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            TokenFeeScheduleUpdate(
                TokenFeeScheduleUpdateTransactionBody {
                    token_id: Some(
                        TokenId {
                            shard_num: 0,
                            realm_num: 0,
                            token_num: 8798,
                        },
                    ),
                    custom_fees: [
                        CustomFee {
                            fee_collector_account_id: Some(
                                AccountId {
                                    shard_num: 0,
                                    realm_num: 0,
                                    account: Some(
                                        AccountNum(
                                            4322,
                                        ),
                                    ),
                                },
                            ),
                            all_collectors_are_exempt: false,
                            fee: Some(
                                FixedFee(
                                    FixedFee {
                                        amount: 10,
                                        denominating_token_id: Some(
                                            TokenId {
                                                shard_num: 0,
                                                realm_num: 0,
                                                token_num: 483902,
                                            },
                                        ),
                                    },
                                ),
                            ),
                        },
                        CustomFee {
                            fee_collector_account_id: Some(
                                AccountId {
                                    shard_num: 0,
                                    realm_num: 0,
                                    account: Some(
                                        AccountNum(
                                            389042,
                                        ),
                                    ),
                                },
                            ),
                            all_collectors_are_exempt: false,
                            fee: Some(
                                FractionalFee(
                                    FractionalFee {
                                        fractional_amount: Some(
                                            Fraction {
                                                numerator: 3,
                                                denominator: 7,
                                            },
                                        ),
                                        minimum_amount: 3,
                                        maximum_amount: 100,
                                        net_of_transfers: true,
                                    },
                                ),
                            ),
                        },
                    ],
                },
            )
        "#]]
        .assert_debug_eq(&tx)
    }

    #[test]
    fn to_from_bytes() {
        let tx = make_transaction();

        let tx2 = AnyTransaction::from_bytes(&tx.to_bytes().unwrap()).unwrap();

        let tx = transaction_body(tx);

        let tx2 = transaction_body(tx2);

        assert_eq!(tx, tx2);
    }

    #[test]
    fn from_proto_body() {
        let tx = services::TokenFeeScheduleUpdateTransactionBody {
            token_id: Some(TOKEN_ID.to_protobuf()),
            custom_fees: custom_fees().to_vec().to_protobuf(),
        };

        let data = TokenFeeScheduleUpdateTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(data.token_id, Some(TOKEN_ID));
        assert_eq!(data.custom_fees, custom_fees());
    }

    #[test]
    fn get_set_token_id() {
        let mut tx = TokenFeeScheduleUpdateTransaction::new();
        tx.token_id(TOKEN_ID);

        assert_eq!(tx.get_token_id(), Some(TOKEN_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_token_id_frozen_panic() {
        make_transaction().token_id(TOKEN_ID);
    }

    #[test]
    fn get_set_custom_fees() {
        let mut tx = TokenFeeScheduleUpdateTransaction::new();
        tx.custom_fees(custom_fees());

        assert_eq!(tx.get_custom_fees(), custom_fees());
    }

    #[test]
    #[should_panic]
    fn get_set_custom_fees_frozen_panic() {
        make_transaction().custom_fees(custom_fees());
    }
}
