use hiero_sdk_proto::services;

use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};
use crate::{
    AccountId,
    TokenId,
};

/// A custom transfer fee that was assessed during the handling of a `CryptoTransfer`.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct AssessedCustomFee {
    /// The amount of currency charged to each payer.
    pub amount: i64,

    /// The currency `amount` is charged in, if `None` the fee is in HBar.
    pub token_id: Option<TokenId>,

    /// The account that receives the fees that were charged.
    pub fee_collector_account_id: Option<AccountId>,

    /// A list of all accounts that were charged this fee.
    pub payer_account_id_list: Vec<AccountId>,
}

impl AssessedCustomFee {
    /// Create a new `AssessedCustomFee` from protobuf-encoded `bytes`.
    ///
    /// # Errors
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the bytes fails to produce a valid protobuf.
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the protobuf fails.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        FromProtobuf::from_bytes(bytes)
    }

    /// Convert `self` to a protobuf-encoded [`Vec<u8>`].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        ToProtobuf::to_bytes(self)
    }
}

impl FromProtobuf<services::AssessedCustomFee> for AssessedCustomFee {
    fn from_protobuf(pb: services::AssessedCustomFee) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            amount: pb.amount,
            token_id: Option::from_protobuf(pb.token_id)?,
            fee_collector_account_id: Option::from_protobuf(pb.fee_collector_account_id)?,
            payer_account_id_list: Vec::from_protobuf(pb.effective_payer_account_id)?,
        })
    }
}

impl ToProtobuf for AssessedCustomFee {
    type Protobuf = services::AssessedCustomFee;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::AssessedCustomFee {
            amount: self.amount,
            token_id: self.token_id.to_protobuf(),
            fee_collector_account_id: self.fee_collector_account_id.to_protobuf(),
            effective_payer_account_id: self.payer_account_id_list.to_protobuf(),
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hiero_sdk_proto::services;

    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::{
        AccountId,
        AssessedCustomFee,
        TokenId,
    };

    const AMOUNT: i64 = 1;
    const TOKEN_ID: TokenId = TokenId { shard: 2, realm: 3, num: 4, checksum: None };
    const FEE_COLLECTOR: AccountId =
        AccountId { shard: 5, realm: 6, num: 7, alias: None, evm_address: None, checksum: None };

    const PAYER_ACCOUNT_IDS: [AccountId; 3] = [
        AccountId { shard: 8, realm: 9, num: 10, alias: None, evm_address: None, checksum: None },
        AccountId { shard: 11, realm: 12, num: 13, alias: None, evm_address: None, checksum: None },
        AccountId { shard: 14, realm: 15, num: 16, alias: None, evm_address: None, checksum: None },
    ];

    fn make_fee_proto() -> services::AssessedCustomFee {
        services::AssessedCustomFee {
            amount: AMOUNT,
            token_id: Some(TOKEN_ID.to_protobuf()),
            fee_collector_account_id: Some(FEE_COLLECTOR.to_protobuf()),
            effective_payer_account_id: PAYER_ACCOUNT_IDS
                .iter()
                .map(AccountId::to_protobuf)
                .collect(),
        }
    }

    fn make_fee() -> AssessedCustomFee {
        AssessedCustomFee {
            amount: 201,
            token_id: Some(TokenId::new(1, 2, 3)),
            fee_collector_account_id: Some(AccountId {
                shard: 4,
                realm: 5,
                num: 6,
                alias: None,
                evm_address: None,
                checksum: None,
            }),
            payer_account_id_list: Vec::from([
                AccountId {
                    shard: 0,
                    realm: 0,
                    num: 1,
                    alias: None,
                    evm_address: None,
                    checksum: None,
                },
                AccountId {
                    shard: 0,
                    realm: 0,
                    num: 2,
                    alias: None,
                    evm_address: None,
                    checksum: None,
                },
                AccountId {
                    shard: 0,
                    realm: 0,
                    num: 3,
                    alias: None,
                    evm_address: None,
                    checksum: None,
                },
            ]),
        }
    }

    #[test]
    fn should_serialize() {
        let original_assessed_custom_fee = make_fee();
        let assessed_custom_fee_bytes = original_assessed_custom_fee.to_bytes();
        let copy_assessed_custom_fee =
            AssessedCustomFee::from_bytes(&assessed_custom_fee_bytes).unwrap();

        assert_eq!(original_assessed_custom_fee, copy_assessed_custom_fee);

        expect![[r#"
            AssessedCustomFee {
                amount: 201,
                token_id: Some(
                    "1.2.3",
                ),
                fee_collector_account_id: Some(
                    "4.5.6",
                ),
                payer_account_id_list: [
                    "0.0.1",
                    "0.0.2",
                    "0.0.3",
                ],
            }
        "#]]
        .assert_debug_eq(&original_assessed_custom_fee);
    }

    #[test]
    fn from_protobuf() {
        expect![[r#"
            AssessedCustomFee {
                amount: 1,
                token_id: Some(
                    "2.3.4",
                ),
                fee_collector_account_id: Some(
                    "5.6.7",
                ),
                payer_account_id_list: [
                    "8.9.10",
                    "11.12.13",
                    "14.15.16",
                ],
            }
        "#]]
        .assert_debug_eq(&AssessedCustomFee::from_protobuf(make_fee_proto()).unwrap());
    }

    #[test]
    fn to_protobuf() {
        expect![[r#"
            AssessedCustomFee {
                amount: 1,
                token_id: Some(
                    TokenId {
                        shard_num: 2,
                        realm_num: 3,
                        token_num: 4,
                    },
                ),
                fee_collector_account_id: Some(
                    AccountId {
                        shard_num: 5,
                        realm_num: 6,
                        account: Some(
                            AccountNum(
                                7,
                            ),
                        ),
                    },
                ),
                effective_payer_account_id: [
                    AccountId {
                        shard_num: 8,
                        realm_num: 9,
                        account: Some(
                            AccountNum(
                                10,
                            ),
                        ),
                    },
                    AccountId {
                        shard_num: 11,
                        realm_num: 12,
                        account: Some(
                            AccountNum(
                                13,
                            ),
                        ),
                    },
                    AccountId {
                        shard_num: 14,
                        realm_num: 15,
                        account: Some(
                            AccountNum(
                                16,
                            ),
                        ),
                    },
                ],
            }
        "#]]
        .assert_debug_eq(
            &AssessedCustomFee::from_protobuf(make_fee_proto()).unwrap().to_protobuf(),
        );
    }

    #[test]
    fn should_bytes() {
        let assessed_custom_fee = make_fee();
        assert_eq!(
            assessed_custom_fee,
            AssessedCustomFee::from_bytes(&assessed_custom_fee.to_bytes()).unwrap()
        );
    }
}
