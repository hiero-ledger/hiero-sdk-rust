use std::collections::HashMap;

use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountCreateResponse {
    pub account_id: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountUpdateResponse {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateKeyResponse {
    pub key: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub private_keys: Vec<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContractResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_id: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountBalanceResponse {
    pub hbars: String,
    #[serde(default)]
    pub token_balances: HashMap<String, String>,
    #[serde(default)]
    pub token_decimals: HashMap<String, u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContractCallResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_used: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    // Extracted common return value types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bool: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes32: Option<String>,
    // Integer types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub int8: Option<i8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uint8: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub int16: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uint16: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub int24: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uint24: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub int32: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uint32: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub int40: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uint40: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub int48: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uint48: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub int56: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uint56: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub int64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uint64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub int256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uint256: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContractByteCodeResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytecode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContractInfoResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_renew_period: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_renew_account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_memo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_deleted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_automatic_token_associations: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staking_info: Option<StakingInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StakingInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decline_staking_reward: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stake_period_start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_reward: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staked_to_me: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staked_account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staked_node_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InfoResponse {
    pub info: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TopicResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic_id: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TopicInfoResponse {
    pub topic_id: String,
    pub topic_memo: String,
    pub running_hash: String,
    pub sequence_number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submit_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_renew_account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_renew_period: Option<String>,
    pub ledger_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_schedule_key: Option<String>,
    #[serde(default)]
    pub fee_exempt_keys: Vec<String>,
    #[serde(default)]
    pub custom_fees: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleInfoResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payer_account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_memo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_transaction_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_for_expiry: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    pub status: String,
}
