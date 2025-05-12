// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::smart_contract_service_client::SmartContractServiceClient;
use time::{
    Duration,
    OffsetDateTime,
};
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::protobuf::FromProtobuf;
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
    ContractId,
    Error,
    Key,
    ToProtobuf,
    Transaction,
    ValidateChecksums,
};

/// Updates the fields of a smart contract to the given values.
pub type ContractUpdateTransaction = Transaction<ContractUpdateTransactionData>;

#[derive(Debug, Default, Clone)]
pub struct ContractUpdateTransactionData {
    contract_id: Option<ContractId>,

    expiration_time: Option<OffsetDateTime>,

    admin_key: Option<Key>,

    auto_renew_period: Option<Duration>,

    contract_memo: Option<String>,

    max_automatic_token_associations: Option<i32>,

    auto_renew_account_id: Option<AccountId>,

    proxy_account_id: Option<AccountId>,

    /// ID of the account or node to which this contract is staking, if any.
    staked_id: Option<StakedId>,

    decline_staking_reward: Option<bool>,
}

impl ContractUpdateTransaction {
    /// Returns the contract to be updated.
    #[must_use]
    pub fn get_contract_id(&self) -> Option<ContractId> {
        self.data().contract_id
    }

    /// Sets the contract to be updated.
    pub fn contract_id(&mut self, contract_id: ContractId) -> &mut Self {
        self.data_mut().contract_id = Some(contract_id);
        self
    }

    /// Returns the new admin key.
    #[must_use]
    pub fn get_admin_key(&self) -> Option<&Key> {
        self.data().admin_key.as_ref()
    }

    /// Sets the new admin key.
    pub fn admin_key(&mut self, key: impl Into<Key>) -> &mut Self {
        self.data_mut().admin_key = Some(key.into());
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

    /// Returns the auto renew period for this smart contract.
    #[must_use]
    pub fn get_auto_renew_period(&self) -> Option<Duration> {
        self.data().auto_renew_period
    }

    /// Sets the auto renew period for this smart contract.
    pub fn auto_renew_period(&mut self, period: Duration) -> &mut Self {
        self.data_mut().auto_renew_period = Some(period);
        self
    }

    /// Returns the new memo for the smart contract.
    #[must_use]
    pub fn get_contract_memo(&self) -> Option<&str> {
        self.data().contract_memo.as_deref()
    }

    /// Sets the new memo for the smart contract.
    pub fn contract_memo(&mut self, memo: impl Into<String>) -> &mut Self {
        self.data_mut().contract_memo = Some(memo.into());
        self
    }

    /// Returns the maximum number of tokens that this contract can be automatically associated with.
    #[must_use]
    pub fn get_max_automatic_token_associations(&self) -> Option<i32> {
        self.data().max_automatic_token_associations
    }

    /// Sets the maximum number of tokens that this contract can be automatically associated with.
    pub fn max_automatic_token_associations(&mut self, max: i32) -> &mut Self {
        self.data_mut().max_automatic_token_associations = Some(max);
        self
    }

    /// Returns the account to be used at the contract's expiration time to extend the
    /// life of the contract.
    #[must_use]
    pub fn get_auto_renew_account_id(&self) -> Option<AccountId> {
        self.data().auto_renew_account_id
    }

    /// Sets the account to be used at the contract's expiration time to extend the
    /// life of the contract.
    pub fn auto_renew_account_id(&mut self, account_id: AccountId) -> &mut Self {
        self.data_mut().auto_renew_account_id = Some(account_id);
        self
    }

    /// Returns the ID of the account to which this contract is proxy staked.
    #[must_use]
    pub fn get_proxy_account_id(&self) -> Option<AccountId> {
        self.data().proxy_account_id
    }

    /// Sets the ID of the account to which this contract is proxy staked.
    pub fn proxy_account_id(&mut self, id: AccountId) -> &mut Self {
        self.data_mut().proxy_account_id = Some(id);
        self
    }

    /// Returns the ID of the account to which this contract is staking.
    #[must_use]
    pub fn get_staked_account_id(&self) -> Option<AccountId> {
        self.data().staked_id.and_then(StakedId::to_account_id)
    }

    /// Sets the ID of the account to which this contract is staking.
    /// This is mutually exclusive with `staked_node_id`.
    pub fn staked_account_id(&mut self, id: AccountId) -> &mut Self {
        self.data_mut().staked_id = Some(id.into());
        self
    }

    /// Returns the ID of the node to which this contract is staking.
    #[must_use]
    pub fn get_staked_node_id(&self) -> Option<u64> {
        self.data().staked_id.and_then(StakedId::to_node_id)
    }

    /// Sets the ID of the node to which this contract is staking.
    /// This is mutually exclusive with `staked_account_id`.
    pub fn staked_node_id(&mut self, id: u64) -> &mut Self {
        self.data_mut().staked_id = Some(id.into());
        self
    }

    /// Returns `true` if the contract will be updated decline staking rewards,
    /// `false` if it will be updated to _not_,
    /// and `None` if it will not be updated.
    #[must_use]
    pub fn get_decline_staking_reward(&self) -> Option<bool> {
        self.data().decline_staking_reward
    }

    /// Sets to true, the contract declines receiving a staking reward. The default value is false.
    pub fn decline_staking_reward(&mut self, decline: bool) -> &mut Self {
        self.data_mut().decline_staking_reward = Some(decline);
        self
    }
}

impl TransactionData for ContractUpdateTransactionData {}

impl TransactionExecute for ContractUpdateTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { SmartContractServiceClient::new(channel).update_contract(request).await })
    }
}

impl ValidateChecksums for ContractUpdateTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.contract_id.validate_checksums(ledger_id)?;
        self.auto_renew_account_id.validate_checksums(ledger_id)?;
        self.staked_id.validate_checksums(ledger_id)?;
        self.proxy_account_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for ContractUpdateTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::ContractUpdateInstance(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for ContractUpdateTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::ContractUpdateInstance(self.to_protobuf())
    }
}

impl FromProtobuf<services::ContractUpdateTransactionBody> for ContractUpdateTransactionData {
    #[allow(deprecated)]
    fn from_protobuf(pb: services::ContractUpdateTransactionBody) -> crate::Result<Self> {
        use services::contract_update_transaction_body::MemoField;

        Ok(Self {
            contract_id: Option::from_protobuf(pb.contract_id)?,
            expiration_time: pb.expiration_time.map(Into::into),
            admin_key: Option::from_protobuf(pb.admin_key)?,
            auto_renew_period: pb.auto_renew_period.map(Into::into),
            contract_memo: pb.memo_field.map(|it| match it {
                MemoField::Memo(it) | MemoField::MemoWrapper(it) => it,
            }),
            max_automatic_token_associations: pb.max_automatic_token_associations,
            auto_renew_account_id: Option::from_protobuf(pb.auto_renew_account_id)?,
            proxy_account_id: Option::from_protobuf(pb.proxy_account_id)?,
            staked_id: Option::from_protobuf(pb.staked_id)?,
            decline_staking_reward: pb.decline_reward,
        })
    }
}

impl ToProtobuf for ContractUpdateTransactionData {
    type Protobuf = services::ContractUpdateTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        let contract_id = self.contract_id.to_protobuf();
        let expiration_time = self.expiration_time.map(Into::into);
        let admin_key = self.admin_key.to_protobuf();
        let auto_renew_period = self.auto_renew_period.map(Into::into);
        let auto_renew_account_id = self.auto_renew_account_id.to_protobuf();

        let staked_id = self.staked_id.map(|id| match id {
            StakedId::NodeId(id) => {
                services::contract_update_transaction_body::StakedId::StakedNodeId(id as i64)
            }

            StakedId::AccountId(id) => {
                services::contract_update_transaction_body::StakedId::StakedAccountId(
                    id.to_protobuf(),
                )
            }
        });

        let memo_field = self
            .contract_memo
            .clone()
            .map(services::contract_update_transaction_body::MemoField::MemoWrapper);

        #[allow(deprecated)]
        services::ContractUpdateTransactionBody {
            contract_id,
            expiration_time,
            admin_key,
            proxy_account_id: self.proxy_account_id.to_protobuf(),
            auto_renew_period,
            max_automatic_token_associations: self
                .max_automatic_token_associations
                .map(|max| max as i32),
            auto_renew_account_id,
            decline_reward: self.decline_staking_reward,
            staked_id,
            file_id: None,
            memo_field,
        }
    }
}

impl From<ContractUpdateTransactionData> for AnyTransactionData {
    fn from(transaction: ContractUpdateTransactionData) -> Self {
        Self::ContractUpdate(transaction)
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

    use crate::contract::ContractUpdateTransactionData;
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
        ContractId,
        ContractUpdateTransaction,
        PublicKey,
    };

    fn admin_key() -> PublicKey {
        unused_private_key().public_key()
    }

    const CONTRACT_ID: ContractId = ContractId::new(0, 0, 5007);

    const MAX_AUTOMATIC_TOKEN_ASSOCIATIONS: i32 = 101;
    const AUTO_RENEW_PERIOD: Duration = Duration::days(1);
    const CONTRACT_MEMO: &str = "3";
    const EXPIRATION_TIME: OffsetDateTime =
        match OffsetDateTime::from_unix_timestamp_nanos(4_000_000) {
            Ok(it) => it,
            Err(_) => panic!("Panic in `const` unwrap"),
        };
    const PROXY_ACCOUNT_ID: AccountId = AccountId::new(0, 0, 4);
    const AUTO_RENEW_ACCOUNT_ID: AccountId = AccountId::new(0, 0, 30);
    const STAKED_ACCOUNT_ID: AccountId = AccountId::new(0, 0, 3);
    const STAKED_NODE_ID: u64 = 4;

    fn make_transaction() -> ContractUpdateTransaction {
        let mut tx = ContractUpdateTransaction::new_for_tests();

        tx.contract_id(CONTRACT_ID)
            .admin_key(admin_key())
            .max_automatic_token_associations(MAX_AUTOMATIC_TOKEN_ASSOCIATIONS)
            .auto_renew_period(AUTO_RENEW_PERIOD)
            .contract_memo(CONTRACT_MEMO)
            .expiration_time(EXPIRATION_TIME)
            .proxy_account_id(PROXY_ACCOUNT_ID)
            .auto_renew_account_id(AUTO_RENEW_ACCOUNT_ID)
            .staked_account_id(STAKED_ACCOUNT_ID)
            .freeze()
            .unwrap();

        tx
    }

    fn make_transaction2() -> ContractUpdateTransaction {
        let mut tx = ContractUpdateTransaction::new_for_tests();

        tx.contract_id(CONTRACT_ID)
            .admin_key(admin_key())
            .max_automatic_token_associations(MAX_AUTOMATIC_TOKEN_ASSOCIATIONS)
            .auto_renew_period(AUTO_RENEW_PERIOD)
            .contract_memo(CONTRACT_MEMO)
            .expiration_time(EXPIRATION_TIME)
            .proxy_account_id(PROXY_ACCOUNT_ID)
            .auto_renew_account_id(AUTO_RENEW_ACCOUNT_ID)
            .staked_node_id(STAKED_NODE_ID)
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
            ContractUpdateInstance(
                ContractUpdateTransactionBody {
                    contract_id: Some(
                        ContractId {
                            shard_num: 0,
                            realm_num: 0,
                            contract: Some(
                                ContractNum(
                                    5007,
                                ),
                            ),
                        },
                    ),
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
                    proxy_account_id: Some(
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
                    auto_renew_period: Some(
                        Duration {
                            seconds: 86400,
                        },
                    ),
                    file_id: None,
                    max_automatic_token_associations: Some(
                        101,
                    ),
                    auto_renew_account_id: Some(
                        AccountId {
                            shard_num: 0,
                            realm_num: 0,
                            account: Some(
                                AccountNum(
                                    30,
                                ),
                            ),
                        },
                    ),
                    decline_reward: None,
                    memo_field: Some(
                        MemoWrapper(
                            "3",
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
            ContractUpdateInstance(
                ContractUpdateTransactionBody {
                    contract_id: Some(
                        ContractId {
                            shard_num: 0,
                            realm_num: 0,
                            contract: Some(
                                ContractNum(
                                    5007,
                                ),
                            ),
                        },
                    ),
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
                    proxy_account_id: Some(
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
                    auto_renew_period: Some(
                        Duration {
                            seconds: 86400,
                        },
                    ),
                    file_id: None,
                    max_automatic_token_associations: Some(
                        101,
                    ),
                    auto_renew_account_id: Some(
                        AccountId {
                            shard_num: 0,
                            realm_num: 0,
                            account: Some(
                                AccountNum(
                                    30,
                                ),
                            ),
                        },
                    ),
                    decline_reward: None,
                    memo_field: Some(
                        MemoWrapper(
                            "3",
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
        let tx = services::ContractUpdateTransactionBody {
            contract_id: Some(CONTRACT_ID.to_protobuf()),
            expiration_time: Some(EXPIRATION_TIME.to_protobuf()),
            admin_key: Some(admin_key().to_protobuf()),
            proxy_account_id: Some(PROXY_ACCOUNT_ID.to_protobuf()),
            auto_renew_period: Some(AUTO_RENEW_PERIOD.to_protobuf()),
            max_automatic_token_associations: Some(MAX_AUTOMATIC_TOKEN_ASSOCIATIONS),
            auto_renew_account_id: Some(AUTO_RENEW_ACCOUNT_ID.to_protobuf()),
            decline_reward: None,
            memo_field: Some(services::contract_update_transaction_body::MemoField::MemoWrapper(
                CONTRACT_MEMO.to_owned(),
            )),
            staked_id: Some(services::contract_update_transaction_body::StakedId::StakedAccountId(
                STAKED_ACCOUNT_ID.to_protobuf(),
            )),
            file_id: None,
        };

        let tx = ContractUpdateTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(tx.contract_id, Some(CONTRACT_ID));
        assert_eq!(tx.admin_key, Some(admin_key().into()));
        assert_eq!(tx.max_automatic_token_associations, Some(MAX_AUTOMATIC_TOKEN_ASSOCIATIONS));
        assert_eq!(tx.auto_renew_period, Some(AUTO_RENEW_PERIOD));
        assert_eq!(tx.contract_memo, Some(CONTRACT_MEMO.to_owned()));
        assert_eq!(tx.expiration_time, Some(EXPIRATION_TIME));
        assert_eq!(tx.proxy_account_id, Some(PROXY_ACCOUNT_ID));
        assert_eq!(tx.auto_renew_account_id, Some(AUTO_RENEW_ACCOUNT_ID));
        assert_eq!(tx.staked_id, Some(crate::staked_id::StakedId::AccountId(STAKED_ACCOUNT_ID)));
    }

    mod get_set {
        use super::*;

        #[test]
        fn contract_id() {
            let mut tx = ContractUpdateTransaction::new();
            tx.contract_id(CONTRACT_ID);

            assert_eq!(tx.get_contract_id(), Some(CONTRACT_ID));
        }

        #[test]
        #[should_panic]
        fn contract_id_frozen_panics() {
            make_transaction().contract_id(CONTRACT_ID);
        }

        #[test]
        fn admin_key() {
            let mut tx = ContractUpdateTransaction::new();
            tx.admin_key(super::admin_key());

            assert_eq!(tx.get_admin_key(), Some(&super::admin_key().into()));
        }

        #[test]
        #[should_panic]
        fn admin_key_frozen_panics() {
            make_transaction().admin_key(super::admin_key());
        }

        #[test]
        fn max_automatic_token_associations() {
            let mut tx = ContractUpdateTransaction::new();
            tx.max_automatic_token_associations(MAX_AUTOMATIC_TOKEN_ASSOCIATIONS);

            assert_eq!(
                tx.get_max_automatic_token_associations(),
                Some(MAX_AUTOMATIC_TOKEN_ASSOCIATIONS)
            );
        }

        #[test]
        #[should_panic]
        fn max_automatic_token_associations_frozen_panics() {
            make_transaction().max_automatic_token_associations(MAX_AUTOMATIC_TOKEN_ASSOCIATIONS);
        }

        #[test]
        fn auto_renew_period() {
            let mut tx = ContractUpdateTransaction::new();
            tx.auto_renew_period(AUTO_RENEW_PERIOD);

            assert_eq!(tx.get_auto_renew_period(), Some(AUTO_RENEW_PERIOD));
        }

        #[test]
        #[should_panic]
        fn auto_renew_period_frozen_panics() {
            make_transaction().auto_renew_period(AUTO_RENEW_PERIOD);
        }

        #[test]
        fn contract_memo() {
            let mut tx = ContractUpdateTransaction::new();
            tx.contract_memo(CONTRACT_MEMO);

            assert_eq!(tx.get_contract_memo(), Some(CONTRACT_MEMO));
        }

        #[test]
        #[should_panic]
        fn contract_memo_frozen_panics() {
            make_transaction().contract_memo(CONTRACT_MEMO);
        }

        #[test]
        fn expiration_time() {
            let mut tx = ContractUpdateTransaction::new();
            tx.expiration_time(EXPIRATION_TIME);

            assert_eq!(tx.get_expiration_time(), Some(EXPIRATION_TIME));
        }

        #[test]
        #[should_panic]
        fn expiration_time_frozen_panics() {
            make_transaction().expiration_time(EXPIRATION_TIME);
        }

        #[test]
        fn proxy_account_id() {
            let mut tx = ContractUpdateTransaction::new();
            tx.proxy_account_id(PROXY_ACCOUNT_ID);

            assert_eq!(tx.get_proxy_account_id(), Some(PROXY_ACCOUNT_ID));
        }

        #[test]
        #[should_panic]
        fn proxy_account_id_frozen_panics() {
            make_transaction().proxy_account_id(PROXY_ACCOUNT_ID);
        }

        #[test]
        fn auto_renew_account_id() {
            let mut tx = ContractUpdateTransaction::new();
            tx.auto_renew_account_id(AUTO_RENEW_ACCOUNT_ID);

            assert_eq!(tx.get_auto_renew_account_id(), Some(AUTO_RENEW_ACCOUNT_ID));
        }

        #[test]
        #[should_panic]
        fn auto_renew_account_id_frozen_panics() {
            make_transaction().auto_renew_account_id(AUTO_RENEW_ACCOUNT_ID);
        }

        #[test]
        fn staked_account_id() {
            let mut tx = ContractUpdateTransaction::new();
            tx.staked_account_id(STAKED_ACCOUNT_ID);

            assert_eq!(tx.get_staked_account_id(), Some(STAKED_ACCOUNT_ID));
        }

        #[test]
        #[should_panic]
        fn staked_account_id_frozen_panics() {
            make_transaction().staked_account_id(STAKED_ACCOUNT_ID);
        }

        #[test]
        fn staked_node_id() {
            let mut tx = ContractUpdateTransaction::new();
            tx.staked_node_id(STAKED_NODE_ID);

            assert_eq!(tx.get_staked_node_id(), Some(STAKED_NODE_ID));
        }

        #[test]
        #[should_panic]
        fn staked_node_id_frozen_panics() {
            make_transaction().staked_node_id(STAKED_NODE_ID);
        }
    }
}
