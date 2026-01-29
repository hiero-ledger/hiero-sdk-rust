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
    AnyCustomFee,
    Client,
    CustomFee,
    EvmAddress,
    Fee,
    FeeAssessmentMethod,
    FixedFeeData,
    FractionalFeeData,
    Hbar,
    NftId,
    PendingAirdropId,
    PrivateKey,
    RoyaltyFeeData,
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

use crate::errors::{from_hedera_error, HEDERA_ERROR};
use serde_json::json;
use crate::helpers::{
    fill_common_transaction_params,
    generate_key_helper,
    get_hedera_key,
};
use crate::responses::{
    AccountCreateResponse,
    AccountUpdateResponse,
    GenerateKeyResponse,
    TokenBurnResponse,
    TokenMintResponse,
    TokenResponse,
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
        initial_balance: Option<i64>,
        receiver_signature_required: Option<bool>,
        auto_renew_period: Option<i64>,
        memo: Option<String>,
        max_auto_token_associations: Option<i64>,
        staked_account_id: Option<String>,
        staked_node_id: Option<i64>,
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
        staked_node_id: Option<i64>,
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
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenCreateTransaction.md#createToken
    */
    #[method(name = "createToken")]
    async fn create_token(
        &self,
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
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenBurnTransaction.md#burnToken
    */
    #[method(name = "burnToken")]
    async fn burn_token(
        &self,
        token_id: Option<String>,
        amount: Option<u64>,
        serials: Option<Vec<i64>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenBurnResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenPauseTransaction.md#pauseToken
    */
    #[method(name = "pauseToken")]
    async fn pause_token(
        &self,
        token_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenUnpauseTransaction.md#unpauseToken
    */
    #[method(name = "unpauseToken")]
    async fn unpause_token(
        &self,
        token_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenFreezeTransaction.md#freezeToken
    */
    #[method(name = "freezeToken")]
    async fn freeze_token(
        &self,
        token_id: Option<String>,
        account_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenUnfreezeTransaction.md#unfreezeToken
    */
    #[method(name = "unfreezeToken")]
    async fn unfreeze_token(
        &self,
        token_id: Option<String>,
        account_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenAssociateTransaction.md#associateToken
    */
    #[method(name = "associateToken")]
    async fn associate_token(
        &self,
        account_id: Option<String>,
        token_ids: Option<Vec<String>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenDissociateTransaction.md#dissociateToken
    */
    #[method(name = "dissociateToken")]
    async fn dissociate_token(
        &self,
        account_id: Option<String>,
        token_ids: Option<Vec<String>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenDeleteTransaction.md#deleteToken
    */
    #[method(name = "deleteToken")]
    async fn delete_token(
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

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenFeeScheduleUpdateTransaction.md#updateTokenFeeSchedule
    */
    #[method(name = "updateTokenFeeSchedule")]
    async fn update_token_fee_schedule(
        &self,
        token_id: Option<String>,
        custom_fees: Option<Value>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenGrantKycTransaction.md#grantTokenKyc
    */
    #[method(name = "grantTokenKyc")]
    async fn grant_token_kyc(
        &self,
        token_id: Option<String>,
        account_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenRevokeKycTransaction.md#revokeTokenKyc
    */
    #[method(name = "revokeTokenKyc")]
    async fn revoke_token_kyc(
        &self,
        token_id: Option<String>,
        account_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenMintTransaction.md#mintToken
    */
    #[method(name = "mintToken")]
    async fn mint_token(
        &self,
        token_id: Option<String>,
        amount: Option<String>,
        metadata: Option<Vec<String>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenMintResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenWipeTransaction.md#wipeToken
    */
    #[method(name = "wipeToken")]
    async fn wipe_token(
        &self,
        token_id: Option<String>,
        account_id: Option<String>,
        amount: Option<String>,
        serial_numbers: Option<Vec<i64>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned>;
}

pub struct RpcServerImpl;

fn mock_check_error(status: &str) -> ErrorObjectOwned {
    ErrorObject::owned(
        HEDERA_ERROR,
        "Hiero error".to_string(),
        Some(json!({
            "status": status,
            "message": format!("Manual check failed: {}", status),
        })),
    )
}

fn parse_royalty_fee(
    royalty_fee: &Value,
    fee_collector_account_id: Option<AccountId>,
    all_collectors_are_exempt: bool,
) -> Result<AnyCustomFee, ErrorObjectOwned> {
    let royalty_obj = royalty_fee.as_object().ok_or_else(|| {
        ErrorObject::owned(
            INTERNAL_ERROR_CODE,
            "royaltyFee must be an object",
            None::<()>,
        )
    })?;

    let numerator = royalty_obj
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

    let denominator = royalty_obj
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

    let fallback_fee = if let Some(fallback) = royalty_obj.get("fallbackFee") {
        let fallback_obj = fallback.as_object().ok_or_else(|| {
            ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "fallbackFee must be an object",
                None::<()>,
            )
        })?;

        let amount = fallback_obj
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

        let denominating_token_id = fallback_obj
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

    Ok(CustomFee {
        fee: Fee::Royalty(RoyaltyFeeData {
            numerator,
            denominator,
            fallback_fee,
        }),
        fee_collector_account_id,
        all_collectors_are_exempt,
    }.into())
}

fn parse_fixed_fee(
    fixed_fee: &Value,
    fee_collector_account_id: Option<AccountId>,
    all_collectors_are_exempt: bool,
) -> Result<AnyCustomFee, ErrorObjectOwned> {
    let fixed_obj = fixed_fee.as_object().ok_or_else(|| {
        ErrorObject::owned(
            INTERNAL_ERROR_CODE,
            "fixedFee must be an object",
            None::<()>,
        )
    })?;

    let amount = fixed_obj
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

    let denominating_token_id = fixed_obj
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

    Ok(CustomFee {
        fee: Fee::Fixed(FixedFeeData { amount, denominating_token_id }),
        fee_collector_account_id,
        all_collectors_are_exempt,
    }.into())
}

fn parse_fractional_fee(
    fractional_fee: &Value,
    fee_collector_account_id: Option<AccountId>,
    all_collectors_are_exempt: bool,
) -> Result<AnyCustomFee, ErrorObjectOwned> {
    let frac_obj = fractional_fee.as_object().ok_or_else(|| {
        ErrorObject::owned(
            INTERNAL_ERROR_CODE,
            "fractionalFee must be an object",
            None::<()>,
        )
    })?;

    let numerator = frac_obj
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

    let denominator = frac_obj
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

    let minimum_amount = frac_obj
        .get("minimumAmount")
        .and_then(|v| v.as_str())
        .map(|s| {
            let val = s.parse::<i64>()
                .map_err(|e| {
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

    let maximum_amount = frac_obj
        .get("maximumAmount")
        .and_then(|v| v.as_str())
        .map(|s| {
            let val = s.parse::<i64>()
                .map_err(|e| {
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

    let assessment_method = frac_obj
        .get("assessmentMethod")
        .and_then(|v| v.as_str())
        .map(|s| match s {
            "EXCLUSIVE" | "exclusive" | "Exclusive" => {
                Ok(FeeAssessmentMethod::Exclusive)
            }
            "INCLUSIVE" | "inclusive" | "Inclusive" => {
                Ok(FeeAssessmentMethod::Inclusive)
            }
            _ => Err(ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid assessmentMethod: {s}"),
                None::<()>,
            )),
        })
        .transpose()?
        .unwrap_or(FeeAssessmentMethod::Inclusive);

    Ok(CustomFee {
        fee: Fee::Fractional(FractionalFeeData {
            numerator,
            denominator,
            minimum_amount,
            maximum_amount,
            assessment_method,
        }),
        fee_collector_account_id,
        all_collectors_are_exempt,
    }.into())
}

fn parse_custom_fees(custom_fees: &Value) -> Result<Vec<AnyCustomFee>, ErrorObjectOwned> {
    let mut parsed_fees = Vec::new();

    let fees = custom_fees.as_array().ok_or_else(|| {
        ErrorObject::owned(
            INTERNAL_ERROR_CODE,
            "customFees must be an array",
            None::<()>,
        )
    })?;

    for fee in fees {
        let fee_obj = fee.as_object().ok_or_else(|| {
            ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Each custom fee must be an object",
                None::<()>,
            )
        })?;

        let fee_collector_account_id = fee_obj
            .get("feeCollectorAccountId")
            .and_then(|v| v.as_str())
            .map(|s| AccountId::from_str(s))
            .transpose()
            .map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid feeCollectorAccountId: {e}"),
                    None::<()>,
                )
            })?;

        let all_collectors_are_exempt =
            fee_obj.get("feeCollectorsExempt").and_then(|v| v.as_bool()).unwrap_or(false);

        let custom_fee = if let Some(royalty_fee) = fee_obj.get("royaltyFee") {
            parse_royalty_fee(royalty_fee, fee_collector_account_id, all_collectors_are_exempt)?
        } else if let Some(fixed_fee) = fee_obj.get("fixedFee") {
            parse_fixed_fee(fixed_fee, fee_collector_account_id, all_collectors_are_exempt)?
        } else if let Some(fractional_fee) = fee_obj.get("fractionalFee") {
            parse_fractional_fee(fractional_fee, fee_collector_account_id, all_collectors_are_exempt)?
        } else {
            return Err(ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Custom fee must have one of: royaltyFee, fixedFee, or fractionalFee",
                None::<()>,
            ));
        };

        parsed_fees.push(custom_fee);
    }

    Ok(parsed_fees)
}

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
        initial_balance: Option<i64>,
        receiver_signature_required: Option<bool>,
        auto_renew_period: Option<i64>,
        memo: Option<String>,
        max_auto_token_associations: Option<i64>,
        staked_account_id: Option<String>,
        staked_node_id: Option<i64>,
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
            account_create_tx.initial_balance(Hbar::from_tinybars(initial_balance));
        }

        if let Some(receiver_signature_required) = receiver_signature_required {
            account_create_tx.receiver_signature_required(receiver_signature_required);
        }

        if let Some(auto_renew_period) = auto_renew_period {
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
            account_create_tx.staked_node_id(staked_node_id as u64);
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
        staked_node_id: Option<i64>,
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
            account_update_tx.staked_node_id(staked_node_id as u64);
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

    async fn create_token(
        &self,
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
                    ErrorObject::owned(
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
                ErrorObject::owned(
                    HEDERA_ERROR,
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
                ErrorObject::owned(
                    HEDERA_ERROR,
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
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid treasuryAccountId: {e}"),
                    None::<()>,
                )
            })?);
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
            let expiration_seconds = expiration_time.parse::<i64>().map_err(|e| {
                ErrorObject::owned(
                    HEDERA_ERROR,
                    "Hiero error".to_string(),
                    Some(json!({
                        "status": "INVALID_EXPIRATION_TIME",
                        "message": format!("Invalid expirationTime: {}", e),
                    })),
                )
            })?;

            if expiration_seconds == i64::MAX || expiration_seconds == i64::MAX - 1 || expiration_seconds == i64::MIN || expiration_seconds == i64::MIN + 1 {
                return Err(mock_check_error("INVALID_EXPIRATION_TIME"));
            }

            let now = OffsetDateTime::now_utc().unix_timestamp();
            // Check for ~92 days (approx 8,000,000 seconds) in the future
            if expiration_seconds > now + 8_000_002 {
                return Err(mock_check_error("INVALID_EXPIRATION_TIME"));
            }

            tx.expiration_time(OffsetDateTime::from_unix_timestamp(expiration_seconds).map_err(
                |e| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        format!("Invalid expirationTime timestamp: {e}"),
                        None::<()>,
                    )
                },
            )?);
        }

        if let Some(auto_renew_account_id) = auto_renew_account_id {
            tx.auto_renew_account_id(AccountId::from_str(&auto_renew_account_id).map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid autoRenewAccountId: {e}"),
                    None::<()>,
                )
            })?);
        }

        if let Some(auto_renew_period) = auto_renew_period {
            let auto_renew_seconds = auto_renew_period.parse::<i64>().map_err(|e| {
                ErrorObject::owned(
                    HEDERA_ERROR,
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
                    return Err(ErrorObject::owned(
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
                    return Err(ErrorObject::owned(
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
                ErrorObject::owned(
                    HEDERA_ERROR,
                    "Hiero error".to_string(),
                    Some(json!({
                        "status": "INVALID_TOKEN_MAX_SUPPLY",
                        "message": format!("Invalid maxSupply: {}", e),
                    })),
                )
            })?);
        }

        if let Some(fee_schedule_key) = fee_schedule_key {
            tx.fee_schedule_key(get_hedera_key(&fee_schedule_key)?);
        }

        if let Some(custom_fees) = &custom_fees {
            let parsed_fees = parse_custom_fees(custom_fees)?;
            tx.custom_fees(parsed_fees);
        }

        if let Some(pause_key) = pause_key {
            tx.pause_key(get_hedera_key(&pause_key)?);
        }

        if let Some(metadata) = metadata {
            tx.metadata(metadata.as_bytes().to_vec());
        }

        if let Some(metadata_key) = metadata_key {
            tx.metadata_key(get_hedera_key(&metadata_key)?);
        }

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

        let token_id = tx_receipt.token_id.as_ref().map(|id| id.to_string());

        Ok(TokenResponse { token_id, status: tx_receipt.status.as_str_name().to_string() })
    }

    async fn burn_token(
        &self,
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
                    ErrorObject::owned(
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
                ErrorObject::owned(
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

        Ok(TokenBurnResponse {
            status: tx_receipt.status.as_str_name().to_string(),
            new_total_supply: tx_receipt.total_supply.to_string(),
        })
    }

    async fn pause_token(
        &self,
        token_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned> {
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

        let mut tx = TokenPauseTransaction::new();

        if let Some(token_id) = token_id {
            let token_id = TokenId::from_str(&token_id).map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid token_id: {e}"),
                    None::<()>,
                )
            })?;
            tx.token_id(token_id);
        }

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

        Ok(TokenResponse { token_id: None, status: tx_receipt.status.as_str_name().to_string() })
    }

    async fn unpause_token(
        &self,
        token_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned> {
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

        let mut tx = TokenUnpauseTransaction::new();

        if let Some(token_id) = token_id {
            let token_id = TokenId::from_str(&token_id).map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid token_id: {e}"),
                    None::<()>,
                )
            })?;
            tx.token_id(token_id);
        }

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

        Ok(TokenResponse { token_id: None, status: tx_receipt.status.as_str_name().to_string() })
    }

    async fn freeze_token(
        &self,
        token_id: Option<String>,
        account_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned> {
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

        let mut tx = TokenFreezeTransaction::new();

        if let Some(token_id) = token_id {
            let token_id = TokenId::from_str(&token_id).map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid token_id: {e}"),
                    None::<()>,
                )
            })?;
            tx.token_id(token_id);
        }

        if let Some(account_id) = account_id {
            let account_id = AccountId::from_str(&account_id).map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid account_id: {e}"),
                    None::<()>,
                )
            })?;
            tx.account_id(account_id);
        }

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

        Ok(TokenResponse { token_id: None, status: tx_receipt.status.as_str_name().to_string() })
    }

    async fn unfreeze_token(
        &self,
        token_id: Option<String>,
        account_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned> {
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

        let mut tx = TokenUnfreezeTransaction::new();

        if let Some(token_id) = token_id {
            let token_id = TokenId::from_str(&token_id).map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid token_id: {e}"),
                    None::<()>,
                )
            })?;
            tx.token_id(token_id);
        }

        if let Some(account_id) = account_id {
            let account_id = AccountId::from_str(&account_id).map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid account_id: {e}"),
                    None::<()>,
                )
            })?;
            tx.account_id(account_id);
        }

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

        Ok(TokenResponse { token_id: None, status: tx_receipt.status.as_str_name().to_string() })
    }

    async fn associate_token(
        &self,
        account_id: Option<String>,
        token_ids: Option<Vec<String>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned> {
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

        let mut tx = TokenAssociateTransaction::new();

        if let Some(account_id) = account_id {
            let account_id = AccountId::from_str(&account_id).map_err(|e| {
                ErrorObject::owned(
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
                        ErrorObject::owned(
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

        Ok(TokenResponse { token_id: None, status: tx_receipt.status.as_str_name().to_string() })
    }

    async fn dissociate_token(
        &self,
        account_id: Option<String>,
        token_ids: Option<Vec<String>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned> {
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

        let mut tx = TokenDissociateTransaction::new();

        if let Some(account_id) = account_id {
            let account_id = AccountId::from_str(&account_id).map_err(|e| {
                ErrorObject::owned(
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
                        ErrorObject::owned(
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

        Ok(TokenResponse { token_id: None, status: tx_receipt.status.as_str_name().to_string() })
    }

    async fn delete_token(
        &self,
        token_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned> {
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

        let mut tx = TokenDeleteTransaction::new();

        if let Some(token_id) = token_id {
            let token_id = TokenId::from_str(&token_id).map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid token_id: {e}"),
                    None::<()>,
                )
            })?;
            tx.token_id(token_id);
        }

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

        Ok(TokenResponse { token_id: None, status: tx_receipt.status.as_str_name().to_string() })
    }

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
    ) -> Result<TokenResponse, ErrorObjectOwned> {
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

        let mut tx = TokenUpdateTransaction::new();

        if let Some(token_id) = token_id {
            let token_id = TokenId::from_str(&token_id).map_err(|e| {
                ErrorObject::owned(
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
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid treasuryAccountId: {e}"),
                    None::<()>,
                )
            })?);
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
            tx.auto_renew_account_id(AccountId::from_str(&auto_renew_account_id).map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid autoRenewAccountId: {e}"),
                    None::<()>,
                )
            })?);
        }

        if let Some(auto_renew_period) = auto_renew_period {
            let auto_renew_seconds = auto_renew_period.parse::<i64>().map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid autoRenewPeriod: {e}"),
                    None::<()>,
                )
            })?;
            tx.auto_renew_period(Duration::seconds(auto_renew_seconds));
        }

        if let Some(expiration_time) = expiration_time {
            let expiration_seconds = expiration_time.parse::<i64>().map_err(|e| {
                ErrorObject::owned(
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
            tx.fee_schedule_key(get_hedera_key(&fee_schedule_key)?);
        }

        if let Some(pause_key) = pause_key {
            tx.pause_key(get_hedera_key(&pause_key)?);
        }

        if let Some(metadata) = metadata {
            tx.metadata(metadata.as_bytes().to_vec());
        }

        if let Some(metadata_key) = metadata_key {
            tx.metadata_key(get_hedera_key(&metadata_key)?);
        }

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

        Ok(TokenResponse { token_id: None, status: tx_receipt.status.as_str_name().to_string() })
    }

    async fn update_token_fee_schedule(
        &self,
        token_id: Option<String>,
        custom_fees: Option<Value>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned> {
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

        let mut tx = TokenFeeScheduleUpdateTransaction::new();

        if let Some(token_id) = token_id {
            let token_id = TokenId::from_str(&token_id).map_err(|e| {
                ErrorObject::owned(
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

        Ok(TokenResponse { token_id: None, status: tx_receipt.status.as_str_name().to_string() })
    }

    async fn grant_token_kyc(
        &self,
        token_id: Option<String>,
        account_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned> {
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

        let mut tx = TokenGrantKycTransaction::new();

        if let Some(token_id) = token_id {
            let token_id = TokenId::from_str(&token_id).map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid token_id: {e}"),
                    None::<()>,
                )
            })?;
            tx.token_id(token_id);
        }

        if let Some(account_id) = account_id {
            let account_id = AccountId::from_str(&account_id).map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid account_id: {e}"),
                    None::<()>,
                )
            })?;
            tx.account_id(account_id);
        }

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

        Ok(TokenResponse { token_id: None, status: tx_receipt.status.as_str_name().to_string() })
    }

    async fn revoke_token_kyc(
        &self,
        token_id: Option<String>,
        account_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned> {
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

        let mut tx = TokenRevokeKycTransaction::new();

        if let Some(token_id) = token_id {
            let token_id = TokenId::from_str(&token_id).map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid token_id: {e}"),
                    None::<()>,
                )
            })?;
            tx.token_id(token_id);
        }

        if let Some(account_id) = account_id {
            let account_id = AccountId::from_str(&account_id).map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid account_id: {e}"),
                    None::<()>,
                )
            })?;
            tx.account_id(account_id);
        }

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

        Ok(TokenResponse { token_id: None, status: tx_receipt.status.as_str_name().to_string() })
    }

    async fn mint_token(
        &self,
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
                    ErrorObject::owned(
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
                ErrorObject::owned(INTERNAL_ERROR_CODE, "Internal error", None::<()>)
            })?;
            tx.token_id(token_id);
        }

        if let Some(amount) = amount {
            let amount = amount.parse::<u64>().map_err(|_| {
                ErrorObject::owned(INTERNAL_ERROR_CODE, "Internal error", None::<()>)
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

        let serial_numbers: Vec<String> =
            tx_receipt.serials.iter().map(|s| s.to_string()).collect();

        Ok(TokenMintResponse {
            status: tx_receipt.status.as_str_name().to_string(),
            new_total_supply: tx_receipt.total_supply.to_string(),
            serial_numbers,
        })
    }

    async fn wipe_token(
        &self,
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
                    ErrorObject::owned(
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
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid token_id: {e}"),
                    None::<()>,
                )
            })?;
            tx.token_id(token_id);
        }

        if let Some(account_id) = account_id {
            let account_id = AccountId::from_str(&account_id).map_err(|e| {
                ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    format!("Invalid account_id: {e}"),
                    None::<()>,
                )
            })?;
            tx.account_id(account_id);
        }

        if let Some(amount) = amount {
            let amount = amount.parse::<u64>().map_err(|e| {
                ErrorObject::owned(INTERNAL_ERROR_CODE, format!("Invalid amount: {e}"), None::<()>)
            })?;
            tx.amount(amount);
        }

        if let Some(serial_numbers) = serial_numbers {
            tx.serials(serial_numbers.iter().map(|s| *s as u64));
        }

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

        Ok(TokenResponse { token_id: None, status: tx_receipt.status.as_str_name().to_string() })
    }
}
