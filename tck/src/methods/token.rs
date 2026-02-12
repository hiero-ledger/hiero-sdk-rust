use std::collections::HashMap;
use std::str::FromStr;

use hiero_sdk::{
    AccountId,
    AnyCustomFee,
    Client,
    FixedFeeData,
    FractionalFeeData,
    NftId,
    PendingAirdropId,
    RoyaltyFeeData,
    TokenAssociateTransaction,
    TokenBurnTransaction,
    TokenCancelAirdropTransaction,
    TokenClaimAirdropTransaction,
    TokenCreateTransaction,
    TokenDeleteTransaction,
    TokenFeeScheduleUpdateTransaction,
    TokenFreezeTransaction,
    TokenId,
    TokenMintTransaction,
    TokenPauseTransaction,
    TokenSupplyType,
    TokenType,
    TokenUpdateTransaction,
};
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::error::INTERNAL_ERROR_CODE;
use jsonrpsee::types::{
    ErrorObject,
    ErrorObjectOwned,
};
use serde_json::{
    json,
    Value,
};
use time::{
    Duration,
    OffsetDateTime,
};

use crate::common::{
    internal_error,
    mock_check_error,
    mock_consensus_error,
};
use crate::errors::{
    from_hedera_error,
    HEDERA_ERROR,
};
use crate::helpers::{
    fill_common_transaction_params,
    get_hedera_key,
};
use crate::responses::{
    TokenMintResponse,
    TokenResponse,
};

#[rpc(server, client)]
pub trait TokenRpc {
    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenCreateTransaction.md#createToken
    */
    #[method(name = "createToken")]
    async fn create_token(
        &self,
        name: Option<String>,
        symbol: Option<String>,
        decimals: Option<u32>,
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
    ) -> Result<HashMap<String, String>, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenClaimAirdropTransaction.md#tokenClaim
    */
    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenDeleteTransaction.md#deleteToken
    */
    #[method(name = "deleteToken")]
    async fn delete_token(
        &self,
        token_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<HashMap<String, String>, ErrorObjectOwned>;

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

    #[method(name = "associateToken")]
    async fn associate_token(
        &self,
        account_id: Option<String>,
        token_ids: Option<Vec<String>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<HashMap<String, String>, ErrorObjectOwned>;

    #[method(name = "mintToken")]
    async fn mint_token(
        &self,
        token_id: Option<String>,
        amount: Option<String>,
        metadata: Option<Vec<String>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenMintResponse, ErrorObjectOwned>;

    #[method(name = "freezeToken")]
    async fn freeze_token(
        &self,
        token_id: Option<String>,
        account_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    #[method(name = "pauseToken")]
    async fn pause_token(
        &self,
        token_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenUpdateTransaction.md#updateToken
    */
    #[method(name = "updateToken")]
    async fn update_token(
        &self,
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
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    #[method(name = "updateTokenFeeSchedule")]
    async fn update_token_fee_schedule(
        &self,
        token_id: Option<String>,
        custom_fees: Option<Value>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned>;
}

pub async fn create_token(
    client: &Client,
    name: Option<String>,
    symbol: Option<String>,
    decimals: Option<u32>,
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
    _custom_fees: Option<Value>, // TODO: Implement custom fees for tokens
    pause_key: Option<String>,
    metadata: Option<String>,
    metadata_key: Option<String>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<HashMap<String, String>, ErrorObjectOwned> {
    let mut tx = TokenCreateTransaction::new();

    if let Some(name) = name {
        tx.name(name);
    }

    if let Some(symbol) = symbol {
        tx.symbol(symbol);
    }

    if let Some(decimals) = decimals {
        tx.decimals(decimals);
    }

    if let Some(initial_supply) = initial_supply {
        let supply = initial_supply.parse::<u64>().map_err(|e| {
            ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid initial_supply: {e}"),
                None::<()>,
            )
        })?;
        tx.initial_supply(supply);
    }

    if let Some(treasury_account_id) = treasury_account_id {
        let account_id = AccountId::from_str(&treasury_account_id).map_err(|e| {
            ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid treasury_account_id: {e}"),
                None::<()>,
            )
        })?;
        tx.treasury_account_id(account_id);
    }

    if let Some(admin_key) = admin_key {
        tx.admin_key(get_hedera_key(&admin_key)?);
    }

    if let Some(kyc_key) = kyc_key {
        tx.kyc_key(get_hedera_key(&kyc_key)?);
    }

    if let Some(freeze_key) = freeze_key {
        tx.freeze_key(get_hedera_key(&freeze_key)?);
    }

    if let Some(wipe_key) = wipe_key {
        tx.wipe_key(get_hedera_key(&wipe_key)?);
    }

    if let Some(supply_key) = supply_key {
        tx.supply_key(get_hedera_key(&supply_key)?);
    }

    if let Some(freeze_default) = freeze_default {
        tx.freeze_default(freeze_default);
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

        tx.expiration_time(
            OffsetDateTime::from_unix_timestamp(timestamp)
                .map_err(|e| mock_consensus_error("INVALID_EXPIRATION_TIME", &e.to_string()))?,
        );
    }

    if let Some(auto_renew_account_id) = auto_renew_account_id {
        let account_id = AccountId::from_str(&auto_renew_account_id).map_err(|e| {
            ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid auto_renew_account_id: {e}"),
                None::<()>,
            )
        })?;
        tx.auto_renew_account_id(account_id);
    }

    if let Some(auto_renew_period) = auto_renew_period {
        let seconds = auto_renew_period.parse::<i64>().map_err(|e| {
            ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid auto_renew_period: {e}"),
                None::<()>,
            )
        })?;
        tx.auto_renew_period(Duration::seconds(seconds));
    }

    if let Some(memo) = memo {
        tx.token_memo(memo);
    }

    if let Some(token_type) = token_type {
        let tt = match token_type.as_str() {
            "ft" => TokenType::FungibleCommon,
            "nft" => TokenType::NonFungibleUnique,
            _ => {
                return Err(ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid token_type: {}", token_type),
                    None::<()>,
                ))
            }
        };
        tx.token_type(tt);
    }

    if let Some(supply_type) = supply_type {
        let st = match supply_type.as_str() {
            "infinite" => TokenSupplyType::Infinite,
            "finite" => TokenSupplyType::Finite,
            _ => {
                return Err(ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid supply_type: {}", supply_type),
                    None::<()>,
                ))
            }
        };
        tx.token_supply_type(st);
    }

    if let Some(max_supply) = max_supply {
        let supply = max_supply
            .parse::<u64>()
            .map_err(|e| internal_error(format!("Invalid max_supply: {e}")))?;
        tx.max_supply(supply);
    }

    if let Some(fee_schedule_key) = fee_schedule_key {
        tx.fee_schedule_key(get_hedera_key(&fee_schedule_key)?);
    }

    if let Some(pause_key) = pause_key {
        tx.pause_key(get_hedera_key(&pause_key)?);
    }

    if let Some(metadata) = metadata {
        tx.metadata(metadata.into_bytes());
    }

    if let Some(metadata_key) = metadata_key {
        tx.metadata_key(get_hedera_key(&metadata_key)?);
    }

    if let Some(common_transaction_params) = &common_transaction_params {
        fill_common_transaction_params(&mut tx, common_transaction_params, client);
    }

    let tx_response = tx.execute(client).await.map_err(|e| from_hedera_error(e))?;
    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(HashMap::from([
        ("tokenId".to_string(), tx_receipt.token_id.map(|id| id.to_string()).unwrap_or_default()),
        ("status".to_string(), tx_receipt.status.as_str_name().to_string()),
    ]))
}

pub async fn delete_token(
    client: &Client,
    token_id: Option<String>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<HashMap<String, String>, ErrorObjectOwned> {
    let mut tx = TokenDeleteTransaction::new();

    if let Some(token_id_str) = token_id {
        let token_id = TokenId::from_str(&token_id_str)
            .map_err(|e| internal_error(format!("Invalid tokenId: {e}")))?;
        tx.token_id(token_id);
    }

    if let Some(common_transaction_params) = &common_transaction_params {
        fill_common_transaction_params(&mut tx, common_transaction_params, client);
    }

    let tx_response = tx.execute(client).await.map_err(|e| from_hedera_error(e))?;
    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(HashMap::from([("status".to_string(), tx_receipt.status.as_str_name().to_string())]))
}

pub async fn token_claim(
    client: &Client,
    pending_airdrop_ids: Vec<HashMap<String, String>>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<HashMap<String, String>, ErrorObjectOwned> {
    // Parse pending_airdrop_ids from HashMap<String, String> to PendingAirdropId
    let mut parsed_ids = Vec::new();
    for id_map in pending_airdrop_ids {
        let sender_id =
            id_map.get("sender_id").ok_or_else(|| internal_error("Missing sender_id"))?;
        let receiver_id =
            id_map.get("receiver_id").ok_or_else(|| internal_error("Missing receiver_id"))?;
        let token_id = id_map.get("token_id");
        let nft_id = id_map.get("nft_id");

        let sender_id = AccountId::from_str(sender_id)
            .map_err(|e| internal_error(format!("Invalid sender_id: {e}")))?;
        let receiver_id = AccountId::from_str(receiver_id)
            .map_err(|e| internal_error(format!("Invalid receiver_id: {e}")))?;

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
            let nft_id = NftId::from_str(nft_id)
                .map_err(|e| internal_error(format!("Invalid nft_id: {e}")))?;
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
        fill_common_transaction_params(&mut tx, common_transaction_params, client);
    }

    let tx_response = tx.execute(client).await.map_err(|e| from_hedera_error(e))?;
    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(HashMap::from([("status".to_string(), tx_receipt.status.as_str_name().to_string())]))
}

pub async fn token_cancel(
    client: &Client,
    pending_airdrop_ids: Vec<HashMap<String, String>>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<HashMap<String, String>, ErrorObjectOwned> {
    // Parse pending_airdrop_ids from HashMap<String, String> to PendingAirdropId
    let mut parsed_ids = Vec::new();
    for id_map in pending_airdrop_ids {
        let sender_id =
            id_map.get("sender_id").ok_or_else(|| internal_error("Missing sender_id"))?;
        let receiver_id =
            id_map.get("receiver_id").ok_or_else(|| internal_error("Missing receiver_id"))?;
        let token_id = id_map.get("token_id");
        let nft_id = id_map.get("nft_id");

        let sender_id = AccountId::from_str(sender_id)
            .map_err(|e| internal_error(format!("Invalid sender_id: {e}")))?;
        let receiver_id = AccountId::from_str(receiver_id)
            .map_err(|e| internal_error(format!("Invalid receiver_id: {e}")))?;

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
            let nft_id = NftId::from_str(nft_id)
                .map_err(|e| internal_error(format!("Invalid nft_id: {e}")))?;
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
        fill_common_transaction_params(&mut tx, common_transaction_params, client);
    }

    let tx_response = tx.execute(client).await.map_err(|e| from_hedera_error(e))?;
    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(HashMap::from([("status".to_string(), tx_receipt.status.as_str_name().to_string())]))
}

// Helper function for building token mint transaction (used in schedule creation)
pub(crate) fn build_token_mint_tx_from_value(
    params: &Value,
) -> Result<TokenMintTransaction, ErrorObjectOwned> {
    let mut tx = TokenMintTransaction::new();

    let token_id = params.get("tokenId").and_then(Value::as_str).ok_or_else(|| {
        ErrorObject::owned(
            INTERNAL_ERROR_CODE,
            "tokenId is required in mintToken".to_string(),
            None::<()>,
        )
    })?;
    let token_id =
        TokenId::from_str(token_id).map_err(|e| internal_error(format!("Invalid tokenId: {e}")))?;
    tx.token_id(token_id);

    if let Some(amount) =
        params.get("amount").and_then(|v| v.as_u64().or_else(|| v.as_str()?.parse::<u64>().ok()))
    {
        tx.amount(amount);
    }

    if let Some(metadata_array) = params.get("metadata").and_then(Value::as_array) {
        if !metadata_array.is_empty() {
            let mut metas = Vec::new();
            for m in metadata_array {
                if let Some(hex_str) = m.as_str() {
                    let bytes = hex::decode(hex_str).map_err(|e| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            format!("Invalid metadata hex: {e}"),
                            None::<()>,
                        )
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

// Helper function for building token burn transaction (used in schedule creation)
pub(crate) fn build_token_burn_tx_from_value(
    params: &Value,
) -> Result<TokenBurnTransaction, ErrorObjectOwned> {
    let mut tx = TokenBurnTransaction::new();

    let token_id = params.get("tokenId").and_then(Value::as_str).ok_or_else(|| {
        ErrorObject::owned(
            INTERNAL_ERROR_CODE,
            "tokenId is required in burnToken".to_string(),
            None::<()>,
        )
    })?;
    let token_id =
        TokenId::from_str(token_id).map_err(|e| internal_error(format!("Invalid tokenId: {e}")))?;
    tx.token_id(token_id);

    if let Some(amount) =
        params.get("amount").and_then(|v| v.as_u64().or_else(|| v.as_str()?.parse::<u64>().ok()))
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

pub async fn associate_token(
    client: &Client,
    account_id: Option<String>,
    token_ids: Option<Vec<String>>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<HashMap<String, String>, ErrorObjectOwned> {
    let mut tx = TokenAssociateTransaction::new();

    if let Some(account_id) = account_id {
        let account_id = AccountId::from_str(&account_id).map_err(|e| {
            ErrorObjectOwned::owned(
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
                    ErrorObjectOwned::owned(
                        INTERNAL_ERROR_CODE,
                        format!("Invalid token_id: {e}"),
                        None::<()>,
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        tx.token_ids(parsed_token_ids);
    }

    if let Some(common_transaction_params) = common_transaction_params {
        fill_common_transaction_params(&mut tx, &common_transaction_params, client);
    }

    let tx_response = tx.execute(client).await.map_err(|e| from_hedera_error(e))?;
    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(HashMap::from([("status".to_string(), tx_receipt.status.as_str_name().to_string())]))
}

pub async fn mint_token(
    client: &Client,
    token_id: Option<String>,
    amount: Option<String>,
    metadata: Option<Vec<String>>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<TokenMintResponse, ErrorObjectOwned> {
    let mut tx = TokenMintTransaction::new();

    if let Some(token_id) = token_id {
        tx.token_id(TokenId::from_str(&token_id).map_err(internal_error)?);
    }

    if let Some(amount) = amount {
        let amount_val = amount.parse::<u64>().map_err(internal_error)?;
        tx.amount(amount_val);
    }

    if let Some(metadata) = metadata {
        let decoded_metadata: Result<Vec<Vec<u8>>, _> = metadata
            .iter()
            .map(|m| {
                hex::decode(m).map_err(|e| internal_error(format!("Invalid hex metadata: {e}")))
            })
            .collect();
        tx.metadata(decoded_metadata?);
    }

    if let Some(params) = common_transaction_params {
        fill_common_transaction_params(&mut tx, &params, client);
    }

    let tx_response = tx.execute(client).await.map_err(from_hedera_error)?;
    let tx_receipt = tx_response.get_receipt(client).await.map_err(from_hedera_error)?;

    let serial_numbers: Vec<String> = tx_receipt.serials.iter().map(|s| s.to_string()).collect();

    Ok(TokenMintResponse {
        status: tx_receipt.status.as_str_name().to_string(),
        new_total_supply: Some(tx_receipt.total_supply.to_string()),
        serial_numbers,
    })
}

pub async fn freeze_token(
    client: &Client,
    token_id: Option<String>,
    account_id: Option<String>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<TokenResponse, ErrorObjectOwned> {
    let mut tx = TokenFreezeTransaction::new();

    if let Some(token_id) = token_id {
        tx.token_id(TokenId::from_str(&token_id).map_err(internal_error)?);
    }

    if let Some(account_id) = account_id {
        tx.account_id(AccountId::from_str(&account_id).map_err(internal_error)?);
    }

    if let Some(params) = common_transaction_params {
        fill_common_transaction_params(&mut tx, &params, client);
    }

    let tx_response = tx.execute(client).await.map_err(from_hedera_error)?;
    let tx_receipt = tx_response.get_receipt(client).await.map_err(from_hedera_error)?;

    Ok(TokenResponse { status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn pause_token(
    client: &Client,
    token_id: Option<String>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<TokenResponse, ErrorObjectOwned> {
    let mut tx = TokenPauseTransaction::new();

    if let Some(token_id) = token_id {
        tx.token_id(TokenId::from_str(&token_id).map_err(internal_error)?);
    }

    if let Some(params) = common_transaction_params {
        fill_common_transaction_params(&mut tx, &params, client);
    }

    let tx_response = tx.execute(client).await.map_err(from_hedera_error)?;
    let tx_receipt = tx_response.get_receipt(client).await.map_err(from_hedera_error)?;

    Ok(TokenResponse { status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn update_token(
    client: &Client,
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
    let mut tx = TokenUpdateTransaction::new();

    if let Some(token_id_str) = token_id {
        let token_id = TokenId::from_str(&token_id_str)
            .map_err(|e| internal_error(format!("Invalid tokenId: {e}")))?;
        tx.token_id(token_id);
    }

    if let Some(symbol) = symbol {
        tx.token_symbol(symbol);
    }

    if let Some(name) = name {
        tx.token_name(name);
    }

    if let Some(treasury_account_id) = treasury_account_id {
        let account_id = AccountId::from_str(&treasury_account_id).map_err(|e| {
            ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid treasury_account_id: {e}"),
                None::<()>,
            )
        })?;
        tx.treasury_account_id(account_id);
    }

    if let Some(admin_key) = admin_key {
        tx.admin_key(get_hedera_key(&admin_key)?);
    }

    if let Some(kyc_key) = kyc_key {
        tx.kyc_key(get_hedera_key(&kyc_key)?);
    }

    if let Some(freeze_key) = freeze_key {
        tx.freeze_key(get_hedera_key(&freeze_key)?);
    }

    if let Some(wipe_key) = wipe_key {
        tx.wipe_key(get_hedera_key(&wipe_key)?);
    }

    if let Some(supply_key) = supply_key {
        tx.supply_key(get_hedera_key(&supply_key)?);
    }

    if let Some(auto_renew_account_id) = auto_renew_account_id {
        let account_id = AccountId::from_str(&auto_renew_account_id).map_err(|e| {
            ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid auto_renew_account_id: {e}"),
                None::<()>,
            )
        })?;
        tx.auto_renew_account_id(account_id);
    }

    if let Some(auto_renew_period) = auto_renew_period {
        let seconds = auto_renew_period.parse::<i64>().map_err(|e| {
            ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid auto_renew_period: {e}"),
                None::<()>,
            )
        })?;
        tx.auto_renew_period(Duration::seconds(seconds));
    }

    if let Some(expiration_time) = expiration_time {
        let timestamp = expiration_time
            .parse::<i64>()
            .map_err(|e| mock_consensus_error("INVALID_EXPIRATION_TIME", &e.to_string()))?;

        tx.expiration_time(
            OffsetDateTime::from_unix_timestamp(timestamp)
                .map_err(|e| mock_consensus_error("INVALID_EXPIRATION_TIME", &e.to_string()))?,
        );
    }

    if let Some(memo) = memo {
        tx.token_memo(Some(memo));
    }

    if let Some(fee_schedule_key) = fee_schedule_key {
        tx.fee_schedule_key(get_hedera_key(&fee_schedule_key)?);
    }

    if let Some(pause_key) = pause_key {
        tx.pause_key(get_hedera_key(&pause_key)?);
    }

    if let Some(metadata) = metadata {
        tx.metadata(metadata.into_bytes());
    }

    if let Some(metadata_key) = metadata_key {
        tx.metadata_key(get_hedera_key(&metadata_key)?);
    }

    if let Some(common_transaction_params) = &common_transaction_params {
        fill_common_transaction_params(&mut tx, common_transaction_params, client);
    }

    let tx_response = tx.execute(client).await.map_err(|e| from_hedera_error(e))?;
    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(TokenResponse { status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn update_token_fee_schedule(
    client: &Client,
    token_id: Option<String>,
    custom_fees: Option<Value>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<TokenResponse, ErrorObjectOwned> {
    let mut tx = TokenFeeScheduleUpdateTransaction::new();

    if let Some(token_id_str) = token_id {
        let token_id = TokenId::from_str(&token_id_str)
            .map_err(|e| internal_error(format!("Invalid tokenId: {e}")))?;
        tx.token_id(token_id);
    }

    if let Some(custom_fees) = custom_fees {
        let parsed_fees = parse_custom_fees(&custom_fees)?;
        tx.custom_fees(parsed_fees);
    }

    if let Some(common_transaction_params) = &common_transaction_params {
        fill_common_transaction_params(&mut tx, common_transaction_params, client);
    }

    let tx_response = tx.execute(client).await.map_err(|e| from_hedera_error(e))?;
    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(TokenResponse { status: tx_receipt.status.as_str_name().to_string() })
}

// Helper function to parse royalty fee from JSON
fn parse_royalty_fee(
    royalty_fee_obj: &Value,
    fee_collector_account_id: Option<AccountId>,
    all_collectors_are_exempt: bool,
) -> Result<AnyCustomFee, ErrorObjectOwned> {
    let royalty_fee = royalty_fee_obj.as_object().ok_or_else(|| {
        ErrorObject::owned(INTERNAL_ERROR_CODE, "royaltyFee must be an object", None::<()>)
    })?;

    let numerator = royalty_fee
        .get("numerator")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Missing or invalid royaltyFee.numerator",
                None::<()>,
            )
        })?
        .parse::<i64>()
        .map_err(|e| {
            ErrorObject::owned(
                HEDERA_ERROR,
                "Hiero error".to_string(),
                Some(json!({
                    "status": "CUSTOM_FEE_MUST_BE_POSITIVE",
                    "message": format!("Invalid royaltyFee.numerator: {}", e),
                })),
            )
        })?;

    if numerator <= 0 {
        return Err(mock_check_error("CUSTOM_FEE_MUST_BE_POSITIVE"));
    }
    let numerator = numerator as u64;

    let denominator = royalty_fee
        .get("denominator")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Missing or invalid royaltyFee.denominator",
                None::<()>,
            )
        })?
        .parse::<i64>()
        .map_err(|e| {
            ErrorObject::owned(
                HEDERA_ERROR,
                "Hiero error".to_string(),
                Some(json!({
                    "status": "CUSTOM_FEE_MUST_BE_POSITIVE",
                    "message": format!("Invalid royaltyFee.denominator: {}", e),
                })),
            )
        })?;

    if denominator == 0 {
        return Err(mock_check_error("FRACTION_DIVIDES_BY_ZERO"));
    }
    if denominator < 0 {
        return Err(mock_check_error("CUSTOM_FEE_MUST_BE_POSITIVE"));
    }
    let denominator = denominator as u64;

    let fallback_fee = if let Some(fallback_obj) = royalty_fee.get("fallbackFee") {
        let fallback = fallback_obj.as_object().ok_or_else(|| {
            ErrorObject::owned(INTERNAL_ERROR_CODE, "fallbackFee must be an object", None::<()>)
        })?;

        let amount = fallback
            .get("amount")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "Missing or invalid fallbackFee.amount",
                    None::<()>,
                )
            })?
            .parse::<i64>()
            .map_err(|e| {
                ErrorObject::owned(
                    HEDERA_ERROR,
                    "Hiero error".to_string(),
                    Some(json!({
                        "status": "CUSTOM_FEE_MUST_BE_POSITIVE",
                        "message": format!("Invalid fallbackFee.amount: {}", e),
                    })),
                )
            })?;

        if amount <= 0 {
            return Err(mock_check_error("CUSTOM_FEE_MUST_BE_POSITIVE"));
        }

        let denominating_token_id = fallback
            .get("denominatingTokenId")
            .and_then(|v| v.as_str())
            .map(|s| TokenId::from_str(s))
            .transpose()
            .map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid denominatingTokenId: {e}"),
                    None::<()>,
                )
            })?;

        Some(FixedFeeData { amount, denominating_token_id })
    } else {
        None
    };

    let royalty_fee_data = RoyaltyFeeData { numerator, denominator, fallback_fee };

    Ok(AnyCustomFee {
        fee: hiero_sdk::Fee::Royalty(royalty_fee_data),
        fee_collector_account_id,
        all_collectors_are_exempt,
    })
}

// Helper function to parse fixed fee from JSON
fn parse_fixed_fee(
    fixed_fee_obj: &Value,
    fee_collector_account_id: Option<AccountId>,
    all_collectors_are_exempt: bool,
) -> Result<AnyCustomFee, ErrorObjectOwned> {
    let fixed_fee = fixed_fee_obj.as_object().ok_or_else(|| {
        ErrorObject::owned(INTERNAL_ERROR_CODE, "fixedFee must be an object", None::<()>)
    })?;

    let amount = fixed_fee
        .get("amount")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Missing or invalid fixedFee.amount",
                None::<()>,
            )
        })?
        .parse::<i64>()
        .map_err(|e| {
            ErrorObject::owned(
                HEDERA_ERROR,
                "Hiero error".to_string(),
                Some(json!({
                    "status": "CUSTOM_FEE_MUST_BE_POSITIVE",
                    "message": format!("Invalid fixedFee.amount: {}", e),
                })),
            )
        })?;

    if amount <= 0 {
        return Err(mock_check_error("CUSTOM_FEE_MUST_BE_POSITIVE"));
    }

    let denominating_token_id = fixed_fee
        .get("denominatingTokenId")
        .and_then(|v| v.as_str())
        .map(|s| TokenId::from_str(s))
        .transpose()
        .map_err(|e| {
            ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid denominatingTokenId: {e}"),
                None::<()>,
            )
        })?;

    let fixed_fee_data = FixedFeeData { amount, denominating_token_id };

    Ok(AnyCustomFee {
        fee: hiero_sdk::Fee::Fixed(fixed_fee_data),
        fee_collector_account_id,
        all_collectors_are_exempt,
    })
}

// Helper function to parse fractional fee from JSON
fn parse_fractional_fee(
    fractional_fee_obj: &Value,
    fee_collector_account_id: Option<AccountId>,
    all_collectors_are_exempt: bool,
) -> Result<AnyCustomFee, ErrorObjectOwned> {
    let fractional_fee = fractional_fee_obj.as_object().ok_or_else(|| {
        ErrorObject::owned(INTERNAL_ERROR_CODE, "fractionalFee must be an object", None::<()>)
    })?;

    let numerator = fractional_fee
        .get("numerator")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Missing or invalid fractionalFee.numerator",
                None::<()>,
            )
        })?
        .parse::<i64>()
        .map_err(|e| {
            ErrorObject::owned(
                HEDERA_ERROR,
                "Hiero error".to_string(),
                Some(json!({
                    "status": "CUSTOM_FEE_MUST_BE_POSITIVE",
                    "message": format!("Invalid fractionalFee.numerator: {}", e),
                })),
            )
        })?;

    if numerator <= 0 {
        return Err(mock_check_error("CUSTOM_FEE_MUST_BE_POSITIVE"));
    }
    let numerator = numerator as u64;

    let denominator = fractional_fee
        .get("denominator")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Missing or invalid fractionalFee.denominator",
                None::<()>,
            )
        })?
        .parse::<i64>()
        .map_err(|e| {
            ErrorObject::owned(
                HEDERA_ERROR,
                "Hiero error".to_string(),
                Some(json!({
                    "status": "CUSTOM_FEE_MUST_BE_POSITIVE",
                    "message": format!("Invalid fractionalFee.denominator: {}", e),
                })),
            )
        })?;

    if denominator == 0 {
        return Err(mock_check_error("FRACTION_DIVIDES_BY_ZERO"));
    }
    if denominator < 0 {
        return Err(mock_check_error("CUSTOM_FEE_MUST_BE_POSITIVE"));
    }
    let denominator = denominator as u64;

    let minimum_amount = fractional_fee
        .get("minimumAmount")
        .and_then(|v| v.as_str())
        .map(|s| {
            let val = s.parse::<i64>().map_err(|e| {
                ErrorObject::owned(
                    HEDERA_ERROR,
                    "Hiero error".to_string(),
                    Some(json!({
                        "status": "CUSTOM_FEE_MUST_BE_POSITIVE",
                        "message": format!("Invalid fractionalFee.minimumAmount: {}", e),
                    })),
                )
            })?;
            if val < 0 {
                return Err(mock_check_error("CUSTOM_FEE_MUST_BE_POSITIVE"));
            }
            Ok(val)
        })
        .transpose()?
        .unwrap_or(0);

    let maximum_amount = fractional_fee
        .get("maximumAmount")
        .and_then(|v| v.as_str())
        .map(|s| {
            let val = s.parse::<i64>().map_err(|e| {
                ErrorObject::owned(
                    HEDERA_ERROR,
                    "Hiero error".to_string(),
                    Some(json!({
                        "status": "CUSTOM_FEE_MUST_BE_POSITIVE",
                        "message": format!("Invalid fractionalFee.maximumAmount: {}", e),
                    })),
                )
            })?;
            if val < 0 {
                return Err(mock_check_error("CUSTOM_FEE_MUST_BE_POSITIVE"));
            }
            Ok(val)
        })
        .transpose()?
        .unwrap_or(0);

    let assessment_method = fractional_fee
        .get("assessmentMethod")
        .and_then(|v| v.as_str())
        .map(|s| match s {
            "EXCLUSIVE" | "exclusive" | "Exclusive" => {
                Ok(hiero_sdk::FeeAssessmentMethod::Exclusive)
            }
            "INCLUSIVE" | "inclusive" | "Inclusive" => {
                Ok(hiero_sdk::FeeAssessmentMethod::Inclusive)
            }
            _ => Err(ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid assessmentMethod: {s}"),
                None::<()>,
            )),
        })
        .transpose()?
        .unwrap_or(hiero_sdk::FeeAssessmentMethod::Inclusive);

    let fractional_fee_data = FractionalFeeData {
        numerator,
        denominator,
        minimum_amount,
        maximum_amount,
        assessment_method,
    };

    Ok(AnyCustomFee {
        fee: hiero_sdk::Fee::Fractional(fractional_fee_data),
        fee_collector_account_id,
        all_collectors_are_exempt,
    })
}

// Helper function to parse custom fees from JSON Value
fn parse_custom_fees(custom_fees: &Value) -> Result<Vec<AnyCustomFee>, ErrorObjectOwned> {
    let mut parsed_fees = Vec::new();

    let fees_array = custom_fees
        .as_array()
        .ok_or_else(|| internal_error("customFees must be an array".to_string()))?;

    for fee in fees_array {
        let fee_obj = fee
            .as_object()
            .ok_or_else(|| internal_error("Each custom fee must be an object".to_string()))?;

        let fee_collector_account_id = fee_obj
            .get("feeCollectorAccountId")
            .and_then(|v| v.as_str())
            .map(|s| AccountId::from_str(s))
            .transpose()
            .map_err(|e| internal_error(format!("Invalid feeCollectorAccountId: {e}")))?;

        let all_collectors_are_exempt =
            fee_obj.get("feeCollectorsExempt").and_then(|v| v.as_bool()).unwrap_or(false);

        let custom_fee = if let Some(royalty_fee) = fee_obj.get("royaltyFee") {
            parse_royalty_fee(royalty_fee, fee_collector_account_id, all_collectors_are_exempt)?
        } else if let Some(fixed_fee) = fee_obj.get("fixedFee") {
            parse_fixed_fee(fixed_fee, fee_collector_account_id, all_collectors_are_exempt)?
        } else if let Some(fractional_fee) = fee_obj.get("fractionalFee") {
            parse_fractional_fee(
                fractional_fee,
                fee_collector_account_id,
                all_collectors_are_exempt,
            )?
        } else {
            return Err(internal_error(
                "Custom fee must have one of: royaltyFee, fixedFee, or fractionalFee".to_string(),
            ));
        };

        parsed_fees.push(custom_fee);
    }

    Ok(parsed_fees)
}
