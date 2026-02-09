use std::collections::HashMap;
use std::str::FromStr;

use hiero_sdk::{
    AccountId,
    Client,
    NftId,
    PendingAirdropId,
    TokenAssociateTransaction,
    TokenBurnTransaction,
    TokenCancelAirdropTransaction,
    TokenClaimAirdropTransaction,
    TokenCreateTransaction,
    TokenDeleteTransaction,
    TokenId,
    TokenMintTransaction,
    TokenSupplyType,
    TokenType,
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

use crate::common::{
    internal_error,
    mock_consensus_error,
};
use crate::errors::from_hedera_error;
use crate::helpers::{
    fill_common_transaction_params,
    get_hedera_key,
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
