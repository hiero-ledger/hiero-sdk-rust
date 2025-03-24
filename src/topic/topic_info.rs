// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use time::{
    Duration,
    OffsetDateTime,
};

use crate::custom_fixed_fee::CustomFixedFee;
use crate::protobuf::ToProtobuf;
use crate::{
    AccountId,
    FromProtobuf,
    Key,
    LedgerId,
    TopicId,
};

/// Response from [`TopicInfoQuery`][crate::TopicInfoQuery].

#[derive(Debug, Clone)]
pub struct TopicInfo {
    /// The ID of the topic for which information is requested.
    pub topic_id: TopicId,

    /// Short publicly visible memo about the topic. No guarantee of uniqueness
    pub topic_memo: String,

    /// SHA-384 running hash of (previousRunningHash, topicId, consensusTimestamp, sequenceNumber, message).
    pub running_hash: Vec<u8>,

    /// Sequence number (starting at 1 for the first submitMessage) of messages on the topic.
    pub sequence_number: u64,

    /// Effective consensus timestamp at (and after) which submitMessage calls will no longer succeed on the topic.
    pub expiration_time: Option<OffsetDateTime>,

    /// Access control for update/delete of the topic.
    pub admin_key: Option<Key>,

    /// Access control for submit message.
    pub submit_key: Option<Key>,

    /// An account which will be automatically charged to renew the topic's expiration, at
    /// `auto_renew_period` interval.
    pub auto_renew_account_id: Option<AccountId>,

    /// The interval at which the auto-renew account will be charged to extend the topic's expiry.
    pub auto_renew_period: Option<Duration>,

    /// The ledger ID the response was returned from
    pub ledger_id: LedgerId,

    /// Access control for update/delete of custom fees.
    pub fee_schedule_key: Option<Key>,

    /// If the transaction contains a signer from this list, no custom fees are applied.
    pub fee_exempt_keys: Vec<Key>,

    /// List of custom fees.
    pub custom_fees: Vec<CustomFixedFee>,
}

impl TopicInfo {
    /// Create a new `TopicInfo` from protobuf-encoded `bytes`.
    ///
    /// # Errors
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the bytes fails to produce a valid protobuf.
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the protobuf fails.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        FromProtobuf::<services::ConsensusGetTopicInfoResponse>::from_bytes(bytes)
    }

    /// Convert `self` to a protobuf-encoded [`Vec<u8>`].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        ToProtobuf::to_bytes(self)
    }
}

impl FromProtobuf<services::response::Response> for TopicInfo {
    fn from_protobuf(pb: services::response::Response) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let response = pb_getv!(pb, ConsensusGetTopicInfo, services::response::Response);
        Self::from_protobuf(response)
    }
}

impl FromProtobuf<services::ConsensusGetTopicInfoResponse> for TopicInfo {
    fn from_protobuf(pb: services::ConsensusGetTopicInfoResponse) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let topic_id = pb_getf!(pb, topic_id)?;
        let info = pb_getf!(pb, topic_info)?;
        let admin_key = Option::from_protobuf(info.admin_key)?;
        let submit_key = Option::from_protobuf(info.submit_key)?;
        let expiration_time = info.expiration_time.map(Into::into);
        let auto_renew_period = info.auto_renew_period.map(Into::into);
        let auto_renew_account_id = Option::from_protobuf(info.auto_renew_account)?;
        let ledger_id = LedgerId::from_bytes(info.ledger_id);
        let fee_schedule_key = Option::from_protobuf(info.fee_schedule_key)?;

        let mut fee_exempt_keys = Vec::new();
        for pb_key in info.fee_exempt_key_list {
            fee_exempt_keys.push(Key::from_protobuf(pb_key)?);
        }

        let mut custom_fees = Vec::new();
        for pb_fee in info.custom_fees {
            custom_fees.push(CustomFixedFee::from_protobuf(pb_fee)?);
        }

        Ok(Self {
            topic_id: TopicId::from_protobuf(topic_id)?,
            admin_key,
            submit_key,
            auto_renew_period,
            auto_renew_account_id,
            running_hash: info.running_hash,
            sequence_number: info.sequence_number,
            expiration_time,
            topic_memo: info.memo,
            ledger_id,
            fee_schedule_key,
            fee_exempt_keys,
            custom_fees,
        })
    }
}

impl ToProtobuf for TopicInfo {
    type Protobuf = services::ConsensusGetTopicInfoResponse;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::ConsensusGetTopicInfoResponse {
            topic_id: Some(self.topic_id.to_protobuf()),
            topic_info: Some(services::ConsensusTopicInfo {
                memo: self.topic_memo.clone(),
                running_hash: self.running_hash.clone(),
                sequence_number: self.sequence_number,
                expiration_time: self.expiration_time.to_protobuf(),
                admin_key: self.admin_key.to_protobuf(),
                submit_key: self.submit_key.to_protobuf(),
                auto_renew_period: self.auto_renew_period.to_protobuf(),
                auto_renew_account: self.auto_renew_account_id.to_protobuf(),
                ledger_id: self.ledger_id.to_bytes(),
                custom_fees: vec![],
                fee_exempt_key_list: vec![],
                fee_schedule_key: None,
            }),
            header: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hedera_proto::services;
    use prost::Message;

    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::transaction::test_helpers::unused_private_key;
    use crate::{
        LedgerId,
        TopicInfo,
    };

    fn make_info() -> services::ConsensusGetTopicInfoResponse {
        services::ConsensusGetTopicInfoResponse {
            header: None,
            topic_id: Some(services::TopicId { shard_num: 1, realm_num: 2, topic_num: 3 }),
            topic_info: Some(services::ConsensusTopicInfo {
                memo: "1".to_owned(),
                running_hash: Vec::from([2]),
                sequence_number: 3,
                expiration_time: Some(services::Timestamp { seconds: 0, nanos: 4_000_000 }),
                admin_key: Some(unused_private_key().public_key().to_protobuf()),
                submit_key: Some(unused_private_key().public_key().to_protobuf()),
                auto_renew_period: Some(services::Duration { seconds: 5 * 24 * 60 * 60 }),
                auto_renew_account: Some(services::AccountId {
                    shard_num: 0,
                    realm_num: 0,
                    account: Some(services::account_id::Account::AccountNum(4)),
                }),
                ledger_id: LedgerId::testnet().to_bytes(),
                custom_fees: vec![],
                fee_exempt_key_list: vec![],
                fee_schedule_key: None,
            }),
        }
    }

    #[test]
    fn from_protobuf() {
        expect![[r#"
            TopicInfo {
                topic_id: "1.2.3",
                topic_memo: "1",
                running_hash: [
                    2,
                ],
                sequence_number: 3,
                expiration_time: Some(
                    1970-01-01 0:00:00.004 +00:00:00,
                ),
                admin_key: Some(
                    Single(
                        "302a300506032b6570032100e0c8ec2758a5879ffac226a13c0c516b799e72e35141a0dd828f94d37988a4b7",
                    ),
                ),
                submit_key: Some(
                    Single(
                        "302a300506032b6570032100e0c8ec2758a5879ffac226a13c0c516b799e72e35141a0dd828f94d37988a4b7",
                    ),
                ),
                auto_renew_account_id: Some(
                    "0.0.4",
                ),
                auto_renew_period: Some(
                    Duration {
                        seconds: 432000,
                        nanoseconds: 0,
                    },
                ),
                ledger_id: "testnet",
                fee_schedule_key: None,
                fee_exempt_keys: [],
                custom_fees: [],
            }
        "#]]
        .assert_debug_eq(&TopicInfo::from_protobuf(make_info()).unwrap())
    }

    #[test]
    fn to_protobuf() {
        expect![[r#"
            ConsensusGetTopicInfoResponse {
                header: None,
                topic_id: Some(
                    TopicId {
                        shard_num: 1,
                        realm_num: 2,
                        topic_num: 3,
                    },
                ),
                topic_info: Some(
                    ConsensusTopicInfo {
                        memo: "1",
                        running_hash: [
                            2,
                        ],
                        sequence_number: 3,
                        expiration_time: Some(
                            Timestamp {
                                seconds: 0,
                                nanos: 4000000,
                            },
                        ),
                        admin_key: Some(
                            Key {
                                key: Some(
                                    Ed25519(
                                        [
                                            224,
                                            200,
                                            236,
                                            39,
                                            88,
                                            165,
                                            135,
                                            159,
                                            250,
                                            194,
                                            38,
                                            161,
                                            60,
                                            12,
                                            81,
                                            107,
                                            121,
                                            158,
                                            114,
                                            227,
                                            81,
                                            65,
                                            160,
                                            221,
                                            130,
                                            143,
                                            148,
                                            211,
                                            121,
                                            136,
                                            164,
                                            183,
                                        ],
                                    ),
                                ),
                            },
                        ),
                        submit_key: Some(
                            Key {
                                key: Some(
                                    Ed25519(
                                        [
                                            224,
                                            200,
                                            236,
                                            39,
                                            88,
                                            165,
                                            135,
                                            159,
                                            250,
                                            194,
                                            38,
                                            161,
                                            60,
                                            12,
                                            81,
                                            107,
                                            121,
                                            158,
                                            114,
                                            227,
                                            81,
                                            65,
                                            160,
                                            221,
                                            130,
                                            143,
                                            148,
                                            211,
                                            121,
                                            136,
                                            164,
                                            183,
                                        ],
                                    ),
                                ),
                            },
                        ),
                        auto_renew_period: Some(
                            Duration {
                                seconds: 432000,
                            },
                        ),
                        auto_renew_account: Some(
                            AccountId {
                                shard_num: 0,
                                realm_num: 0,
                                account: Some(
                                    AccountNum(
                                        4,
                                    ),
                                ),
                            },
                        ),
                        ledger_id: [
                            1,
                        ],
                        fee_schedule_key: None,
                        fee_exempt_key_list: [],
                        custom_fees: [],
                    },
                ),
            }
        "#]]
        .assert_debug_eq(&TopicInfo::from_protobuf(make_info()).unwrap().to_protobuf())
    }

    #[test]
    fn from_bytes() {
        expect![[r#"
            TopicInfo {
                topic_id: "1.2.3",
                topic_memo: "1",
                running_hash: [
                    2,
                ],
                sequence_number: 3,
                expiration_time: Some(
                    1970-01-01 0:00:00.004 +00:00:00,
                ),
                admin_key: Some(
                    Single(
                        "302a300506032b6570032100e0c8ec2758a5879ffac226a13c0c516b799e72e35141a0dd828f94d37988a4b7",
                    ),
                ),
                submit_key: Some(
                    Single(
                        "302a300506032b6570032100e0c8ec2758a5879ffac226a13c0c516b799e72e35141a0dd828f94d37988a4b7",
                    ),
                ),
                auto_renew_account_id: Some(
                    "0.0.4",
                ),
                auto_renew_period: Some(
                    Duration {
                        seconds: 432000,
                        nanoseconds: 0,
                    },
                ),
                ledger_id: "testnet",
                fee_schedule_key: None,
                fee_exempt_keys: [],
                custom_fees: [],
            }
        "#]]
        .assert_debug_eq(&TopicInfo::from_bytes(&make_info().encode_to_vec()).unwrap())
    }
}
