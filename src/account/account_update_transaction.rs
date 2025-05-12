// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::crypto_service_client::CryptoServiceClient;
use time::{
    Duration,
    OffsetDateTime,
};
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};
use crate::staked_id::StakedId;
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

/// Change properties for the given account.
///
/// Any null field is ignored (left unchanged). This
/// transaction must be signed by the existing key for this account. If
/// the transaction is changing the key field, then the transaction must be
/// signed by both the old key (from before the change) and the new key.
///
pub type AccountUpdateTransaction = Transaction<AccountUpdateTransactionData>;

// TODO: shard_id: Option<ShardId>
// TODO: realm_id: Option<RealmId>
// TODO: new_realm_admin_key: Option<Key>,

#[derive(Debug, Clone, Default)]
pub struct AccountUpdateTransactionData {
    /// The account ID which is being updated in this transaction.
    account_id: Option<AccountId>,

    /// The new key.
    key: Option<Key>,

    /// If true, this account's key must sign any transaction depositing into this account.
    receiver_signature_required: Option<bool>,

    /// The account is charged to extend its expiration date every this many seconds.
    auto_renew_period: Option<Duration>,

    auto_renew_account_id: Option<AccountId>,

    /// The ID of the account to which this account is proxy staked.
    ///
    /// If `proxy_account_id` is `None`, or is an invalid account, or is an account
    /// that isn't a node, then this account is automatically proxy staked to
    /// a node chosen by the network, but without earning payments.
    ///
    /// If the `proxy_account_id` account refuses to accept proxy staking, or
    /// if it is not currently running a node, then it
    /// will behave as if `proxy_account_id` was `None`.
    #[deprecated]
    proxy_account_id: Option<AccountId>,

    /// The new expiration time to extend to (ignored if equal to or before the current one).
    expiration_time: Option<OffsetDateTime>,

    /// The memo associated with the account.
    account_memo: Option<String>,

    /// The maximum number of tokens that an Account can be implicitly associated with.
    ///
    /// Defaults to `0`. Allows up to a maximum value of `1000`.
    /// If the value is set to `-1`, unlimited automatic token associations are allowed.
    max_automatic_token_associations: Option<i32>,

    /// ID of the account or node to which this account is staking, if any.
    staked_id: Option<StakedId>,

    /// If true, the account declines receiving a staking reward. The default value is false.
    decline_staking_reward: Option<bool>,
}

impl AccountUpdateTransaction {
    /// Returns the ID for the account that is being updated.
    #[must_use]
    pub fn get_account_id(&self) -> Option<AccountId> {
        self.data().account_id
    }

    /// Sets the ID for the account that is being updated.
    pub fn account_id(&mut self, id: AccountId) -> &mut Self {
        self.data_mut().account_id = Some(id);
        self
    }

    /// Gets the new expiration time to extend to (ignored if equal to or before the current one).
    #[must_use]
    pub fn get_expiration_time(&self) -> Option<OffsetDateTime> {
        self.data().expiration_time
    }

    /// Sets the new expiration time to extend to (ignored if equal to or before the current one).
    pub fn expiration_time(&mut self, at: OffsetDateTime) -> &mut Self {
        self.data_mut().expiration_time = Some(at);
        self
    }

    /// Returns the key that the account will be updated to.
    #[must_use]
    pub fn get_key(&self) -> Option<&Key> {
        self.data().key.as_ref()
    }

    /// Sets the key for this account.
    pub fn key(&mut self, key: impl Into<Key>) -> &mut Self {
        self.data_mut().key = Some(key.into());
        self
    }

    /// If true, this account's key must sign any transaction depositing hbar into this account.
    #[must_use]
    pub fn get_receiver_signature_required(&self) -> Option<bool> {
        self.data().receiver_signature_required
    }

    /// Set to true to require this account to sign any transfer of hbars to this account.
    pub fn receiver_signature_required(&mut self, required: bool) -> &mut Self {
        self.data_mut().receiver_signature_required = Some(required);
        self
    }

    /// Gets the ID of the account to which this account will be updated to be proxy staked to.
    #[deprecated]
    #[allow(deprecated)]
    #[must_use]
    pub fn get_proxy_account_id(&self) -> Option<AccountId> {
        self.data().proxy_account_id
    }

    /// Sets the proxy account ID for this account.
    ///
    /// If `proxy_account_id` is `None`, or is an invalid account, or is an account
    /// that isn't a node, then this account is automatically proxy staked to
    /// a node chosen by the network, but without earning payments.
    ///
    /// If the `proxy_account_id` account refuses to accept proxy staking, or
    /// if it is not currently running a node, then it
    /// will behave as if `proxy_account_id` was `None`.
    #[deprecated]
    #[allow(deprecated)]
    pub fn proxy_account_id(&mut self, proxy_account_id: AccountId) -> &mut Self {
        self.data_mut().proxy_account_id = Some(proxy_account_id);
        self
    }

    /// Returns the new auto renew period.
    #[must_use]
    pub fn get_auto_renew_period(&self) -> Option<Duration> {
        self.data().auto_renew_period
    }

    /// Sets the auto renew period for this account.
    pub fn auto_renew_period(&mut self, period: Duration) -> &mut Self {
        self.data_mut().auto_renew_period = Some(period);
        self
    }

    /// Returns the new auto renew account id.
    ///
    /// # Network Support
    /// Please note that this not supported on any hedera network at this time.
    #[must_use]
    pub fn get_auto_renew_account_id(&self) -> Option<AccountId> {
        self.data().auto_renew_account_id
    }

    /// Sets the account to be used at this account's expiration time to extend the
    /// life of the account.  If `None`, this account pays for its own auto renewal fee.
    ///
    /// # Network Support
    /// Please note that this not supported on any hedera network at this time.
    pub fn auto_renew_account_id(&mut self, id: AccountId) -> &mut Self {
        self.data_mut().auto_renew_account_id = Some(id);
        self
    }

    /// Returns the memo associated with the account.
    #[must_use]
    pub fn get_account_memo(&self) -> Option<&str> {
        self.data().account_memo.as_deref()
    }

    /// Sets the memo associated with the account.
    pub fn account_memo(&mut self, memo: impl Into<String>) -> &mut Self {
        self.data_mut().account_memo = Some(memo.into());
        self
    }

    /// Returns the maximum number of tokens that an Account can be implicitly associated with.
    #[must_use]
    pub fn get_max_automatic_token_associations(&self) -> Option<i32> {
        self.data().max_automatic_token_associations
    }

    /// Sets the maximum number of tokens that an Account can be implicitly associated with.
    ///
    pub fn max_automatic_token_associations(&mut self, amount: i32) -> &mut Self {
        self.data_mut().max_automatic_token_associations = Some(amount);
        self
    }

    /// Returns the ID of the account to which this account is staking.
    /// This is mutually exclusive with `staked_node_id`.
    #[must_use]
    pub fn get_staked_account_id(&self) -> Option<AccountId> {
        self.data().staked_id.and_then(StakedId::to_account_id)
    }

    /// Sets the ID of the account to which this account is staking.
    /// This is mutually exclusive with `staked_node_id`.
    pub fn staked_account_id(&mut self, id: AccountId) -> &mut Self {
        self.data_mut().staked_id = Some(id.into());
        self
    }

    /// Clears the account's staked account ID.
    pub fn clear_staked_account_id(&mut self) -> &mut Self {
        self.staked_account_id(AccountId::from(0))
    }

    /// Returns the ID of the node to which this account is staking.
    /// This is mutually exclusive with `staked_account_id`.
    #[must_use]
    pub fn get_staked_node_id(&self) -> Option<u64> {
        self.data().staked_id.and_then(StakedId::to_node_id)
    }

    /// Sets the ID of the node to which this account is staking.
    /// This is mutually exclusive with `staked_account_id`.
    pub fn staked_node_id(&mut self, id: u64) -> &mut Self {
        self.data_mut().staked_id = Some(id.into());
        self
    }

    /// Clears the account's staked node ID.
    pub fn clear_staked_node_id(&mut self) -> &mut Self {
        self.staked_node_id(u64::MAX)
    }

    /// Returns `true` if this account should decline receiving a staking reward,
    /// `false` if it should _not_,
    /// and `None` if the value should remain unchanged.
    #[must_use]
    pub fn get_decline_staking_reward(&self) -> Option<bool> {
        self.data().decline_staking_reward
    }

    /// If set to true, the account declines receiving a staking reward. The default value is false.
    pub fn decline_staking_reward(&mut self, decline: bool) -> &mut Self {
        self.data_mut().decline_staking_reward = Some(decline);
        self
    }
}

impl TransactionData for AccountUpdateTransactionData {}

impl TransactionExecute for AccountUpdateTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { CryptoServiceClient::new(channel).update_account(request).await })
    }
}

impl ValidateChecksums for AccountUpdateTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.account_id.validate_checksums(ledger_id)?;
        self.staked_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for AccountUpdateTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::CryptoUpdateAccount(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for AccountUpdateTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::CryptoUpdateAccount(self.to_protobuf())
    }
}

impl From<AccountUpdateTransactionData> for AnyTransactionData {
    fn from(transaction: AccountUpdateTransactionData) -> Self {
        Self::AccountUpdate(transaction)
    }
}

impl FromProtobuf<services::CryptoUpdateTransactionBody> for AccountUpdateTransactionData {
    #[allow(deprecated)]
    fn from_protobuf(pb: services::CryptoUpdateTransactionBody) -> crate::Result<Self> {
        use services::crypto_update_transaction_body::ReceiverSigRequiredField;

        let receiver_signature_required = pb.receiver_sig_required_field.map(|it| match it {
            ReceiverSigRequiredField::ReceiverSigRequired(it)
            | ReceiverSigRequiredField::ReceiverSigRequiredWrapper(it) => it,
        });

        Ok(Self {
            account_id: Option::from_protobuf(pb.account_id_to_update)?,
            key: Option::from_protobuf(pb.key)?,
            receiver_signature_required,
            auto_renew_period: pb.auto_renew_period.map(Into::into),
            auto_renew_account_id: None,
            proxy_account_id: Option::from_protobuf(pb.proxy_account_id)?,
            expiration_time: pb.expiration_time.map(Into::into),
            account_memo: pb.memo,
            max_automatic_token_associations: pb.max_automatic_token_associations,
            staked_id: Option::from_protobuf(pb.staked_id)?,
            decline_staking_reward: pb.decline_reward,
        })
    }
}

impl ToProtobuf for AccountUpdateTransactionData {
    type Protobuf = services::CryptoUpdateTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        let account_id = self.account_id.to_protobuf();
        let key = self.key.to_protobuf();
        let auto_renew_period = self.auto_renew_period.to_protobuf();
        let expiration_time = self.expiration_time.to_protobuf();

        let receiver_signature_required = self.receiver_signature_required.map(|required| {
            services::crypto_update_transaction_body::ReceiverSigRequiredField::ReceiverSigRequiredWrapper(required)
        });

        let staked_id = self.staked_id.map(|id| match id {
            StakedId::NodeId(id) => {
                services::crypto_update_transaction_body::StakedId::StakedNodeId(id as i64)
            }
            StakedId::AccountId(id) => {
                services::crypto_update_transaction_body::StakedId::StakedAccountId(
                    id.to_protobuf(),
                )
            }
        });

        #[allow(deprecated)]
        services::CryptoUpdateTransactionBody {
            account_id_to_update: account_id,
            key,
            proxy_account_id: self.proxy_account_id.to_protobuf(),
            proxy_fraction: 0,
            auto_renew_period,
            expiration_time,
            memo: self.account_memo.clone(),
            max_automatic_token_associations: self.max_automatic_token_associations.map(Into::into),
            decline_reward: self.decline_staking_reward,
            send_record_threshold_field: None,
            receive_record_threshold_field: None,
            receiver_sig_required_field: receiver_signature_required,
            staked_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hedera_proto::services;
    use time::{
        Duration,
        OffsetDateTime,
    };

    use crate::account::AccountUpdateTransactionData;
    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::staked_id::StakedId;
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
        unused_private_key,
    };
    use crate::{
        AccountId,
        AccountUpdateTransaction,
        AnyTransaction,
        PublicKey,
    };

    fn key() -> PublicKey {
        unused_private_key().public_key()
    }

    const ACCOUNT_ID: AccountId = AccountId::new(0, 0, 2002);
    const PROXY_ACCOUNT_ID: AccountId = AccountId::new(0, 0, 1001);
    const AUTO_RENEW_PERIOD: Duration = Duration::hours(10);
    const EXPIRATION_TIME: OffsetDateTime = match OffsetDateTime::from_unix_timestamp(1554158543) {
        Ok(it) => it,
        Err(_) => panic!("Panic in `const` unwrap"),
    };

    const RECEIVER_SIGNATURE_REQUIRED: bool = false;
    const MAX_AUTOMATIC_TOKEN_ASSOCIATIONS: i32 = 100;
    const ACCOUNT_MEMO: &str = "Some memo";
    const STAKED_ACCOUNT_ID: AccountId = AccountId::new(0, 0, 3);
    const STAKED_NODE_ID: u64 = 4;

    #[allow(deprecated)]
    fn make_transaction() -> AccountUpdateTransaction {
        let mut tx = AccountUpdateTransaction::new_for_tests();

        tx.key(key())
            .account_id(ACCOUNT_ID)
            .proxy_account_id(PROXY_ACCOUNT_ID)
            .auto_renew_period(AUTO_RENEW_PERIOD)
            .expiration_time(EXPIRATION_TIME)
            .receiver_signature_required(RECEIVER_SIGNATURE_REQUIRED)
            .max_automatic_token_associations(MAX_AUTOMATIC_TOKEN_ASSOCIATIONS)
            .account_memo(ACCOUNT_MEMO)
            .staked_account_id(STAKED_ACCOUNT_ID)
            .freeze()
            .unwrap();

        return tx;
    }

    #[allow(deprecated)]
    fn make_transaction2() -> AccountUpdateTransaction {
        let mut tx = AccountUpdateTransaction::new_for_tests();

        tx.key(key())
            .account_id(ACCOUNT_ID)
            .proxy_account_id(PROXY_ACCOUNT_ID)
            .auto_renew_period(AUTO_RENEW_PERIOD)
            .expiration_time(EXPIRATION_TIME)
            .receiver_signature_required(RECEIVER_SIGNATURE_REQUIRED)
            .max_automatic_token_associations(MAX_AUTOMATIC_TOKEN_ASSOCIATIONS)
            .account_memo(ACCOUNT_MEMO)
            .staked_node_id(STAKED_NODE_ID)
            .freeze()
            .unwrap();

        return tx;
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            CryptoUpdateAccount(
                CryptoUpdateTransactionBody {
                    account_id_to_update: Some(
                        AccountId {
                            shard_num: 0,
                            realm_num: 0,
                            account: Some(
                                AccountNum(
                                    2002,
                                ),
                            ),
                        },
                    ),
                    key: Some(
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
                    proxy_account_id: Some(
                        AccountId {
                            shard_num: 0,
                            realm_num: 0,
                            account: Some(
                                AccountNum(
                                    1001,
                                ),
                            ),
                        },
                    ),
                    proxy_fraction: 0,
                    auto_renew_period: Some(
                        Duration {
                            seconds: 36000,
                        },
                    ),
                    expiration_time: Some(
                        Timestamp {
                            seconds: 1554158543,
                            nanos: 0,
                        },
                    ),
                    memo: Some(
                        "Some memo",
                    ),
                    max_automatic_token_associations: Some(
                        100,
                    ),
                    decline_reward: None,
                    send_record_threshold_field: None,
                    receive_record_threshold_field: None,
                    receiver_sig_required_field: Some(
                        ReceiverSigRequiredWrapper(
                            false,
                        ),
                    ),
                    staked_id: Some(
                        StakedAccountId(
                            AccountId {
                                shard_num: 0,
                                realm_num: 0,
                                account: Some(
                                    AccountNum(
                                        3,
                                    ),
                                ),
                            },
                        ),
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
    fn serialize2() {
        let tx = make_transaction2();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            CryptoUpdateAccount(
                CryptoUpdateTransactionBody {
                    account_id_to_update: Some(
                        AccountId {
                            shard_num: 0,
                            realm_num: 0,
                            account: Some(
                                AccountNum(
                                    2002,
                                ),
                            ),
                        },
                    ),
                    key: Some(
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
                    proxy_account_id: Some(
                        AccountId {
                            shard_num: 0,
                            realm_num: 0,
                            account: Some(
                                AccountNum(
                                    1001,
                                ),
                            ),
                        },
                    ),
                    proxy_fraction: 0,
                    auto_renew_period: Some(
                        Duration {
                            seconds: 36000,
                        },
                    ),
                    expiration_time: Some(
                        Timestamp {
                            seconds: 1554158543,
                            nanos: 0,
                        },
                    ),
                    memo: Some(
                        "Some memo",
                    ),
                    max_automatic_token_associations: Some(
                        100,
                    ),
                    decline_reward: None,
                    send_record_threshold_field: None,
                    receive_record_threshold_field: None,
                    receiver_sig_required_field: Some(
                        ReceiverSigRequiredWrapper(
                            false,
                        ),
                    ),
                    staked_id: Some(
                        StakedNodeId(
                            4,
                        ),
                    ),
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
    fn from_proto_body() {
        #[allow(deprecated)]
        let tx = services::CryptoUpdateTransactionBody {
            account_id_to_update: Some(ACCOUNT_ID.to_protobuf()),
            key: Some(key().to_protobuf()),
            proxy_account_id: Some(PROXY_ACCOUNT_ID.to_protobuf()),
            send_record_threshold_field: None,
            receive_record_threshold_field: None,
            receiver_sig_required_field: Some(services::crypto_update_transaction_body::ReceiverSigRequiredField::ReceiverSigRequiredWrapper(RECEIVER_SIGNATURE_REQUIRED)),
            auto_renew_period: Some(AUTO_RENEW_PERIOD.to_protobuf()),
            memo: Some(ACCOUNT_MEMO.to_owned()),
            max_automatic_token_associations: Some(MAX_AUTOMATIC_TOKEN_ASSOCIATIONS as i32),
            decline_reward: None,
            staked_id: Some(services::crypto_update_transaction_body::StakedId::StakedAccountId(
                STAKED_ACCOUNT_ID.to_protobuf(),
            )),
            proxy_fraction: 0,
            expiration_time: Some(EXPIRATION_TIME.to_protobuf()),
        };

        let tx = AccountUpdateTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(tx.account_id, Some(ACCOUNT_ID));
        #[allow(deprecated)]
        {
            assert_eq!(tx.proxy_account_id, Some(PROXY_ACCOUNT_ID));
        }
        assert_eq!(tx.auto_renew_period, Some(AUTO_RENEW_PERIOD));
        assert_eq!(tx.expiration_time, Some(EXPIRATION_TIME));
        assert_eq!(tx.receiver_signature_required, Some(RECEIVER_SIGNATURE_REQUIRED));
        assert_eq!(tx.max_automatic_token_associations, Some(MAX_AUTOMATIC_TOKEN_ASSOCIATIONS));
        assert_eq!(tx.account_memo, Some(ACCOUNT_MEMO.to_owned()));
        assert_eq!(tx.staked_id.and_then(StakedId::to_account_id), Some(STAKED_ACCOUNT_ID));
    }

    #[test]
    fn get_set_account_id() {
        let mut tx = AccountUpdateTransaction::new();
        tx.account_id(ACCOUNT_ID);

        assert_eq!(tx.get_account_id(), Some(ACCOUNT_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_account_id_frozen_panics() {
        let mut tx = make_transaction();
        tx.account_id(ACCOUNT_ID);
    }

    #[test]
    #[allow(deprecated)]
    fn get_set_proxy_account_id() {
        let mut tx = AccountUpdateTransaction::new();
        tx.proxy_account_id(PROXY_ACCOUNT_ID);

        assert_eq!(tx.get_proxy_account_id(), Some(PROXY_ACCOUNT_ID));
    }

    #[test]
    #[should_panic]
    #[allow(deprecated)]
    fn get_set_proxy_account_id_frozen_panics() {
        let mut tx = make_transaction();
        tx.proxy_account_id(PROXY_ACCOUNT_ID);
    }

    #[test]
    fn get_set_auto_renew_period() {
        let mut tx = AccountUpdateTransaction::new();
        tx.auto_renew_period(AUTO_RENEW_PERIOD);

        assert_eq!(tx.get_auto_renew_period(), Some(AUTO_RENEW_PERIOD));
    }

    #[test]
    #[should_panic]
    fn get_set_auto_renew_period_frozen_panics() {
        let mut tx = make_transaction();
        tx.auto_renew_period(AUTO_RENEW_PERIOD);
    }

    #[test]
    fn get_set_expiration_time() {
        let mut tx = AccountUpdateTransaction::new();
        tx.expiration_time(EXPIRATION_TIME);

        assert_eq!(tx.get_expiration_time(), Some(EXPIRATION_TIME));
    }

    #[test]
    #[should_panic]
    fn get_set_expiration_time_frozen_panics() {
        let mut tx = make_transaction();
        tx.expiration_time(EXPIRATION_TIME);
    }

    #[test]
    fn get_set_receiver_signature_required() {
        let mut tx = AccountUpdateTransaction::new();
        tx.receiver_signature_required(RECEIVER_SIGNATURE_REQUIRED);

        assert_eq!(tx.get_receiver_signature_required(), Some(RECEIVER_SIGNATURE_REQUIRED));
    }

    #[test]
    #[should_panic]
    fn get_set_receiver_signature_required_frozen_panics() {
        let mut tx = make_transaction();
        tx.receiver_signature_required(RECEIVER_SIGNATURE_REQUIRED);
    }

    #[test]
    fn get_set_max_automatic_token_associations() {
        let mut tx = AccountUpdateTransaction::new();
        tx.max_automatic_token_associations(MAX_AUTOMATIC_TOKEN_ASSOCIATIONS);

        assert_eq!(
            tx.get_max_automatic_token_associations(),
            Some(MAX_AUTOMATIC_TOKEN_ASSOCIATIONS)
        );
    }

    #[test]
    #[should_panic]
    fn get_set_max_automatic_token_associations_frozen_panics() {
        let mut tx = make_transaction();
        tx.max_automatic_token_associations(MAX_AUTOMATIC_TOKEN_ASSOCIATIONS);
    }

    #[test]
    fn get_set_account_memo() {
        let mut tx = AccountUpdateTransaction::new();
        tx.account_memo(ACCOUNT_MEMO);

        assert_eq!(tx.get_account_memo(), Some(ACCOUNT_MEMO));
    }

    #[test]
    #[should_panic]
    fn get_set_account_memo_frozen_panics() {
        let mut tx = make_transaction();
        tx.account_memo(ACCOUNT_MEMO);
    }

    #[test]
    fn get_set_staked_account_id() {
        let mut tx = AccountUpdateTransaction::new();
        tx.staked_account_id(STAKED_ACCOUNT_ID);

        assert_eq!(tx.get_staked_account_id(), Some(STAKED_ACCOUNT_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_staked_account_id_frozen_panics() {
        let mut tx = make_transaction();
        tx.staked_account_id(STAKED_ACCOUNT_ID);
    }

    #[test]
    fn get_set_staked_node_id() {
        let mut tx = AccountUpdateTransaction::new();
        tx.staked_node_id(STAKED_NODE_ID);

        assert_eq!(tx.get_staked_node_id(), Some(STAKED_NODE_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_staked_node_id_frozen_panics() {
        let mut tx = make_transaction();
        tx.staked_node_id(STAKED_NODE_ID);
    }
}
