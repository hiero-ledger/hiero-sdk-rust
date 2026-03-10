use std::collections::HashMap;
use std::str::FromStr;

use hiero_sdk::{
    AccountId,
    Client,
    CustomFeeLimit,
    CustomFixedFee,
    TokenId,
    TopicCreateTransaction,
    TopicDeleteTransaction,
    TopicId,
    TopicInfoQuery,
    TopicMessageSubmitTransaction,
    TopicUpdateTransaction,
};
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::error::INTERNAL_ERROR_CODE;
use jsonrpsee::types::ErrorObjectOwned;
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
};
use crate::responses::{
    TopicInfoResponse,
    TopicResponse,
};

#[rpc(server, client)]
pub trait TopicRpc {
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
        max_chunks: Option<i64>,
        chunk_size: Option<i64>,
        custom_fee_limits: Option<Value>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TopicResponse, ErrorObjectOwned>;

    #[method(name = "getTopicInfo")]
    async fn get_topic_info(
        &self,
        topic_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TopicInfoResponse, ErrorObjectOwned>;
}

pub async fn create_topic(
    client: &Client,
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
            .map_err(|e| ErrorObjectOwned::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
        topic_create_tx.auto_renew_period(Duration::seconds(period));
    }

    if let Some(auto_renew_account_id) = auto_renew_account_id {
        topic_create_tx.auto_renew_account_id(
            AccountId::from_str(&auto_renew_account_id).map_err(|e| {
                ErrorObjectOwned::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
            })?,
        );
    }

    if let Some(fee_schedule_key) = fee_schedule_key {
        let key = get_hedera_key(&fee_schedule_key)?;
        topic_create_tx.fee_schedule_key(key);
    }

    if let Some(fee_exempt_keys) = fee_exempt_keys {
        if !fee_exempt_keys.is_empty() {
            let keys: Result<Vec<_>, _> =
                fee_exempt_keys.iter().map(|k| get_hedera_key(k)).collect();
            topic_create_tx.fee_exempt_keys(keys?);
        }
    }

    if let Some(custom_fees) = custom_fees {
        if let Value::Array(fees_array) = custom_fees {
            if !fees_array.is_empty() {
                let mut sdk_custom_fees = Vec::new();
                for fee in fees_array {
                    if let Value::Object(fee_obj) = fee {
                        let fixed_fee_obj = fee_obj.get("fixedFee").ok_or_else(|| {
                            ErrorObjectOwned::owned(
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
                                    ErrorObjectOwned::owned(
                                        INTERNAL_ERROR_CODE,
                                        "Missing amount in fixedFee".to_string(),
                                        None::<()>,
                                    )
                                })?;

                            let amount = amount_str.parse::<u64>().map_err(|e| {
                                mock_consensus_error(
                                    "CUSTOM_FEE_MUST_BE_POSITIVE",
                                    &format!("Invalid amount: {e}"),
                                )
                            })?;

                            let denominating_token_id = fixed_fee
                                .get("denominatingTokenId")
                                .and_then(|v| v.as_str())
                                .map(|s| {
                                    TokenId::from_str(s).map_err(|e| {
                                        ErrorObjectOwned::owned(
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
                                        ErrorObjectOwned::owned(
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
        fill_common_transaction_params(&mut topic_create_tx, &common_transaction_params, client);
    }

    let tx_response = topic_create_tx.execute(client).await.map_err(|e| from_hedera_error(e))?;

    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(TopicResponse {
        topic_id: tx_receipt.topic_id.map(|id| id.to_string()),
        status: tx_receipt.status.as_str_name().to_string(),
    })
}

pub async fn update_topic(
    client: &Client,
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
    let mut topic_update_tx = TopicUpdateTransaction::new();

    if let Some(topic_id) = topic_id {
        topic_update_tx.topic_id(TopicId::from_str(&topic_id).map_err(|e| {
            ErrorObjectOwned::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
        })?);
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
            .map_err(|e| ErrorObjectOwned::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
        topic_update_tx.auto_renew_period(Duration::seconds(period));
    }

    if let Some(auto_renew_account_id) = auto_renew_account_id {
        topic_update_tx.auto_renew_account_id(
            AccountId::from_str(&auto_renew_account_id).map_err(|e| {
                ErrorObjectOwned::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
            })?,
        );
    }

    if let Some(expiration_time) = expiration_time {
        let timestamp = expiration_time
            .parse::<i64>()
            .map_err(|e| ErrorObjectOwned::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
        topic_update_tx.expiration_time(
            OffsetDateTime::from_unix_timestamp(timestamp)
                .map_err(|e| mock_consensus_error("INVALID_EXPIRATION_TIME", &e.to_string()))?,
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
            let keys: Result<Vec<_>, _> =
                fee_exempt_keys.iter().map(|k| get_hedera_key(k)).collect();
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
                        let fixed_fee_obj = fee_obj.get("fixedFee").ok_or_else(|| {
                            ErrorObjectOwned::owned(
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
                                    ErrorObjectOwned::owned(
                                        INTERNAL_ERROR_CODE,
                                        "Missing amount in fixedFee".to_string(),
                                        None::<()>,
                                    )
                                })?;

                            let amount = amount_str.parse::<u64>().map_err(|e| {
                                mock_consensus_error(
                                    "CUSTOM_FEE_MUST_BE_POSITIVE",
                                    &format!("Invalid amount: {e}"),
                                )
                            })?;

                            let denominating_token_id = fixed_fee
                                .get("denominatingTokenId")
                                .and_then(|v| v.as_str())
                                .map(|s| {
                                    TokenId::from_str(s).map_err(|e| {
                                        ErrorObjectOwned::owned(
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
                                        ErrorObjectOwned::owned(
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
        fill_common_transaction_params(&mut topic_update_tx, &common_transaction_params, client);
    }

    let tx_response = topic_update_tx.execute(client).await.map_err(|e| from_hedera_error(e))?;

    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(TopicResponse { topic_id: None, status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn delete_topic(
    client: &Client,
    topic_id: Option<String>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<TopicResponse, ErrorObjectOwned> {
    let mut topic_delete_tx = TopicDeleteTransaction::new();

    if let Some(topic_id) = topic_id {
        topic_delete_tx.topic_id(TopicId::from_str(&topic_id).map_err(|e| {
            ErrorObjectOwned::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
        })?);
    }

    if let Some(common_transaction_params) = common_transaction_params {
        fill_common_transaction_params(&mut topic_delete_tx, &common_transaction_params, client);
    }

    let tx_response = topic_delete_tx.execute(client).await.map_err(|e| from_hedera_error(e))?;

    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(TopicResponse { topic_id: None, status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn submit_topic_message(
    client: &Client,
    topic_id: Option<String>,
    message: Option<String>,
    max_chunks: Option<i64>,
    chunk_size: Option<i64>,
    custom_fee_limits: Option<Value>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<TopicResponse, ErrorObjectOwned> {
    let mut topic_message_submit_tx = TopicMessageSubmitTransaction::new();

    if let Some(topic_id) = topic_id {
        topic_message_submit_tx.topic_id(TopicId::from_str(&topic_id).map_err(|e| {
            ErrorObjectOwned::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
        })?);
    }

    match message {
        Some(msg) if !msg.is_empty() => {
            topic_message_submit_tx.message(msg.into_bytes());
        }
        _ => {
            return Err(ErrorObjectOwned::owned(
                INTERNAL_ERROR_CODE,
                "Message is required".to_string(),
                None::<()>,
            ));
        }
    }

    if let Some(max_chunks) = max_chunks {
        if max_chunks < 0 {
            return Err(ErrorObjectOwned::owned(
                INTERNAL_ERROR_CODE,
                "Max chunks cannot be negative".to_string(),
                None::<()>,
            ));
        }
        if max_chunks == 0 {
            return Err(ErrorObjectOwned::owned(
                INTERNAL_ERROR_CODE,
                "Max chunks must be greater than zero".to_string(),
                None::<()>,
            ));
        }
        if max_chunks > 1000 {
            return Err(ErrorObjectOwned::owned(
                INTERNAL_ERROR_CODE,
                "Max chunks exceeds maximum allowed value".to_string(),
                None::<()>,
            ));
        }
        topic_message_submit_tx.max_chunks(max_chunks as usize);
    }

    if let Some(chunk_size) = chunk_size {
        if chunk_size < 0 {
            return Err(ErrorObjectOwned::owned(
                INTERNAL_ERROR_CODE,
                "Chunk size cannot be negative".to_string(),
                None::<()>,
            ));
        }
        if chunk_size == 0 {
            return Err(ErrorObjectOwned::owned(
                INTERNAL_ERROR_CODE,
                "Chunk size must be greater than zero".to_string(),
                None::<()>,
            ));
        }
        if chunk_size > 1000000 {
            return Err(ErrorObjectOwned::owned(
                INTERNAL_ERROR_CODE,
                "Chunk size exceeds maximum allowed value".to_string(),
                None::<()>,
            ));
        }
        topic_message_submit_tx.chunk_size(chunk_size as usize);
    }

    if let Some(custom_fee_limits) = custom_fee_limits {
        if let Value::Array(limits_array) = custom_fee_limits {
            let mut sdk_custom_fee_limits = Vec::new();
            for limit in limits_array {
                if let Value::Object(limit_obj) = limit {
                    let payer_id =
                        limit_obj.get("payerId").and_then(|v| v.as_str()).ok_or_else(|| {
                            ErrorObjectOwned::owned(
                                INTERNAL_ERROR_CODE,
                                "Missing payerId in customFeeLimit".to_string(),
                                None::<()>,
                            )
                        })?;

                    let account_id = AccountId::from_str(payer_id).map_err(|e| {
                        ErrorObjectOwned::owned(
                            INTERNAL_ERROR_CODE,
                            format!("Invalid payerId: {e}"),
                            None::<()>,
                        )
                    })?;

                    let fixed_fees_array =
                        limit_obj.get("fixedFees").and_then(|v| v.as_array()).ok_or_else(|| {
                            ErrorObjectOwned::owned(
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
                                    ErrorObjectOwned::owned(
                                        INTERNAL_ERROR_CODE,
                                        "Missing amount in fixedFee".to_string(),
                                        None::<()>,
                                    )
                                })?;

                            let amount = amount_str.parse::<u64>().map_err(|e| {
                                mock_consensus_error(
                                    "INVALID_MAX_CUSTOM_FEES",
                                    &format!("Invalid amount: {e}"),
                                )
                            })?;

                            let denominating_token_id = fee_obj
                                .get("denominatingTokenId")
                                .and_then(|v| v.as_str())
                                .map(|s| {
                                    TokenId::from_str(s).map_err(|_| {
                                        ErrorObjectOwned::owned(
                                            INTERNAL_ERROR_CODE,
                                            "Internal error".to_string(),
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

                    sdk_custom_fee_limits
                        .push(CustomFeeLimit::new(Some(account_id), custom_fixed_fees));
                }
            }
            topic_message_submit_tx.custom_fee_limits(sdk_custom_fee_limits);
        }
    }

    if let Some(common_transaction_params) = common_transaction_params {
        fill_common_transaction_params(
            &mut topic_message_submit_tx,
            &common_transaction_params,
            client,
        );
    }

    let tx_response =
        topic_message_submit_tx.execute(client).await.map_err(|e| from_hedera_error(e))?;

    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(TopicResponse { topic_id: None, status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn get_topic_info(
    client: &Client,
    topic_id: Option<String>,
    _common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<TopicInfoResponse, ErrorObjectOwned> {
    let mut query = TopicInfoQuery::new();
    if let Some(topic_id) = topic_id {
        query.topic_id(TopicId::from_str(&topic_id).map_err(|e| {
            ErrorObjectOwned::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
        })?);
    }
    let info = query.execute(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(TopicInfoResponse {
        topic_id: info.topic_id.to_string(),
        topic_memo: info.topic_memo,
        running_hash: hex::encode(&info.running_hash),
        sequence_number: info.sequence_number.to_string(),
        expiration_time: info.expiration_time.map(|t| t.unix_timestamp().to_string()),
        admin_key: info.admin_key.map(|k| crate::helpers::key_to_der_string(&k)),
        submit_key: info.submit_key.map(|k| crate::helpers::key_to_der_string(&k)),
        auto_renew_account_id: info.auto_renew_account_id.map(|id| id.to_string()),
        auto_renew_period: info.auto_renew_period.map(|d| d.whole_seconds().to_string()),
        ledger_id: info.ledger_id.to_string(),
        fee_schedule_key: info.fee_schedule_key.map(|k| crate::helpers::key_to_der_string(&k)),
        fee_exempt_keys: info.fee_exempt_keys.iter().map(|k| crate::helpers::key_to_der_string(k)).collect(),
        custom_fees: info.custom_fees.iter().map(|f| {
            serde_json::json!({
                "feeCollectorAccountId": f.fee_collector_account_id.as_ref().map(|id| id.to_string()),
                "amount": f.amount,
                "denominatingTokenId": f.denominating_token_id.as_ref().map(|id| id.to_string()),
            })
        }).collect(),
    })
}

pub fn build_topic_create_tx_from_value(
    params: &Value,
) -> Result<TopicCreateTransaction, ErrorObjectOwned> {
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
        let period = auto_renew_period
            .parse::<i64>()
            .map_err(|e| ErrorObjectOwned::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
        tx.auto_renew_period(Duration::seconds(period));
    }

    if let Some(auto_renew_account_id) = params.get("autoRenewAccountId").and_then(Value::as_str) {
        tx.auto_renew_account_id(AccountId::from_str(auto_renew_account_id).map_err(|e| {
            ErrorObjectOwned::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
        })?);
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
                        ErrorObjectOwned::owned(
                            INTERNAL_ERROR_CODE,
                            "Missing fixedFee in custom fee".to_string(),
                            None::<()>,
                        )
                    })?;

                    if let Value::Object(fixed_fee) = fixed_fee_obj {
                        let amount_str =
                            fixed_fee.get("amount").and_then(Value::as_str).ok_or_else(|| {
                                ErrorObjectOwned::owned(
                                    INTERNAL_ERROR_CODE,
                                    "Missing amount in fixedFee".to_string(),
                                    None::<()>,
                                )
                            })?;

                        let amount = amount_str.parse::<u64>().map_err(|e| {
                            mock_consensus_error(
                                "CUSTOM_FEE_MUST_BE_POSITIVE",
                                &format!("Invalid amount: {e}"),
                            )
                        })?;

                        let denominating_token_id = fixed_fee
                            .get("denominatingTokenId")
                            .and_then(Value::as_str)
                            .map(|s| {
                                TokenId::from_str(s).map_err(|e| {
                                    ErrorObjectOwned::owned(
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
                                    ErrorObjectOwned::owned(
                                        INTERNAL_ERROR_CODE,
                                        format!("Invalid account ID: {e}"),
                                        None::<()>,
                                    )
                                })
                            })
                            .transpose()?;

                        fees.push(CustomFixedFee::new(
                            amount,
                            denominating_token_id,
                            fee_collector_account_id,
                        ));
                    }
                }
            }
            tx.custom_fees(fees);
        }
    }

    Ok(tx)
}

pub fn build_topic_message_submit_tx_from_value(
    params: &Value,
) -> Result<TopicMessageSubmitTransaction, ErrorObjectOwned> {
    let mut tx = TopicMessageSubmitTransaction::new();

    if let Some(topic_id) = params.get("topicId").and_then(Value::as_str) {
        tx.topic_id(TopicId::from_str(topic_id).map_err(|e| {
            ErrorObjectOwned::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
        })?);
    }

    match params.get("message").and_then(Value::as_str) {
        Some(msg) if !msg.is_empty() => {
            tx.message(msg.as_bytes());
        }
        _ => {
            return Err(ErrorObjectOwned::owned(
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
            return Err(ErrorObjectOwned::owned(
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
                let payer_id =
                    limit_obj.get("payerId").and_then(Value::as_str).ok_or_else(|| {
                        ErrorObjectOwned::owned(
                            INTERNAL_ERROR_CODE,
                            "Missing payerId in customFeeLimit".to_string(),
                            None::<()>,
                        )
                    })?;

                let account_id = AccountId::from_str(payer_id).map_err(|e| {
                    ErrorObjectOwned::owned(
                        INTERNAL_ERROR_CODE,
                        format!("Invalid payerId: {e}"),
                        None::<()>,
                    )
                })?;

                let fixed_fees_array =
                    limit_obj.get("fixedFees").and_then(Value::as_array).ok_or_else(|| {
                        ErrorObjectOwned::owned(
                            INTERNAL_ERROR_CODE,
                            "Missing fixedFees in customFeeLimit".to_string(),
                            None::<()>,
                        )
                    })?;

                let mut fixed_fees = Vec::new();
                for fee in fixed_fees_array {
                    if let Value::Object(fee_obj) = fee {
                        let amount_str =
                            fee_obj.get("amount").and_then(Value::as_str).ok_or_else(|| {
                                ErrorObjectOwned::owned(
                                    INTERNAL_ERROR_CODE,
                                    "Missing amount in fixedFee".to_string(),
                                    None::<()>,
                                )
                            })?;

                        let amount = amount_str.parse::<u64>().map_err(|e| {
                            mock_consensus_error(
                                "CUSTOM_FEE_MUST_BE_POSITIVE",
                                &format!("Invalid amount: {e}"),
                            )
                        })?;

                        let denominating_token_id = fee_obj
                            .get("denominatingTokenId")
                            .and_then(Value::as_str)
                            .map(|s| {
                                TokenId::from_str(s).map_err(|e| {
                                    ErrorObjectOwned::owned(
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
