use std::str::FromStr;
use std::sync::{
    Arc,
    Mutex,
};

use hiero_sdk::{
    AccountId,
    AnyCustomFee,
    Client,
    CustomFee,
    Fee,
    FeeAssessmentMethod,
    FixedFeeData,
    FractionalFeeData,
    RoyaltyFeeData,
    TokenId,
};
use jsonrpsee::types::error::INTERNAL_ERROR_CODE;
use jsonrpsee::types::ErrorObjectOwned;
use once_cell::sync::Lazy;
use serde_json::{
    json,
    Value,
};

use crate::errors::HEDERA_ERROR;

pub static GLOBAL_SDK_CLIENT: Lazy<Arc<Mutex<Option<Client>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

pub fn mock_check_error(status: &str) -> ErrorObjectOwned {
    jsonrpsee::types::ErrorObject::owned(
        HEDERA_ERROR,
        "Hiero error".to_string(),
        Some(json!({
            "status": status,
            "message": format!("Manual check failed: {}", status),
        })),
    )
}

pub fn parse_royalty_fee(
    royalty_fee: &Value,
    fee_collector_account_id: Option<AccountId>,
    all_collectors_are_exempt: bool,
) -> Result<AnyCustomFee, ErrorObjectOwned> {
    let royalty_obj = royalty_fee.as_object().ok_or_else(|| {
        jsonrpsee::types::ErrorObject::owned(
            INTERNAL_ERROR_CODE,
            "royaltyFee must be an object",
            None::<()>,
        )
    })?;

    let numerator = royalty_obj
        .get("numerator")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Missing or invalid royaltyFee.numerator",
                None::<()>,
            )
        })?
        .parse::<i64>()
        .map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
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
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Missing or invalid royaltyFee.denominator",
                None::<()>,
            )
        })?
        .parse::<i64>()
        .map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
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
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "fallbackFee must be an object",
                None::<()>,
            )
        })?;

        let amount = fallback_obj
            .get("amount")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "Missing or invalid fallbackFee.amount",
                    None::<()>,
                )
            })?
            .parse::<i64>()
            .map_err(|e| {
                jsonrpsee::types::ErrorObject::owned(
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
                jsonrpsee::types::ErrorObject::owned(
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
        fee: Fee::Royalty(RoyaltyFeeData { numerator, denominator, fallback_fee }),
        fee_collector_account_id,
        all_collectors_are_exempt,
    }
    .into())
}

pub fn parse_fixed_fee(
    fixed_fee: &Value,
    fee_collector_account_id: Option<AccountId>,
    all_collectors_are_exempt: bool,
) -> Result<AnyCustomFee, ErrorObjectOwned> {
    let fixed_obj = fixed_fee.as_object().ok_or_else(|| {
        jsonrpsee::types::ErrorObject::owned(
            INTERNAL_ERROR_CODE,
            "fixedFee must be an object",
            None::<()>,
        )
    })?;

    let amount = fixed_obj
        .get("amount")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Missing or invalid fixedFee.amount",
                None::<()>,
            )
        })?
        .parse::<i64>()
        .map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
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
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                format!("Invalid denominatingTokenId: {e}"),
                None::<()>,
            )
        })?;

    Ok(CustomFee {
        fee: Fee::Fixed(FixedFeeData { amount, denominating_token_id }),
        fee_collector_account_id,
        all_collectors_are_exempt,
    }
    .into())
}

pub fn parse_fractional_fee(
    fractional_fee: &Value,
    fee_collector_account_id: Option<AccountId>,
    all_collectors_are_exempt: bool,
) -> Result<AnyCustomFee, ErrorObjectOwned> {
    let frac_obj = fractional_fee.as_object().ok_or_else(|| {
        jsonrpsee::types::ErrorObject::owned(
            INTERNAL_ERROR_CODE,
            "fractionalFee must be an object",
            None::<()>,
        )
    })?;

    let numerator = frac_obj
        .get("numerator")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Missing or invalid fractionalFee.numerator",
                None::<()>,
            )
        })?
        .parse::<i64>()
        .map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
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
            jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Missing or invalid fractionalFee.denominator",
                None::<()>,
            )
        })?
        .parse::<i64>()
        .map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
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
            let val = s.parse::<i64>().map_err(|e| {
                jsonrpsee::types::ErrorObject::owned(
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
            let val = s.parse::<i64>().map_err(|e| {
                jsonrpsee::types::ErrorObject::owned(
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
            "EXCLUSIVE" | "exclusive" | "Exclusive" => Ok(FeeAssessmentMethod::Exclusive),
            "INCLUSIVE" | "inclusive" | "Inclusive" => Ok(FeeAssessmentMethod::Inclusive),
            _ => Err(jsonrpsee::types::ErrorObject::owned(
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
    }
    .into())
}

pub fn parse_custom_fees(custom_fees: &Value) -> Result<Vec<AnyCustomFee>, ErrorObjectOwned> {
    let mut parsed_fees = Vec::new();

    let fees = custom_fees.as_array().ok_or_else(|| {
        jsonrpsee::types::ErrorObject::owned(
            INTERNAL_ERROR_CODE,
            "customFees must be an array",
            None::<()>,
        )
    })?;

    for fee in fees {
        let fee_obj = fee.as_object().ok_or_else(|| {
            jsonrpsee::types::ErrorObject::owned(
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
                jsonrpsee::types::ErrorObject::owned(
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
            parse_fractional_fee(
                fractional_fee,
                fee_collector_account_id,
                all_collectors_are_exempt,
            )?
        } else {
            return Err(jsonrpsee::types::ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Custom fee must have one of: royaltyFee, fixedFee, or fractionalFee",
                None::<()>,
            ));
        };

        parsed_fees.push(custom_fee);
    }

    Ok(parsed_fees)
}
