// SPDX-License-Identifier: Apache-2.0

use std::ops::Not;

use hedera_proto::services;
#[cfg(test)]
pub(super) use tests::make_receipt;

use crate::protobuf::ToProtobuf;
use crate::{
    AccountId,
    ContractId,
    Error,
    ExchangeRates,
    FileId,
    FromProtobuf,
    ScheduleId,
    Status,
    TokenId,
    TopicId,
    TransactionId,
};

/// The summary of a transaction's result so far, if the transaction has reached consensus.
/// Response from [`TransactionReceiptQuery`][crate::TransactionReceiptQuery].

#[derive(Debug, Clone)]
pub struct TransactionReceipt {
    // fixme(sr): better doc comment.
    /// The ID of the transaction that this is a receipt for.
    pub transaction_id: Option<TransactionId>,

    /// The consensus status of the transaction; is UNKNOWN if consensus has not been reached, or if
    /// the associated transaction did not have a valid payer signature.
    pub status: Status,

    /// In the receipt for an `AccountCreateTransaction`, the id of the newly created account.
    pub account_id: Option<AccountId>,

    /// In the receipt for a `FileCreateTransaction`, the id of the newly created file.
    pub file_id: Option<FileId>,

    /// In the receipt for a `ContractCreateTransaction`, the id of the newly created contract.
    pub contract_id: Option<ContractId>,

    /// The exchange rates in effect when the transaction reached consensus.
    pub exchange_rates: Option<ExchangeRates>,

    /// In the receipt for a `TopicCreateTransaction`, the id of the newly created topic.
    pub topic_id: Option<TopicId>,

    /// In the receipt for a `TopicMessageSubmitTransaction`, the new sequence number of the topic
    /// that received the message.
    pub topic_sequence_number: u64,

    // TODO: use a hash type (for display/debug/serialize purposes)
    /// In the receipt for a `TopicMessageSubmitTransaction`, the new running hash of the
    /// topic that received the message.
    pub topic_running_hash: Option<Vec<u8>>,

    /// In the receipt of a `TopicMessageSubmitTransaction`, the version of the SHA-384
    /// digest used to update the running hash.
    pub topic_running_hash_version: u64,

    /// In the receipt for a `TokenCreateTransaction`, the id of the newly created token.
    pub token_id: Option<TokenId>,

    /// Populated in the receipt of `TokenMint`, `TokenWipe`, and `TokenBurn` transactions.
    ///
    /// For fungible tokens, the current total supply of this token.
    /// For non-fungible tokens, the total number of NFTs issued for a given token id.
    pub total_supply: u64,

    /// In the receipt for a `ScheduleCreateTransaction`, the id of the newly created schedule.
    pub schedule_id: Option<ScheduleId>,

    /// In the receipt of a `ScheduleCreateTransaction` or `ScheduleSignTransaction` that resolves
    /// to `Success`, the `TransactionId` that should be used to query for the receipt or
    /// record of the relevant scheduled transaction.
    pub scheduled_transaction_id: Option<TransactionId>,

    /// In the receipt of a `TokenMintTransaction` for tokens of type `NonFungibleUnique`,
    /// the serial numbers of the newly created NFTs.
    pub serials: Vec<i64>,

    /// The receipts of processing all transactions with the given id, in consensus time order.
    pub duplicates: Vec<TransactionReceipt>,

    /// The receipts (if any) of all child transactions spawned by the transaction with the
    /// given top-level id, in consensus order.
    pub children: Vec<TransactionReceipt>,

    /// In the receipt of a NodeCreate, NodeUpdate, NodeDelete, the id of the newly created node.
    /// An affected node identifier.
    pub node_id: u64,
}

impl TransactionReceipt {
    /// Create a new `TransactionReceipt` from protobuf-encoded `bytes`.
    ///
    /// # Errors
    /// - [`Error::FromProtobuf`] if decoding the bytes fails to produce a valid protobuf.
    /// - [`Error::FromProtobuf`] if decoding the protobuf fails.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        FromProtobuf::<services::TransactionReceipt>::from_bytes(bytes)
    }

    /// Convert `self` to a protobuf-encoded [`Vec<u8>`].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        ToProtobuf::to_bytes(self)
    }

    /// Validate [`status`](Self.status) and return an `Err` if it isn't [`Status::Success`]
    ///
    /// # Errors
    /// - [`Error::ReceiptStatus`] if `validate && self.status != Status::Success`
    pub fn validate_status(&self, validate: bool) -> crate::Result<&Self> {
        if validate && self.status != Status::Success {
            Err(Error::ReceiptStatus {
                status: self.status,
                transaction_id: self.transaction_id.map(Box::new),
            })
        } else {
            Ok(self)
        }
    }

    fn from_protobuf(
        receipt: services::TransactionReceipt,
        duplicates: Vec<Self>,
        children: Vec<Self>,
        transaction_id: Option<&TransactionId>,
    ) -> crate::Result<Self> {
        let status = Status::try_from(receipt.status).unwrap_or_default();

        let account_id = Option::from_protobuf(receipt.account_id)?;
        let file_id = Option::from_protobuf(receipt.file_id)?;
        let contract_id = Option::from_protobuf(receipt.contract_id)?;
        let topic_id = Option::from_protobuf(receipt.topic_id)?;
        let token_id = Option::from_protobuf(receipt.token_id)?;
        let schedule_id = Option::from_protobuf(receipt.schedule_id)?;
        let exchange_rates = Option::from_protobuf(receipt.exchange_rate)?;

        let scheduled_transaction_id = Option::from_protobuf(receipt.scheduled_transaction_id)?;

        Ok(Self {
            status,
            total_supply: receipt.new_total_supply,
            serials: receipt.serial_numbers,
            exchange_rates,
            topic_running_hash_version: receipt.topic_running_hash_version,
            topic_sequence_number: receipt.topic_sequence_number,
            topic_running_hash: receipt
                .topic_running_hash
                .is_empty()
                .not()
                .then_some(receipt.topic_running_hash),
            scheduled_transaction_id,
            account_id,
            file_id,
            contract_id,
            topic_id,
            token_id,
            schedule_id,
            duplicates,
            children,
            transaction_id: transaction_id.copied(),
            node_id: receipt.node_id,
        })
    }

    pub(crate) fn from_response_protobuf(
        pb: services::response::Response,
        transaction_id: Option<&TransactionId>,
    ) -> crate::Result<Self> {
        let pb = pb_getv!(pb, TransactionGetReceipt, services::response::Response);

        let receipt = pb_getf!(pb, receipt)?;

        let duplicates = Vec::from_protobuf(pb.duplicate_transaction_receipts)?;

        let children = Vec::from_protobuf(pb.child_transaction_receipts)?;

        Self::from_protobuf(receipt, duplicates, children, transaction_id)
    }
}

impl FromProtobuf<services::response::Response> for TransactionReceipt {
    fn from_protobuf(pb: services::response::Response) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Self::from_response_protobuf(pb, None)
    }
}

impl FromProtobuf<services::TransactionReceipt> for TransactionReceipt {
    fn from_protobuf(receipt: services::TransactionReceipt) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Self::from_protobuf(receipt, Vec::new(), Vec::new(), None)
    }
}

impl ToProtobuf for TransactionReceipt {
    type Protobuf = services::TransactionReceipt;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::TransactionReceipt {
            status: self.status as i32,
            account_id: self.account_id.to_protobuf(),
            file_id: self.file_id.to_protobuf(),
            contract_id: self.contract_id.to_protobuf(),
            exchange_rate: self.exchange_rates.to_protobuf(),
            topic_id: self.topic_id.to_protobuf(),
            topic_sequence_number: self.topic_sequence_number,
            topic_running_hash: self.topic_running_hash.clone().unwrap_or_default(),
            topic_running_hash_version: self.topic_running_hash_version,
            token_id: self.token_id.to_protobuf(),
            new_total_supply: self.total_supply,
            schedule_id: self.schedule_id.to_protobuf(),
            scheduled_transaction_id: self.scheduled_transaction_id.to_protobuf(),
            serial_numbers: self.serials.clone(),
            node_id: self.node_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use time::OffsetDateTime;

    use crate::protobuf::ToProtobuf;
    use crate::transaction::test_helpers::{
        TEST_TX_ID,
        VALID_START,
    };
    use crate::{
        AccountId,
        ContractId,
        ExchangeRate,
        ExchangeRates,
        FileId,
        ScheduleId,
        Status,
        TokenId,
        TopicId,
        TransactionReceipt,
    };

    const EXPIRATION_TIME: OffsetDateTime = VALID_START;

    // needed in `transaction_record`.
    pub(crate) fn make_receipt() -> TransactionReceipt {
        TransactionReceipt {
            transaction_id: None,
            status: Status::ScheduleAlreadyDeleted,
            account_id: Some(AccountId::new(1, 2, 3)),
            file_id: Some(FileId::new(4, 5, 6)),
            exchange_rates: Some(ExchangeRates {
                current_rate: ExchangeRate {
                    hbars: 100,
                    cents: 100,
                    expiration_time: EXPIRATION_TIME,
                    exchange_rate_in_cents: f64::from(100) / f64::from(100),
                },
                next_rate: ExchangeRate {
                    hbars: 200,
                    cents: 200,
                    expiration_time: EXPIRATION_TIME,
                    exchange_rate_in_cents: f64::from(200) / f64::from(200),
                },
            }),
            contract_id: Some(ContractId::new(3, 2, 1)),
            topic_id: Some(TopicId::new(9, 8, 7)),
            topic_sequence_number: 3,
            topic_running_hash: Some(b"how now brown cow".to_vec()),
            topic_running_hash_version: 0,
            token_id: Some(TokenId::new(6, 5, 4)),
            total_supply: 30,
            schedule_id: Some(ScheduleId::new(1, 1, 1)),
            scheduled_transaction_id: Some(TEST_TX_ID),
            serials: Vec::from([1, 2, 3]),
            duplicates: Vec::new(),
            children: Vec::new(),
            node_id: 1,
        }
    }

    #[test]
    fn serialize() {
        expect![[r#"
            TransactionReceipt {
                status: ScheduleAlreadyDeleted,
                account_id: Some(
                    AccountId {
                        shard_num: 1,
                        realm_num: 2,
                        account: Some(
                            AccountNum(
                                3,
                            ),
                        ),
                    },
                ),
                file_id: Some(
                    FileId {
                        shard_num: 4,
                        realm_num: 5,
                        file_num: 6,
                    },
                ),
                contract_id: Some(
                    ContractId {
                        shard_num: 3,
                        realm_num: 2,
                        contract: Some(
                            ContractNum(
                                1,
                            ),
                        ),
                    },
                ),
                exchange_rate: Some(
                    ExchangeRateSet {
                        current_rate: Some(
                            ExchangeRate {
                                hbar_equiv: 100,
                                cent_equiv: 100,
                                expiration_time: Some(
                                    TimestampSeconds {
                                        seconds: 1554158542,
                                    },
                                ),
                            },
                        ),
                        next_rate: Some(
                            ExchangeRate {
                                hbar_equiv: 200,
                                cent_equiv: 200,
                                expiration_time: Some(
                                    TimestampSeconds {
                                        seconds: 1554158542,
                                    },
                                ),
                            },
                        ),
                    },
                ),
                topic_id: Some(
                    TopicId {
                        shard_num: 9,
                        realm_num: 8,
                        topic_num: 7,
                    },
                ),
                topic_sequence_number: 3,
                topic_running_hash: [
                    104,
                    111,
                    119,
                    32,
                    110,
                    111,
                    119,
                    32,
                    98,
                    114,
                    111,
                    119,
                    110,
                    32,
                    99,
                    111,
                    119,
                ],
                topic_running_hash_version: 0,
                token_id: Some(
                    TokenId {
                        shard_num: 6,
                        realm_num: 5,
                        token_num: 4,
                    },
                ),
                new_total_supply: 30,
                schedule_id: Some(
                    ScheduleId {
                        shard_num: 1,
                        realm_num: 1,
                        schedule_num: 1,
                    },
                ),
                scheduled_transaction_id: Some(
                    TransactionId {
                        transaction_valid_start: Some(
                            Timestamp {
                                seconds: 1554158542,
                                nanos: 0,
                            },
                        ),
                        account_id: Some(
                            AccountId {
                                shard_num: 0,
                                realm_num: 0,
                                account: Some(
                                    AccountNum(
                                        5006,
                                    ),
                                ),
                            },
                        ),
                        scheduled: false,
                        nonce: 0,
                    },
                ),
                serial_numbers: [
                    1,
                    2,
                    3,
                ],
                node_id: 1,
            }
        "#]]
        .assert_debug_eq(&make_receipt().to_protobuf())
    }

    #[test]
    fn to_from_bytes() {
        let a = make_receipt();
        let b = TransactionReceipt::from_bytes(&a.to_bytes()).unwrap();

        assert_eq!(a.to_protobuf(), b.to_protobuf());
    }
}
