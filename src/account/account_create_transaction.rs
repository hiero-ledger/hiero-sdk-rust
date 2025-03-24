// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::crypto_service_client::CryptoServiceClient;
use time::Duration;
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
    EvmAddress,
    Hbar,
    Key,
    Transaction,
    ValidateChecksums,
};

/// Create a new Hedera™ account.
pub type AccountCreateTransaction = Transaction<AccountCreateTransactionData>;

// TODO: shard_id: Option<ShardId>
// TODO: realm_id: Option<RealmId>
// TODO: new_realm_admin_key: Option<Key>,

#[derive(Debug, Clone)]
pub struct AccountCreateTransactionData {
    /// The key that must sign each transfer out of the account.
    ///
    /// If `receiver_signature_required` is true, then it must also sign any transfer
    /// into the account.
    key: Option<Key>,

    /// The initial number of Hbar to put into the account.
    initial_balance: Hbar,

    /// If true, this account's key must sign any transaction depositing into this account.
    receiver_signature_required: bool,

    /// The account is charged to extend its expiration date every this many seconds.
    auto_renew_period: Option<Duration>,

    /// The account to be used at this account's expiration time to extend the
    /// life of the account.  If `None`, this account pays for its own auto renewal fee.
    auto_renew_account_id: Option<AccountId>,

    /// The memo associated with the account.
    account_memo: String,

    /// The maximum number of tokens that an Account can be implicitly associated with.
    ///
    /// Defaults to `0`. Allows up to a maximum value of `1000`.
    /// If the value is set to `-1`, unlimited automatic token associations are allowed.
    max_automatic_token_associations: i32,

    // notably *not* a PublicKey.
    /// A 20-byte EVM address to be used as the account's alias.
    alias: Option<EvmAddress>,

    /// ID of the account or node to which this account is staking, if any.
    staked_id: Option<StakedId>,

    /// If true, the account declines receiving a staking reward. The default value is false.
    decline_staking_reward: bool,
}

impl Default for AccountCreateTransactionData {
    fn default() -> Self {
        Self {
            key: None,
            initial_balance: Hbar::ZERO,
            receiver_signature_required: false,
            auto_renew_period: Some(Duration::days(90)),
            auto_renew_account_id: None,
            account_memo: String::new(),
            max_automatic_token_associations: 0,
            alias: None,
            staked_id: None,
            decline_staking_reward: false,
        }
    }
}

impl AccountCreateTransaction {
    /// Get the key this account will be created with.
    ///
    /// Returns `Some(key)` if previously set, `None` otherwise.
    #[must_use]
    pub fn get_key(&self) -> Option<&Key> {
        self.data().key.as_ref()
    }

    /// Sets the key for this account.
    pub fn key(&mut self, key: impl Into<Key>) -> &mut Self {
        self.data_mut().key = Some(key.into());
        self
    }

    /// Get the balance that will be transferred to this account on creation.
    ///
    /// Returns `initial_balance` if previously set, `0` otherwise.
    #[must_use]
    pub fn get_initial_balance(&self) -> Hbar {
        self.data().initial_balance
    }

    /// Sets the balance that will be transferred to this account on creation.
    pub fn initial_balance(&mut self, balance: Hbar) -> &mut Self {
        self.data_mut().initial_balance = balance;
        self
    }

    /// Returns `true` if this account must sign any transfer of hbars _to_ itself.
    #[must_use]
    pub fn get_receiver_signature_required(&self) -> bool {
        self.data().receiver_signature_required
    }

    /// Sets to true to require this account to sign any transfer of hbars to this account.
    pub fn receiver_signature_required(&mut self, required: bool) -> &mut Self {
        self.data_mut().receiver_signature_required = required;
        self
    }

    /// Returns the auto renew period for this account.
    #[must_use]
    pub fn get_auto_renew_period(&self) -> Option<Duration> {
        self.data().auto_renew_period
    }

    /// Sets the auto renew period for this account.
    pub fn auto_renew_period(&mut self, period: Duration) -> &mut Self {
        self.data_mut().auto_renew_period = Some(period);
        self
    }

    /// Gets the account to be used at this account's expiration time to extend the
    /// life of the account.  If `None`, this account pays for its own auto renewal fee.
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

    /// Get the memo associated with the account
    #[must_use]
    pub fn get_account_memo(&self) -> &str {
        &self.data().account_memo
    }

    /// Sets the memo associated with the account.
    pub fn account_memo(&mut self, memo: impl Into<String>) -> &mut Self {
        self.data_mut().account_memo = memo.into();
        self
    }

    /// Get the maximum number of tokens that an Account can be implicitly associated with.
    ///
    /// Defaults to `0`. Allows up to a maximum value of `1000`.
    #[must_use]
    pub fn get_max_automatic_token_associations(&self) -> i32 {
        self.data().max_automatic_token_associations
    }

    /// Sets the maximum number of tokens that an Account can be implicitly associated with.
    pub fn max_automatic_token_associations(&mut self, amount: i32) -> &mut Self {
        self.data_mut().max_automatic_token_associations = amount;
        self
    }

    /// Returns the evm address the account will be created with as an alias.
    ///
    /// # Network Support
    /// Please note that this not currently supported on mainnet.
    #[must_use]
    pub fn get_alias(&self) -> Option<EvmAddress> {
        self.data().alias
    }

    /// Sets the evm address the account will be created with as an alias.
    ///
    /// The last 20 bytes of the keccak-256 hash of a `ECDSA_SECP256K1` primitive key.
    ///
    /// # Network Support
    /// Please note that this not currently supported on mainnet.
    pub fn alias(&mut self, alias: EvmAddress) -> &mut Self {
        self.data_mut().alias = Some(alias);
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
        self.data_mut().staked_id = Some(StakedId::AccountId(id));
        self
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
        self.data_mut().staked_id = Some(StakedId::NodeId(id));
        self
    }

    /// Returns `true` if the account should decline receiving staking rewards, `false` otherwise.
    #[must_use]
    pub fn get_decline_staking_reward(&self) -> bool {
        self.data().decline_staking_reward
    }

    /// If `true`, the account declines receiving a staking reward. The default value is false.
    pub fn decline_staking_reward(&mut self, decline: bool) -> &mut Self {
        self.data_mut().decline_staking_reward = decline;
        self
    }
}

impl TransactionData for AccountCreateTransactionData {}

impl TransactionExecute for AccountCreateTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { CryptoServiceClient::new(channel).create_account(request).await })
    }
}

impl ValidateChecksums for AccountCreateTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.staked_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for AccountCreateTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::CryptoCreateAccount(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for AccountCreateTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::CryptoCreateAccount(self.to_protobuf())
    }
}

impl From<AccountCreateTransactionData> for AnyTransactionData {
    fn from(transaction: AccountCreateTransactionData) -> Self {
        Self::AccountCreate(transaction)
    }
}

impl FromProtobuf<services::CryptoCreateTransactionBody> for AccountCreateTransactionData {
    fn from_protobuf(pb: services::CryptoCreateTransactionBody) -> crate::Result<Self> {
        let alias = (!pb.alias.is_empty()).then(|| EvmAddress::try_from(pb.alias)).transpose()?;

        Ok(Self {
            key: Option::from_protobuf(pb.key)?,
            initial_balance: Hbar::from_tinybars(pb.initial_balance as i64),
            receiver_signature_required: pb.receiver_sig_required,
            auto_renew_period: pb.auto_renew_period.map(Into::into),
            auto_renew_account_id: None,
            account_memo: pb.memo,
            max_automatic_token_associations: pb.max_automatic_token_associations,
            alias,
            staked_id: Option::from_protobuf(pb.staked_id)?,
            decline_staking_reward: pb.decline_reward,
        })
    }
}

impl ToProtobuf for AccountCreateTransactionData {
    type Protobuf = services::CryptoCreateTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        let key = self.key.to_protobuf();
        let auto_renew_period = self.auto_renew_period.to_protobuf();
        let staked_id = self.staked_id.map(|it| match it {
            StakedId::NodeId(id) => {
                services::crypto_create_transaction_body::StakedId::StakedNodeId(id as i64)
            }
            StakedId::AccountId(id) => {
                services::crypto_create_transaction_body::StakedId::StakedAccountId(
                    id.to_protobuf(),
                )
            }
        });

        #[allow(deprecated)]
        services::CryptoCreateTransactionBody {
            key,
            initial_balance: self.initial_balance.to_tinybars() as u64,
            proxy_account_id: None,
            send_record_threshold: i64::MAX as u64,
            receive_record_threshold: i64::MAX as u64,
            receiver_sig_required: self.receiver_signature_required,
            auto_renew_period,
            shard_id: None,
            realm_id: None,
            new_realm_admin_key: None,
            memo: self.account_memo.clone(),
            max_automatic_token_associations: i32::from(self.max_automatic_token_associations),
            alias: self.alias.map_or(vec![], |it| it.to_bytes().to_vec()),
            decline_reward: self.decline_staking_reward,
            staked_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hedera_proto::services;
    use hex_literal::hex;
    use time::Duration;

    use crate::account::AccountCreateTransactionData;
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
        AccountCreateTransaction,
        AccountId,
        AnyTransaction,
        EvmAddress,
        Hbar,
        PublicKey,
    };

    fn key() -> PublicKey {
        unused_private_key().public_key()
    }

    const INITIAL_BALANCE: Hbar = Hbar::from_tinybars(450);
    const ACCOUNT_MEMO: &str = "some memo";
    const RECEIVER_SIGNATURE_REQUIRED: bool = true;
    const AUTO_RENEW_PERIOD: Duration = Duration::hours(10);
    const STAKED_ACCOUNT_ID: AccountId = AccountId::new(0, 0, 3);
    const STAKED_NODE_ID: u64 = 4;
    const ALIAS: EvmAddress = EvmAddress(hex!("5c562e90feaf0eebd33ea75d21024f249d451417"));
    const MAX_AUTOMATIC_TOKEN_ASSOCIATIONS: i32 = 100;

    fn make_transaction() -> AccountCreateTransaction {
        let mut tx = AccountCreateTransaction::new_for_tests();

        tx.key(key())
            .initial_balance(INITIAL_BALANCE)
            .account_memo(ACCOUNT_MEMO)
            .receiver_signature_required(RECEIVER_SIGNATURE_REQUIRED)
            .auto_renew_period(AUTO_RENEW_PERIOD)
            .staked_account_id(STAKED_ACCOUNT_ID)
            .alias(ALIAS)
            .max_automatic_token_associations(MAX_AUTOMATIC_TOKEN_ASSOCIATIONS)
            .freeze()
            .unwrap();

        return tx;
    }

    fn make_transaction2() -> AccountCreateTransaction {
        let mut tx = AccountCreateTransaction::new_for_tests();

        tx.key(key())
            .initial_balance(INITIAL_BALANCE)
            .account_memo(ACCOUNT_MEMO)
            .receiver_signature_required(RECEIVER_SIGNATURE_REQUIRED)
            .auto_renew_period(AUTO_RENEW_PERIOD)
            .staked_node_id(STAKED_NODE_ID)
            .alias(ALIAS)
            .max_automatic_token_associations(MAX_AUTOMATIC_TOKEN_ASSOCIATIONS)
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
            CryptoCreateAccount(
                CryptoCreateTransactionBody {
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
                    initial_balance: 450,
                    proxy_account_id: None,
                    send_record_threshold: 9223372036854775807,
                    receive_record_threshold: 9223372036854775807,
                    receiver_sig_required: true,
                    auto_renew_period: Some(
                        Duration {
                            seconds: 36000,
                        },
                    ),
                    shard_id: None,
                    realm_id: None,
                    new_realm_admin_key: None,
                    memo: "some memo",
                    max_automatic_token_associations: 100,
                    decline_reward: false,
                    alias: [
                        92,
                        86,
                        46,
                        144,
                        254,
                        175,
                        14,
                        235,
                        211,
                        62,
                        167,
                        93,
                        33,
                        2,
                        79,
                        36,
                        157,
                        69,
                        20,
                        23,
                    ],
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
            CryptoCreateAccount(
                CryptoCreateTransactionBody {
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
                    initial_balance: 450,
                    proxy_account_id: None,
                    send_record_threshold: 9223372036854775807,
                    receive_record_threshold: 9223372036854775807,
                    receiver_sig_required: true,
                    auto_renew_period: Some(
                        Duration {
                            seconds: 36000,
                        },
                    ),
                    shard_id: None,
                    realm_id: None,
                    new_realm_admin_key: None,
                    memo: "some memo",
                    max_automatic_token_associations: 100,
                    decline_reward: false,
                    alias: [
                        92,
                        86,
                        46,
                        144,
                        254,
                        175,
                        14,
                        235,
                        211,
                        62,
                        167,
                        93,
                        33,
                        2,
                        79,
                        36,
                        157,
                        69,
                        20,
                        23,
                    ],
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
        let tx = services::CryptoCreateTransactionBody {
            key: Some(key().to_protobuf()),
            initial_balance: INITIAL_BALANCE.to_tinybars() as u64,
            proxy_account_id: None,
            send_record_threshold: i64::MAX as u64,
            receive_record_threshold: i64::MAX as u64,
            receiver_sig_required: RECEIVER_SIGNATURE_REQUIRED,
            auto_renew_period: Some(AUTO_RENEW_PERIOD.to_protobuf()),
            shard_id: None,
            realm_id: None,
            new_realm_admin_key: None,
            memo: ACCOUNT_MEMO.to_owned(),
            max_automatic_token_associations: MAX_AUTOMATIC_TOKEN_ASSOCIATIONS,
            decline_reward: false,
            alias: ALIAS.to_bytes().to_vec(),
            staked_id: Some(services::crypto_create_transaction_body::StakedId::StakedAccountId(
                STAKED_ACCOUNT_ID.to_protobuf(),
            )),
        };

        let tx = AccountCreateTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(tx.key, Some(key().into()));
        assert_eq!(tx.initial_balance, INITIAL_BALANCE);
        assert_eq!(tx.account_memo, ACCOUNT_MEMO);
        assert_eq!(tx.receiver_signature_required, RECEIVER_SIGNATURE_REQUIRED);
        assert_eq!(tx.auto_renew_period, Some(AUTO_RENEW_PERIOD));
        assert_eq!(tx.staked_id.and_then(StakedId::to_account_id), Some(STAKED_ACCOUNT_ID));
        assert_eq!(tx.alias, Some(ALIAS));
        assert_eq!(tx.max_automatic_token_associations, MAX_AUTOMATIC_TOKEN_ASSOCIATIONS);
    }

    #[test]
    fn properties() {
        let tx = make_transaction();

        assert_eq!(tx.get_key(), Some(&key().into()));
        assert_eq!(tx.get_initial_balance(), INITIAL_BALANCE);
        assert_eq!(tx.get_account_memo(), ACCOUNT_MEMO);
        assert_eq!(tx.get_receiver_signature_required(), RECEIVER_SIGNATURE_REQUIRED);
        assert_eq!(tx.get_auto_renew_period(), Some(AUTO_RENEW_PERIOD));
        assert_eq!(tx.get_staked_account_id(), Some(STAKED_ACCOUNT_ID));
        assert_eq!(tx.get_alias(), Some(ALIAS));
        assert_eq!(tx.get_max_automatic_token_associations(), MAX_AUTOMATIC_TOKEN_ASSOCIATIONS);
    }

    #[test]
    fn get_set_key() {
        let mut tx = AccountCreateTransaction::new();
        tx.key(key());

        assert_eq!(tx.get_key(), Some(&key().into()));
    }

    #[test]
    #[should_panic]
    fn get_set_key_frozen_panics() {
        let mut tx = make_transaction();

        tx.key(key());
    }

    #[test]
    fn get_set_initial_balance() {
        let mut tx = AccountCreateTransaction::new();
        tx.initial_balance(INITIAL_BALANCE);

        assert_eq!(tx.get_initial_balance(), INITIAL_BALANCE);
    }

    #[test]
    #[should_panic]
    fn get_set_initial_balance_frozen_panics() {
        let mut tx = make_transaction();

        tx.initial_balance(INITIAL_BALANCE);
    }

    #[test]
    fn get_set_account_memo() {
        let mut tx = AccountCreateTransaction::new();
        tx.account_memo(ACCOUNT_MEMO);

        assert_eq!(tx.get_account_memo(), ACCOUNT_MEMO);
    }

    #[test]
    #[should_panic]
    fn get_set_account_memo_frozen_panics() {
        let mut tx = make_transaction();

        tx.account_memo(ACCOUNT_MEMO);
    }

    #[test]
    fn get_set_receiver_signature_required() {
        let mut tx = AccountCreateTransaction::new();
        tx.receiver_signature_required(RECEIVER_SIGNATURE_REQUIRED);

        assert_eq!(tx.get_receiver_signature_required(), RECEIVER_SIGNATURE_REQUIRED);
    }

    #[test]
    #[should_panic]
    fn get_set_receiver_signature_required_frozen_panics() {
        let mut tx = make_transaction();

        tx.receiver_signature_required(RECEIVER_SIGNATURE_REQUIRED);
    }

    #[test]
    fn get_set_auto_renew_period() {
        let mut tx = AccountCreateTransaction::new();
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
    fn get_set_staked_account_id() {
        let mut tx = AccountCreateTransaction::new();
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
    fn get_set_alias() {
        let mut tx = AccountCreateTransaction::new();
        tx.alias(ALIAS);

        assert_eq!(tx.get_alias(), Some(ALIAS));
    }

    #[test]
    #[should_panic]
    fn get_set_alias_frozen_panics() {
        let mut tx = make_transaction();

        tx.alias(ALIAS);
    }

    #[test]
    fn get_set_max_automatic_token_associations() {
        let mut tx = AccountCreateTransaction::new();
        tx.max_automatic_token_associations(MAX_AUTOMATIC_TOKEN_ASSOCIATIONS);

        assert_eq!(tx.get_max_automatic_token_associations(), MAX_AUTOMATIC_TOKEN_ASSOCIATIONS);
    }

    #[test]
    #[should_panic]
    fn get_set_max_automatic_token_associations_frozen_panics() {
        let mut tx = make_transaction();

        tx.max_automatic_token_associations(MAX_AUTOMATIC_TOKEN_ASSOCIATIONS);
    }
}
