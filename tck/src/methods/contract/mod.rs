use std::collections::HashMap;
use std::str::FromStr;

use hiero_sdk::{
    AccountId,
    Client,
    ContractBytecodeQuery,
    ContractCallQuery,
    ContractCreateTransaction,
    ContractDeleteTransaction,
    ContractExecuteTransaction,
    ContractId,
    ContractInfoQuery,
    ContractUpdateTransaction,
    FileId,
    Hbar,
};
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::error::INTERNAL_ERROR_CODE;
use jsonrpsee::types::{
    ErrorObject,
    ErrorObjectOwned,
};
use serde_json::Value;
use time::{
    Duration,
    OffsetDateTime,
};

use crate::common::mock_consensus_error;
use crate::errors::from_hedera_error;
use crate::helpers::{
    fill_common_transaction_params,
    get_hedera_key,
    key_to_der_string,
};
use crate::responses::{
    ContractByteCodeResponse,
    ContractCallResponse,
    ContractInfoResponse,
    ContractResponse,
    StakingInfo,
};

#[rpc(server, client)]
pub trait ContractRpc {
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
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/contract-service/contractCallQuery.md#contractCallQuery
    */
    #[method(name = "contractCallQuery")]
    async fn contract_call_query(
        &self,
        contract_id: Option<String>,
        gas: Option<String>,
        function_name: Option<String>,
        function_parameters: Option<String>,
        sender_account_id: Option<String>,
        max_query_payment: Option<String>,
    ) -> Result<ContractCallResponse, ErrorObjectOwned>;

    #[method(name = "contractByteCodeQuery")]
    async fn contract_bytecode_query(
        &self,
        contract_id: Option<String>,
        query_payment: Option<String>,
        max_query_payment: Option<String>,
    ) -> Result<ContractByteCodeResponse, ErrorObjectOwned>;

    #[method(name = "contractInfoQuery")]
    async fn contract_info_query(
        &self,
        contract_id: Option<String>,
        query_payment: Option<String>,
        max_query_payment: Option<String>,
    ) -> Result<ContractInfoResponse, ErrorObjectOwned>;
}

pub async fn create_contract(
    client: &Client,
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
            AccountId::from_str(&auto_renew_account_id)
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?,
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
            FileId::from_str(&bytecode_file_id)
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?,
        );
    }

    if let Some(staked_account_id) = staked_account_id {
        contract_create_tx.staked_account_id(
            AccountId::from_str(&staked_account_id)
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?,
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
        fill_common_transaction_params(&mut contract_create_tx, &common_transaction_params, client);
    }

    let tx_response = contract_create_tx.execute(client).await.map_err(|e| from_hedera_error(e))?;

    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(ContractResponse {
        contract_id: tx_receipt.contract_id.map(|id| id.to_string()),
        status: tx_receipt.status.as_str_name().to_string(),
    })
}

pub async fn update_contract(
    client: &Client,
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
    let mut contract_update_tx = ContractUpdateTransaction::new();

    if let Some(contract_id) = contract_id {
        contract_update_tx.contract_id(
            ContractId::from_str(&contract_id)
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?,
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
            AccountId::from_str(&auto_renew_account_id)
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?,
        );
    }

    if let Some(staked_account_id) = staked_account_id {
        contract_update_tx.staked_account_id(
            AccountId::from_str(&staked_account_id)
                .map_err(|e| mock_consensus_error("INVALID_STAKING_ID", &e.to_string()))?,
        );
    }

    if let Some(staked_node_id) = staked_node_id {
        let node_id = staked_node_id
            .parse::<u64>()
            .map_err(|e| mock_consensus_error("INVALID_STAKING_ID", &e.to_string()))?;
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
            .map_err(|e| mock_consensus_error("INVALID_EXPIRATION_TIME", &e.to_string()))?;

        // Check timestamp boundaries
        if timestamp > 253402300799 {
            return Err(mock_consensus_error(
                "INVALID_EXPIRATION_TIME",
                "Expiration time exceeds maximum allowed value",
            ));
        }

        if timestamp < -377705116800 {
            return Err(mock_consensus_error(
                "EXPIRATION_REDUCTION_NOT_ALLOWED",
                "Expiration time is below minimum allowed value",
            ));
        }

        contract_update_tx.expiration_time(
            OffsetDateTime::from_unix_timestamp(timestamp)
                .map_err(|e| mock_consensus_error("INVALID_EXPIRATION_TIME", &e.to_string()))?,
        );
    }

    if let Some(common_transaction_params) = common_transaction_params {
        fill_common_transaction_params(&mut contract_update_tx, &common_transaction_params, client);
    }

    let tx_response = contract_update_tx.execute(client).await.map_err(|e| from_hedera_error(e))?;

    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(ContractResponse { contract_id: None, status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn delete_contract(
    client: &Client,
    contract_id: Option<String>,
    transfer_account_id: Option<String>,
    transfer_contract_id: Option<String>,
    permanent_removal: Option<bool>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<ContractResponse, ErrorObjectOwned> {
    let mut contract_delete_tx = ContractDeleteTransaction::new();

    if let Some(contract_id) = contract_id {
        contract_delete_tx.contract_id(
            ContractId::from_str(&contract_id)
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?,
        );
    }

    if let Some(permanent_removal) = permanent_removal {
        contract_delete_tx.permanent_removal(permanent_removal);
    }

    // Depend on how I order transferContractId and transferAccountId the last will stay if both are called
    if let Some(transfer_contract_id) = transfer_contract_id {
        contract_delete_tx.transfer_contract_id(
            ContractId::from_str(&transfer_contract_id)
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?,
        );
    }

    if let Some(transfer_account_id) = transfer_account_id {
        contract_delete_tx.transfer_account_id(
            AccountId::from_str(&transfer_account_id)
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?,
        );
    }

    if let Some(common_transaction_params) = common_transaction_params {
        fill_common_transaction_params(&mut contract_delete_tx, &common_transaction_params, client);
    }

    let tx_response = contract_delete_tx.execute(client).await.map_err(|e| from_hedera_error(e))?;

    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(ContractResponse { contract_id: None, status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn execute_contract(
    client: &Client,
    contract_id: Option<String>,
    gas: Option<String>,
    amount: Option<String>,
    function_parameters: Option<String>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<ContractResponse, ErrorObjectOwned> {
    let mut contract_execute_tx = ContractExecuteTransaction::new();

    if let Some(contract_id) = contract_id {
        contract_execute_tx.contract_id(
            ContractId::from_str(&contract_id)
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?,
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
        fill_common_transaction_params(
            &mut contract_execute_tx,
            &common_transaction_params,
            client,
        );
    }

    let tx_response =
        contract_execute_tx.execute(client).await.map_err(|e| from_hedera_error(e))?;

    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(ContractResponse { contract_id: None, status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn contract_call_query(
    client: &Client,
    contract_id: Option<String>,
    gas: Option<String>,
    function_name: Option<String>,
    function_parameters: Option<String>,
    sender_account_id: Option<String>,
    max_query_payment: Option<String>,
) -> Result<ContractCallResponse, ErrorObjectOwned> {
    let mut query = ContractCallQuery::new();

    if let Some(contract_id) = contract_id {
        query.contract_id(
            ContractId::from_str(&contract_id)
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?,
        );
    }

    if let Some(gas) = gas {
        let gas_val = gas
            .parse::<u64>()
            .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
        query.gas(gas_val);
    }

    if let Some(function_parameters) = &function_parameters {
        let bytes = hex::decode(function_parameters).map_err(|e| {
            ErrorObject::owned(INTERNAL_ERROR_CODE, format!("Invalid hex: {e}"), None::<()>)
        })?;
        query.function_parameters(bytes);
    }

    if let Some(function_name) = &function_name {
        query.function(function_name);
    }

    if let Some(max_query_payment) = max_query_payment {
        let amount = max_query_payment
            .parse::<i64>()
            .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
        query.max_payment_amount(Hbar::from_tinybars(amount));
    }

    if let Some(sender_account_id) = &sender_account_id {
        query.sender_account_id(
            AccountId::from_str(sender_account_id)
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?,
        );
    }

    let result = query.execute(client).await.map_err(|e| from_hedera_error(e))?;

    // Build response matching the JS SDK structure
    Ok(ContractCallResponse {
        bytes: if !result.bytes.is_empty() {
            Some(format!("0x{}", hex::encode(&result.bytes)))
        } else {
            None
        },
        contract_id: Some(result.contract_id.to_string()),
        gas_used: Some(result.gas_used.to_string()),
        error_message: result.error_message.clone(),
        // Try to extract common return value types
        string: result.get_str(0).map(|s| s.to_string()),
        bool: result.get_bool(0),
        address: result.get_address(0).map(|addr| format!("0x{}", addr)),
        bytes32: if result.bytes.len() >= 32 {
            result.get_bytes32(0).map(|b| format!("0x{}", hex::encode(b)))
        } else {
            None
        },
        // Integer types
        int8: result.get_i8(0),
        uint8: result.get_u8(0),
        int16: result.get_i32(0).and_then(|v| i16::try_from(v).ok()),
        uint16: result.get_u32(0).and_then(|v| u16::try_from(v).ok()),
        int24: result.get_i32(0).map(|v| v.to_string()),
        uint24: result.get_u32(0).map(|v| v.to_string()),
        int32: result.get_i32(0),
        uint32: result.get_u32(0),
        int40: result.get_i64(0).map(|v| v.to_string()),
        uint40: result.get_u64(0).map(|v| v.to_string()),
        int48: result.get_i64(0).map(|v| v.to_string()),
        uint48: result.get_u64(0).map(|v| v.to_string()),
        int56: result.get_i64(0).map(|v| v.to_string()),
        uint56: result.get_u64(0).map(|v| v.to_string()),
        int64: result.get_i64(0).map(|v| v.to_string()),
        uint64: result.get_u64(0).map(|v| v.to_string()),
        int256: result.get_i256(0).map(|v| v.to_string()),
        uint256: result.get_u256(0).map(|v| v.to_string()),
    })
}

pub async fn contract_bytecode_query(
    client: &Client,
    contract_id: Option<String>,
    query_payment: Option<String>,
    max_query_payment: Option<String>,
) -> Result<ContractByteCodeResponse, ErrorObjectOwned> {
    if contract_id.is_none() {
        return Err(crate::common::mock_consensus_error(
            "INVALID_CONTRACT_ID",
            "The contract ID is invalid or missing",
        ));
    }

    let mut query = ContractBytecodeQuery::new();

    if let Some(contract_id_str) = &contract_id {
        query.contract_id(
            ContractId::from_str(contract_id_str)
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?,
        );
    }

    if let Some(query_payment) = query_payment {
        let amount = query_payment
            .parse::<i64>()
            .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
        query.payment_amount(Hbar::from_tinybars(amount));
    }

    if let Some(max_query_payment) = max_query_payment {
        let amount = max_query_payment
            .parse::<i64>()
            .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
        query.max_payment_amount(Hbar::from_tinybars(amount));
    }
    let result = query.execute(client).await.map_err(|e| from_hedera_error(e))?;

    let bytecode =
        if !result.is_empty() { Some(format!("0x{}", hex::encode(&result))) } else { None };

    Ok(ContractByteCodeResponse { bytecode, contract_id: contract_id.clone() })
}

pub async fn contract_info_query(
    client: &Client,
    contract_id: Option<String>,
    query_payment: Option<String>,
    max_query_payment: Option<String>,
) -> Result<ContractInfoResponse, ErrorObjectOwned> {
    // Mock consensus node error if contract_id is missing
    if contract_id.is_none() {
        return Err(crate::common::mock_consensus_error(
            "INVALID_CONTRACT_ID",
            "The contract ID is invalid or missing",
        ));
    }

    let mut query = ContractInfoQuery::new();

    if let Some(contract_id) = contract_id {
        query.contract_id(
            ContractId::from_str(&contract_id)
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?,
        );
    }

    if let Some(query_payment) = query_payment {
        let amount = query_payment
            .parse::<i64>()
            .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
        query.payment_amount(Hbar::from_tinybars(amount));
    }

    if let Some(max_query_payment) = max_query_payment {
        let amount = max_query_payment
            .parse::<i64>()
            .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
        query.max_payment_amount(Hbar::from_tinybars(amount));
    }

    let info = query.execute(client).await.map_err(|e| from_hedera_error(e))?;

    println!("🔍 ContractInfo: {:?}", info);

    Ok(ContractInfoResponse {
        contract_id: Some(info.contract_id.to_string()),
        account_id: Some(info.account_id.to_string()),
        contract_account_id: Some(info.contract_account_id.clone()),
        admin_key: info.admin_key.map(|k| key_to_der_string(&k)),
        expiration_time: info.expiration_time.map(|t| t.unix_timestamp().to_string()),
        auto_renew_period: info.auto_renew_period.map(|d| d.whole_seconds().to_string()),
        auto_renew_account_id: info.auto_renew_account_id.map(|id| id.to_string()),
        storage: Some(info.storage.to_string()),
        contract_memo: Some(info.contract_memo.clone()),
        balance: Some(Hbar::from_tinybars(info.balance as i64).to_tinybars().to_string()),
        is_deleted: Some(info.is_deleted),
        max_automatic_token_associations: Some(info.max_automatic_token_associations.to_string()),
        ledger_id: Some(info.ledger_id.to_string()),
        staking_info: info.staking_info.map(|si| StakingInfo {
            decline_staking_reward: Some(si.decline_staking_reward),
            stake_period_start: si.stake_period_start.map(|t| t.unix_timestamp().to_string()),
            pending_reward: Some(si.pending_reward.to_tinybars().to_string()),
            staked_to_me: Some(si.staked_to_me.to_tinybars().to_string()),
            staked_account_id: si.staked_account_id.map(|id| id.to_string()),
            staked_node_id: si.staked_node_id.map(|id| id.to_string()),
        }),
    })
}
