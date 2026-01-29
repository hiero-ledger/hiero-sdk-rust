use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{
    Arc,
    Mutex,
};

use hiero_sdk::{
    AccountCreateTransaction,
    AccountId,
    AccountUpdateTransaction,
    AccountAllowanceApproveTransaction,
    AccountBalanceQuery,
    Client,
    ContractCreateTransaction,
    ContractDeleteTransaction,
    ContractExecuteTransaction,
    ContractId,
    ContractUpdateTransaction,
    CustomFeeLimit,
    CustomFixedFee,
    EvmAddress,
    FileId,
    Hbar,
    NftId,
    PendingAirdropId,
    PrivateKey,
    TransferTransaction,
    TokenMintTransaction,
    TokenBurnTransaction,
    ScheduleCreateTransaction,
    ScheduleDeleteTransaction,
    ScheduleId,
    ScheduleSignTransaction,
    ContractCallQuery,
    ContractInfoQuery,
    TopicInfoQuery,
    ScheduleInfoQuery,
    TokenCancelAirdropTransaction,
    TokenClaimAirdropTransaction,
    TokenId,
    TopicCreateTransaction,
    TopicDeleteTransaction,
    TopicId,
    TopicMessageSubmitTransaction,
    TopicUpdateTransaction,
};
use jsonrpsee::core::async_trait;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::error::INTERNAL_ERROR_CODE;
use jsonrpsee::types::{
    ErrorObject,
    ErrorObjectOwned,
};
use once_cell::sync::Lazy;
use serde_json::Value;
use time::{
    Duration,
    OffsetDateTime,
};

use crate::errors::from_hedera_error;
use crate::helpers::{
    fill_common_transaction_params,
    generate_key_helper,
    get_hedera_key,
};
use crate::responses::{
    AccountBalanceResponse,
    AccountCreateResponse,
    AccountUpdateResponse,
    ContractCallResponse,
    ContractResponse,
    GenerateKeyResponse,
    InfoResponse,
    ScheduleResponse,
    TopicResponse,
};

static GLOBAL_SDK_CLIENT: Lazy<Arc<Mutex<Option<Client>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

#[rpc(server, client)]
pub trait Rpc {
    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/utility.md#generateKey
    */
    #[method(name = "generateKey")]
    fn generate_key(
        &self,
        _type: String,
        from_key: Option<String>,
        threshold: Option<i32>,
        keys: Option<Value>,
    ) -> Result<GenerateKeyResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/utility.md#setup
    */
    #[method(name = "setup")]
    fn setup(
        &self,
        operator_account_id: Option<String>,
        operator_private_key: Option<String>,
        node_ip: Option<String>,
        node_account_id: Option<String>,
        mirror_network_ip: Option<String>,
    ) -> Result<String, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/utility.md#reset
    */
    #[method(name = "reset")]
    fn reset(&self) -> Result<HashMap<String, String>, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/crypto-service/accountCreateTransaction.md#createAccount
    */
    #[method(name = "createAccount")]
    async fn create_account(
        &self,
        key: Option<String>,
        initial_balance: Option<String>,
        receiver_signature_required: Option<bool>,
        auto_renew_period: Option<String>,
        memo: Option<String>,
        max_auto_token_associations: Option<i64>,
        staked_account_id: Option<String>,
        staked_node_id: Option<String>,
        decline_staking_reward: Option<bool>,
        alias: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<AccountCreateResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/crypto-service/accountUpdateTransaction.md#updateAccount
    */
    #[method(name = "updateAccount")]
    async fn update_account(
        &self,
        account_id: Option<String>,
        key: Option<String>,
        auto_renew_period: Option<i64>,
        expiration_time: Option<i64>,
        receiver_signature_required: Option<bool>,
        memo: Option<String>,
        max_auto_token_associations: Option<i64>,
        staked_account_id: Option<String>,
        staked_node_id: Option<String>,
        decline_staking_reward: Option<bool>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<AccountUpdateResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenClaimAirdropTransaction.md#tokenClaim
    */
    #[method(name = "tokenClaim")]
    async fn token_claim(
        &self,
        pending_airdrop_ids: Vec<HashMap<String, String>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<HashMap<String, String>, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenCancelAirdropTransaction.md#tokenCancel
    */
    #[method(name = "tokenCancel")]
    async fn token_cancel(
        &self,
        pending_airdrop_ids: Vec<HashMap<String, String>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<HashMap<String, String>, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/contract-service/contractCreateTransaction.md#createContract
    */
    #[method(name = "createContract")]
    async fn create_contract(
        &self,
        admin_key: Option<String>,
        auto_renew_period: Option<String>,
        auto_renew_account_id: Option<String>,
        initial_balance: Option<String>,
        bytecode_file_id: Option<String>,
        initcode: Option<String>,
        staked_account_id: Option<String>,
        staked_node_id: Option<String>,
        gas: Option<String>,
        decline_staking_reward: Option<bool>,
        memo: Option<String>,
        max_automatic_token_associations: Option<i32>,
        constructor_parameters: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ContractResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/contract-service/contractUpdateTransaction.md#updateContract
    */
    #[method(name = "updateContract")]
    async fn update_contract(
        &self,
        contract_id: Option<String>,
        admin_key: Option<String>,
        auto_renew_period: Option<String>,
        auto_renew_account_id: Option<String>,
        staked_account_id: Option<String>,
        staked_node_id: Option<String>,
        decline_staking_reward: Option<bool>,
        memo: Option<String>,
        max_automatic_token_associations: Option<i32>,
        expiration_time: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ContractResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/contract-service/contractDeleteTransaction.md#deleteContract
    */
    #[method(name = "deleteContract")]
    async fn delete_contract(
        &self,
        contract_id: Option<String>,
        transfer_account_id: Option<String>,
        transfer_contract_id: Option<String>,
        permanent_removal: Option<bool>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ContractResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/contract-service/contractExecuteTransaction.md#executeContract
    */
    #[method(name = "executeContract")]
    async fn execute_contract(
        &self,
        contract_id: Option<String>,
        gas: Option<String>,
        amount: Option<String>,
        function_parameters: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ContractResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/topic-service/topicCreateTransaction.md#createTopic
    */
    #[method(name = "createTopic")]
    async fn create_topic(
        &self,
        memo: Option<String>,
        admin_key: Option<String>,
        submit_key: Option<String>,
        auto_renew_period: Option<String>,
        auto_renew_account_id: Option<String>,
        fee_schedule_key: Option<String>,
        fee_exempt_keys: Option<Vec<String>>,
        custom_fees: Option<Value>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TopicResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/topic-service/topicUpdateTransaction.md#updateTopic
    */
    #[method(name = "updateTopic")]
    async fn update_topic(
        &self,
        topic_id: Option<String>,
        memo: Option<String>,
        admin_key: Option<String>,
        submit_key: Option<String>,
        auto_renew_period: Option<String>,
        auto_renew_account_id: Option<String>,
        expiration_time: Option<String>,
        fee_schedule_key: Option<String>,
        fee_exempt_keys: Option<Vec<String>>,
        custom_fees: Option<Value>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TopicResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/topic-service/topicDeleteTransaction.md#deleteTopic
    */
    #[method(name = "deleteTopic")]
    async fn delete_topic(
        &self,
        topic_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TopicResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/topic-service/topicMessageSubmitTransaction.md#submitTopicMessage
    */
    #[method(name = "submitTopicMessage")]
    async fn submit_topic_message(
        &self,
        topic_id: Option<String>,
        message: Option<String>,
        max_chunks: Option<usize>,
        chunk_size: Option<usize>,
        custom_fee_limits: Option<Value>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TopicResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/schedule-service/scheduleCreateTransaction.md#createSchedule
    */
    #[method(name = "createSchedule")]
    async fn create_schedule(
        &self,
        scheduled_transaction: Option<Value>,
        memo: Option<String>,
        admin_key: Option<String>,
        payer_account_id: Option<String>,
        expiration_time: Option<String>,
        wait_for_expiry: Option<bool>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ScheduleResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/schedule-service/scheduleSignTransaction.md#signSchedule
    */
    #[method(name = "signSchedule")]
    async fn sign_schedule(
        &self,
        schedule_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ScheduleResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/schedule-service/scheduleDeleteTransaction.md#deleteSchedule
    */
    #[method(name = "deleteSchedule")]
    async fn delete_schedule(
        &self,
        schedule_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ScheduleResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/crypto-service/accountBalanceQuery.md#getAccountBalance
    */
    #[method(name = "getAccountBalance")]
    async fn get_account_balance(
        &self,
        account_id: Option<String>,
        contract_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<AccountBalanceResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/contract-service/contractCallQuery.md#contractCallQuery
    */
    #[method(name = "contractCallQuery")]
    async fn contract_call_query(
        &self,
        contract_id: Option<String>,
        gas: Option<String>,
        function_parameters: Option<String>,
        sender_account_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ContractCallResponse, ErrorObjectOwned>;

    #[method(name = "contractInfoQuery")]
    async fn contract_info_query(
        &self,
        contract_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<InfoResponse, ErrorObjectOwned>;

    #[method(name = "topicInfoQuery")]
    async fn topic_info_query(
        &self,
        topic_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<InfoResponse, ErrorObjectOwned>;

    #[method(name = "scheduleInfoQuery")]
    async fn schedule_info_query(
        &self,
        schedule_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<InfoResponse, ErrorObjectOwned>;
}

pub struct RpcServerImpl;

#[async_trait]
impl RpcServer for RpcServerImpl {
    fn setup(
        &self,
        operator_account_id: Option<String>,
        operator_private_key: Option<String>,
        node_ip: Option<String>,
        node_account_id: Option<String>,
        mirror_network_ip: Option<String>,
    ) -> Result<String, ErrorObjectOwned> {
        let mut network: HashMap<String, AccountId> = HashMap::new();

        // Client setup, if the network is not set, it will be created using testnet.
        // If the network is manually set, the network will be configured using the
        // provided ips and account id.
        let client = match (node_ip, node_account_id, mirror_network_ip) {
            (Some(node_ip), Some(node_account_id), Some(mirror_network_ip)) => {
                let account_id = AccountId::from_str(node_account_id.as_str()).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?;
                network.insert(node_ip, account_id);

                let client = Client::for_network(network).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?;
                client.set_mirror_network([mirror_network_ip]);
                client
            }
            (None, None, None) => Client::for_testnet(),
            _ => {
                return Err(ErrorObject::borrowed(
                    INTERNAL_ERROR_CODE,
                    "Failed to setup client",
                    None,
                ))
            }
        };

        let operator_id = if let Some(operator_account_id) = operator_account_id {
            AccountId::from_str(operator_account_id.as_str())
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?
        } else {
            return Err(ErrorObject::borrowed(
                INTERNAL_ERROR_CODE,
                "Missing operator account id",
                None,
            ));
        };

        let operator_key = if let Some(operator_private_key) = operator_private_key {
            PrivateKey::from_str(operator_private_key.as_str())
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?
        } else {
            return Err(ErrorObject::borrowed(
                INTERNAL_ERROR_CODE,
                "Missing operator private key",
                None,
            ));
        };

        client.set_operator(operator_id, operator_key);

        let mut global_client = GLOBAL_SDK_CLIENT.lock().unwrap();
        *global_client = Some(client);

        Ok("SUCCESS".to_owned())
    }

    fn reset(&self) -> Result<HashMap<String, String>, ErrorObjectOwned> {
        let mut global_client = GLOBAL_SDK_CLIENT.lock().unwrap();
        *global_client = None;
        Ok(HashMap::from([("status".to_string(), "SUCCESS".to_string())].to_owned()))
    }

    fn generate_key(
        &self,
        _type: String,
        from_key: Option<String>,
        threshold: Option<i32>,
        keys: Option<Value>,
    ) -> Result<GenerateKeyResponse, ErrorObjectOwned> {
        let mut private_keys: Vec<Value> = Vec::new();

        let key = generate_key_helper(_type, from_key, threshold, keys, &mut private_keys, false)?;

        Ok(GenerateKeyResponse { key: key, private_keys: private_keys })
    }

    async fn create_account(
        &self,
        key: Option<String>,
        initial_balance: Option<String>,
        receiver_signature_required: Option<bool>,
        auto_renew_period: Option<String>,
        memo: Option<String>,
        max_auto_token_associations: Option<i64>,
        staked_account_id: Option<String>,
        staked_node_id: Option<String>,
        decline_staking_reward: Option<bool>,
        alias: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<AccountCreateResponse, ErrorObjectOwned> {
        let client = {
            let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
            guard
                .as_ref()
                .ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Client not initialized".to_string(),
                        None::<()>,
                    )
                })?
                .clone()
        };
        let mut account_create_tx = AccountCreateTransaction::new();
  
        if let Some(key) = key {
            let key = get_hedera_key(&key)?;

            account_create_tx.set_key_without_alias(key);
        }

        if let Some(initial_balance) = initial_balance {
            let initial_balance = initial_balance.parse::<i64>().map_err(|e| {
                ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
            })?;
            account_create_tx.initial_balance(Hbar::from_tinybars(initial_balance));
        }

        if let Some(receiver_signature_required) = receiver_signature_required {
            account_create_tx.receiver_signature_required(receiver_signature_required);
        }

        if let Some(auto_renew_period) = auto_renew_period {
            let auto_renew_period = auto_renew_period.parse::<i64>().map_err(|e| {
                ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
            })?;
            account_create_tx.auto_renew_period(Duration::seconds(auto_renew_period));
        }

        if let Some(memo) = memo {
            account_create_tx.account_memo(memo);
        }

        if let Some(max_auto_token_associations) = max_auto_token_associations {
            account_create_tx.max_automatic_token_associations(max_auto_token_associations as i32);
        }

        if let Some(staked_account_id) = staked_account_id {
            account_create_tx.staked_account_id(
                AccountId::from_str(&staked_account_id).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }

        if let Some(alias) = alias {
            account_create_tx.alias(
                EvmAddress::from_str(&alias).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }

        if let Some(staked_node_id) = staked_node_id {
            let node_id = staked_node_id.parse::<i64>().map_err(|e| {
                ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
            })?;
            if node_id < 0 {
                return Err(ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "INVALID_STAKING_ID",
                    None::<()>,
                ));
            }
            account_create_tx.staked_node_id(node_id as u64);
        }

        if let Some(decline_staking_reward) = decline_staking_reward {
            account_create_tx.decline_staking_reward(decline_staking_reward);
        }

        if let Some(common_transaction_params) = common_transaction_params {
            let _ =
                fill_common_transaction_params(&mut account_create_tx, &common_transaction_params);

            account_create_tx.freeze_with(&client).unwrap();

            if let Some(signers) = common_transaction_params.get("signers") {
                if let Value::Array(signers) = signers {
                    for signer in signers {
                        if let Value::String(signer_str) = signer {
                            account_create_tx.sign(PrivateKey::from_str_der(signer_str).unwrap());
                        }
                    }
                }
            }
        }

        let tx_response =
            account_create_tx.execute(&client).await.map_err(|e| from_hedera_error(e))?;

        let tx_receipt =
            tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

        Ok(AccountCreateResponse {
            account_id: tx_receipt.account_id.unwrap().to_string(),
            status: tx_receipt.status.as_str_name().to_string(),
        })
    }

    async fn update_account(
        &self,
        account_id: Option<String>,
        key: Option<String>,
        auto_renew_period: Option<i64>,
        expiration_time: Option<i64>,
        receiver_signature_required: Option<bool>,
        memo: Option<String>,
        max_auto_token_associations: Option<i64>,
        staked_account_id: Option<String>,
        staked_node_id: Option<String>,
        decline_staking_reward: Option<bool>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<AccountUpdateResponse, ErrorObjectOwned> {
        let client = {
            let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
            guard
                .as_ref()
                .ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Client not initialized".to_string(),
                        None::<()>,
                    )
                })?
                .clone()
        };

        let mut account_update_tx = AccountUpdateTransaction::new();

        if let Some(account_id) = account_id {
            account_update_tx.account_id(account_id.parse().unwrap());
        }

        if let Some(key) = key {
            println!("key: {}", key);
            let key = get_hedera_key(&key)?;

            account_update_tx.key(key);
        }

        if let Some(receiver_signature_required) = receiver_signature_required {
            account_update_tx.receiver_signature_required(receiver_signature_required);
        }

        if let Some(auto_renew_period) = auto_renew_period {
            account_update_tx.auto_renew_period(Duration::seconds(auto_renew_period));
        }

        if let Some(expiration_time) = expiration_time {
            account_update_tx.expiration_time(
                OffsetDateTime::from_unix_timestamp(expiration_time).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }

        if let Some(memo) = memo {
            account_update_tx.account_memo(memo);
        }

        if let Some(max_auto_token_associations) = max_auto_token_associations {
            account_update_tx.max_automatic_token_associations(max_auto_token_associations as i32);
        }

        if let Some(staked_account_id) = staked_account_id {
            account_update_tx.staked_account_id(
                AccountId::from_str(&staked_account_id).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }

        if let Some(staked_node_id) = staked_node_id {
            let node_id = staked_node_id.parse::<i64>().map_err(|e| {
                ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
            })?;
            if node_id < 0 {
                return Err(ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "INVALID_STAKING_ID",
                    None::<()>,
                ));
            }
            account_update_tx.staked_node_id(node_id as u64);
        }

        if let Some(decline_staking_reward) = decline_staking_reward {
            account_update_tx.decline_staking_reward(decline_staking_reward);
        }

        if let Some(common_transaction_params) = common_transaction_params {
            let _ =
                fill_common_transaction_params(&mut account_update_tx, &common_transaction_params);

            account_update_tx.freeze_with(&client).unwrap();

            if let Some(signers) = common_transaction_params.get("signers") {
                if let Value::Array(signers) = signers {
                    for signer in signers {
                        if let Value::String(signer_str) = signer {
                            account_update_tx.sign(PrivateKey::from_str_der(signer_str).unwrap());
                        }
                    }
                }
            }
        }

        let tx_response =
            account_update_tx.execute(&client).await.map_err(|e| from_hedera_error(e))?;

        let tx_receipt =
            tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

        Ok(AccountUpdateResponse { status: tx_receipt.status.as_str_name().to_string() })
    }

    async fn token_claim(
        &self,
        pending_airdrop_ids: Vec<HashMap<String, String>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<HashMap<String, String>, ErrorObjectOwned> {
        let client = {
            let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
            guard
                .as_ref()
                .ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Client not initialized".to_string(),
                        None::<()>,
                    )
                })?
                .clone()
        };

        // Parse pending_airdrop_ids from HashMap<String, String> to PendingAirdropId
        let mut parsed_ids = Vec::new();
        for id_map in pending_airdrop_ids {
            let sender_id = id_map.get("sender_id").ok_or_else(|| {
                ErrorObject::owned(INTERNAL_ERROR_CODE, "Missing sender_id", None::<()>)
            })?;
            let receiver_id = id_map.get("receiver_id").ok_or_else(|| {
                ErrorObject::owned(INTERNAL_ERROR_CODE, "Missing receiver_id", None::<()>)
            })?;
            let token_id = id_map.get("token_id");
            let nft_id = id_map.get("nft_id");

            let sender_id = AccountId::from_str(sender_id).map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid sender_id: {e}"),
                    None::<()>,
                )
            })?;
            let receiver_id = AccountId::from_str(receiver_id).map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid receiver_id: {e}"),
                    None::<()>,
                )
            })?;

            let pending_id = if let Some(token_id) = token_id {
                let token_id = TokenId::from_str(token_id).map_err(|e| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        format!("Invalid token_id: {e}"),
                        None::<()>,
                    )
                })?;
                PendingAirdropId::new_token_id(sender_id, receiver_id, token_id)
            } else if let Some(nft_id) = nft_id {
                let nft_id = NftId::from_str(nft_id).map_err(|e| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        format!("Invalid nft_id: {e}"),
                        None::<()>,
                    )
                })?;
                PendingAirdropId::new_nft_id(sender_id, receiver_id, nft_id)
            } else {
                return Err(ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "Must provide either token_id or nft_id",
                    None::<()>,
                ));
            };
            parsed_ids.push(pending_id);
        }

        let mut tx = TokenClaimAirdropTransaction::new();
        tx.pending_airdrop_ids(parsed_ids);

        if let Some(common_transaction_params) = &common_transaction_params {
            let _ = fill_common_transaction_params(&mut tx, common_transaction_params);
            tx.freeze_with(&client).map_err(|e| from_hedera_error(e.into()))?;
            if let Some(signers) = common_transaction_params.get("signers") {
                if let Value::Array(signers) = signers {
                    for signer in signers {
                        if let Value::String(signer_str) = signer {
                            tx.sign(PrivateKey::from_str_der(signer_str).unwrap());
                        }
                    }
                }
            }
        }

        let tx_response = tx.execute(&client).await.map_err(|e| from_hedera_error(e))?;
        let tx_receipt =
            tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

        Ok(HashMap::from([("status".to_string(), tx_receipt.status.as_str_name().to_string())]))
    }

    async fn token_cancel(
        &self,
        pending_airdrop_ids: Vec<HashMap<String, String>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<HashMap<String, String>, ErrorObjectOwned> {
        let client = {
            let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
            guard
                .as_ref()
                .ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Client not initialized".to_string(),
                        None::<()>,
                    )
                })?
                .clone()
        };

        // Parse pending_airdrop_ids from HashMap<String, String> to PendingAirdropId
        let mut parsed_ids = Vec::new();
        for id_map in pending_airdrop_ids {
            let sender_id = id_map.get("sender_id").ok_or_else(|| {
                ErrorObject::owned(INTERNAL_ERROR_CODE, "Missing sender_id", None::<()>)
            })?;
            let receiver_id = id_map.get("receiver_id").ok_or_else(|| {
                ErrorObject::owned(INTERNAL_ERROR_CODE, "Missing receiver_id", None::<()>)
            })?;
            let token_id = id_map.get("token_id");
            let nft_id = id_map.get("nft_id");

            let sender_id = AccountId::from_str(sender_id).map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid sender_id: {e}"),
                    None::<()>,
                )
            })?;
            let receiver_id = AccountId::from_str(receiver_id).map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid receiver_id: {e}"),
                    None::<()>,
                )
            })?;

            let pending_id = if let Some(token_id) = token_id {
                let token_id = TokenId::from_str(token_id).map_err(|e| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        format!("Invalid token_id: {e}"),
                        None::<()>,
                    )
                })?;
                PendingAirdropId::new_token_id(sender_id, receiver_id, token_id)
            } else if let Some(nft_id) = nft_id {
                let nft_id = NftId::from_str(nft_id).map_err(|e| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        format!("Invalid nft_id: {e}"),
                        None::<()>,
                    )
                })?;
                PendingAirdropId::new_nft_id(sender_id, receiver_id, nft_id)
            } else {
                return Err(ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "Must provide either token_id or nft_id",
                    None::<()>,
                ));
            };
            parsed_ids.push(pending_id);
        }

        let mut tx = TokenCancelAirdropTransaction::new();
        tx.pending_airdrop_ids(parsed_ids);

        if let Some(common_transaction_params) = &common_transaction_params {
            let _ = fill_common_transaction_params(&mut tx, common_transaction_params);
            tx.freeze_with(&client).map_err(|e| from_hedera_error(e.into()))?;
            if let Some(signers) = common_transaction_params.get("signers") {
                if let Value::Array(signers) = signers {
                    for signer in signers {
                        if let Value::String(signer_str) = signer {
                            tx.sign(PrivateKey::from_str_der(signer_str).unwrap());
                        }
                    }
                }
            }
        }

        let tx_response = tx.execute(&client).await.map_err(|e| from_hedera_error(e))?;
        let tx_receipt =
            tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

        Ok(HashMap::from([("status".to_string(), tx_receipt.status.as_str_name().to_string())]))
    }

    async fn create_contract(
        &self,
        admin_key: Option<String>,
        auto_renew_period: Option<String>,
        auto_renew_account_id: Option<String>,
        initial_balance: Option<String>,
        bytecode_file_id: Option<String>,
        initcode: Option<String>,
        staked_account_id: Option<String>,
        staked_node_id: Option<String>,
        gas: Option<String>,
        decline_staking_reward: Option<bool>,
        memo: Option<String>,
        max_automatic_token_associations: Option<i32>,
        constructor_parameters: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ContractResponse, ErrorObjectOwned> {
        let client = {
            let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
            guard
                .as_ref()
                .ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Client not initialized".to_string(),
                        None::<()>,
                    )
                })?
                .clone()
        };

        let mut contract_create_tx = ContractCreateTransaction::new();

        if let Some(admin_key) = admin_key {
            let key = get_hedera_key(&admin_key)?;
            contract_create_tx.admin_key(key);
        }

        if let Some(auto_renew_period) = auto_renew_period {
            let period = auto_renew_period
                .parse::<i64>()
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
            contract_create_tx.auto_renew_period(Duration::seconds(period));
        }

        if let Some(gas) = gas {
            let gas_value = gas
                .parse::<u64>()
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
            contract_create_tx.gas(gas_value);
        }

        if let Some(auto_renew_account_id) = auto_renew_account_id {
            contract_create_tx.auto_renew_account_id(
                AccountId::from_str(&auto_renew_account_id).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }

        if let Some(initial_balance) = initial_balance {
            let balance = initial_balance
                .parse::<i64>()
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
            contract_create_tx.initial_balance(Hbar::from_tinybars(balance));
        }

        if let Some(initcode) = initcode {
            let initcode_bytes = hex::decode(&initcode).map_err(|e| {
                ErrorObject::owned(INTERNAL_ERROR_CODE, format!("Invalid hex: {e}"), None::<()>)
            })?;
            contract_create_tx.bytecode(initcode_bytes);
        }

        if let Some(bytecode_file_id) = bytecode_file_id {
            contract_create_tx.bytecode_file_id(
                FileId::from_str(&bytecode_file_id).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }

        if let Some(staked_account_id) = staked_account_id {
            contract_create_tx.staked_account_id(
                AccountId::from_str(&staked_account_id).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }

        if let Some(staked_node_id) = staked_node_id {
            let node_id = staked_node_id
                .parse::<u64>()
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
            contract_create_tx.staked_node_id(node_id);
        }

        if let Some(decline_staking_reward) = decline_staking_reward {
            contract_create_tx.decline_staking_reward(decline_staking_reward);
        }

        if let Some(memo) = memo {
            contract_create_tx.contract_memo(memo);
        }

        if let Some(max_automatic_token_associations) = max_automatic_token_associations {
            contract_create_tx.max_automatic_token_associations(max_automatic_token_associations);
        }

        if let Some(constructor_parameters) = constructor_parameters {
            let constructor_params = hex::decode(&constructor_parameters).map_err(|e| {
                ErrorObject::owned(INTERNAL_ERROR_CODE, format!("Invalid hex: {e}"), None::<()>)
            })?;
            contract_create_tx.constructor_parameters(constructor_params);
        }

        if let Some(common_transaction_params) = common_transaction_params {
            let _ = fill_common_transaction_params(&mut contract_create_tx, &common_transaction_params);

            contract_create_tx.freeze_with(&client).unwrap();

            if let Some(signers) = common_transaction_params.get("signers") {
                if let Value::Array(signers) = signers {
                    for signer in signers {
                        if let Value::String(signer_str) = signer {
                            contract_create_tx.sign(PrivateKey::from_str_der(signer_str).unwrap());
                        }
                    }
                }
            }
        }

        let tx_response =
            contract_create_tx.execute(&client).await.map_err(|e| from_hedera_error(e))?;

        let tx_receipt =
            tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

        Ok(ContractResponse {
            contract_id: tx_receipt.contract_id.map(|id| id.to_string()),
            status: tx_receipt.status.as_str_name().to_string(),
        })
    }

    async fn update_contract(
        &self,
        contract_id: Option<String>,
        admin_key: Option<String>,
        auto_renew_period: Option<String>,
        auto_renew_account_id: Option<String>,
        staked_account_id: Option<String>,
        staked_node_id: Option<String>,
        decline_staking_reward: Option<bool>,
        memo: Option<String>,
        max_automatic_token_associations: Option<i32>,
        expiration_time: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ContractResponse, ErrorObjectOwned> {
        let client = {
            let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
            guard
                .as_ref()
                .ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Client not initialized".to_string(),
                        None::<()>,
                    )
                })?
                .clone()
        };

        let mut contract_update_tx = ContractUpdateTransaction::new();

        if let Some(contract_id) = contract_id {
            contract_update_tx.contract_id(
                ContractId::from_str(&contract_id).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }

        if let Some(admin_key) = admin_key {
            let key = get_hedera_key(&admin_key)?;
            contract_update_tx.admin_key(key);
        }

        if let Some(auto_renew_period) = auto_renew_period {
            let period = auto_renew_period
                .parse::<i64>()
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
            contract_update_tx.auto_renew_period(Duration::seconds(period));
        }

        if let Some(auto_renew_account_id) = auto_renew_account_id {
            contract_update_tx.auto_renew_account_id(
                AccountId::from_str(&auto_renew_account_id).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }

        if let Some(staked_account_id) = staked_account_id {
            contract_update_tx.staked_account_id(
                AccountId::from_str(&staked_account_id).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }

        if let Some(staked_node_id) = staked_node_id {
            let node_id = staked_node_id
                .parse::<u64>()
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
            contract_update_tx.staked_node_id(node_id);
        }

        if let Some(decline_staking_reward) = decline_staking_reward {
            contract_update_tx.decline_staking_reward(decline_staking_reward);
        }

        if let Some(memo) = memo {
            contract_update_tx.contract_memo(memo);
        }

        if let Some(max_automatic_token_associations) = max_automatic_token_associations {
            contract_update_tx.max_automatic_token_associations(max_automatic_token_associations);
        }

        if let Some(expiration_time) = expiration_time {
            let timestamp = expiration_time
                .parse::<i64>()
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
            contract_update_tx.expiration_time(
                OffsetDateTime::from_unix_timestamp(timestamp).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }

        if let Some(common_transaction_params) = common_transaction_params {
            let _ = fill_common_transaction_params(&mut contract_update_tx, &common_transaction_params);

            contract_update_tx.freeze_with(&client).unwrap();

            if let Some(signers) = common_transaction_params.get("signers") {
                if let Value::Array(signers) = signers {
                    for signer in signers {
                        if let Value::String(signer_str) = signer {
                            contract_update_tx.sign(PrivateKey::from_str_der(signer_str).unwrap());
                        }
                    }
                }
            }
        }

        let tx_response =
            contract_update_tx.execute(&client).await.map_err(|e| from_hedera_error(e))?;

        let tx_receipt =
            tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

        Ok(ContractResponse {
            contract_id: None,
            status: tx_receipt.status.as_str_name().to_string(),
        })
    }

    async fn delete_contract(
        &self,
        contract_id: Option<String>,
        transfer_account_id: Option<String>,
        transfer_contract_id: Option<String>,
        permanent_removal: Option<bool>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ContractResponse, ErrorObjectOwned> {
        let client = {
            let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
            guard
                .as_ref()
                .ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Client not initialized".to_string(),
                        None::<()>,
                    )
                })?
                .clone()
        };

        let mut contract_delete_tx = ContractDeleteTransaction::new();

        if let Some(contract_id) = contract_id {
            contract_delete_tx.contract_id(
                ContractId::from_str(&contract_id).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }

        // Note: The Rust SDK doesn't expose permanent_removal as a settable field,
        // it's hardcoded to false in the protobuf conversion
        let _ = permanent_removal;

        // Depend on how I order transferContractId and transferAccountId the last will stay if both are called
        if let Some(transfer_contract_id) = transfer_contract_id {
            contract_delete_tx.transfer_contract_id(
                ContractId::from_str(&transfer_contract_id).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }

        if let Some(transfer_account_id) = transfer_account_id {
            contract_delete_tx.transfer_account_id(
                AccountId::from_str(&transfer_account_id).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }

        if let Some(common_transaction_params) = common_transaction_params {
            let _ = fill_common_transaction_params(&mut contract_delete_tx, &common_transaction_params);

            contract_delete_tx.freeze_with(&client).unwrap();

            if let Some(signers) = common_transaction_params.get("signers") {
                if let Value::Array(signers) = signers {
                    for signer in signers {
                        if let Value::String(signer_str) = signer {
                            contract_delete_tx.sign(PrivateKey::from_str_der(signer_str).unwrap());
                        }
                    }
                }
            }
        }

        let tx_response =
            contract_delete_tx.execute(&client).await.map_err(|e| from_hedera_error(e))?;

        let tx_receipt =
            tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

        Ok(ContractResponse {
            contract_id: None,
            status: tx_receipt.status.as_str_name().to_string(),
        })
    }

    async fn execute_contract(
        &self,
        contract_id: Option<String>,
        gas: Option<String>,
        amount: Option<String>,
        function_parameters: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ContractResponse, ErrorObjectOwned> {
        let client = {
            let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
            guard
                .as_ref()
                .ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Client not initialized".to_string(),
                        None::<()>,
                    )
                })?
                .clone()
        };

        let mut contract_execute_tx = ContractExecuteTransaction::new();

        if let Some(contract_id) = contract_id {
            contract_execute_tx.contract_id(
                ContractId::from_str(&contract_id).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }

        if let Some(gas) = gas {
            let gas_value = gas
                .parse::<u64>()
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
            contract_execute_tx.gas(gas_value);
        }

        if let Some(amount) = amount {
            let amount_value = amount
                .parse::<i64>()
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
            contract_execute_tx.payable_amount(Hbar::from_tinybars(amount_value));
        }

        if let Some(function_parameters) = function_parameters {
            let function_params = hex::decode(&function_parameters).map_err(|e| {
                ErrorObject::owned(INTERNAL_ERROR_CODE, format!("Invalid hex: {e}"), None::<()>)
            })?;
            contract_execute_tx.function_parameters(function_params);
        }

        if let Some(common_transaction_params) = common_transaction_params {
            let _ = fill_common_transaction_params(&mut contract_execute_tx, &common_transaction_params);

            contract_execute_tx.freeze_with(&client).unwrap();

            if let Some(signers) = common_transaction_params.get("signers") {
                if let Value::Array(signers) = signers {
                    for signer in signers {
                        if let Value::String(signer_str) = signer {
                            contract_execute_tx.sign(PrivateKey::from_str_der(signer_str).unwrap());
                        }
                    }
                }
            }
        }

        let tx_response =
            contract_execute_tx.execute(&client).await.map_err(|e| from_hedera_error(e))?;

        let tx_receipt =
            tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

        Ok(ContractResponse {
            contract_id: None,
            status: tx_receipt.status.as_str_name().to_string(),
        })
    }

    async fn create_topic(
        &self,
        memo: Option<String>,
        admin_key: Option<String>,
        submit_key: Option<String>,
        auto_renew_period: Option<String>,
        auto_renew_account_id: Option<String>,
        fee_schedule_key: Option<String>,
        fee_exempt_keys: Option<Vec<String>>,
        custom_fees: Option<Value>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TopicResponse, ErrorObjectOwned> {
        let client = {
            let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
            guard
                .as_ref()
                .ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Client not initialized".to_string(),
                        None::<()>,
                    )
                })?
                .clone()
        };

        let mut topic_create_tx = TopicCreateTransaction::new();

        if let Some(memo) = memo {
            topic_create_tx.topic_memo(memo);
        }

        if let Some(admin_key) = admin_key {
            let key = get_hedera_key(&admin_key)?;
            topic_create_tx.admin_key(key);
        }

        if let Some(submit_key) = submit_key {
            let key = get_hedera_key(&submit_key)?;
            topic_create_tx.submit_key(key);
        }

        if let Some(auto_renew_period) = auto_renew_period {
            let period = auto_renew_period
                .parse::<i64>()
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
            topic_create_tx.auto_renew_period(Duration::seconds(period));
        }

        if let Some(auto_renew_account_id) = auto_renew_account_id {
            topic_create_tx.auto_renew_account_id(
                AccountId::from_str(&auto_renew_account_id).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }

        if let Some(fee_schedule_key) = fee_schedule_key {
            let key = get_hedera_key(&fee_schedule_key)?;
            topic_create_tx.fee_schedule_key(key);
        }

        if let Some(fee_exempt_keys) = fee_exempt_keys {
            if !fee_exempt_keys.is_empty() {
                let keys: Result<Vec<_>, _> = fee_exempt_keys
                    .iter()
                    .map(|k| get_hedera_key(k))
                    .collect();
                topic_create_tx.fee_exempt_keys(keys?);
            }
        }

        if let Some(custom_fees) = custom_fees {
            if let Value::Array(fees_array) = custom_fees {
                if !fees_array.is_empty() {
                    let mut sdk_custom_fees = Vec::new();
                    for fee in fees_array {
                        if let Value::Object(fee_obj) = fee {
                            let fixed_fee_obj = fee_obj
                                .get("fixedFee")
                                .ok_or_else(|| {
                                    ErrorObject::owned(
                                        INTERNAL_ERROR_CODE,
                                        "Missing fixedFee in custom fee".to_string(),
                                        None::<()>,
                                    )
                                })?;

                            if let Value::Object(fixed_fee) = fixed_fee_obj {
                                let amount_str = fixed_fee
                                    .get("amount")
                                    .and_then(|v| v.as_str())
                                    .ok_or_else(|| {
                                        ErrorObject::owned(
                                            INTERNAL_ERROR_CODE,
                                            "Missing amount in fixedFee".to_string(),
                                            None::<()>,
                                        )
                                    })?;

                                let amount = amount_str
                                    .parse::<u64>()
                                    .map_err(|e| {
                                        ErrorObject::owned(
                                            INTERNAL_ERROR_CODE,
                                            format!("Invalid amount: {e}"),
                                            None::<()>,
                                        )
                                    })?;

                                let denominating_token_id = fixed_fee
                                    .get("denominatingTokenId")
                                    .and_then(|v| v.as_str())
                                    .map(|s| {
                                        TokenId::from_str(s).map_err(|e| {
                                            ErrorObject::owned(
                                                INTERNAL_ERROR_CODE,
                                                format!("Invalid token ID: {e}"),
                                                None::<()>,
                                            )
                                        })
                                    })
                                    .transpose()?;

                                let fee_collector_account_id = fee_obj
                                    .get("feeCollectorAccountId")
                                    .and_then(|v| v.as_str())
                                    .map(|s| {
                                        AccountId::from_str(s).map_err(|e| {
                                            ErrorObject::owned(
                                                INTERNAL_ERROR_CODE,
                                                format!("Invalid account ID: {e}"),
                                                None::<()>,
                                            )
                                        })
                                    })
                                    .transpose()?;

                                sdk_custom_fees.push(CustomFixedFee::new(
                                    amount,
                                    denominating_token_id,
                                    fee_collector_account_id,
                                ));
                            }
                        }
                    }
                    topic_create_tx.custom_fees(sdk_custom_fees);
                }
            }
        }

        if let Some(common_transaction_params) = common_transaction_params {
            let _ = fill_common_transaction_params(&mut topic_create_tx, &common_transaction_params);

            topic_create_tx.freeze_with(&client).unwrap();

            if let Some(signers) = common_transaction_params.get("signers") {
                if let Value::Array(signers) = signers {
                    for signer in signers {
                        if let Value::String(signer_str) = signer {
                            topic_create_tx.sign(PrivateKey::from_str_der(signer_str).unwrap());
                        }
                    }
                }
            }
        }

        let tx_response =
            topic_create_tx.execute(&client).await.map_err(|e| from_hedera_error(e))?;

        let tx_receipt =
            tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

        Ok(TopicResponse {
            topic_id: tx_receipt.topic_id.map(|id| id.to_string()),
            status: tx_receipt.status.as_str_name().to_string(),
        })
    }

    async fn update_topic(
        &self,
        topic_id: Option<String>,
        memo: Option<String>,
        admin_key: Option<String>,
        submit_key: Option<String>,
        auto_renew_period: Option<String>,
        auto_renew_account_id: Option<String>,
        expiration_time: Option<String>,
        fee_schedule_key: Option<String>,
        fee_exempt_keys: Option<Vec<String>>,
        custom_fees: Option<Value>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TopicResponse, ErrorObjectOwned> {
        let client = {
            let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
            guard
                .as_ref()
                .ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Client not initialized".to_string(),
                        None::<()>,
                    )
                })?
                .clone()
        };

        let mut topic_update_tx = TopicUpdateTransaction::new();

        if let Some(topic_id) = topic_id {
            topic_update_tx.topic_id(
                TopicId::from_str(&topic_id).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }

        if let Some(memo) = memo {
            topic_update_tx.topic_memo(memo);
        }

        if let Some(admin_key) = admin_key {
            let key = get_hedera_key(&admin_key)?;
            topic_update_tx.admin_key(key);
        }

        if let Some(submit_key) = submit_key {
            let key = get_hedera_key(&submit_key)?;
            topic_update_tx.submit_key(key);
        }

        if let Some(auto_renew_period) = auto_renew_period {
            let period = auto_renew_period
                .parse::<i64>()
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
            topic_update_tx.auto_renew_period(Duration::seconds(period));
        }

        if let Some(auto_renew_account_id) = auto_renew_account_id {
            topic_update_tx.auto_renew_account_id(
                AccountId::from_str(&auto_renew_account_id).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }

        if let Some(expiration_time) = expiration_time {
            let timestamp = expiration_time
                .parse::<i64>()
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
            topic_update_tx.expiration_time(
                OffsetDateTime::from_unix_timestamp(timestamp).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }

        if let Some(fee_schedule_key) = fee_schedule_key {
            let key = get_hedera_key(&fee_schedule_key)?;
            topic_update_tx.fee_schedule_key(key);
        }

        if let Some(fee_exempt_keys) = fee_exempt_keys {
            if fee_exempt_keys.is_empty() {
                topic_update_tx.clear_fee_exempt_keys();
            } else {
                let keys: Result<Vec<_>, _> = fee_exempt_keys
                    .iter()
                    .map(|k| get_hedera_key(k))
                    .collect();
                topic_update_tx.fee_exempt_keys(keys?);
            }
        }

        if let Some(custom_fees) = custom_fees {
            if let Value::Array(fees_array) = custom_fees {
                if fees_array.is_empty() {
                    topic_update_tx.clear_custom_fees();
                } else {
                    let mut sdk_custom_fees = Vec::new();
                    for fee in fees_array {
                        if let Value::Object(fee_obj) = fee {
                            let fixed_fee_obj = fee_obj
                                .get("fixedFee")
                                .ok_or_else(|| {
                                    ErrorObject::owned(
                                        INTERNAL_ERROR_CODE,
                                        "Missing fixedFee in custom fee".to_string(),
                                        None::<()>,
                                    )
                                })?;

                            if let Value::Object(fixed_fee) = fixed_fee_obj {
                                let amount_str = fixed_fee
                                    .get("amount")
                                    .and_then(|v| v.as_str())
                                    .ok_or_else(|| {
                                        ErrorObject::owned(
                                            INTERNAL_ERROR_CODE,
                                            "Missing amount in fixedFee".to_string(),
                                            None::<()>,
                                        )
                                    })?;

                                let amount = amount_str
                                    .parse::<u64>()
                                    .map_err(|e| {
                                        ErrorObject::owned(
                                            INTERNAL_ERROR_CODE,
                                            format!("Invalid amount: {e}"),
                                            None::<()>,
                                        )
                                    })?;

                                let denominating_token_id = fixed_fee
                                    .get("denominatingTokenId")
                                    .and_then(|v| v.as_str())
                                    .map(|s| {
                                        TokenId::from_str(s).map_err(|e| {
                                            ErrorObject::owned(
                                                INTERNAL_ERROR_CODE,
                                                format!("Invalid token ID: {e}"),
                                                None::<()>,
                                            )
                                        })
                                    })
                                    .transpose()?;

                                let fee_collector_account_id = fee_obj
                                    .get("feeCollectorAccountId")
                                    .and_then(|v| v.as_str())
                                    .map(|s| {
                                        AccountId::from_str(s).map_err(|e| {
                                            ErrorObject::owned(
                                                INTERNAL_ERROR_CODE,
                                                format!("Invalid account ID: {e}"),
                                                None::<()>,
                                            )
                                        })
                                    })
                                    .transpose()?;

                                sdk_custom_fees.push(CustomFixedFee::new(
                                    amount,
                                    denominating_token_id,
                                    fee_collector_account_id,
                                ));
                            }
                        }
                    }
                    topic_update_tx.custom_fees(sdk_custom_fees);
                }
            }
        }

        if let Some(common_transaction_params) = common_transaction_params {
            let _ = fill_common_transaction_params(&mut topic_update_tx, &common_transaction_params);

            topic_update_tx.freeze_with(&client).unwrap();

            if let Some(signers) = common_transaction_params.get("signers") {
                if let Value::Array(signers) = signers {
                    for signer in signers {
                        if let Value::String(signer_str) = signer {
                            topic_update_tx.sign(PrivateKey::from_str_der(signer_str).unwrap());
                        }
                    }
                }
            }
        }

        let tx_response =
            topic_update_tx.execute(&client).await.map_err(|e| from_hedera_error(e))?;

        let tx_receipt =
            tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

        Ok(TopicResponse {
            topic_id: None,
            status: tx_receipt.status.as_str_name().to_string(),
        })
    }

    async fn delete_topic(
        &self,
        topic_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TopicResponse, ErrorObjectOwned> {
        let client = {
            let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
            guard
                .as_ref()
                .ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Client not initialized".to_string(),
                        None::<()>,
                    )
                })?
                .clone()
        };

        let mut topic_delete_tx = TopicDeleteTransaction::new();

        if let Some(topic_id) = topic_id {
            topic_delete_tx.topic_id(
                TopicId::from_str(&topic_id).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }

        if let Some(common_transaction_params) = common_transaction_params {
            let _ = fill_common_transaction_params(&mut topic_delete_tx, &common_transaction_params);

            topic_delete_tx.freeze_with(&client).unwrap();

            if let Some(signers) = common_transaction_params.get("signers") {
                if let Value::Array(signers) = signers {
                    for signer in signers {
                        if let Value::String(signer_str) = signer {
                            topic_delete_tx.sign(PrivateKey::from_str_der(signer_str).unwrap());
                        }
                    }
                }
            }
        }

        let tx_response =
            topic_delete_tx.execute(&client).await.map_err(|e| from_hedera_error(e))?;

        let tx_receipt =
            tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

        Ok(TopicResponse {
            topic_id: None,
            status: tx_receipt.status.as_str_name().to_string(),
        })
    }

    async fn submit_topic_message(
        &self,
        topic_id: Option<String>,
        message: Option<String>,
        max_chunks: Option<usize>,
        chunk_size: Option<usize>,
        custom_fee_limits: Option<Value>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TopicResponse, ErrorObjectOwned> {
        let client = {
            let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
            guard
                .as_ref()
                .ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Client not initialized".to_string(),
                        None::<()>,
                    )
                })?
                .clone()
        };
        let mut topic_message_submit_tx = TopicMessageSubmitTransaction::new();

        if let Some(topic_id) = topic_id {
            topic_message_submit_tx.topic_id(
                TopicId::from_str(&topic_id).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }

        match message {
            Some(msg) if !msg.is_empty() => {
                topic_message_submit_tx.message(msg.into_bytes());
            }
            _ => {
                return Err(ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "Message is required".to_string(),
                    None::<()>,
                ));
            }
        }

        if let Some(max_chunks) = max_chunks {
            topic_message_submit_tx.max_chunks(max_chunks);
        }

        if let Some(chunk_size) = chunk_size {
            if chunk_size == 0 {
                return Err(ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "Chunk size must be greater than zero".to_string(),
                    None::<()>,
                ));
            }
            topic_message_submit_tx.chunk_size(chunk_size);
        }
        if let Some(custom_fee_limits) = custom_fee_limits {
            if let Value::Array(limits_array) = custom_fee_limits {
                let mut sdk_custom_fee_limits = Vec::new();
                for limit in limits_array {
                    if let Value::Object(limit_obj) = limit {
                        let payer_id = limit_obj
                            .get("payerId")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                ErrorObject::owned(
                                    INTERNAL_ERROR_CODE,
                                    "Missing payerId in customFeeLimit".to_string(),
                                    None::<()>,
                                )
                            })?;

                        let account_id = AccountId::from_str(payer_id).map_err(|e| {
                            ErrorObject::owned(
                                INTERNAL_ERROR_CODE,
                                format!("Invalid payerId: {e}"),
                                None::<()>,
                            )
                        })?;

                        let fixed_fees_array = limit_obj
                            .get("fixedFees")
                            .and_then(|v| v.as_array())
                            .ok_or_else(|| {
                                ErrorObject::owned(
                                    INTERNAL_ERROR_CODE,
                                    "Missing fixedFees in customFeeLimit".to_string(),
                                    None::<()>,
                                )
                            })?;

                        let mut custom_fixed_fees = Vec::new();
                        for fee in fixed_fees_array {
                            if let Value::Object(fee_obj) = fee {
                                let amount_str = fee_obj
                                    .get("amount")
                                    .and_then(|v| v.as_str())
                                    .ok_or_else(|| {
                                        ErrorObject::owned(
                                            INTERNAL_ERROR_CODE,
                                            "Missing amount in fixedFee".to_string(),
                                            None::<()>,
                                        )
                                    })?;

                                let amount = amount_str
                                    .parse::<u64>()
                                    .map_err(|e| {
                                        ErrorObject::owned(
                                            INTERNAL_ERROR_CODE,
                                            format!("Invalid amount: {e}"),
                                            None::<()>,
                                        )
                                    })?;

                                let denominating_token_id = fee_obj
                                    .get("denominatingTokenId")
                                    .and_then(|v| v.as_str())
                                    .map(|s| {
                                        TokenId::from_str(s).map_err(|e| {
                                            ErrorObject::owned(
                                                INTERNAL_ERROR_CODE,
                                                format!("Invalid token ID: {e}"),
                                                None::<()>,
                                            )
                                        })
                                    })
                                    .transpose()?;

                                custom_fixed_fees.push(CustomFixedFee::new(
                                    amount,
                                    denominating_token_id,
                                    None,
                                ));
                            }
                        }

                        sdk_custom_fee_limits.push(CustomFeeLimit::new(
                            Some(account_id),
                            custom_fixed_fees,
                        ));
                    }
                }
                topic_message_submit_tx.custom_fee_limits(sdk_custom_fee_limits);
            }
        }

        if let Some(common_transaction_params) = common_transaction_params {
            let _ = fill_common_transaction_params(
                &mut topic_message_submit_tx,
                &common_transaction_params,
            );

            topic_message_submit_tx.freeze_with(&client).unwrap();

            if let Some(signers) = common_transaction_params.get("signers") {
                if let Value::Array(signers) = signers {
                    for signer in signers {
                        if let Value::String(signer_str) = signer {
                            topic_message_submit_tx
                                .sign(PrivateKey::from_str_der(signer_str).unwrap());
                        }
                    }
                }
            }
        }
        let tx_response = topic_message_submit_tx
            .execute(&client)
            .await
            .map_err(|e| from_hedera_error(e))?;
        let tx_receipt =
            tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

        Ok(TopicResponse {
            topic_id: None,
            status: tx_receipt.status.as_str_name().to_string(),
        })
    }

    async fn create_schedule(
        &self,
        scheduled_transaction: Option<Value>,
        memo: Option<String>,
        admin_key: Option<String>,
        payer_account_id: Option<String>,
        expiration_time: Option<String>,
        wait_for_expiry: Option<bool>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ScheduleResponse, ErrorObjectOwned> {
        let client = {
            let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
            guard
                .as_ref()
                .ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Client not initialized".to_string(),
                        None::<()>,
                    )
                })?
                .clone()
        };

        let mut schedule_create_tx = ScheduleCreateTransaction::new();

        if let Some(scheduled_transaction) = &scheduled_transaction {
            build_scheduled_transaction(scheduled_transaction, &client, &mut schedule_create_tx)?;
        }

        if let Some(memo) = memo {
            schedule_create_tx.schedule_memo(memo);
        }

        if let Some(admin_key) = admin_key {
            let key = get_hedera_key(&admin_key)?;
            schedule_create_tx.admin_key(key);
        }

        if let Some(payer_account_id) = payer_account_id {
            schedule_create_tx.payer_account_id(
                AccountId::from_str(&payer_account_id).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }

        if let Some(expiration_time) = expiration_time {
            let ts = expiration_time
                .parse::<i64>()
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
            schedule_create_tx.expiration_time(
                OffsetDateTime::from_unix_timestamp(ts)
                    .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?,
            );
        }

        if let Some(wait_for_expiry) = wait_for_expiry {
            schedule_create_tx.wait_for_expiry(wait_for_expiry);
        }

        if let Some(common_transaction_params) = common_transaction_params {
            let _ = fill_common_transaction_params(&mut schedule_create_tx, &common_transaction_params);

            schedule_create_tx.freeze_with(&client).unwrap();

            if let Some(signers) = common_transaction_params.get("signers") {
                if let Value::Array(signers) = signers {
                    for signer in signers {
                        if let Value::String(signer_str) = signer {
                            schedule_create_tx.sign(PrivateKey::from_str_der(signer_str).unwrap());
                        }
                    }
                }
            }
        }

        let tx_response =
            schedule_create_tx.execute(&client).await.map_err(|e| from_hedera_error(e))?;

        let tx_receipt =
            tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

        Ok(ScheduleResponse {
            schedule_id: tx_receipt.schedule_id.map(|id| id.to_string()),
            transaction_id: tx_receipt.scheduled_transaction_id.map(|id| id.to_string()),
            status: tx_receipt.status.as_str_name().to_string(),
        })
    }

    async fn sign_schedule(
        &self,
        schedule_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ScheduleResponse, ErrorObjectOwned> {
        let client = {
            let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
            guard
                .as_ref()
                .ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Client not initialized".to_string(),
                        None::<()>,
                    )
                })?
                .clone()
        };

        let mut schedule_sign_tx = ScheduleSignTransaction::new();

        if let Some(schedule_id) = schedule_id {
            let sid = ScheduleId::from_str(&schedule_id).map_err(|e| {
                ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
            })?;
            schedule_sign_tx.schedule_id(sid);
        }

        if let Some(common_transaction_params) = common_transaction_params {
            let _ = fill_common_transaction_params(&mut schedule_sign_tx, &common_transaction_params);

            schedule_sign_tx.freeze_with(&client).unwrap();

            if let Some(signers) = common_transaction_params.get("signers") {
                if let Value::Array(signers) = signers {
                    for signer in signers {
                        if let Value::String(signer_str) = signer {
                            schedule_sign_tx.sign(PrivateKey::from_str_der(signer_str).unwrap());
                        }
                    }
                }
            }
        }

        let tx_response =
            schedule_sign_tx.execute(&client).await.map_err(|e| from_hedera_error(e))?;

        let tx_receipt =
            tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

        Ok(ScheduleResponse {
            schedule_id: None,
            transaction_id: None,
            status: tx_receipt.status.as_str_name().to_string(),
        })
    }

    async fn delete_schedule(
        &self,
        schedule_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ScheduleResponse, ErrorObjectOwned> {
        let client = {
            let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
            guard
                .as_ref()
                .ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Client not initialized".to_string(),
                        None::<()>,
                    )
                })?
                .clone()
        };

        let mut schedule_delete_tx = ScheduleDeleteTransaction::new();

        if let Some(schedule_id) = schedule_id {
            let sid = ScheduleId::from_str(&schedule_id).map_err(|e| {
                ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
            })?;
            schedule_delete_tx.schedule_id(sid);
        }

        if let Some(common_transaction_params) = common_transaction_params {
            let _ = fill_common_transaction_params(&mut schedule_delete_tx, &common_transaction_params);

            schedule_delete_tx.freeze_with(&client).unwrap();

            if let Some(signers) = common_transaction_params.get("signers") {
                if let Value::Array(signers) = signers {
                    for signer in signers {
                        if let Value::String(signer_str) = signer {
                            schedule_delete_tx.sign(PrivateKey::from_str_der(signer_str).unwrap());
                        }
                    }
                }
            }
        }

        let tx_response =
            schedule_delete_tx.execute(&client).await.map_err(|e| from_hedera_error(e))?;

        let tx_receipt =
            tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

        Ok(ScheduleResponse {
            schedule_id: None,
            transaction_id: None,
            status: tx_receipt.status.as_str_name().to_string(),
        })
    }

    async fn get_account_balance(
        &self,
        account_id: Option<String>,
        contract_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<AccountBalanceResponse, ErrorObjectOwned> {
        let _ = common_transaction_params;
        let client = {
            let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
            guard
                .as_ref()
                .ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Client not initialized".to_string(),
                        None::<()>,
                    )
                })?
                .clone()
        };

        let mut query = AccountBalanceQuery::new();

        if let Some(account_id) = account_id {
            query.account_id(
                AccountId::from_str(&account_id).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }

        if let Some(contract_id) = contract_id {
            query.contract_id(
                ContractId::from_str(&contract_id).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }

        let tx_response = query.execute(&client).await.map_err(|e| from_hedera_error(e))?;

        let mut token_balances = HashMap::new();
        for (token_id, amount) in tx_response.tokens {
            token_balances.insert(token_id.to_string(), amount.to_string());
        }

        #[allow(deprecated)]
        let mut token_decimals = HashMap::new();
        #[allow(deprecated)]
        for (token_id, decimals) in tx_response.token_decimals {
            token_decimals.insert(token_id.to_string(), decimals);
        }

        Ok(AccountBalanceResponse {
            hbars: tx_response.hbars.to_tinybars().to_string(),
            token_balances,
            token_decimals,
        })
    }

    async fn contract_call_query(
        &self,
        contract_id: Option<String>,
        gas: Option<String>,
        function_parameters: Option<String>,
        sender_account_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ContractCallResponse, ErrorObjectOwned> {
        let _ = common_transaction_params;
        let client = {
            let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
            guard
                .as_ref()
                .ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Client not initialized".to_string(),
                        None::<()>,
                    )
                })?
                .clone()
        };

        let mut query = ContractCallQuery::new();
        if let Some(contract_id) = contract_id {
            query.contract_id(
                ContractId::from_str(&contract_id).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }
        if let Some(gas) = gas {
            let gas_val = gas
                .parse::<u64>()
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
            query.gas(gas_val);
        }
        if let Some(sender_account_id) = sender_account_id {
            query.sender_account_id(
                AccountId::from_str(&sender_account_id).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }
        if let Some(function_parameters) = function_parameters {
            let bytes = hex::decode(&function_parameters).map_err(|e| {
                ErrorObject::owned(INTERNAL_ERROR_CODE, format!("Invalid hex: {e}"), None::<()>)
            })?;
            query.function_parameters(bytes);
        }

        let result = query.execute(&client).await.map_err(|e| from_hedera_error(e))?;
        Ok(ContractCallResponse {
            contract_id: Some(result.contract_id.to_string()),
            gas_used: result.gas,
            result: hex::encode(result.bytes),
        })
    }

    async fn contract_info_query(
        &self,
        contract_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<InfoResponse, ErrorObjectOwned> {
        let client = {
            let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
            guard
                .as_ref()
                .ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Client not initialized".to_string(),
                        None::<()>,
                    )
                })?
                .clone()
        };

        let mut query = ContractInfoQuery::new();
        if let Some(contract_id) = contract_id {
            query.contract_id(
                ContractId::from_str(&contract_id).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }
        let _ = common_transaction_params;
        let info = query.execute(&client).await.map_err(|e| from_hedera_error(e))?;
        Ok(InfoResponse { info: format!("{info:?}") })
    }

    async fn topic_info_query(
        &self,
        topic_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<InfoResponse, ErrorObjectOwned> {
        let client = {
            let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
            guard
                .as_ref()
                .ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Client not initialized".to_string(),
                        None::<()>,
                    )
                })?
                .clone()
        };

        let mut query = TopicInfoQuery::new();
        if let Some(topic_id) = topic_id {
            query.topic_id(
                TopicId::from_str(&topic_id)
                    .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?,
            );
        }
        let _ = common_transaction_params;
        let info = query.execute(&client).await.map_err(|e| from_hedera_error(e))?;
        Ok(InfoResponse { info: format!("{info:?}") })
    }

    async fn schedule_info_query(
        &self,
        schedule_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<InfoResponse, ErrorObjectOwned> {
        let client = {
            let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
            guard
                .as_ref()
                .ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Client not initialized".to_string(),
                        None::<()>,
                    )
                })?
                .clone()
        };

        let mut query = ScheduleInfoQuery::new();
        if let Some(schedule_id) = schedule_id {
            query.schedule_id(
                ScheduleId::from_str(&schedule_id).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
                })?,
            );
        }
        let _ = common_transaction_params;
        let info = query.execute(&client).await.map_err(|e| from_hedera_error(e))?;
        Ok(InfoResponse { info: format!("{info:?}") })
    }
}

fn build_scheduled_transaction(
    scheduled_tx: &Value,
    _client: &Client,
    schedule_tx: &mut ScheduleCreateTransaction,
) -> Result<(), ErrorObjectOwned> {
    let method = scheduled_tx
        .get("method")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "scheduledTransaction.method is required".to_string(),
                None::<()>,
            )
        })?;

    let params = scheduled_tx.get("params").unwrap_or(&Value::Null).clone();

    match method {
        "createTopic" => {
            let tx = build_topic_create_tx_from_value(&params)?;
            schedule_tx.scheduled_transaction(tx);
            Ok(())
        }
        "submitMessage" => {
            let tx = build_topic_message_submit_tx_from_value(&params)?;
            schedule_tx.scheduled_transaction(tx);
            Ok(())
        }
        "createAccount" => {
            let tx = build_account_create_tx_from_value(&params)?;
            schedule_tx.scheduled_transaction(tx);
            Ok(())
        }
        "transferCrypto" => {
            let tx = build_transfer_tx_from_value(&params)?;
            schedule_tx.scheduled_transaction(tx);
            Ok(())
        }
        "approveAllowance" => {
            let tx = build_allowance_approve_tx_from_value(&params)?;
            schedule_tx.scheduled_transaction(tx);
            Ok(())
        }
        "mintToken" => {
            let tx = build_token_mint_tx_from_value(&params)?;
            schedule_tx.scheduled_transaction(tx);
            Ok(())
        }
        "burnToken" => {
            let tx = build_token_burn_tx_from_value(&params)?;
            schedule_tx.scheduled_transaction(tx);
            Ok(())
        }
        _ => Err(ErrorObject::owned(
            INTERNAL_ERROR_CODE,
            format!("Unsupported scheduled transaction method: {method}"),
            None::<()>,
        )),
    }
}

fn build_account_create_tx_from_value(params: &Value) -> Result<AccountCreateTransaction, ErrorObjectOwned> {
    let mut tx = AccountCreateTransaction::new();

    if let Some(key) = params.get("key").and_then(Value::as_str) {
        let key = get_hedera_key(key)?;
        tx.set_key_without_alias(key);
    }

    if let Some(initial_balance) = params.get("initialBalance").and_then(Value::as_i64) {
        tx.initial_balance(Hbar::from_tinybars(initial_balance));
    }

    if let Some(receiver_signature_required) =
        params.get("receiverSignatureRequired").and_then(Value::as_bool)
    {
        tx.receiver_signature_required(receiver_signature_required);
    }

    if let Some(auto_renew_period) = params.get("autoRenewPeriod").and_then(Value::as_i64) {
        tx.auto_renew_period(Duration::seconds(auto_renew_period));
    }

    if let Some(memo) = params.get("memo").and_then(Value::as_str) {
        tx.account_memo(memo.to_string());
    }

    if let Some(max_auto_token_associations) =
        params.get("maxAutoTokenAssociations").and_then(Value::as_i64)
    {
        tx.max_automatic_token_associations(max_auto_token_associations as i32);
    }

    if let Some(staked_account_id) = params.get("stakedAccountId").and_then(Value::as_str) {
        tx.staked_account_id(
            AccountId::from_str(staked_account_id)
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?,
        );
    }

    if let Some(staked_node_id) = params.get("stakedNodeId").and_then(Value::as_str) {
        let node_id = staked_node_id
            .parse::<i64>()
            .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
        if node_id < 0 {
            return Err(ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "INVALID_STAKING_ID",
                None::<()>,
            ));
        }
        tx.staked_node_id(node_id as u64);
    }

    if let Some(decline_staking_reward) =
        params.get("declineStakingReward").and_then(Value::as_bool)
    {
        tx.decline_staking_reward(decline_staking_reward);
    }

    if let Some(alias) = params.get("alias").and_then(Value::as_str) {
        tx.alias(
            EvmAddress::from_str(alias)
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?,
        );
    }

    Ok(tx)
}

fn build_topic_create_tx_from_value(params: &Value) -> Result<TopicCreateTransaction, ErrorObjectOwned> {
    let mut tx = TopicCreateTransaction::new();

    if let Some(memo) = params.get("memo").and_then(Value::as_str) {
        tx.topic_memo(memo.to_string());
    }

    if let Some(admin_key) = params.get("adminKey").and_then(Value::as_str) {
        let key = get_hedera_key(admin_key)?;
        tx.admin_key(key);
    }

    if let Some(submit_key) = params.get("submitKey").and_then(Value::as_str) {
        let key = get_hedera_key(submit_key)?;
        tx.submit_key(key);
    }

    if let Some(auto_renew_period) = params.get("autoRenewPeriod").and_then(Value::as_str) {
        let period = auto_renew_period.parse::<i64>().map_err(|e| {
            ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
        })?;
        tx.auto_renew_period(Duration::seconds(period));
    }

    if let Some(auto_renew_account_id) = params.get("autoRenewAccountId").and_then(Value::as_str) {
        tx.auto_renew_account_id(
            AccountId::from_str(auto_renew_account_id)
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?,
        );
    }

    if let Some(fee_schedule_key) = params.get("feeScheduleKey").and_then(Value::as_str) {
        let key = get_hedera_key(fee_schedule_key)?;
        tx.fee_schedule_key(key);
    }

    if let Some(fee_exempt_keys) = params.get("feeExemptKeys").and_then(Value::as_array) {
        if !fee_exempt_keys.is_empty() {
            let mut keys = Vec::new();
            for k in fee_exempt_keys {
                if let Some(kstr) = k.as_str() {
                    keys.push(get_hedera_key(kstr)?);
                }
            }
            tx.fee_exempt_keys(keys);
        } else {
            tx.clear_fee_exempt_keys();
        }
    }

    if let Some(custom_fees) = params.get("customFees").and_then(Value::as_array) {
        if !custom_fees.is_empty() {
            let mut fees = Vec::new();
            for fee in custom_fees {
                if let Value::Object(fee_obj) = fee {
                    let fixed_fee_obj = fee_obj.get("fixedFee").ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "Missing fixedFee in custom fee".to_string(),
                            None::<()>,
                        )
                    })?;

                    if let Value::Object(fixed_fee) = fixed_fee_obj {
                        let amount_str = fixed_fee
                            .get("amount")
                            .and_then(Value::as_str)
                            .ok_or_else(|| {
                                ErrorObject::owned(
                                    INTERNAL_ERROR_CODE,
                                    "Missing amount in fixedFee".to_string(),
                                    None::<()>,
                                )
                            })?;

                        let amount = amount_str.parse::<u64>().map_err(|e| {
                            ErrorObject::owned(
                                INTERNAL_ERROR_CODE,
                                format!("Invalid amount: {e}"),
                                None::<()>,
                            )
                        })?;

                        let denominating_token_id = fixed_fee
                            .get("denominatingTokenId")
                            .and_then(Value::as_str)
                            .map(|s| {
                                TokenId::from_str(s).map_err(|e| {
                                    ErrorObject::owned(
                                        INTERNAL_ERROR_CODE,
                                        format!("Invalid token ID: {e}"),
                                        None::<()>,
                                    )
                                })
                            })
                            .transpose()?;

                        let fee_collector_account_id = fee_obj
                            .get("feeCollectorAccountId")
                            .and_then(Value::as_str)
                            .map(|s| {
                                AccountId::from_str(s).map_err(|e| {
                                    ErrorObject::owned(
                                        INTERNAL_ERROR_CODE,
                                        format!("Invalid account ID: {e}"),
                                        None::<()>,
                                    )
                                })
                            })
                            .transpose()?;

                        fees.push(CustomFixedFee::new(amount, denominating_token_id, fee_collector_account_id));
                    }
                }
            }
            tx.custom_fees(fees);
        }
    }

    Ok(tx)
}

fn build_topic_message_submit_tx_from_value(
    params: &Value,
) -> Result<TopicMessageSubmitTransaction, ErrorObjectOwned> {
    let mut tx = TopicMessageSubmitTransaction::new();

    if let Some(topic_id) = params.get("topicId").and_then(Value::as_str) {
        tx.topic_id(
            TopicId::from_str(topic_id)
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?,
        );
    }

    match params.get("message").and_then(Value::as_str) {
        Some(msg) if !msg.is_empty() => {
            tx.message(msg.as_bytes());
        }
        _ => {
            return Err(ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Message is required".to_string(),
                None::<()>,
            ));
        }
    }

    if let Some(max_chunks) = params.get("maxChunks").and_then(Value::as_u64) {
        tx.max_chunks(max_chunks as usize);
    }

    if let Some(chunk_size) = params.get("chunkSize").and_then(Value::as_u64) {
        if chunk_size == 0 {
            return Err(ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Chunk size must be greater than zero".to_string(),
                None::<()>,
            ));
        }
        tx.chunk_size(chunk_size as usize);
    }

    if let Some(custom_fee_limits) = params.get("customFeeLimits").and_then(Value::as_array) {
        let mut limits = Vec::new();
        for limit in custom_fee_limits {
            if let Value::Object(limit_obj) = limit {
                let payer_id = limit_obj
                    .get("payerId")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "Missing payerId in customFeeLimit".to_string(),
                            None::<()>,
                        )
                    })?;

                let account_id = AccountId::from_str(payer_id).map_err(|e| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        format!("Invalid payerId: {e}"),
                        None::<()>,
                    )
                })?;

                let fixed_fees_array = limit_obj
                    .get("fixedFees")
                    .and_then(Value::as_array)
                    .ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "Missing fixedFees in customFeeLimit".to_string(),
                            None::<()>,
                        )
                    })?;

                let mut fixed_fees = Vec::new();
                for fee in fixed_fees_array {
                    if let Value::Object(fee_obj) = fee {
                        let amount_str = fee_obj
                            .get("amount")
                            .and_then(Value::as_str)
                            .ok_or_else(|| {
                                ErrorObject::owned(
                                    INTERNAL_ERROR_CODE,
                                    "Missing amount in fixedFee".to_string(),
                                    None::<()>,
                                )
                            })?;

                        let amount = amount_str.parse::<u64>().map_err(|e| {
                            ErrorObject::owned(
                                INTERNAL_ERROR_CODE,
                                format!("Invalid amount: {e}"),
                                None::<()>,
                            )
                        })?;

                        let denominating_token_id = fee_obj
                            .get("denominatingTokenId")
                            .and_then(Value::as_str)
                            .map(|s| {
                                TokenId::from_str(s).map_err(|e| {
                                    ErrorObject::owned(
                                        INTERNAL_ERROR_CODE,
                                        format!("Invalid token ID: {e}"),
                                        None::<()>,
                                    )
                                })
                            })
                            .transpose()?;

                        fixed_fees.push(CustomFixedFee::new(amount, denominating_token_id, None));
                    }
                }

                limits.push(CustomFeeLimit::new(Some(account_id), fixed_fees));
            }
        }
        tx.custom_fee_limits(limits);
    }

    Ok(tx)
}

fn build_transfer_tx_from_value(params: &Value) -> Result<TransferTransaction, ErrorObjectOwned> {
    let mut tx = TransferTransaction::new();

    let transfers = params
        .get("transfers")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "transfers array is required for transferCrypto".to_string(),
                None::<()>,
            )
        })?;

    if transfers.is_empty() {
        return Err(ErrorObject::owned(
            INTERNAL_ERROR_CODE,
            "No transfers provided".to_string(),
            None::<()>,
        ));
    }

    for t in transfers {
        if let Value::Object(obj) = t {
            let is_approved = obj.get("approved").and_then(Value::as_bool).unwrap_or(false);

            // Handle HBAR transfers
            if let Some(hbar_obj) = obj.get("hbar").and_then(Value::as_object) {
                let amount_i64 = hbar_obj
                    .get("amount")
                    .and_then(|v| v.as_i64().or_else(|| v.as_str()?.parse::<i64>().ok()))
                    .ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "amount is required in hbar transfers".to_string(),
                            None::<()>,
                        )
                    })?;
                let amount = Hbar::from_tinybars(amount_i64);

                if let Some(account_id_str) = hbar_obj.get("accountId").and_then(Value::as_str) {
                    let account_id = AccountId::from_str(account_id_str).map_err(|e| {
                        ErrorObject::owned(INTERNAL_ERROR_CODE, format!("Invalid account ID: {e}"), None::<()>)
                    })?;

                    if is_approved {
                        tx.approved_hbar_transfer(account_id, amount);
                    } else {
                        tx.hbar_transfer(account_id, amount);
                    }
                } else if let Some(evm_address_str) = hbar_obj.get("evmAddress").and_then(Value::as_str) {
                    let evm_address = EvmAddress::from_str(evm_address_str).map_err(|e| {
                        ErrorObject::owned(INTERNAL_ERROR_CODE, format!("Invalid EVM address: {e}"), None::<()>)
                    })?;
                    let account_id = AccountId::from_evm_address(&evm_address, 0, 0);

                    if is_approved {
                        tx.approved_hbar_transfer(account_id, amount);
                    } else {
                        tx.hbar_transfer(account_id, amount);
                    }
                } else {
                    return Err(ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Either accountId or evmAddress is required in hbar transfers".to_string(),
                        None::<()>,
                    ));
                }
            }
            // Handle token transfers
            else if let Some(token_obj) = obj.get("token").and_then(Value::as_object) {
                let account_id_str = token_obj
                    .get("accountId")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "accountId is required in token transfers".to_string(),
                            None::<()>,
                        )
                    })?;
                let account_id = AccountId::from_str(account_id_str).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, format!("Invalid account ID: {e}"), None::<()>)
                })?;

                let token_id_str = token_obj
                    .get("tokenId")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "tokenId is required in token transfers".to_string(),
                            None::<()>,
                        )
                    })?;
                let token_id = TokenId::from_str(token_id_str).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, format!("Invalid token ID: {e}"), None::<()>)
                })?;

                let amount = token_obj
                    .get("amount")
                    .and_then(|v| v.as_i64().or_else(|| v.as_str()?.parse::<i64>().ok()))
                    .ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "amount is required in token transfers".to_string(),
                            None::<()>,
                        )
                    })?;

                let decimals = token_obj.get("decimals").and_then(Value::as_u64).map(|d| d as u32);

                if is_approved {
                    tx.approved_token_transfer(token_id, account_id, amount);
                } else if let Some(dec) = decimals {
                    tx.token_transfer_with_decimals(token_id, account_id, amount, dec);
                } else {
                    tx.token_transfer(token_id, account_id, amount);
                }
            }
            // Handle NFT transfers
            else if let Some(nft_obj) = obj.get("nft").and_then(Value::as_object) {
                let sender_account_id_str = nft_obj
                    .get("senderAccountId")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "senderAccountId is required in NFT transfers".to_string(),
                            None::<()>,
                        )
                    })?;
                let sender_account_id = AccountId::from_str(sender_account_id_str).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, format!("Invalid sender account ID: {e}"), None::<()>)
                })?;

                let receiver_account_id_str = nft_obj
                    .get("receiverAccountId")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "receiverAccountId is required in NFT transfers".to_string(),
                            None::<()>,
                        )
                    })?;
                let receiver_account_id = AccountId::from_str(receiver_account_id_str).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, format!("Invalid receiver account ID: {e}"), None::<()>)
                })?;

                let token_id_str = nft_obj
                    .get("tokenId")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "tokenId is required in NFT transfers".to_string(),
                            None::<()>,
                        )
                    })?;
                let token_id = TokenId::from_str(token_id_str).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, format!("Invalid token ID: {e}"), None::<()>)
                })?;

                let serial_number = nft_obj
                    .get("serialNumber")
                    .and_then(|v| v.as_i64().or_else(|| v.as_str()?.parse::<i64>().ok()))
                    .ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "serialNumber is required in NFT transfers".to_string(),
                            None::<()>,
                        )
                    })?;

                let nft_id = NftId::from((token_id, serial_number as u64));

                if is_approved {
                    tx.approved_nft_transfer(nft_id, sender_account_id, receiver_account_id);
                } else {
                    tx.nft_transfer(nft_id, sender_account_id, receiver_account_id);
                }
            }
        }
    }

    Ok(tx)
}

fn build_allowance_approve_tx_from_value(
    params: &Value,
) -> Result<AccountAllowanceApproveTransaction, ErrorObjectOwned> {
    let mut tx = AccountAllowanceApproveTransaction::new();

    let allowances = params
        .get("allowances")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "allowances array is required".to_string(),
                None::<()>,
            )
        })?;

    for allowance in allowances {
        if let Value::Object(obj) = allowance {
            // Get owner and spender (required for all allowance types)
            let owner_str = obj
                .get("ownerAccountId")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "ownerAccountId is required in allowances".to_string(),
                        None::<()>,
                    )
                })?;
            let spender_str = obj
                .get("spenderAccountId")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "spenderAccountId is required in allowances".to_string(),
                        None::<()>,
                    )
                })?;

            let owner = AccountId::from_str(owner_str).map_err(|e| {
                ErrorObject::owned(INTERNAL_ERROR_CODE, format!("Invalid ownerAccountId: {e}"), None::<()>)
            })?;
            let spender = AccountId::from_str(spender_str).map_err(|e| {
                ErrorObject::owned(INTERNAL_ERROR_CODE, format!("Invalid spenderAccountId: {e}"), None::<()>)
            })?;

            // Check which type of allowance
            if let Some(hbar_obj) = obj.get("hbar").and_then(Value::as_object) {
                let amount = hbar_obj
                    .get("amount")
                    .and_then(|v| v.as_i64().or_else(|| v.as_str()?.parse::<i64>().ok()))
                    .ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "amount is required in hbar allowance".to_string(),
                            None::<()>,
                        )
                    })?;

                tx.approve_hbar_allowance(owner, spender, Hbar::from_tinybars(amount));
            } else if let Some(token_obj) = obj.get("token").and_then(Value::as_object) {
                let token_id_str = token_obj
                    .get("tokenId")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "tokenId is required in token allowance".to_string(),
                            None::<()>,
                        )
                    })?;
                let token_id = TokenId::from_str(token_id_str).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, format!("Invalid tokenId: {e}"), None::<()>)
                })?;

                let amount = token_obj
                    .get("amount")
                    .and_then(|v| v.as_i64().or_else(|| v.as_str()?.parse::<i64>().ok()))
                    .ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "amount is required in token allowance".to_string(),
                            None::<()>,
                        )
                    })?;

                tx.approve_token_allowance(token_id, owner, spender, amount as u64);
            } else if let Some(nft_obj) = obj.get("nft").and_then(Value::as_object) {
                // Handle NFT allowances
                let token_id_str = nft_obj
                    .get("tokenId")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "tokenId is required in nft allowance".to_string(),
                            None::<()>,
                        )
                    })?;
                let token_id = TokenId::from_str(token_id_str).map_err(|e| {
                    ErrorObject::owned(INTERNAL_ERROR_CODE, format!("Invalid tokenId: {e}"), None::<()>)
                })?;

                // Check if this is an "approve all" or specific serial numbers
                if let Some(approved_for_all) = nft_obj.get("approvedForAll").and_then(Value::as_bool) {
                    if approved_for_all {
                        tx.approve_token_nft_allowance_all_serials(token_id, owner, spender);
                    }
                } else if let Some(serial_numbers) = nft_obj.get("serialNumbers").and_then(Value::as_array) {
                    for serial in serial_numbers {
                        let serial_num = serial
                            .as_i64()
                            .or_else(|| serial.as_str()?.parse::<i64>().ok())
                            .ok_or_else(|| {
                                ErrorObject::owned(
                                    INTERNAL_ERROR_CODE,
                                    "Invalid serial number in nft allowance".to_string(),
                                    None::<()>,
                                )
                            })?;
                        
                        let nft_id = NftId::from((token_id, serial_num as u64));
                        tx.approve_token_nft_allowance(nft_id, owner, spender);
                    }
                } else {
                    return Err(ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Either approvedForAll or serialNumbers is required in nft allowance".to_string(),
                        None::<()>,
                    ));
                }
            } else {
                return Err(ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "No valid allowance type provided (hbar, token, or nft)".to_string(),
                    None::<()>,
                ));
            }
        }
    }

    Ok(tx)
}

fn build_token_mint_tx_from_value(params: &Value) -> Result<TokenMintTransaction, ErrorObjectOwned> {
    let mut tx = TokenMintTransaction::new();

    let token_id = params
        .get("tokenId")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "tokenId is required in mintToken".to_string(),
                None::<()>,
            )
        })?;
    let token_id = TokenId::from_str(token_id).map_err(|e| {
        ErrorObject::owned(INTERNAL_ERROR_CODE, format!("Invalid tokenId: {e}"), None::<()>)
    })?;
    tx.token_id(token_id);

    if let Some(amount) = params
        .get("amount")
        .and_then(|v| v.as_u64().or_else(|| v.as_str()?.parse::<u64>().ok()))
    {
        tx.amount(amount);
    }

    if let Some(metadata_array) = params.get("metadata").and_then(Value::as_array) {
        if !metadata_array.is_empty() {
            let mut metas = Vec::new();
            for m in metadata_array {
                if let Some(hex_str) = m.as_str() {
                    let bytes = hex::decode(hex_str).map_err(|e| {
                        ErrorObject::owned(INTERNAL_ERROR_CODE, format!("Invalid metadata hex: {e}"), None::<()>)
                    })?;
                    metas.push(bytes);
                }
            }
            if !metas.is_empty() {
                tx.metadata(metas);
            }
        }
    }

    Ok(tx)
}

fn build_token_burn_tx_from_value(params: &Value) -> Result<TokenBurnTransaction, ErrorObjectOwned> {
    let mut tx = TokenBurnTransaction::new();

    let token_id = params
        .get("tokenId")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "tokenId is required in burnToken".to_string(),
                None::<()>,
            )
        })?;
    let token_id = TokenId::from_str(token_id).map_err(|e| {
        ErrorObject::owned(INTERNAL_ERROR_CODE, format!("Invalid tokenId: {e}"), None::<()>)
    })?;
    tx.token_id(token_id);

    if let Some(amount) = params
        .get("amount")
        .and_then(|v| v.as_u64().or_else(|| v.as_str()?.parse::<u64>().ok()))
    {
        tx.amount(amount);
    }

    if let Some(serials) = params.get("serials").and_then(Value::as_array) {
        if !serials.is_empty() {
            let mut s = Vec::new();
            for sn in serials {
                if let Some(num) = sn.as_i64().or_else(|| sn.as_str()?.parse::<i64>().ok()) {
                    s.push(num);
                }
            }
            if !s.is_empty() {
                tx.serials(s);
            }
        }
    }

    Ok(tx)
}
