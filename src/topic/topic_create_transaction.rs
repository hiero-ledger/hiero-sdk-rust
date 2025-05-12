// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::consensus_service_client::ConsensusServiceClient;
use time::Duration;
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
    Hbar,
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

    /// The key that can be used to update the custom fees for this topic.
    fee_schedule_key: Option<Key>,

    /// If the transaction contains a signer from this list, no custom fees are applied.
    fee_exempt_keys: Vec<Key>,

    /// The custom fee to be assessed during a message submission to this topic. Empty if no custom fees are applied.
    custom_fees: Vec<CustomFixedFee>,
}

impl Default for TopicCreateTransactionData {
    fn default() -> Self {
        Self {
            topic_memo: String::new(),
            admin_key: None,
            submit_key: None,
            auto_renew_period: Some(Duration::days(90)),
            auto_renew_account_id: None,
            fee_schedule_key: None,
            fee_exempt_keys: vec![],
            custom_fees: vec![],
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

    /// Sets the key that can be used to update the fee schedule for the topic.
    pub fn fee_schedule_key(&mut self, key: impl Into<Key>) -> &mut Self {
        self.data_mut().fee_schedule_key = Some(key.into());
        self
    }

    /// The keys that can be used to update the fee schedule for the topic.
    #[must_use]
    pub fn get_fee_schedule_key(&self) -> Option<&Key> {
        self.data().fee_schedule_key.as_ref()
    }

    /// Sets the keys that can be used to update the fee schedule for the topic.
    pub fn fee_exempt_keys(&mut self, keys: Vec<Key>) -> &mut Self {
        self.data_mut().fee_exempt_keys = keys;
        self
    }

    /// The keys exempt from custom fees for this topic.
    #[must_use]
    pub fn get_fee_exempt_keys(&self) -> &Vec<Key> {
        &self.data().fee_exempt_keys
    }

    /// Clears the keys exempt from custom fees for this topic.
    pub fn clear_fee_exempt_keys(&mut self) -> &mut Self {
        self.data_mut().fee_exempt_keys.clear();
        self
    }

    /// Adds a key to the list of keys exempt from custom fees for this topic.
    pub fn add_fee_exempt_key(&mut self, key: impl Into<Key>) -> &mut Self {
        self.data_mut().fee_exempt_keys.push(key.into());
        self
    }

    /// The custom fees to be assessed during a message submission to this topic.
    #[must_use]
    pub fn get_custom_fees(&self) -> &Vec<CustomFixedFee> {
        &self.data().custom_fees
    }

    /// Sets the custom fees to be assessed during a message submission to this topic.
    pub fn custom_fees(&mut self, fees: Vec<CustomFixedFee>) -> &mut Self {
        self.data_mut().custom_fees = fees;
        self
    }

    /// Clears the custom fees for this topic.
    pub fn clear_custom_fees(&mut self) -> &mut Self {
        self.data_mut().custom_fees.clear();
        self
    }

    /// Adds a custom fee to the list of custom fees for this topic.
    pub fn add_custom_fee(&mut self, fee: CustomFixedFee) -> &mut Self {
        self.data_mut().custom_fees.push(fee);
        self
    }
}

impl TransactionData for TopicCreateTransactionData {
    fn default_max_transaction_fee(&self) -> Hbar {
        Hbar::new(25)
    }
}

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

        services::transaction_body::Data::ConsensusCreateTopic(self.to_protobuf())
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
        let custom_fees = pb
            .custom_fees
            .into_iter()
            .map(CustomFixedFee::from_protobuf)
            .collect::<Result<Vec<_>, _>>()?;

        let fee_exempt_keys = pb
            .fee_exempt_key_list
            .into_iter()
            .map(Key::from_protobuf)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
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

impl ToProtobuf for TopicCreateTransactionData {
    type Protobuf = services::ConsensusCreateTopicTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        let custom_fees = self.custom_fees.iter().map(|fee| fee.to_protobuf()).collect::<Vec<_>>();
        let fee_exempt_key_list =
            self.fee_exempt_keys.iter().map(|key| key.to_protobuf()).collect::<Vec<_>>();
        let fee_schedule_key = self.fee_schedule_key.as_ref().map(|key| key.to_protobuf());

        services::ConsensusCreateTopicTransactionBody {
            auto_renew_account: self.auto_renew_account_id.to_protobuf(),
            memo: self.topic_memo.clone(),
            admin_key: self.admin_key.to_protobuf(),
            submit_key: self.submit_key.to_protobuf(),
            auto_renew_period: self.auto_renew_period.to_protobuf(),
            custom_fees,
            fee_exempt_key_list,
            fee_schedule_key,
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hedera_proto::services;
    use time::Duration;

    use super::TopicCreateTransactionData;
    use crate::custom_fixed_fee::CustomFixedFee;
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
        PrivateKey,
        PublicKey,
        TokenId,
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
                    fee_schedule_key: None,
                    fee_exempt_key_list: [],
                    custom_fees: [],
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
            custom_fees: vec![],
            fee_exempt_key_list: vec![],
            fee_schedule_key: None,
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

    #[test]
    fn get_set_fee_schedule_key() {
        let mut tx = TopicCreateTransaction::new();
        tx.fee_schedule_key(key());

        assert_eq!(tx.get_fee_schedule_key(), Some(&key().into()));
    }

    #[test]
    #[should_panic]
    fn get_set_fee_schedule_key_frozen_panics() {
        make_transaction().fee_schedule_key(key());
    }

    #[test]
    fn get_set_fee_exempt_keys() {
        let keys = vec![PrivateKey::generate_ecdsa(), PrivateKey::generate_ecdsa()];
        let mut tx = TopicCreateTransaction::new();
        tx.fee_exempt_keys(keys.iter().map(|key| key.public_key().into()).collect());

        assert_eq!(
            tx.get_fee_exempt_keys(),
            &keys.iter().map(|key| key.public_key().into()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn get_set_custom_fees() {
        let mut tx = TopicCreateTransaction::new();
        tx.custom_fees(vec![
            CustomFixedFee::new(100, Some(TokenId::new(1, 2, 3)), Some(AccountId::new(4, 5, 6))),
            CustomFixedFee::new(200, None, None),
        ]);

        assert_eq!(
            tx.get_custom_fees(),
            &vec![
                CustomFixedFee::new(
                    100,
                    Some(TokenId::new(1, 2, 3)),
                    Some(AccountId::new(4, 5, 6))
                ),
                CustomFixedFee::new(200, None, None)
            ]
        );
    }

    #[test]
    fn add_topic_custom_fee_to_list() {
        let custom_fixed_fees = vec![
            CustomFixedFee::new(1, Some(TokenId::new(0, 0, 0)), None),
            CustomFixedFee::new(2, Some(TokenId::new(0, 0, 1)), None),
            CustomFixedFee::new(3, Some(TokenId::new(0, 0, 2)), None),
        ];

        let custom_fee_to_add = CustomFixedFee::new(4, Some(TokenId::new(0, 0, 3)), None);

        let mut expected_custom_fees = custom_fixed_fees.clone();
        expected_custom_fees.push(custom_fee_to_add.clone());

        let mut tx = TopicCreateTransaction::new();
        tx.custom_fees(custom_fixed_fees);
        tx.add_custom_fee(custom_fee_to_add);

        assert_eq!(tx.get_custom_fees().len(), expected_custom_fees.len());
        assert_eq!(tx.get_custom_fees(), &expected_custom_fees);
    }

    #[test]
    fn add_topic_custom_fee_to_empty_list() {
        let custom_fee_to_add = CustomFixedFee::new(4, Some(TokenId::new(0, 0, 3)), None);

        let mut tx = TopicCreateTransaction::new();
        tx.add_custom_fee(custom_fee_to_add.clone());

        assert_eq!(tx.get_custom_fees().len(), 1);
        assert_eq!(tx.get_custom_fees(), &vec![custom_fee_to_add]);
    }

    #[test]
    fn add_fee_exempt_key_to_empty_list() {
        let mut tx = TopicCreateTransaction::new();

        let fee_exempt_key = PrivateKey::generate_ecdsa();
        tx.add_fee_exempt_key(fee_exempt_key.public_key());

        assert_eq!(tx.get_fee_exempt_keys().len(), 1);
        assert_eq!(tx.get_fee_exempt_keys(), &vec![fee_exempt_key.public_key().into()]);
    }

    #[test]
    fn add_fee_exempt_key_to_list() {
        let fee_exempt_key = PrivateKey::generate_ecdsa();
        let mut tx = TopicCreateTransaction::new();
        tx.fee_exempt_keys(vec![fee_exempt_key.public_key().into()]);

        let fee_exempt_key_to_add = PrivateKey::generate_ecdsa();
        tx.add_fee_exempt_key(fee_exempt_key_to_add.public_key());

        let expected_keys =
            vec![fee_exempt_key.public_key().into(), fee_exempt_key_to_add.public_key().into()];

        assert_eq!(tx.get_fee_exempt_keys().len(), 2);
        assert_eq!(tx.get_fee_exempt_keys(), &expected_keys);
    }
}
