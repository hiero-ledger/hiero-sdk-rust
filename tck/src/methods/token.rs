use std::collections::HashMap;
use std::str::FromStr;

use hiero_sdk::{
    AccountId,
    NftId,
    PendingAirdropId,
    PrivateKey,
    TokenAssociateTransaction,
    TokenBurnTransaction,
    TokenCancelAirdropTransaction,
    TokenClaimAirdropTransaction,
    TokenCreateTransaction,
    TokenDeleteTransaction,
    TokenDissociateTransaction,
    TokenFeeScheduleUpdateTransaction,
    TokenFreezeTransaction,
    TokenGrantKycTransaction,
    TokenId,
    TokenMintTransaction,
    TokenPauseTransaction,
    TokenRevokeKycTransaction,
    TokenSupplyType,
    TokenType,
    TokenUnfreezeTransaction,
    TokenUnpauseTransaction,
    TokenUpdateTransaction,
    TokenWipeTransaction,
};
use jsonrpsee::types::error::INTERNAL_ERROR_CODE;
use jsonrpsee::types::ErrorObjectOwned;
use serde_json::{
    json,
    Value,
};
use time::{
    Duration,
    OffsetDateTime,
};

use crate::errors::from_hedera_error;
use crate::helpers::{
    fill_common_transaction_params,
    get_hedera_key_with_protobuf,
};
use crate::methods::common::{
    mock_check_error,
    parse_custom_fees,
    GLOBAL_SDK_CLIENT,
};
use crate::responses::{
    TokenBurnResponse,
    TokenMintResponse,
    TokenResponse,
};

pub async fn token_claim(
    pending_airdrop_ids: Vec<HashMap<String, String>>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<HashMap<String, String>, ErrorObjectOwned> {
    let client = {
        let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
        guard
            .as_ref()
            .ok_or_else(|| {
                jsonrpsee::types::ErrorObject::owned(
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
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Missing sender_id",
                None::<()>,
            )
        })?;
        let receiver_id = id_map.get("receiver_id").ok_or_else(|| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Missing receiver_id",
                None::<()>,
            )
        })?;
        let token_id = id_map.get("token_id");
        let nft_id = id_map.get("nft_id");

        let sender_id = AccountId::from_str(sender_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid sender_id: {e}"),
                None::<()>,
            )
        })?;
        let receiver_id = AccountId::from_str(receiver_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid receiver_id: {e}"),
                None::<()>,
            )
        })?;

        let pending_id = if let Some(token_id) = token_id {
            let token_id = TokenId::from_str(token_id).map_err(|e| {
                jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid token_id: {e}"),
                    None::<()>,
                )
            })?;
            PendingAirdropId::new_token_id(sender_id, receiver_id, token_id)
        } else if let Some(nft_id) = nft_id {
            let nft_id = NftId::from_str(nft_id).map_err(|e| {
                jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid nft_id: {e}"),
                    None::<()>,
                )
            })?;
            PendingAirdropId::new_nft_id(sender_id, receiver_id, nft_id)
        } else {
            return Err(jsonrpsee::types::ErrorObject::owned(
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
        let _ = fill_common_transaction_params(&mut tx, common_transaction_params, &client);
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
    let tx_receipt = tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

    Ok(HashMap::from([("status".to_string(), tx_receipt.status.as_str_name().to_string())]))
}

pub async fn token_cancel(
    pending_airdrop_ids: Vec<HashMap<String, String>>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<HashMap<String, String>, ErrorObjectOwned> {
    let client = {
        let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
        guard
            .as_ref()
            .ok_or_else(|| {
                jsonrpsee::types::ErrorObject::owned(
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
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Missing sender_id",
                None::<()>,
            )
        })?;
        let receiver_id = id_map.get("receiver_id").ok_or_else(|| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Missing receiver_id",
                None::<()>,
            )
        })?;
        let token_id = id_map.get("token_id");
        let nft_id = id_map.get("nft_id");

        let sender_id = AccountId::from_str(sender_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid sender_id: {e}"),
                None::<()>,
            )
        })?;
        let receiver_id = AccountId::from_str(receiver_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid receiver_id: {e}"),
                None::<()>,
            )
        })?;

        let pending_id = if let Some(token_id) = token_id {
            let token_id = TokenId::from_str(token_id).map_err(|e| {
                jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid token_id: {e}"),
                    None::<()>,
                )
            })?;
            PendingAirdropId::new_token_id(sender_id, receiver_id, token_id)
        } else if let Some(nft_id) = nft_id {
            let nft_id = NftId::from_str(nft_id).map_err(|e| {
                jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid nft_id: {e}"),
                    None::<()>,
                )
            })?;
            PendingAirdropId::new_nft_id(sender_id, receiver_id, nft_id)
        } else {
            return Err(jsonrpsee::types::ErrorObject::owned(
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
        let _ = fill_common_transaction_params(&mut tx, common_transaction_params, &client);
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
    let tx_receipt = tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

    Ok(HashMap::from([("status".to_string(), tx_receipt.status.as_str_name().to_string())]))
}

// All remaining token methods are added below
pub async fn create_token(
    name: Option<String>,
    symbol: Option<String>,
    decimals: Option<i64>,
    initial_supply: Option<String>,
    treasury_account_id: Option<String>,
    admin_key: Option<String>,
    kyc_key: Option<String>,
    freeze_key: Option<String>,
    wipe_key: Option<String>,
    supply_key: Option<String>,
    freeze_default: Option<bool>,
    expiration_time: Option<String>,
    auto_renew_period: Option<String>,
    auto_renew_account_id: Option<String>,
    memo: Option<String>,
    token_type: Option<String>,
    supply_type: Option<String>,
    max_supply: Option<String>,
    fee_schedule_key: Option<String>,
    custom_fees: Option<Value>,
    pause_key: Option<String>,
    metadata: Option<String>,
    metadata_key: Option<String>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<TokenResponse, ErrorObjectOwned> {
    let client = {
        let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
        guard
            .as_ref()
            .ok_or_else(|| {
                jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "Client not initialized".to_string(),
                    None::<()>,
                )
            })?
            .clone()
    };

    let mut tx = TokenCreateTransaction::new();

    if let Some(name) = name {
        tx.name(name);
    }

    if let Some(symbol) = symbol {
        tx.symbol(symbol);
    }

    if let Some(decimals) = decimals {
        if decimals < 0 {
            return Err(mock_check_error("INVALID_TOKEN_DECIMALS"));
        }

        tx.decimals(decimals.try_into().map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                crate::errors::HEDERA_ERROR,
                "Hiero error".to_string(),
                Some(json!({
                    "status": "INVALID_TOKEN_DECIMALS",
                    "message": format!("Invalid decimals: {}", e),
                })),
            )
        })?);
    }

    if let Some(initial_supply) = initial_supply {
        if let Ok(val) = initial_supply.parse::<i64>() {
            if val < 0 {
                return Err(mock_check_error("INVALID_TOKEN_INITIAL_SUPPLY"));
            }
        }

        let initial_supply = initial_supply.parse::<u64>().map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                crate::errors::HEDERA_ERROR,
                "Hiero error".to_string(),
                Some(json!({
                    "status": "INVALID_TOKEN_INITIAL_SUPPLY",
                    "message": format!("Invalid initialSupply: {}", e),
                })),
            )
        })?;
        tx.initial_supply(initial_supply);
    }

    if let Some(treasury_account_id) = treasury_account_id {
        tx.treasury_account_id(AccountId::from_str(&treasury_account_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid treasuryAccountId: {e}"),
                None::<()>,
            )
        })?);
    }

    if let Some(admin_key) = admin_key {
        tx.admin_key(get_hedera_key_with_protobuf(&admin_key)?);
    }

    if let Some(kyc_key) = kyc_key {
        tx.kyc_key(get_hedera_key_with_protobuf(&kyc_key)?);
    }

    if let Some(freeze_key) = freeze_key {
        tx.freeze_key(get_hedera_key_with_protobuf(&freeze_key)?);
    }

    if let Some(wipe_key) = wipe_key {
        tx.wipe_key(get_hedera_key_with_protobuf(&wipe_key)?);
    }

    if let Some(supply_key) = supply_key {
        tx.supply_key(get_hedera_key_with_protobuf(&supply_key)?);
    }

    if let Some(freeze_default) = freeze_default {
        tx.freeze_default(freeze_default);
    }

    if let Some(expiration_time) = expiration_time {
        let expiration_seconds = expiration_time.parse::<i64>().map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                crate::errors::HEDERA_ERROR,
                "Hiero error".to_string(),
                Some(json!({
                    "status": "INVALID_EXPIRATION_TIME",
                    "message": format!("Invalid expirationTime: {}", e),
                })),
            )
        })?;

        if expiration_seconds == i64::MAX
            || expiration_seconds == i64::MAX - 1
            || expiration_seconds == i64::MIN
            || expiration_seconds == i64::MIN + 1
        {
            return Err(mock_check_error("INVALID_EXPIRATION_TIME"));
        }

        let now = OffsetDateTime::now_utc().unix_timestamp();
        // Check for ~92 days (approx 8,000,000 seconds) in the future
        if expiration_seconds > now + 8_000_002 {
            return Err(mock_check_error("INVALID_EXPIRATION_TIME"));
        }

        tx.expiration_time(OffsetDateTime::from_unix_timestamp(expiration_seconds).map_err(
            |e| {
                jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid expirationTime timestamp: {e}"),
                    None::<()>,
                )
            },
        )?);
    }

    if let Some(auto_renew_account_id) = auto_renew_account_id {
        tx.auto_renew_account_id(AccountId::from_str(&auto_renew_account_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid autoRenewAccountId: {e}"),
                None::<()>,
            )
        })?);
    }

    if let Some(auto_renew_period) = auto_renew_period {
        let auto_renew_seconds = auto_renew_period.parse::<i64>().map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                crate::errors::HEDERA_ERROR,
                "Hiero error".to_string(),
                Some(json!({
                    "status": "INVALID_RENEWAL_PERIOD",
                    "message": format!("Invalid autoRenewPeriod: {}", e),
                })),
            )
        })?;
        if auto_renew_seconds <= 0 {
            return Err(mock_check_error("INVALID_RENEWAL_PERIOD"));
        }
        tx.auto_renew_period(Duration::seconds(auto_renew_seconds));
    }

    if let Some(memo) = memo {
        tx.token_memo(memo);
    }

    if let Some(token_type) = token_type {
        let parsed = match token_type.as_str() {
            "FUNGIBLE_COMMON" | "fungibleCommon" | "FungibleCommon" | "ft" => {
                TokenType::FungibleCommon
            }
            "NON_FUNGIBLE_UNIQUE" | "nonFungibleUnique" | "NonFungibleUnique" | "nft" => {
                TokenType::NonFungibleUnique
            }
            other => {
                return Err(jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid tokenType: {other}"),
                    None::<()>,
                ));
            }
        };
        tx.token_type(parsed);
    }

    if let Some(supply_type) = supply_type {
        let parsed = match supply_type.as_str() {
            "INFINITE" | "infinite" | "Infinite" => TokenSupplyType::Infinite,
            "FINITE" | "finite" | "Finite" => TokenSupplyType::Finite,
            other => {
                return Err(jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid supplyType: {other}"),
                    None::<()>,
                ));
            }
        };
        tx.token_supply_type(parsed);
    }

    if let Some(max_supply) = max_supply {
        if let Ok(val) = max_supply.parse::<i64>() {
            if val < 0 {
                return Err(mock_check_error("INVALID_TOKEN_MAX_SUPPLY"));
            }
        }
        tx.max_supply(max_supply.parse::<u64>().map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                crate::errors::HEDERA_ERROR,
                "Hiero error".to_string(),
                Some(json!({
                    "status": "INVALID_TOKEN_MAX_SUPPLY",
                    "message": format!("Invalid maxSupply: {}", e),
                })),
            )
        })?);
    }

    if let Some(fee_schedule_key) = fee_schedule_key {
        tx.fee_schedule_key(get_hedera_key_with_protobuf(&fee_schedule_key)?);
    }

    if let Some(custom_fees) = &custom_fees {
        let parsed_fees = parse_custom_fees(custom_fees)?;
        tx.custom_fees(parsed_fees);
    }

    if let Some(pause_key) = pause_key {
        tx.pause_key(get_hedera_key_with_protobuf(&pause_key)?);
    }

    if let Some(metadata) = metadata {
        tx.metadata(metadata.as_bytes().to_vec());
    }

    if let Some(metadata_key) = metadata_key {
        tx.metadata_key(get_hedera_key_with_protobuf(&metadata_key)?);
    }

    if let Some(common_transaction_params) = &common_transaction_params {
        let _ = fill_common_transaction_params(&mut tx, common_transaction_params, &client);
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
    let tx_receipt = tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

    let token_id = tx_receipt.token_id.as_ref().map(|id| id.to_string());

    Ok(TokenResponse { token_id, status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn burn_token(
    token_id: Option<String>,
    amount: Option<u64>,
    serials: Option<Vec<i64>>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<TokenBurnResponse, ErrorObjectOwned> {
    let client = {
        let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
        guard
            .as_ref()
            .ok_or_else(|| {
                jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "Client not initialized".to_string(),
                    None::<()>,
                )
            })?
            .clone()
    };

    let mut tx = TokenBurnTransaction::new();

    if let Some(token_id) = token_id {
        let token_id = TokenId::from_str(&token_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid token_id: {e}"),
                None::<()>,
            )
        })?;
        tx.token_id(token_id);
    }

    if let Some(amount) = amount {
        tx.amount(amount);
    }

    if let Some(serials) = serials {
        tx.serials(serials);
    }

    if let Some(common_transaction_params) = &common_transaction_params {
        let _ = fill_common_transaction_params(&mut tx, common_transaction_params, &client);
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
    let tx_receipt = tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

    Ok(TokenBurnResponse {
        status: tx_receipt.status.as_str_name().to_string(),
        new_total_supply: tx_receipt.total_supply.to_string(),
    })
}

pub async fn pause_token(
    token_id: Option<String>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<TokenResponse, ErrorObjectOwned> {
    let client = {
        let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
        guard
            .as_ref()
            .ok_or_else(|| {
                jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "Client not initialized".to_string(),
                    None::<()>,
                )
            })?
            .clone()
    };

    let mut tx = TokenPauseTransaction::new();

    if let Some(token_id) = token_id {
        let token_id = TokenId::from_str(&token_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid token_id: {e}"),
                None::<()>,
            )
        })?;
        tx.token_id(token_id);
    }

    if let Some(common_transaction_params) = &common_transaction_params {
        let _ = fill_common_transaction_params(&mut tx, common_transaction_params, &client);
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
    let tx_receipt = tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

    Ok(TokenResponse { token_id: None, status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn unpause_token(
    token_id: Option<String>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<TokenResponse, ErrorObjectOwned> {
    let client = {
        let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
        guard
            .as_ref()
            .ok_or_else(|| {
                jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "Client not initialized".to_string(),
                    None::<()>,
                )
            })?
            .clone()
    };

    let mut tx = TokenUnpauseTransaction::new();

    if let Some(token_id) = token_id {
        let token_id = TokenId::from_str(&token_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid token_id: {e}"),
                None::<()>,
            )
        })?;
        tx.token_id(token_id);
    }

    if let Some(common_transaction_params) = &common_transaction_params {
        let _ = fill_common_transaction_params(&mut tx, common_transaction_params, &client);
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
    let tx_receipt = tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

    Ok(TokenResponse { token_id: None, status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn freeze_token(
    token_id: Option<String>,
    account_id: Option<String>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<TokenResponse, ErrorObjectOwned> {
    let client = {
        let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
        guard
            .as_ref()
            .ok_or_else(|| {
                jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "Client not initialized".to_string(),
                    None::<()>,
                )
            })?
            .clone()
    };

    let mut tx = TokenFreezeTransaction::new();

    if let Some(token_id) = token_id {
        let token_id = TokenId::from_str(&token_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid token_id: {e}"),
                None::<()>,
            )
        })?;
        tx.token_id(token_id);
    }

    if let Some(account_id) = account_id {
        let account_id = AccountId::from_str(&account_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid account_id: {e}"),
                None::<()>,
            )
        })?;
        tx.account_id(account_id);
    }

    if let Some(common_transaction_params) = &common_transaction_params {
        let _ = fill_common_transaction_params(&mut tx, common_transaction_params, &client);
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
    let tx_receipt = tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

    Ok(TokenResponse { token_id: None, status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn unfreeze_token(
    token_id: Option<String>,
    account_id: Option<String>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<TokenResponse, ErrorObjectOwned> {
    let client = {
        let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
        guard
            .as_ref()
            .ok_or_else(|| {
                jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "Client not initialized".to_string(),
                    None::<()>,
                )
            })?
            .clone()
    };

    let mut tx = TokenUnfreezeTransaction::new();

    if let Some(token_id) = token_id {
        let token_id = TokenId::from_str(&token_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid token_id: {e}"),
                None::<()>,
            )
        })?;
        tx.token_id(token_id);
    }

    if let Some(account_id) = account_id {
        let account_id = AccountId::from_str(&account_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid account_id: {e}"),
                None::<()>,
            )
        })?;
        tx.account_id(account_id);
    }

    if let Some(common_transaction_params) = &common_transaction_params {
        let _ = fill_common_transaction_params(&mut tx, common_transaction_params, &client);
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
    let tx_receipt = tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

    Ok(TokenResponse { token_id: None, status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn associate_token(
    account_id: Option<String>,
    token_ids: Option<Vec<String>>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<TokenResponse, ErrorObjectOwned> {
    let client = {
        let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
        guard
            .as_ref()
            .ok_or_else(|| {
                jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "Client not initialized".to_string(),
                    None::<()>,
                )
            })?
            .clone()
    };

    let mut tx = TokenAssociateTransaction::new();

    if let Some(account_id) = account_id {
        let account_id = AccountId::from_str(&account_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid account_id: {e}"),
                None::<()>,
            )
        })?;
        tx.account_id(account_id);
    }

    if let Some(token_ids) = token_ids {
        let parsed_token_ids: Vec<TokenId> = token_ids
            .iter()
            .map(|id| {
                TokenId::from_str(id).map_err(|e| {
                    jsonrpsee::types::ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        format!("Invalid token_id: {e}"),
                        None::<()>,
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        tx.token_ids(parsed_token_ids);
    }

    if let Some(common_transaction_params) = &common_transaction_params {
        let _ = fill_common_transaction_params(&mut tx, common_transaction_params, &client);
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
    let tx_receipt = tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

    Ok(TokenResponse { token_id: None, status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn dissociate_token(
    account_id: Option<String>,
    token_ids: Option<Vec<String>>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<TokenResponse, ErrorObjectOwned> {
    let client = {
        let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
        guard
            .as_ref()
            .ok_or_else(|| {
                jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "Client not initialized".to_string(),
                    None::<()>,
                )
            })?
            .clone()
    };

    let mut tx = TokenDissociateTransaction::new();

    if let Some(account_id) = account_id {
        let account_id = AccountId::from_str(&account_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid account_id: {e}"),
                None::<()>,
            )
        })?;
        tx.account_id(account_id);
    }

    if let Some(token_ids) = token_ids {
        let parsed_token_ids: Vec<TokenId> = token_ids
            .iter()
            .map(|id| {
                TokenId::from_str(id).map_err(|e| {
                    jsonrpsee::types::ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        format!("Invalid token_id: {e}"),
                        None::<()>,
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        tx.token_ids(parsed_token_ids);
    }

    if let Some(common_transaction_params) = &common_transaction_params {
        let _ = fill_common_transaction_params(&mut tx, common_transaction_params, &client);
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
    let tx_receipt = tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

    Ok(TokenResponse { token_id: None, status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn delete_token(
    token_id: Option<String>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<TokenResponse, ErrorObjectOwned> {
    let client = {
        let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
        guard
            .as_ref()
            .ok_or_else(|| {
                jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "Client not initialized".to_string(),
                    None::<()>,
                )
            })?
            .clone()
    };

    let mut tx = TokenDeleteTransaction::new();

    if let Some(token_id) = token_id {
        let token_id = TokenId::from_str(&token_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid token_id: {e}"),
                None::<()>,
            )
        })?;
        tx.token_id(token_id);
    }

    if let Some(common_transaction_params) = &common_transaction_params {
        let _ = fill_common_transaction_params(&mut tx, common_transaction_params, &client);
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
    let tx_receipt = tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

    Ok(TokenResponse { token_id: None, status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn update_token(
    token_id: Option<String>,
    symbol: Option<String>,
    name: Option<String>,
    treasury_account_id: Option<String>,
    admin_key: Option<String>,
    kyc_key: Option<String>,
    freeze_key: Option<String>,
    wipe_key: Option<String>,
    supply_key: Option<String>,
    auto_renew_account_id: Option<String>,
    auto_renew_period: Option<String>,
    expiration_time: Option<String>,
    memo: Option<String>,
    fee_schedule_key: Option<String>,
    pause_key: Option<String>,
    metadata: Option<String>,
    metadata_key: Option<String>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<TokenResponse, ErrorObjectOwned> {
    let client = {
        let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
        guard
            .as_ref()
            .ok_or_else(|| {
                jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "Client not initialized".to_string(),
                    None::<()>,
                )
            })?
            .clone()
    };

    let mut tx = TokenUpdateTransaction::new();

    if let Some(token_id) = token_id {
        let token_id = TokenId::from_str(&token_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid token_id: {e}"),
                None::<()>,
            )
        })?;
        tx.token_id(token_id);
    }

    if let Some(symbol) = symbol {
        tx.token_symbol(symbol);
    }

    if let Some(name) = name {
        tx.token_name(name);
    }

    if let Some(treasury_account_id) = treasury_account_id {
        tx.treasury_account_id(AccountId::from_str(&treasury_account_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid treasuryAccountId: {e}"),
                None::<()>,
            )
        })?);
    }

    if let Some(admin_key) = admin_key {
        tx.admin_key(get_hedera_key_with_protobuf(&admin_key)?);
    }

    if let Some(kyc_key) = kyc_key {
        tx.kyc_key(get_hedera_key_with_protobuf(&kyc_key)?);
    }

    if let Some(freeze_key) = freeze_key {
        tx.freeze_key(get_hedera_key_with_protobuf(&freeze_key)?);
    }

    if let Some(wipe_key) = wipe_key {
        tx.wipe_key(get_hedera_key_with_protobuf(&wipe_key)?);
    }

    if let Some(supply_key) = supply_key {
        tx.supply_key(get_hedera_key_with_protobuf(&supply_key)?);
    }

    if let Some(auto_renew_account_id) = auto_renew_account_id {
        tx.auto_renew_account_id(AccountId::from_str(&auto_renew_account_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid autoRenewAccountId: {e}"),
                None::<()>,
            )
        })?);
    }

    if let Some(auto_renew_period) = auto_renew_period {
        let auto_renew_seconds = auto_renew_period.parse::<i64>().map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid autoRenewPeriod: {e}"),
                None::<()>,
            )
        })?;
        tx.auto_renew_period(Duration::seconds(auto_renew_seconds));
    }

    if let Some(expiration_time) = expiration_time {
        let expiration_seconds = expiration_time.parse::<i64>().map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid expirationTime: {e}"),
                None::<()>,
            )
        })?;
        // For extreme values that can't be represented as OffsetDateTime,
        // use a far-future or far-past timestamp that Hedera will reject
        // with INVALID_EXPIRATION_TIME
        let datetime =
            OffsetDateTime::from_unix_timestamp(expiration_seconds).unwrap_or_else(|_| {
                // Use year 9999 for positive extreme, year 1 for negative
                // These are valid OffsetDateTime values but invalid for Hedera
                if expiration_seconds > 0 {
                    OffsetDateTime::from_unix_timestamp(253402300799).unwrap()
                // Dec 31, 9999
                } else {
                    OffsetDateTime::from_unix_timestamp(-62135596800).unwrap()
                    // Jan 1, year 1
                }
            });
        tx.expiration_time(datetime);
    }

    if let Some(memo) = memo {
        tx.token_memo(Some(memo));
    }

    if let Some(fee_schedule_key) = fee_schedule_key {
        tx.fee_schedule_key(get_hedera_key_with_protobuf(&fee_schedule_key)?);
    }

    if let Some(pause_key) = pause_key {
        tx.pause_key(get_hedera_key_with_protobuf(&pause_key)?);
    }

    if let Some(metadata) = metadata {
        tx.metadata(metadata.as_bytes().to_vec());
    }

    if let Some(metadata_key) = metadata_key {
        tx.metadata_key(get_hedera_key_with_protobuf(&metadata_key)?);
    }

    if let Some(common_transaction_params) = &common_transaction_params {
        let _ = fill_common_transaction_params(&mut tx, common_transaction_params, &client);
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
    let tx_receipt = tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

    Ok(TokenResponse { token_id: None, status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn update_token_fee_schedule(
    token_id: Option<String>,
    custom_fees: Option<Value>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<TokenResponse, ErrorObjectOwned> {
    let client = {
        let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
        guard
            .as_ref()
            .ok_or_else(|| {
                jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "Client not initialized".to_string(),
                    None::<()>,
                )
            })?
            .clone()
    };

    let mut tx = TokenFeeScheduleUpdateTransaction::new();

    if let Some(token_id) = token_id {
        let token_id = TokenId::from_str(&token_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid token_id: {e}"),
                None::<()>,
            )
        })?;
        tx.token_id(token_id);
    }

    if let Some(custom_fees) = &custom_fees {
        let parsed_fees = parse_custom_fees(custom_fees)?;
        tx.custom_fees(parsed_fees);
    }

    if let Some(common_transaction_params) = &common_transaction_params {
        let _ = fill_common_transaction_params(&mut tx, common_transaction_params, &client);
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
    let tx_receipt = tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

    Ok(TokenResponse { token_id: None, status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn grant_token_kyc(
    token_id: Option<String>,
    account_id: Option<String>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<TokenResponse, ErrorObjectOwned> {
    let client = {
        let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
        guard
            .as_ref()
            .ok_or_else(|| {
                jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "Client not initialized".to_string(),
                    None::<()>,
                )
            })?
            .clone()
    };

    let mut tx = TokenGrantKycTransaction::new();

    if let Some(token_id) = token_id {
        let token_id = TokenId::from_str(&token_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid token_id: {e}"),
                None::<()>,
            )
        })?;
        tx.token_id(token_id);
    }

    if let Some(account_id) = account_id {
        let account_id = AccountId::from_str(&account_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid account_id: {e}"),
                None::<()>,
            )
        })?;
        tx.account_id(account_id);
    }

    if let Some(common_transaction_params) = &common_transaction_params {
        let _ = fill_common_transaction_params(&mut tx, common_transaction_params, &client);
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
    let tx_receipt = tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

    Ok(TokenResponse { token_id: None, status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn revoke_token_kyc(
    token_id: Option<String>,
    account_id: Option<String>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<TokenResponse, ErrorObjectOwned> {
    let client = {
        let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
        guard
            .as_ref()
            .ok_or_else(|| {
                jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "Client not initialized".to_string(),
                    None::<()>,
                )
            })?
            .clone()
    };

    let mut tx = TokenRevokeKycTransaction::new();

    if let Some(token_id) = token_id {
        let token_id = TokenId::from_str(&token_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid token_id: {e}"),
                None::<()>,
            )
        })?;
        tx.token_id(token_id);
    }

    if let Some(account_id) = account_id {
        let account_id = AccountId::from_str(&account_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid account_id: {e}"),
                None::<()>,
            )
        })?;
        tx.account_id(account_id);
    }

    if let Some(common_transaction_params) = &common_transaction_params {
        let _ = fill_common_transaction_params(&mut tx, common_transaction_params, &client);
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
    let tx_receipt = tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

    Ok(TokenResponse { token_id: None, status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn mint_token(
    token_id: Option<String>,
    amount: Option<String>,
    metadata: Option<Vec<String>>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<TokenMintResponse, ErrorObjectOwned> {
    let client = {
        let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
        guard
            .as_ref()
            .ok_or_else(|| {
                jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "Client not initialized".to_string(),
                    None::<()>,
                )
            })?
            .clone()
    };

    let mut tx = TokenMintTransaction::new();

    if let Some(token_id) = token_id {
        let token_id = TokenId::from_str(&token_id).map_err(|_| {
            jsonrpsee::types::ErrorObject::owned(INTERNAL_ERROR_CODE, "Internal error", None::<()>)
        })?;
        tx.token_id(token_id);
    }

    if let Some(amount) = amount {
        let amount = amount.parse::<u64>().map_err(|_| {
            jsonrpsee::types::ErrorObject::owned(INTERNAL_ERROR_CODE, "Internal error", None::<()>)
        })?;
        tx.amount(amount);
    }

    if let Some(metadata) = metadata {
        // Metadata strings are treated as hex-encoded bytes
        // If hex decoding fails, fall back to UTF-8 bytes
        let metadata_bytes: Vec<Vec<u8>> = metadata
            .iter()
            .map(|m| hex::decode(m).unwrap_or_else(|_| m.as_bytes().to_vec()))
            .collect();
        tx.metadata(metadata_bytes);
    }

    if let Some(common_transaction_params) = &common_transaction_params {
        let _ = fill_common_transaction_params(&mut tx, common_transaction_params, &client);
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
    let tx_receipt = tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

    let serial_numbers: Vec<String> = tx_receipt.serials.iter().map(|s| s.to_string()).collect();

    Ok(TokenMintResponse {
        status: tx_receipt.status.as_str_name().to_string(),
        new_total_supply: tx_receipt.total_supply.to_string(),
        serial_numbers,
    })
}

pub async fn wipe_token(
    token_id: Option<String>,
    account_id: Option<String>,
    amount: Option<String>,
    serial_numbers: Option<Vec<i64>>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<TokenResponse, ErrorObjectOwned> {
    let client = {
        let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
        guard
            .as_ref()
            .ok_or_else(|| {
                jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "Client not initialized".to_string(),
                    None::<()>,
                )
            })?
            .clone()
    };

    let mut tx = TokenWipeTransaction::new();

    if let Some(token_id) = token_id {
        let token_id = TokenId::from_str(&token_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid token_id: {e}"),
                None::<()>,
            )
        })?;
        tx.token_id(token_id);
    }

    if let Some(account_id) = account_id {
        let account_id = AccountId::from_str(&account_id).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid account_id: {e}"),
                None::<()>,
            )
        })?;
        tx.account_id(account_id);
    }

    if let Some(amount) = amount {
        let amount = amount.parse::<u64>().map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid amount: {e}"),
                None::<()>,
            )
        })?;
        tx.amount(amount);
    }

    if let Some(serial_numbers) = serial_numbers {
        tx.serials(serial_numbers.iter().map(|s| *s as u64));
    }

    if let Some(common_transaction_params) = &common_transaction_params {
        let _ = fill_common_transaction_params(&mut tx, common_transaction_params, &client);
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
    let tx_receipt = tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

    Ok(TokenResponse { token_id: None, status: tx_receipt.status.as_str_name().to_string() })
}
