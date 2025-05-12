// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::consensus_service_client::ConsensusServiceClient;
use time::{
    Duration,
    OffsetDateTime,
};
use tonic::transport::Channel;

use crate::custom_fixed_fee::CustomFixedFee;
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
    TopicId,
    Transaction,
    ValidateChecksums,
};

/// Change properties for the given topic.
///
/// Any null field is ignored (left unchanged).
///
pub type TopicUpdateTransaction = Transaction<TopicUpdateTransactionData>;

#[derive(Debug, Clone, Default)]
pub struct TopicUpdateTransactionData {
    /// The topic ID which is being updated in this transaction.
    topic_id: Option<TopicId>,

    /// The new expiration time to extend to (ignored if equal to or before the current one).
    expiration_time: Option<OffsetDateTime>,

    /// Short publicly visible memo about the topic. No guarantee of uniqueness.
    topic_memo: Option<String>,

    /// Access control for `TopicUpdateTransaction` and `TopicDeleteTransaction`.
    admin_key: Option<Key>,

    /// Access control for `TopicMessageSubmitTransaction`.
    submit_key: Option<Key>,

    /// The initial lifetime of the topic and the amount of time to attempt to
    /// extend the topic's lifetime by automatically at the topic's expiration time, if
    /// the `auto_renew_account_id` is configured.
    auto_renew_period: Option<Duration>,

    /// Optional account to be used at the topic's expiration time to extend the life of the topic.
    auto_renew_account_id: Option<AccountId>,

    /// Access control for update/delete of custom fees.
    /// None if the key should not be updated.
    fee_schedule_key: Option<Key>,

    /// If the transaction contains a signer from this list, no custom fees are applied.
    fee_exempt_keys: Vec<Key>,

    /// The custom fee to be assessed during a message submission to this topic.
    custom_fees: Option<Vec<CustomFixedFee>>,
}

impl TopicUpdateTransaction {
    /// Returns the topic ID which is being updated.
    #[must_use]
    pub fn get_topic_id(&self) -> Option<TopicId> {
        self.data().topic_id
    }

    /// Sets the topic ID which is being updated.
    pub fn topic_id(&mut self, id: impl Into<TopicId>) -> &mut Self {
        self.data_mut().topic_id = Some(id.into());
        self
    }

    /// Returns the new expiration time to extend to (ignored if equal to or before the current one).
    #[must_use]
    pub fn get_expiration_time(&self) -> Option<OffsetDateTime> {
        self.data().expiration_time
    }

    /// Sets the new expiration time to extend to (ignored if equal to or before the current one).
    pub fn expiration_time(&mut self, at: OffsetDateTime) -> &mut Self {
        self.data_mut().expiration_time = Some(at);
        self
    }

    /// Returns the new topic memo for the topic.
    #[must_use]
    pub fn get_topic_memo(&self) -> Option<&str> {
        self.data().topic_memo.as_deref()
    }

    /// Sets the short publicly visible memo about the topic.
    ///
    /// No guarantee of uniqueness.
    pub fn topic_memo(&mut self, memo: impl Into<String>) -> &mut Self {
        self.data_mut().topic_memo = Some(memo.into());
        self
    }

    /// Returns the access control for [`TopicUpdateTransaction`] and [`TopicDeleteTransaction`](crate::TopicDeleteTransaction).
    #[must_use]
    pub fn get_admin_key(&self) -> Option<&Key> {
        self.data().admin_key.as_ref()
    }

    /// Sets the access control for [`TopicUpdateTransaction`] and [`TopicDeleteTransaction`](crate::TopicDeleteTransaction).
    pub fn admin_key(&mut self, key: impl Into<Key>) -> &mut Self {
        self.data_mut().admin_key = Some(key.into());
        self
    }

    /// Clears the access control for [`TopicUpdateTransaction`] and [`TopicDeleteTransaction`](crate::TopicDeleteTransaction).
    pub fn clear_admin_key(&mut self) -> &mut Self {
        self.data_mut().admin_key = Some(Key::KeyList(crate::KeyList::new()));
        self
    }

    /// Returns the access control for [`TopicMessageSubmitTransaction`](crate::TopicMessageSubmitTransaction).
    #[must_use]
    pub fn get_submit_key(&self) -> Option<&Key> {
        self.data().submit_key.as_ref()
    }

    /// Sets the access control for [`TopicMessageSubmitTransaction`](crate::TopicMessageSubmitTransaction).
    pub fn submit_key(&mut self, key: impl Into<Key>) -> &mut Self {
        self.data_mut().submit_key = Some(key.into());
        self
    }
    /// Clears the access control for [`TopicUpdateTransaction`] and [`TopicDeleteTransaction`](crate::TopicDeleteTransaction).
    pub fn clear_submit_key(&mut self) -> &mut Self {
        self.data_mut().submit_key = Some(Key::KeyList(crate::KeyList::new()));
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

    /// Clear the auto renew account ID for this topic.
    pub fn clear_auto_renew_account_id(&mut self) -> &mut Self {
        self.auto_renew_account_id(AccountId {
            shard: 0,
            realm: 0,
            num: 0,
            alias: None,
            evm_address: None,
            checksum: None,
        })
    }

    /// The key that can be used to update the fee schedule for the topic.
    pub fn fee_schedule_key(&mut self, key: impl Into<Key>) -> &mut Self {
        self.data_mut().fee_schedule_key = Some(key.into());
        self
    }

    /// The key that can be used to update the fee schedule for the topic.
    #[must_use]
    pub fn get_fee_schedule_key(&self) -> Option<&Key> {
        self.data().fee_schedule_key.as_ref()
    }

    /// The keys that can be used to update the fee schedule for the topic.
    pub fn fee_exempt_keys(&mut self, keys: Vec<Key>) -> &mut Self {
        self.data_mut().fee_exempt_keys = keys;
        self
    }

    /// The keys that can be used to update the fee schedule for the topic.
    #[must_use]
    pub fn get_fee_exempt_keys(&self) -> &Vec<Key> {
        &self.data().fee_exempt_keys
    }

    /// Clears the keys that can be used to update the fee schedule for the topic.
    pub fn clear_fee_exempt_keys(&mut self) -> &mut Self {
        self.data_mut().fee_exempt_keys = vec![];
        self
    }

    /// Adds a key to the list of keys that can be used to update the fee schedule for the topic.
    pub fn add_fee_exempt_key(&mut self, key: Key) -> &mut Self {
        self.data_mut().fee_exempt_keys.push(key);
        self
    }

    /// The custom fees to be assessed during a message submission to this topic.
    pub fn custom_fees(&mut self, fees: Vec<CustomFixedFee>) -> &mut Self {
        self.data_mut().custom_fees = Some(fees);
        self
    }

    /// Clears the custom fees for this topic.
    pub fn clear_custom_fees(&mut self) -> &mut Self {
        self.data_mut().custom_fees = None;
        self
    }

    /// The custom fees to be assessed during a message submission to this topic.
    #[must_use]
    pub fn get_custom_fees(&self) -> Option<&Vec<CustomFixedFee>> {
        self.data().custom_fees.as_ref()
    }

    /// Adds a custom fee to the list of custom fees for this topic.
    pub fn add_custom_fee(&mut self, fee: CustomFixedFee) -> &mut Self {
        self.data_mut().custom_fees = Some(vec![fee]);
        self
    }
}

impl TransactionData for TopicUpdateTransactionData {}

impl TransactionExecute for TopicUpdateTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { ConsensusServiceClient::new(channel).update_topic(request).await })
    }
}

impl ValidateChecksums for TopicUpdateTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.topic_id.validate_checksums(ledger_id)?;
        self.auto_renew_account_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for TopicUpdateTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::ConsensusUpdateTopic(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for TopicUpdateTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::ConsensusUpdateTopic(self.to_protobuf())
    }
}

impl From<TopicUpdateTransactionData> for AnyTransactionData {
    fn from(transaction: TopicUpdateTransactionData) -> Self {
        Self::TopicUpdate(transaction)
    }
}

impl FromProtobuf<services::ConsensusUpdateTopicTransactionBody> for TopicUpdateTransactionData {
    fn from_protobuf(pb: services::ConsensusUpdateTopicTransactionBody) -> crate::Result<Self> {
        let custom_fees = if let Some(custom_fees) = pb.custom_fees {
            Some(
                custom_fees
                    .fees
                    .into_iter()
                    .map(CustomFixedFee::from_protobuf)
                    .collect::<Result<Vec<_>, _>>()?,
            )
        } else {
            None
        };

        let fee_exempt_keys = if let Some(fee_exempt_keys) = pb.fee_exempt_key_list {
            fee_exempt_keys
                .keys
                .into_iter()
                .map(|pb_key| Key::from_protobuf(pb_key))
                .collect::<Result<Vec<_>, _>>()?
        } else {
            Vec::new()
        };

        Ok(Self {
            topic_id: Option::from_protobuf(pb.topic_id)?,
            expiration_time: pb.expiration_time.map(Into::into),
            topic_memo: pb.memo,
            admin_key: Option::from_protobuf(pb.admin_key)?,
            submit_key: Option::from_protobuf(pb.submit_key)?,
            auto_renew_period: pb.auto_renew_period.map(Into::into),
            auto_renew_account_id: Option::from_protobuf(pb.auto_renew_account)?,
            fee_schedule_key: Option::from_protobuf(pb.fee_schedule_key)?,
            fee_exempt_keys,
            custom_fees,
        })
    }
}

impl ToProtobuf for TopicUpdateTransactionData {
    type Protobuf = services::ConsensusUpdateTopicTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        let topic_id = self.topic_id.to_protobuf();
        let expiration_time = self.expiration_time.map(Into::into);
        let admin_key = self.admin_key.to_protobuf();
        let submit_key = self.submit_key.to_protobuf();
        let fee_schedule_key = self.fee_schedule_key.to_protobuf();

        let auto_renew_period = self.auto_renew_period.map(Into::into);
        let auto_renew_account_id = self.auto_renew_account_id.to_protobuf();
        let custom_fees = self.custom_fees.as_ref().map(|fees| services::FixedCustomFeeList {
            fees: fees.iter().map(|fee| fee.to_protobuf()).collect(),
        });

        let fee_exempt_key_list = if self.fee_exempt_keys.is_empty() {
            None
        } else {
            Some(services::FeeExemptKeyList {
                keys: self.fee_exempt_keys.iter().map(|key| key.to_protobuf()).collect(),
            })
        };

        services::ConsensusUpdateTopicTransactionBody {
            auto_renew_account: auto_renew_account_id,
            memo: self.topic_memo.clone(),
            expiration_time,
            topic_id,
            admin_key,
            submit_key,
            auto_renew_period,
            fee_exempt_key_list,
            fee_schedule_key,
            custom_fees,
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use time::{
        Duration,
        OffsetDateTime,
    };

    use crate::custom_fixed_fee::CustomFixedFee;
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
        unused_private_key,
        VALID_START,
    };
    use crate::{
        AccountId,
        AnyTransaction,
        Key,
        PrivateKey,
        TokenId,
        TopicId,
        TopicUpdateTransaction,
    };

    const TEST_TOPIC_ID: TopicId = TopicId::new(0, 0, 5007);
    const TEST_TOPIC_MEMO: &str = "test memo";
    const TEST_AUTO_RENEW_PERIOD: Duration = Duration::days(1);
    const TEST_AUTO_RENEW_ACCOUNT_ID: AccountId = AccountId::new(0, 0, 5007);
    const TEST_EXPIRATION_TIME: OffsetDateTime = VALID_START;

    fn make_transaction() -> TopicUpdateTransaction {
        let mut tx = TopicUpdateTransaction::new_for_tests();

        tx.topic_id("0.0.5007".parse::<TopicId>().unwrap())
            .clear_admin_key()
            .clear_auto_renew_account_id()
            .clear_submit_key()
            .topic_memo("")
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
            ConsensusUpdateTopic(
                ConsensusUpdateTopicTransactionBody {
                    topic_id: Some(
                        TopicId {
                            shard_num: 0,
                            realm_num: 0,
                            topic_num: 5007,
                        },
                    ),
                    memo: Some(
                        "",
                    ),
                    expiration_time: None,
                    admin_key: Some(
                        Key {
                            key: Some(
                                KeyList(
                                    KeyList {
                                        keys: [],
                                    },
                                ),
                            ),
                        },
                    ),
                    submit_key: Some(
                        Key {
                            key: Some(
                                KeyList(
                                    KeyList {
                                        keys: [],
                                    },
                                ),
                            ),
                        },
                    ),
                    auto_renew_period: None,
                    auto_renew_account: Some(
                        AccountId {
                            shard_num: 0,
                            realm_num: 0,
                            account: Some(
                                AccountNum(
                                    0,
                                ),
                            ),
                        },
                    ),
                    fee_schedule_key: None,
                    fee_exempt_key_list: None,
                    custom_fees: None,
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

    fn make_transaction2() -> TopicUpdateTransaction {
        let mut tx = TopicUpdateTransaction::new_for_tests();

        tx.topic_id("0.0.5007".parse::<TopicId>().unwrap())
            .admin_key(unused_private_key().public_key())
            .auto_renew_account_id("0.0.5009".parse().unwrap())
            .auto_renew_period(Duration::days(1))
            .submit_key(unused_private_key().public_key())
            .topic_memo("Hello memo")
            .expiration_time(VALID_START)
            .freeze()
            .unwrap();

        tx
    }

    #[test]
    fn serialize2() {
        let tx = make_transaction2();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            ConsensusUpdateTopic(
                ConsensusUpdateTopicTransactionBody {
                    topic_id: Some(
                        TopicId {
                            shard_num: 0,
                            realm_num: 0,
                            topic_num: 5007,
                        },
                    ),
                    memo: Some(
                        "Hello memo",
                    ),
                    expiration_time: Some(
                        Timestamp {
                            seconds: 1554158542,
                            nanos: 0,
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
                            seconds: 86400,
                        },
                    ),
                    auto_renew_account: Some(
                        AccountId {
                            shard_num: 0,
                            realm_num: 0,
                            account: Some(
                                AccountNum(
                                    5009,
                                ),
                            ),
                        },
                    ),
                    fee_schedule_key: None,
                    fee_exempt_key_list: None,
                    custom_fees: None,
                },
            )
        "#]]
        .assert_debug_eq(&tx)
    }

    #[test]
    fn to_from_bytes2() {
        let tx = make_transaction2();

        let tx2 = AnyTransaction::from_bytes(&tx.to_bytes().unwrap()).unwrap();

        let tx = transaction_body(tx);

        let tx2 = transaction_body(tx2);

        assert_eq!(tx, tx2);
    }

    #[test]
    fn get_set_topic_id() {
        let mut tx = TopicUpdateTransaction::new();
        tx.topic_id(TEST_TOPIC_ID);

        assert_eq!(tx.get_topic_id(), Some(TEST_TOPIC_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_topic_id_frozen_panics() {
        make_transaction().topic_id(TEST_TOPIC_ID);
    }

    #[test]
    fn get_set_topic_memo() {
        let mut tx = TopicUpdateTransaction::new();
        tx.topic_memo(TEST_TOPIC_MEMO);

        assert_eq!(tx.get_topic_memo(), Some(TEST_TOPIC_MEMO));
    }

    #[test]
    #[should_panic]
    fn get_set_topic_memo_frozen_panics() {
        make_transaction().topic_memo(TEST_TOPIC_MEMO);
    }

    #[test]
    fn get_set_expiration_time() {
        let mut tx = TopicUpdateTransaction::new();
        tx.expiration_time(TEST_EXPIRATION_TIME);

        assert_eq!(tx.get_expiration_time(), Some(TEST_EXPIRATION_TIME));
    }

    #[test]
    #[should_panic]
    fn get_set_expiration_time_frozen_panics() {
        make_transaction().expiration_time(TEST_EXPIRATION_TIME);
    }

    #[test]
    fn get_set_admin_key() {
        let mut tx = TopicUpdateTransaction::new();
        tx.admin_key(unused_private_key().public_key());

        assert_eq!(tx.get_admin_key(), Some(&unused_private_key().public_key().into()));
    }

    #[test]
    #[should_panic]
    fn get_set_admin_key_frozen_panics() {
        make_transaction().admin_key(unused_private_key().public_key());
    }

    #[test]
    fn clear_admin_key() {
        let mut tx = TopicUpdateTransaction::new();
        tx.admin_key(unused_private_key().public_key());
        tx.clear_admin_key();

        assert_eq!(tx.get_admin_key(), Some(&Key::KeyList(crate::KeyList::new())));
    }

    #[test]
    #[should_panic]
    fn clear_admin_key_frozen_panics() {
        make_transaction().clear_admin_key();
    }

    #[test]
    fn get_set_submit_key() {
        let mut tx = TopicUpdateTransaction::new();
        tx.submit_key(unused_private_key().public_key());

        assert_eq!(tx.get_submit_key(), Some(&unused_private_key().public_key().into()));
    }

    #[test]
    #[should_panic]
    fn get_set_submit_key_frozen_panics() {
        make_transaction().submit_key(unused_private_key().public_key());
    }

    #[test]
    fn clear_submit_key() {
        let mut tx = TopicUpdateTransaction::new();
        tx.submit_key(unused_private_key().public_key());
        tx.clear_submit_key();

        assert_eq!(tx.get_submit_key(), Some(&Key::KeyList(crate::KeyList::new())));
    }

    #[test]
    #[should_panic]
    fn clear_submit_key_frozen_panics() {
        make_transaction().clear_submit_key();
    }

    #[test]
    fn get_set_auto_renew_period() {
        let mut tx = TopicUpdateTransaction::new();
        tx.auto_renew_period(TEST_AUTO_RENEW_PERIOD);

        assert_eq!(tx.get_auto_renew_period(), Some(TEST_AUTO_RENEW_PERIOD));
    }

    #[test]
    #[should_panic]
    fn get_set_auto_renew_period_frozen_panics() {
        make_transaction().auto_renew_period(TEST_AUTO_RENEW_PERIOD);
    }

    #[test]
    fn get_set_auto_renew_account_id() {
        let mut tx = TopicUpdateTransaction::new();
        tx.auto_renew_account_id(TEST_AUTO_RENEW_ACCOUNT_ID);

        assert_eq!(tx.get_auto_renew_account_id(), Some(TEST_AUTO_RENEW_ACCOUNT_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_auto_renew_account_id_frozen_panics() {
        make_transaction().auto_renew_account_id(TEST_AUTO_RENEW_ACCOUNT_ID);
    }

    #[test]
    fn clear_auto_renew_account_id() {
        let mut tx = TopicUpdateTransaction::new();
        tx.auto_renew_account_id(TEST_AUTO_RENEW_ACCOUNT_ID);
        tx.clear_auto_renew_account_id();

        assert_eq!(tx.get_auto_renew_account_id(), Some(AccountId::new(0, 0, 0)));
    }

    #[test]
    #[should_panic]
    fn clear_auto_renew_account_id_frozen_panics() {
        make_transaction().clear_auto_renew_account_id();
    }

    #[test]
    fn get_set_fee_schedule_key() {
        let fee_schedule_key = PrivateKey::generate_ecdsa();
        let mut tx = TopicUpdateTransaction::new();
        tx.fee_schedule_key(fee_schedule_key.public_key());

        assert_eq!(tx.get_fee_schedule_key(), Some(&fee_schedule_key.public_key().into()));
    }

    #[test]
    fn get_set_fee_exempt_keys() {
        let fee_exempt_keys = vec![PrivateKey::generate_ecdsa(), PrivateKey::generate_ecdsa()];
        let mut tx = TopicUpdateTransaction::new();
        tx.fee_exempt_keys(fee_exempt_keys.iter().map(|key| key.public_key().into()).collect());

        let expected_keys =
            fee_exempt_keys.iter().map(|key| key.public_key().into()).collect::<Vec<_>>();

        assert_eq!(tx.get_fee_exempt_keys(), &expected_keys);
    }

    #[test]
    fn add_fee_exempt_key_to_empty_list() {
        let mut tx = TopicUpdateTransaction::new();
        let fee_exempt_key = PrivateKey::generate_ecdsa();
        tx.add_fee_exempt_key(fee_exempt_key.public_key().into());

        assert_eq!(tx.get_fee_exempt_keys(), &vec![fee_exempt_key.public_key().into()]);
    }

    #[test]
    fn add_fee_exempt_key_to_list() {
        let fee_exempt_key = PrivateKey::generate_ecdsa();
        let mut tx = TopicUpdateTransaction::new();
        tx.fee_exempt_keys(vec![fee_exempt_key.public_key().into()]);

        let fee_exempt_key_to_add = PrivateKey::generate_ecdsa();
        tx.add_fee_exempt_key(fee_exempt_key_to_add.public_key().into());

        let expected_keys =
            vec![fee_exempt_key.public_key().into(), fee_exempt_key_to_add.public_key().into()];

        assert_eq!(tx.get_fee_exempt_keys(), &expected_keys);
    }

    #[test]
    fn clear_fee_exempt_keys() {
        let fee_exempt_key = PrivateKey::generate_ecdsa();
        let mut tx = TopicUpdateTransaction::new();
        tx.fee_exempt_keys(vec![fee_exempt_key.public_key().into()]);
        tx.clear_fee_exempt_keys();

        assert_eq!(tx.get_fee_exempt_keys(), &vec![]);
    }

    #[test]
    fn get_set_custom_fees() {
        let custom_fees = vec![
            CustomFixedFee::new(1, Some(TokenId::new(0, 0, 0)), None),
            CustomFixedFee::new(2, Some(TokenId::new(0, 0, 1)), None),
            CustomFixedFee::new(3, Some(TokenId::new(0, 0, 2)), None),
        ];

        let mut tx = TopicUpdateTransaction::new();
        tx.custom_fees(custom_fees.clone());

        assert_eq!(tx.get_custom_fees(), Some(&custom_fees));
    }

    #[test]
    fn add_custom_fee_to_list() {
        let custom_fees = vec![
            CustomFixedFee::new(1, Some(TokenId::new(0, 0, 0)), None),
            CustomFixedFee::new(2, Some(TokenId::new(0, 0, 1)), None),
            CustomFixedFee::new(3, Some(TokenId::new(0, 0, 2)), None),
        ];

        let custom_fee_to_add = CustomFixedFee::new(4, Some(TokenId::new(0, 0, 3)), None);

        let mut tx = TopicUpdateTransaction::new();
        tx.custom_fees(custom_fees);
        tx.add_custom_fee(custom_fee_to_add.clone());

        assert_eq!(tx.get_custom_fees(), Some(&vec![custom_fee_to_add]));
    }

    #[test]
    fn add_custom_fee_to_empty_list() {
        let custom_fee_to_add = CustomFixedFee::new(4, Some(TokenId::new(0, 0, 3)), None);

        let mut tx = TopicUpdateTransaction::new();
        tx.add_custom_fee(custom_fee_to_add.clone());

        assert_eq!(tx.get_custom_fees(), Some(&vec![custom_fee_to_add]));
    }

    #[test]
    fn clear_custom_fees() {
        let custom_fees = vec![
            CustomFixedFee::new(1, Some(TokenId::new(0, 0, 0)), None),
            CustomFixedFee::new(2, Some(TokenId::new(0, 0, 1)), None),
            CustomFixedFee::new(3, Some(TokenId::new(0, 0, 2)), None),
        ];

        let mut tx = TopicUpdateTransaction::new();
        tx.custom_fees(custom_fees);
        tx.clear_custom_fees();

        assert_eq!(tx.get_custom_fees(), None);
    }
}
