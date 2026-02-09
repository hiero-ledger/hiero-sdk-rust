use std::collections::HashMap;
use std::str::FromStr;

use hiero_sdk::{
    AccountAllowanceApproveTransaction,
    AccountBalanceQuery,
    AccountCreateTransaction,
    AccountDeleteTransaction,
    AccountId,
    AccountUpdateTransaction,
    Client,
    ContractId,
    EvmAddress,
    Hbar,
    NftId,
    TokenId,
    TransferTransaction,
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

use crate::common::internal_error;
use crate::errors::from_hedera_error;
use crate::helpers::{
    fill_common_transaction_params,
    get_hedera_key,
};
use crate::responses::{
    AccountBalanceResponse,
    AccountCreateResponse,
    AccountUpdateResponse,
};

#[rpc(server, client)]
pub trait AccountRpc {
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
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/crypto-service/accountBalanceQuery.md#getAccountBalance
    */
    #[method(name = "getAccountBalance")]
    async fn get_account_balance(
        &self,
        account_id: Option<String>,
        contract_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<AccountBalanceResponse, ErrorObjectOwned>;

    #[method(name = "deleteAccount")]
    async fn delete_account(
        &self,
        delete_account_id: Option<String>,
        transfer_account_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<AccountUpdateResponse, ErrorObjectOwned>;

    #[method(name = "transferCrypto")]
    async fn transfer_crypto(
        &self,
        transfers: Option<Vec<Value>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<AccountUpdateResponse, ErrorObjectOwned>;
}

pub async fn create_account(
    client: &Client,
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
    let mut account_create_tx = AccountCreateTransaction::new();

    if let Some(key) = key {
        let key = get_hedera_key(&key)?;

        account_create_tx.set_key_without_alias(key);
    }

    if let Some(initial_balance) = initial_balance {
        let initial_balance = initial_balance.parse::<i64>().map_err(internal_error)?;
        account_create_tx.initial_balance(Hbar::from_tinybars(initial_balance));
    }

    if let Some(receiver_signature_required) = receiver_signature_required {
        account_create_tx.receiver_signature_required(receiver_signature_required);
    }

    if let Some(auto_renew_period) = auto_renew_period {
        let auto_renew_period = auto_renew_period.parse::<i64>().map_err(internal_error)?;
        account_create_tx.auto_renew_period(Duration::seconds(auto_renew_period));
    }

    if let Some(memo) = memo {
        account_create_tx.account_memo(memo);
    }

    if let Some(max_auto_token_associations) = max_auto_token_associations {
        account_create_tx.max_automatic_token_associations(max_auto_token_associations as i32);
    }

    if let Some(staked_account_id) = staked_account_id {
        account_create_tx
            .staked_account_id(AccountId::from_str(&staked_account_id).map_err(internal_error)?);
    }

    if let Some(alias) = alias {
        account_create_tx.alias(EvmAddress::from_str(&alias).map_err(internal_error)?);
    }

    if let Some(staked_node_id) = staked_node_id {
        let node_id = staked_node_id.parse::<i64>().map_err(internal_error)?;
        if node_id < 0 {
            return Err(internal_error("INVALID_STAKING_ID"));
        }
        account_create_tx.staked_node_id(node_id as u64);
    }

    if let Some(decline_staking_reward) = decline_staking_reward {
        account_create_tx.decline_staking_reward(decline_staking_reward);
    }

    if let Some(common_transaction_params) = common_transaction_params {
        fill_common_transaction_params(&mut account_create_tx, &common_transaction_params, client);
    }

    let tx_response = account_create_tx.execute(client).await.map_err(|e| from_hedera_error(e))?;

    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(AccountCreateResponse {
        account_id: tx_receipt.account_id.unwrap().to_string(),
        status: tx_receipt.status.as_str_name().to_string(),
    })
}

pub async fn update_account(
    client: &Client,
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
    let mut account_update_tx = AccountUpdateTransaction::new();

    if let Some(account_id) = account_id {
        account_update_tx.account_id(account_id.parse().unwrap());
    }

    if let Some(key) = key {
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
            OffsetDateTime::from_unix_timestamp(expiration_time).map_err(internal_error)?,
        );
    }

    if let Some(memo) = memo {
        account_update_tx.account_memo(memo);
    }

    if let Some(max_auto_token_associations) = max_auto_token_associations {
        account_update_tx.max_automatic_token_associations(max_auto_token_associations as i32);
    }

    if let Some(staked_account_id) = staked_account_id {
        account_update_tx
            .staked_account_id(AccountId::from_str(&staked_account_id).map_err(internal_error)?);
    }

    if let Some(staked_node_id) = staked_node_id {
        let node_id = staked_node_id.parse::<i64>().map_err(internal_error)?;
        if node_id < 0 {
            return Err(internal_error("INVALID_STAKING_ID"));
        }
        account_update_tx.staked_node_id(node_id as u64);
    }

    if let Some(decline_staking_reward) = decline_staking_reward {
        account_update_tx.decline_staking_reward(decline_staking_reward);
    }

    if let Some(common_transaction_params) = common_transaction_params {
        fill_common_transaction_params(&mut account_update_tx, &common_transaction_params, client);
    }

    let tx_response = account_update_tx.execute(client).await.map_err(|e| from_hedera_error(e))?;

    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(AccountUpdateResponse { status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn get_account_balance(
    client: &Client,
    account_id: Option<String>,
    contract_id: Option<String>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<AccountBalanceResponse, ErrorObjectOwned> {
    let _ = common_transaction_params;

    let mut query = AccountBalanceQuery::new();

    if let Some(account_id) = account_id {
        query.account_id(AccountId::from_str(&account_id).map_err(internal_error)?);
    }

    if let Some(contract_id) = contract_id {
        query.contract_id(ContractId::from_str(&contract_id).map_err(internal_error)?);
    }

    let tx_response = query.execute(client).await.map_err(|e| from_hedera_error(e))?;

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

// Helper function used in schedule creation
pub(crate) fn build_account_create_tx_from_value(
    params: &Value,
) -> Result<AccountCreateTransaction, ErrorObjectOwned> {
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
        tx.staked_account_id(AccountId::from_str(staked_account_id).map_err(internal_error)?);
    }

    if let Some(staked_node_id) = params.get("stakedNodeId").and_then(Value::as_str) {
        let node_id = staked_node_id.parse::<i64>().map_err(internal_error)?;
        if node_id < 0 {
            return Err(internal_error("INVALID_STAKING_ID"));
        }
        tx.staked_node_id(node_id as u64);
    }

    if let Some(decline_staking_reward) =
        params.get("declineStakingReward").and_then(Value::as_bool)
    {
        tx.decline_staking_reward(decline_staking_reward);
    }

    if let Some(alias) = params.get("alias").and_then(Value::as_str) {
        tx.alias(EvmAddress::from_str(alias).map_err(internal_error)?);
    }

    Ok(tx)
}

// Helper function for building account allowance approve transaction (used in schedule creation)
pub(crate) fn build_allowance_approve_tx_from_value(
    params: &Value,
) -> Result<AccountAllowanceApproveTransaction, ErrorObjectOwned> {
    let mut tx = AccountAllowanceApproveTransaction::new();

    let allowances = params.get("allowances").and_then(Value::as_array).ok_or_else(|| {
        ErrorObject::owned(
            INTERNAL_ERROR_CODE,
            "allowances array is required".to_string(),
            None::<()>,
        )
    })?;

    for allowance in allowances {
        if let Value::Object(obj) = allowance {
            // Get owner and spender (required for all allowance types)
            let owner_str = obj.get("ownerAccountId").and_then(Value::as_str).ok_or_else(|| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "ownerAccountId is required in allowances".to_string(),
                    None::<()>,
                )
            })?;
            let spender_str =
                obj.get("spenderAccountId").and_then(Value::as_str).ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "spenderAccountId is required in allowances".to_string(),
                        None::<()>,
                    )
                })?;

            let owner = AccountId::from_str(owner_str).map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid ownerAccountId: {e}"),
                    None::<()>,
                )
            })?;
            let spender = AccountId::from_str(spender_str).map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid spenderAccountId: {e}"),
                    None::<()>,
                )
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
                let token_id_str =
                    token_obj.get("tokenId").and_then(Value::as_str).ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "tokenId is required in token allowance".to_string(),
                            None::<()>,
                        )
                    })?;
                let token_id = TokenId::from_str(token_id_str).map_err(|e| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        format!("Invalid tokenId: {e}"),
                        None::<()>,
                    )
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
                let token_id_str =
                    nft_obj.get("tokenId").and_then(Value::as_str).ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "tokenId is required in nft allowance".to_string(),
                            None::<()>,
                        )
                    })?;
                let token_id = TokenId::from_str(token_id_str).map_err(|e| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        format!("Invalid tokenId: {e}"),
                        None::<()>,
                    )
                })?;

                // Check if this is an "approve all" or specific serial numbers
                if let Some(approved_for_all) =
                    nft_obj.get("approvedForAll").and_then(Value::as_bool)
                {
                    if approved_for_all {
                        tx.approve_token_nft_allowance_all_serials(token_id, owner, spender);
                    }
                } else if let Some(serial_numbers) =
                    nft_obj.get("serialNumbers").and_then(Value::as_array)
                {
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
                        "Either approvedForAll or serialNumbers is required in nft allowance"
                            .to_string(),
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

pub async fn delete_account(
    client: &Client,
    delete_account_id: Option<String>,
    transfer_account_id: Option<String>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<AccountUpdateResponse, ErrorObjectOwned> {
    let mut tx = AccountDeleteTransaction::new();

    if let Some(delete_account_id) = delete_account_id {
        tx.account_id(AccountId::from_str(&delete_account_id).map_err(internal_error)?);
    }

    if let Some(transfer_account_id) = transfer_account_id {
        tx.transfer_account_id(AccountId::from_str(&transfer_account_id).map_err(internal_error)?);
    }

    if let Some(params) = common_transaction_params {
        fill_common_transaction_params(&mut tx, &params, client);
    }

    let tx_response = tx.execute(client).await.map_err(|e| from_hedera_error(e))?;
    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(AccountUpdateResponse { status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn transfer_crypto(
    client: &Client,
    transfers: Option<Vec<Value>>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<AccountUpdateResponse, ErrorObjectOwned> {
    let transfers = transfers.ok_or_else(|| internal_error("No transfers provided"))?;

    if transfers.is_empty() {
        return Err(ErrorObject::owned(
            INTERNAL_ERROR_CODE,
            "No transfers provided".to_string(),
            None::<()>,
        ));
    }

    let mut tx = TransferTransaction::new();

    for transfer in transfers {
        let transfer_obj = transfer.as_object().ok_or_else(|| {
            ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Invalid transfer object".to_string(),
                None::<()>,
            )
        })?;

        let is_approved = transfer_obj.get("approved").and_then(|v| v.as_bool()).unwrap_or(false);

        // Handle HBAR transfers
        if let Some(hbar_obj) = transfer_obj.get("hbar").and_then(|v| v.as_object()) {
            let amount_str = hbar_obj.get("amount").and_then(|v| v.as_str()).ok_or_else(|| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "HBAR amount is required".to_string(),
                    None::<()>,
                )
            })?;
            let amount = Hbar::from_tinybars(
                amount_str
                    .parse::<i64>()
                    .map_err(|e| internal_error(format!("Invalid amount: {e}")))?,
            );

            if let Some(account_id_str) = hbar_obj.get("accountId").and_then(|v| v.as_str()) {
                let account_id = AccountId::from_str(account_id_str).map_err(|e| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        format!("Invalid account ID: {e}"),
                        None::<()>,
                    )
                })?;

                if is_approved {
                    tx.approved_hbar_transfer(account_id, amount);
                } else {
                    tx.hbar_transfer(account_id, amount);
                }
            } else if let Some(evm_address_str) =
                hbar_obj.get("evmAddress").and_then(|v| v.as_str())
            {
                let evm_address = EvmAddress::from_str(evm_address_str).map_err(|e| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        format!("Invalid EVM address: {e}"),
                        None::<()>,
                    )
                })?;
                let account_id = AccountId::from_evm_address(&evm_address, 0, 0);

                if is_approved {
                    tx.approved_hbar_transfer(account_id, amount);
                } else {
                    tx.hbar_transfer(account_id, amount);
                }
            }
        }
        // Handle Token transfers
        else if let Some(token_obj) = transfer_obj.get("token").and_then(|v| v.as_object()) {
            let account_id_str =
                token_obj.get("accountId").and_then(|v| v.as_str()).ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Token accountId is required".to_string(),
                        None::<()>,
                    )
                })?;
            let account_id = AccountId::from_str(account_id_str).map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid account ID: {e}"),
                    None::<()>,
                )
            })?;

            let token_id_str =
                token_obj.get("tokenId").and_then(|v| v.as_str()).ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Token tokenId is required".to_string(),
                        None::<()>,
                    )
                })?;
            let token_id = TokenId::from_str(token_id_str).map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid token ID: {e}"),
                    None::<()>,
                )
            })?;

            let amount_str = token_obj.get("amount").and_then(|v| v.as_str()).ok_or_else(|| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "Token amount is required".to_string(),
                    None::<()>,
                )
            })?;
            let amount = amount_str
                .parse::<i64>()
                .map_err(|e| internal_error(format!("Invalid amount: {e}")))?;

            let decimals = token_obj.get("decimals").and_then(|v| v.as_u64()).map(|d| d as u32);

            if is_approved {
                tx.approved_token_transfer(token_id, account_id, amount);
            } else if let Some(decimals) = decimals {
                tx.token_transfer_with_decimals(token_id, account_id, amount, decimals);
            } else {
                tx.token_transfer(token_id, account_id, amount);
            }
        }
        // Handle NFT transfers
        else if let Some(nft_obj) = transfer_obj.get("nft").and_then(|v| v.as_object()) {
            let sender_account_id_str =
                nft_obj.get("senderAccountId").and_then(|v| v.as_str()).ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "NFT senderAccountId is required".to_string(),
                        None::<()>,
                    )
                })?;
            let sender_account_id = AccountId::from_str(sender_account_id_str).map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid sender account ID: {e}"),
                    None::<()>,
                )
            })?;

            let receiver_account_id_str =
                nft_obj.get("receiverAccountId").and_then(|v| v.as_str()).ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "NFT receiverAccountId is required".to_string(),
                        None::<()>,
                    )
                })?;
            let receiver_account_id =
                AccountId::from_str(receiver_account_id_str).map_err(|e| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        format!("Invalid receiver account ID: {e}"),
                        None::<()>,
                    )
                })?;

            let token_id_str =
                nft_obj.get("tokenId").and_then(|v| v.as_str()).ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "NFT tokenId is required".to_string(),
                        None::<()>,
                    )
                })?;
            let token_id = TokenId::from_str(token_id_str).map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid token ID: {e}"),
                    None::<()>,
                )
            })?;

            let serial_number_str =
                nft_obj.get("serialNumber").and_then(|v| v.as_str()).ok_or_else(|| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "NFT serialNumber is required".to_string(),
                        None::<()>,
                    )
                })?;
            let serial_number = serial_number_str.parse::<u64>().map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid serial number: {e}"),
                    None::<()>,
                )
            })?;

            let nft_id = NftId { token_id, serial: serial_number };

            if is_approved {
                tx.approved_nft_transfer(nft_id, sender_account_id, receiver_account_id);
            } else {
                tx.nft_transfer(nft_id, sender_account_id, receiver_account_id);
            }
        }
    }

    if let Some(params) = common_transaction_params {
        fill_common_transaction_params(&mut tx, &params, client);
    }

    let tx_response = tx.execute(client).await.map_err(|e| from_hedera_error(e))?;
    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(AccountUpdateResponse { status: tx_receipt.status.as_str_name().to_string() })
}
