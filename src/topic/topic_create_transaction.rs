/*
 * ‌
 * Hedera Rust SDK
 * ​
 * Copyright (C) 2022 - 2023 Hedera Hashgraph, LLC
 * ​
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * ‍
 */

use hedera_proto::services;
use hedera_proto::services::consensus_service_client::ConsensusServiceClient;
use time::Duration;
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};
use crate::transaction::{
    AnyTransactionData,
    ChunkInfo,
    ToSchedulableTransactionDataProtobuf,
    ToTransactionDataProtobuf,
    TransactionData,
    TransactionExecute,
};
use crate::{
    AccountId,
    BoxGrpcFuture,
    Error,
    Key,
    Transaction,
    ValidateChecksums,
};

/// Create a topic to be used for consensus.
///
/// If an `auto_renew_account_id` is specified, that account must also sign this transaction.
///
/// If an `admin_key` is specified, the adminKey must sign the transaction.
///
/// On success, the resulting `TransactionReceipt` contains the newly created `TopicId`.
///
pub type TopicCreateTransaction = Transaction<TopicCreateTransactionData>;

#[derive(Debug, Clone)]
pub struct TopicCreateTransactionData {
    /// Short publicly visible memo about the topic. No guarantee of uniqueness.
    topic_memo: String,

    /// Access control for `TopicUpdateTransaction` and `TopicDeleteTransaction`.
    admin_key: Option<Key>,

    /// Access control for `TopicMessageSubmitTransaction`.
    submit_key: Option<Key>,

    /// The initial lifetime of the topic and the amount of time to attempt to
    /// extend the topic's lifetime by automatically at the topic's expiration time, if
    /// the `auto_renew_account_id` is configured.
    auto_renew_period: Option<Duration>,

    /// Account to be used at the topic's expiration time to extend the life of the topic.
    auto_renew_account_id: Option<AccountId>,
}

impl Default for TopicCreateTransactionData {
    fn default() -> Self {
        Self {
            topic_memo: String::new(),
            admin_key: None,
            submit_key: None,
            auto_renew_period: Some(Duration::days(90)),
            auto_renew_account_id: None,
        }
    }
}

impl TopicCreateTransaction {
    /// Returns the short, publicly visible, memo about the topic.
    #[must_use]
    pub fn get_topic_memo(&self) -> &str {
        &self.data().topic_memo
    }

    /// Sets the short publicly visible memo about the topic.
    ///
    /// No guarantee of uniqueness.
    pub fn topic_memo(&mut self, memo: impl Into<String>) -> &mut Self {
        self.data_mut().topic_memo = memo.into();
        self
    }

    /// Returns the access control for [`TopicUpdateTransaction`](crate::TopicUpdateTransaction)
    /// and [`TopicDeleteTransaction`](crate::TopicDeleteTransaction).
    #[must_use]
    pub fn get_admin_key(&self) -> Option<&Key> {
        self.data().admin_key.as_ref()
    }

    /// Sets the access control for [`TopicUpdateTransaction`](crate::TopicUpdateTransaction)
    /// and [`TopicDeleteTransaction`](crate::TopicDeleteTransaction).
    pub fn admin_key(&mut self, key: impl Into<Key>) -> &mut Self {
        self.data_mut().admin_key = Some(key.into());
        self
    }

    /// Returns the access control for [`TopicMessageSubmitTransaction`](crate::TopicMessageSubmitTransaction)
    #[must_use]
    pub fn get_submit_key(&self) -> Option<&Key> {
        self.data().submit_key.as_ref()
    }

    /// Sets the access control for [`TopicMessageSubmitTransaction`](crate::TopicMessageSubmitTransaction).
    pub fn submit_key(&mut self, key: impl Into<Key>) -> &mut Self {
        self.data_mut().submit_key = Some(key.into());
        self
    }

    /// Returns the initial lifetime of the topic and the amount of time to attempt to
    /// extend the topic's lifetime by automatically at the topic's expiration time.
    #[must_use]
    pub fn get_auto_renew_period(&self) -> Option<Duration> {
        self.data().auto_renew_period
    }

    /// Sets the initial lifetime of the topic and the amount of time to attempt to
    /// extend the topic's lifetime by automatically at the topic's expiration time.
    pub fn auto_renew_period(&mut self, period: Duration) -> &mut Self {
        self.data_mut().auto_renew_period = Some(period);
        self
    }

    /// Returns the account to be used at the topic's expiration time to extend the life of the topic.
    #[must_use]
    pub fn get_auto_renew_account_id(&self) -> Option<AccountId> {
        self.data().auto_renew_account_id
    }

    /// Sets the account to be used at the topic's expiration time to extend the life of the topic.
    pub fn auto_renew_account_id(&mut self, id: AccountId) -> &mut Self {
        self.data_mut().auto_renew_account_id = Some(id);
        self
    }
}

impl TransactionData for TopicCreateTransactionData {}

impl TransactionExecute for TopicCreateTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { ConsensusServiceClient::new(channel).create_topic(request).await })
    }
}

impl ValidateChecksums for TopicCreateTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.auto_renew_account_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for TopicCreateTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        // Generate the protobuf data
        let mut protobuf_data = self.to_protobuf();

        // Manually assign the auto_renew_account with operator_id if none is set
        if protobuf_data.auto_renew_account.is_none() {
            let operator_id = chunk_info.current_transaction_id.account_id;
            protobuf_data.auto_renew_account = Some(operator_id.to_protobuf());
        }
        services::transaction_body::Data::ConsensusCreateTopic(protobuf_data)
    }
}

impl ToSchedulableTransactionDataProtobuf for TopicCreateTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::ConsensusCreateTopic(self.to_protobuf())
    }
}

impl From<TopicCreateTransactionData> for AnyTransactionData {
    fn from(transaction: TopicCreateTransactionData) -> Self {
        Self::TopicCreate(transaction)
    }
}

impl FromProtobuf<services::ConsensusCreateTopicTransactionBody> for TopicCreateTransactionData {
    fn from_protobuf(pb: services::ConsensusCreateTopicTransactionBody) -> crate::Result<Self> {
        Ok(Self {
            topic_memo: pb.memo,
            admin_key: Option::from_protobuf(pb.admin_key)?,
            submit_key: Option::from_protobuf(pb.submit_key)?,
            auto_renew_period: pb.auto_renew_period.map(Into::into),
            auto_renew_account_id: Option::from_protobuf(pb.auto_renew_account)?,
        })
    }
}

impl ToProtobuf for TopicCreateTransactionData {
    type Protobuf = services::ConsensusCreateTopicTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::ConsensusCreateTopicTransactionBody {
            auto_renew_account: self.auto_renew_account_id.to_protobuf(),
            memo: self.topic_memo.clone(),
            admin_key: self.admin_key.to_protobuf(),
            submit_key: self.submit_key.to_protobuf(),
            auto_renew_period: self.auto_renew_period.to_protobuf(),
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hedera_proto::services;
    use time::Duration;

    use super::TopicCreateTransactionData;
    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
        unused_private_key,
    };
    use crate::{
        AccountId,
        AnyTransaction,
        PublicKey,
        TopicCreateTransaction,
    };

    fn key() -> PublicKey {
        unused_private_key().public_key()
    }

    const AUTO_RENEW_ACCOUNT_ID: AccountId = AccountId::new(0, 0, 5007);
    const AUTO_RENEW_PERIOD: Duration = Duration::days(1);

    fn make_transaction() -> TopicCreateTransaction {
        let mut tx = TopicCreateTransaction::new_for_tests();

        tx.submit_key(key())
            .admin_key(key())
            .auto_renew_account_id(AUTO_RENEW_ACCOUNT_ID)
            .auto_renew_period(AUTO_RENEW_PERIOD)
            .freeze()
            .unwrap();

        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            ConsensusCreateTopic(
                ConsensusCreateTopicTransactionBody {
                    memo: "",
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
                            seconds: 86400,
                        },
                    ),
                    auto_renew_account: Some(
                        AccountId {
                            shard_num: 0,
                            realm_num: 0,
                            account: Some(
                                AccountNum(
                                    5007,
                                ),
                            ),
                        },
                    ),
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
        let tx = services::ConsensusCreateTopicTransactionBody {
            memo: String::new(),
            admin_key: Some(key().to_protobuf()),
            submit_key: Some(key().to_protobuf()),
            auto_renew_period: Some(AUTO_RENEW_PERIOD.to_protobuf()),
            auto_renew_account: Some(AUTO_RENEW_ACCOUNT_ID.to_protobuf()),
        };

        let tx = TopicCreateTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(tx.admin_key, Some(key().into()));
        assert_eq!(tx.submit_key, Some(key().into()));
        assert_eq!(tx.auto_renew_period, Some(AUTO_RENEW_PERIOD));
        assert_eq!(tx.auto_renew_account_id, Some(AUTO_RENEW_ACCOUNT_ID));
    }

    #[test]
    fn get_set_admin_key() {
        let mut tx = TopicCreateTransaction::new();
        tx.admin_key(key());

        assert_eq!(tx.get_admin_key(), Some(&key().into()));
    }

    #[test]
    #[should_panic]
    fn get_set_admin_key_frozen_panics() {
        make_transaction().admin_key(key());
    }

    #[test]
    fn get_set_submit_key() {
        let mut tx = TopicCreateTransaction::new();
        tx.submit_key(key());

        assert_eq!(tx.get_submit_key(), Some(&key().into()));
    }

    #[test]
    #[should_panic]
    fn get_set_submit_key_frozen_panics() {
        make_transaction().submit_key(key());
    }

    #[test]
    fn get_set_auto_renew_period() {
        let mut tx = TopicCreateTransaction::new();
        tx.auto_renew_period(AUTO_RENEW_PERIOD);

        assert_eq!(tx.get_auto_renew_period(), Some(AUTO_RENEW_PERIOD));
    }

    #[test]
    #[should_panic]
    fn get_set_auto_renew_period_frozen_panics() {
        make_transaction().auto_renew_period(AUTO_RENEW_PERIOD);
    }

    #[test]
    fn get_set_auto_renew_account_id() {
        let mut tx = TopicCreateTransaction::new();
        tx.auto_renew_account_id(AUTO_RENEW_ACCOUNT_ID);

        assert_eq!(tx.get_auto_renew_account_id(), Some(AUTO_RENEW_ACCOUNT_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_auto_renew_account_id_frozen_panics() {
        make_transaction().auto_renew_account_id(AUTO_RENEW_ACCOUNT_ID);
    }
}
