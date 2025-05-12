// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::smart_contract_service_client::SmartContractServiceClient;
use time::Duration;
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
    Error,
    FileId,
    Hbar,
    Key,
    ToProtobuf,
    Transaction,
    ValidateChecksums,
};

/// Start a new smart contract instance.
pub type ContractCreateTransaction = Transaction<ContractCreateTransactionData>;

#[derive(Debug, Clone)]
pub struct ContractCreateTransactionData {
    bytecode: Option<Vec<u8>>,

    bytecode_file_id: Option<FileId>,

    admin_key: Option<Key>,

    gas: u64,

    initial_balance: Hbar,

    auto_renew_period: Duration,

    constructor_parameters: Vec<u8>,

    contract_memo: String,

    max_automatic_token_associations: i32,

    auto_renew_account_id: Option<AccountId>,

    /// ID of the account or node to which this contract is staking, if any.
    staked_id: Option<StakedId>,

    decline_staking_reward: bool,
}

impl Default for ContractCreateTransactionData {
    fn default() -> Self {
        Self {
            bytecode: None,
            bytecode_file_id: None,
            admin_key: None,
            gas: 0,
            initial_balance: Hbar::ZERO,
            auto_renew_period: Duration::days(90),
            constructor_parameters: Vec::new(),
            contract_memo: String::new(),
            max_automatic_token_associations: 0,
            auto_renew_account_id: None,
            staked_id: None,
            decline_staking_reward: false,
        }
    }
}

impl ContractCreateTransaction {
    /// Returns the `FileId` to be used as the bytecode for this smart contract.
    #[must_use]
    pub fn get_bytecode_file_id(&self) -> Option<FileId> {
        self.data().bytecode_file_id
    }

    /// Sets the file to use as the bytes for the smart contract.
    pub fn bytecode_file_id(&mut self, file_id: FileId) -> &mut Self {
        self.data_mut().bytecode_file_id = Some(file_id);
        self
    }

    /// Returns the bytecode for the smart contract.
    #[must_use]
    pub fn get_bytecode(&self) -> Option<&[u8]> {
        self.data().bytecode.as_deref()
    }

    /// Sets the bytes of the smart contract.
    pub fn bytecode(&mut self, bytecode: impl AsRef<[u8]>) -> &mut Self {
        self.data_mut().bytecode = Some(bytecode.as_ref().to_vec());
        self
    }

    /// Returns the admin key.
    #[must_use]
    pub fn get_admin_key(&self) -> Option<&Key> {
        self.data().admin_key.as_ref()
    }

    /// Sets the admin key.
    pub fn admin_key(&mut self, key: impl Into<Key>) -> &mut Self {
        self.data_mut().admin_key = Some(key.into());
        self
    }

    /// Returns the gas limit to deploy the smart contract.
    #[must_use]
    pub fn get_gas(&self) -> u64 {
        self.data().gas
    }

    /// Sets the gas limit to deploy the smart contract.
    pub fn gas(&mut self, gas: u64) -> &mut Self {
        self.data_mut().gas = gas;
        self
    }

    /// Returns the initial balance to put into the cryptocurrency account associated with the new smart contract.
    #[must_use]
    pub fn get_initial_balance(&self) -> Hbar {
        self.data().initial_balance
    }

    /// Sets the initial balance to put into the cryptocurrency account associated with the new
    /// smart contract.
    pub fn initial_balance(&mut self, balance: Hbar) -> &mut Self {
        self.data_mut().initial_balance = balance;
        self
    }

    /// Returns the auto renew period for this smart contract.
    #[must_use]
    pub fn get_auto_renew_period(&self) -> Duration {
        self.data().auto_renew_period
    }

    /// Sets the auto renew period for this smart contract.
    pub fn auto_renew_period(&mut self, period: Duration) -> &mut Self {
        self.data_mut().auto_renew_period = period;
        self
    }

    /// Returns the parameters to pass to the constructor.
    #[must_use]
    pub fn get_constructor_parameters(&self) -> &[u8] {
        self.data().constructor_parameters.as_ref()
    }

    /// Sets the parameters to pass to the constructor.
    pub fn constructor_parameters(&mut self, parameters: impl AsRef<[u8]>) -> &mut Self {
        self.data_mut().constructor_parameters = parameters.as_ref().to_vec();
        self
    }

    /// Returns the memo for the new smart contract.
    #[must_use]
    pub fn get_contract_memo(&self) -> &str {
        self.data().contract_memo.as_str()
    }

    /// Sets the memo for the new smart contract.
    pub fn contract_memo(&mut self, memo: impl Into<String>) -> &mut Self {
        self.data_mut().contract_memo = memo.into();
        self
    }

    /// Returns the maximum number of tokens that the contract can be automatically associated with.
    #[must_use]
    pub fn get_max_automatic_token_associations(&self) -> i32 {
        self.data().max_automatic_token_associations
    }

    /// Sets the maximum number of tokens that this contract can be automatically associated with.
    pub fn max_automatic_token_associations(&mut self, max: i32) -> &mut Self {
        self.data_mut().max_automatic_token_associations = max;
        self
    }

    /// Returns the account ot be used at the contract's expiration time to extend the
    /// life of the contract
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

    /// Returns the ID of the account to which this contract is staking.
    #[must_use]
    pub fn get_staked_account_id(&self) -> Option<AccountId> {
        self.data().staked_id.and_then(StakedId::to_account_id)
    }

    /// Sets the ID of the account to which this contract is staking.
    ///
    /// This is mutually exclusive with [`staked_node_id`](Self::staked_node_id).
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
    /// This is mutually exclusive with [`staked_account_id`](Self::staked_account_id).
    pub fn staked_node_id(&mut self, id: u64) -> &mut Self {
        self.data_mut().staked_id = Some(id.into());
        self
    }

    /// Returns `true` if the contract will decline receiving staking rewards, `false` otherwise.
    #[must_use]
    pub fn get_decline_staking_reward(&self) -> bool {
        self.data().decline_staking_reward
    }

    /// If `true` the contract should declie receiving staking rewards. The default value is `false`.
    pub fn decline_staking_reward(&mut self, decline: bool) -> &mut Self {
        self.data_mut().decline_staking_reward = decline;
        self
    }
}

impl TransactionData for ContractCreateTransactionData {
    fn default_max_transaction_fee(&self) -> crate::Hbar {
        crate::Hbar::new(20)
    }
}

impl TransactionExecute for ContractCreateTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { SmartContractServiceClient::new(channel).create_contract(request).await })
    }
}

impl ValidateChecksums for ContractCreateTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.bytecode_file_id.validate_checksums(ledger_id)?;
        self.auto_renew_account_id.validate_checksums(ledger_id)?;
        self.staked_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for ContractCreateTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::ContractCreateInstance(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for ContractCreateTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::ContractCreateInstance(self.to_protobuf())
    }
}

impl From<ContractCreateTransactionData> for AnyTransactionData {
    fn from(transaction: ContractCreateTransactionData) -> Self {
        Self::ContractCreate(transaction)
    }
}

impl FromProtobuf<services::ContractCreateTransactionBody> for ContractCreateTransactionData {
    fn from_protobuf(pb: services::ContractCreateTransactionBody) -> crate::Result<Self> {
        use services::contract_create_transaction_body::InitcodeSource;
        let (bytecode, bytecode_file_id) = match pb.initcode_source {
            Some(InitcodeSource::FileId(it)) => (None, Some(FileId::from_protobuf(it)?)),
            Some(InitcodeSource::Initcode(it)) => (Some(it), None),
            None => (None, None),
        };

        Ok(Self {
            bytecode,
            bytecode_file_id,
            admin_key: Option::from_protobuf(pb.admin_key)?,
            gas: pb.gas as u64,
            initial_balance: Hbar::from_tinybars(pb.initial_balance),
            auto_renew_period: pb_getf!(pb, auto_renew_period)?.into(),
            constructor_parameters: pb.constructor_parameters,
            contract_memo: pb.memo,
            max_automatic_token_associations: pb.max_automatic_token_associations,
            auto_renew_account_id: Option::from_protobuf(pb.auto_renew_account_id)?,
            staked_id: Option::from_protobuf(pb.staked_id)?,
            decline_staking_reward: pb.decline_reward,
        })
    }
}

impl ToProtobuf for ContractCreateTransactionData {
    type Protobuf = services::ContractCreateTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        let admin_key = self.admin_key.to_protobuf();
        let auto_renew_period = self.auto_renew_period.into();
        let auto_renew_account_id = self.auto_renew_account_id.to_protobuf();

        let initcode_source = match (&self.bytecode, &self.bytecode_file_id) {
            (_, Some(file_id)) => {
                Some(services::contract_create_transaction_body::InitcodeSource::FileId(
                    file_id.to_protobuf(),
                ))
            }

            (Some(bytecode), _) => {
                Some(services::contract_create_transaction_body::InitcodeSource::Initcode(
                    bytecode.clone(),
                ))
            }

            _ => None,
        };

        let staked_id = match self.staked_id {
            Some(StakedId::NodeId(node_id)) => Some(
                services::contract_create_transaction_body::StakedId::StakedNodeId(node_id as i64),
            ),

            Some(StakedId::AccountId(account_id)) => {
                Some(services::contract_create_transaction_body::StakedId::StakedAccountId(
                    account_id.to_protobuf(),
                ))
            }

            _ => None,
        };

        #[allow(deprecated)]
        services::ContractCreateTransactionBody {
            admin_key,
            gas: self.gas as i64,
            initial_balance: self.initial_balance.to_tinybars(),
            proxy_account_id: None,
            auto_renew_period: Some(auto_renew_period),
            constructor_parameters: self.constructor_parameters.clone(),
            shard_id: None,
            realm_id: None,
            new_realm_admin_key: None,
            memo: self.contract_memo.clone(),
            max_automatic_token_associations: self.max_automatic_token_associations,
            auto_renew_account_id,
            decline_reward: self.decline_staking_reward,
            initcode_source,
            staked_id,
        }
    }
}

#[cfg(test)]
mod tests {

    use expect_test::expect;
    use hedera_proto::services;
    use time::Duration;

    use crate::contract::ContractCreateTransactionData;
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
        ContractCreateTransaction,
        FileId,
        Hbar,
        PublicKey,
    };

    const BYTECODE: [u8; 4] = [0xde, 0xad, 0xbe, 0xef];
    const BYTECODE_FILE_ID: FileId = FileId::new(0, 0, 3003);

    fn admin_key() -> PublicKey {
        unused_private_key().public_key()
    }

    const GAS: u64 = 0;
    const INITIAL_BALANCE: Hbar = Hbar::from_tinybars(1000);
    const MAX_AUTOMATIC_TOKEN_ASSOCIATIONS: i32 = 101;
    const AUTO_RENEW_PERIOD: Duration = Duration::hours(10);
    const CONSTRUCTOR_PARAMETERS: [u8; 5] = [10, 11, 12, 13, 25];
    const AUTO_RENEW_ACCOUNT_ID: AccountId = AccountId::new(0, 0, 30);
    const STAKED_ACCOUNT_ID: AccountId = AccountId::new(0, 0, 3);
    const STAKED_NODE_ID: u64 = 4;

    fn make_transaction() -> ContractCreateTransaction {
        let mut tx = ContractCreateTransaction::new_for_tests();

        tx.bytecode_file_id(BYTECODE_FILE_ID)
            .admin_key(admin_key())
            .gas(GAS)
            .initial_balance(INITIAL_BALANCE)
            .staked_account_id(STAKED_ACCOUNT_ID)
            .max_automatic_token_associations(MAX_AUTOMATIC_TOKEN_ASSOCIATIONS)
            .auto_renew_period(AUTO_RENEW_PERIOD)
            .constructor_parameters(CONSTRUCTOR_PARAMETERS)
            .auto_renew_account_id(AUTO_RENEW_ACCOUNT_ID)
            .freeze()
            .unwrap();

        tx
    }

    fn make_transaction2() -> ContractCreateTransaction {
        let mut tx = ContractCreateTransaction::new_for_tests();

        tx.bytecode(&BYTECODE)
            .admin_key(admin_key())
            .gas(GAS)
            .initial_balance(INITIAL_BALANCE)
            .staked_node_id(STAKED_NODE_ID)
            .max_automatic_token_associations(MAX_AUTOMATIC_TOKEN_ASSOCIATIONS)
            .auto_renew_period(AUTO_RENEW_PERIOD)
            .constructor_parameters(CONSTRUCTOR_PARAMETERS)
            .auto_renew_account_id(AUTO_RENEW_ACCOUNT_ID)
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
            ContractCreateInstance(
                ContractCreateTransactionBody {
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
                    gas: 0,
                    initial_balance: 1000,
                    proxy_account_id: None,
                    auto_renew_period: Some(
                        Duration {
                            seconds: 36000,
                        },
                    ),
                    constructor_parameters: [
                        10,
                        11,
                        12,
                        13,
                        25,
                    ],
                    shard_id: None,
                    realm_id: None,
                    new_realm_admin_key: None,
                    memo: "",
                    max_automatic_token_associations: 101,
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
                    decline_reward: false,
                    initcode_source: Some(
                        FileId(
                            FileId {
                                shard_num: 0,
                                realm_num: 0,
                                file_num: 3003,
                            },
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
            ContractCreateInstance(
                ContractCreateTransactionBody {
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
                    gas: 0,
                    initial_balance: 1000,
                    proxy_account_id: None,
                    auto_renew_period: Some(
                        Duration {
                            seconds: 36000,
                        },
                    ),
                    constructor_parameters: [
                        10,
                        11,
                        12,
                        13,
                        25,
                    ],
                    shard_id: None,
                    realm_id: None,
                    new_realm_admin_key: None,
                    memo: "",
                    max_automatic_token_associations: 101,
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
                    decline_reward: false,
                    initcode_source: Some(
                        Initcode(
                            [
                                222,
                                173,
                                190,
                                239,
                            ],
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
        let tx = services::ContractCreateTransactionBody {
            admin_key: Some(admin_key().to_protobuf()),
            initial_balance: INITIAL_BALANCE.to_tinybars(),
            proxy_account_id: None,
            auto_renew_period: Some(AUTO_RENEW_PERIOD.to_protobuf()),
            shard_id: None,
            realm_id: None,
            new_realm_admin_key: None,
            memo: String::new(),
            max_automatic_token_associations: MAX_AUTOMATIC_TOKEN_ASSOCIATIONS,
            decline_reward: false,
            staked_id: Some(services::contract_create_transaction_body::StakedId::StakedAccountId(
                STAKED_ACCOUNT_ID.to_protobuf(),
            )),
            gas: GAS as _,
            constructor_parameters: CONSTRUCTOR_PARAMETERS.to_vec(),
            auto_renew_account_id: Some(AUTO_RENEW_ACCOUNT_ID.to_protobuf()),
            initcode_source: Some(
                services::contract_create_transaction_body::InitcodeSource::FileId(
                    BYTECODE_FILE_ID.to_protobuf(),
                ),
            ),
        };
        let tx = ContractCreateTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(tx.bytecode, None);
        assert_eq!(tx.bytecode_file_id, Some(BYTECODE_FILE_ID));
        assert_eq!(tx.admin_key, Some(admin_key().into()));
        assert_eq!(tx.gas, GAS);
        assert_eq!(tx.initial_balance, INITIAL_BALANCE);
        assert_eq!(tx.staked_id, Some(crate::staked_id::StakedId::AccountId(STAKED_ACCOUNT_ID)));
        assert_eq!(tx.max_automatic_token_associations, MAX_AUTOMATIC_TOKEN_ASSOCIATIONS);
        assert_eq!(tx.auto_renew_period, AUTO_RENEW_PERIOD);
        assert_eq!(tx.constructor_parameters, CONSTRUCTOR_PARAMETERS);
        assert_eq!(tx.auto_renew_account_id, Some(AUTO_RENEW_ACCOUNT_ID));
    }

    #[test]
    fn get_set_bytecode() {
        let mut tx = ContractCreateTransaction::new();
        tx.bytecode(BYTECODE);

        assert_eq!(tx.get_bytecode(), Some(BYTECODE.as_slice()));
    }

    #[test]
    #[should_panic]
    fn get_set_bytecode_frozen_panics() {
        make_transaction().bytecode(BYTECODE);
    }

    #[test]
    fn get_set_bytecode_file_id() {
        let mut tx = ContractCreateTransaction::new();
        tx.bytecode_file_id(BYTECODE_FILE_ID);

        assert_eq!(tx.get_bytecode_file_id(), Some(BYTECODE_FILE_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_bytecode_file_id_frozen_panics() {
        make_transaction().bytecode_file_id(BYTECODE_FILE_ID);
    }

    #[test]
    fn get_set_admin_key() {
        let mut tx = ContractCreateTransaction::new();
        tx.admin_key(admin_key());

        assert_eq!(tx.get_admin_key(), Some(&admin_key().into()));
    }

    #[test]
    #[should_panic]
    fn get_set_admin_key_frozen_panics() {
        make_transaction().admin_key(admin_key());
    }

    #[test]
    fn get_set_gas() {
        let mut tx = ContractCreateTransaction::new();
        tx.gas(GAS);

        assert_eq!(tx.get_gas(), GAS);
    }

    #[test]
    #[should_panic]
    fn get_set_gas_frozen_panics() {
        make_transaction().gas(GAS);
    }

    #[test]
    fn get_set_initial_balance() {
        let mut tx = ContractCreateTransaction::new();
        tx.initial_balance(INITIAL_BALANCE);

        assert_eq!(tx.get_initial_balance(), INITIAL_BALANCE);
    }

    #[test]
    #[should_panic]
    fn get_set_initial_balance_frozen_panics() {
        make_transaction().initial_balance(INITIAL_BALANCE);
    }

    #[test]
    fn get_set_max_automatic_token_associations() {
        let mut tx = ContractCreateTransaction::new();
        tx.max_automatic_token_associations(MAX_AUTOMATIC_TOKEN_ASSOCIATIONS);

        assert_eq!(tx.get_max_automatic_token_associations(), MAX_AUTOMATIC_TOKEN_ASSOCIATIONS);
    }

    #[test]
    #[should_panic]
    fn get_set_max_automatic_token_associations_frozen_panics() {
        make_transaction().max_automatic_token_associations(MAX_AUTOMATIC_TOKEN_ASSOCIATIONS);
    }

    #[test]
    fn get_set_auto_renew_period() {
        let mut tx = ContractCreateTransaction::new();
        tx.auto_renew_period(AUTO_RENEW_PERIOD);

        assert_eq!(tx.get_auto_renew_period(), AUTO_RENEW_PERIOD);
    }

    #[test]
    #[should_panic]
    fn get_set_auto_renew_period_frozen_panics() {
        make_transaction().auto_renew_period(AUTO_RENEW_PERIOD);
    }

    #[test]
    fn get_set_constructor_parameters() {
        let mut tx = ContractCreateTransaction::new();
        tx.constructor_parameters(CONSTRUCTOR_PARAMETERS);

        assert_eq!(tx.get_constructor_parameters(), CONSTRUCTOR_PARAMETERS);
    }

    #[test]
    #[should_panic]
    fn get_set_constructor_parameters_frozen_panics() {
        make_transaction().constructor_parameters(CONSTRUCTOR_PARAMETERS);
    }

    #[test]
    fn get_set_auto_renew_account_id() {
        let mut tx = ContractCreateTransaction::new();
        tx.auto_renew_account_id(AUTO_RENEW_ACCOUNT_ID);

        assert_eq!(tx.get_auto_renew_account_id(), Some(AUTO_RENEW_ACCOUNT_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_auto_renew_account_id_frozen_panics() {
        make_transaction().auto_renew_account_id(AUTO_RENEW_ACCOUNT_ID);
    }

    #[test]
    fn get_set_staked_account_id() {
        let mut tx = ContractCreateTransaction::new();
        tx.staked_account_id(STAKED_ACCOUNT_ID);

        assert_eq!(tx.get_staked_account_id(), Some(STAKED_ACCOUNT_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_staked_account_id_frozen_panics() {
        make_transaction().staked_account_id(STAKED_ACCOUNT_ID);
    }

    #[test]
    fn get_set_staked_node_id() {
        let mut tx = ContractCreateTransaction::new();
        tx.staked_node_id(STAKED_NODE_ID);

        assert_eq!(tx.get_staked_node_id(), Some(STAKED_NODE_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_staked_node_id_frozen_panics() {
        make_transaction().staked_node_id(STAKED_NODE_ID);
    }
}
