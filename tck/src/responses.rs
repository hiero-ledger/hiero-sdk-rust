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
    pub contract_id: Option<String>,
    pub gas_used: u64,
    pub result: String,
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
