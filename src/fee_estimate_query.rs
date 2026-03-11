// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use hiero_sdk_proto::services;
use prost::Message;
#[cfg(feature = "serde")]
use serde_derive::{
    Deserialize,
    Serialize,
};
use tokio::time::sleep;

use crate::transaction::{
    ChunkData,
    Transaction,
    TransactionData,
    TransactionExecute,
};
use crate::{
    Client,
    Error,
};

/// Fee estimation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeeEstimateMode {
    /// Estimate based on intrinsic properties plus the latest known state
    State,
    /// Estimate based solely on the transaction's inherent properties
    Intrinsic,
}

impl FeeEstimateMode {
    fn as_str(&self) -> &'static str {
        match self {
            FeeEstimateMode::State => "STATE",
            FeeEstimateMode::Intrinsic => "INTRINSIC",
        }
    }
}

impl Default for FeeEstimateMode {
    fn default() -> Self {
        Self::State
    }
}

/// Fee estimate response from the mirror node
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FeeEstimateResponse {
    /// The mode that was used to calculate the fees
    /// Defaults to "STATE" if not present in the response
    #[cfg_attr(feature = "serde", serde(default = "default_mode_string"))]
    pub mode: String,
    /// The network fee component
    pub network: NetworkFee,
    /// The node fee component
    pub node: FeeEstimate,
    /// The service fee component
    pub service: FeeEstimate,
    /// The sum of the network, node, and service subtotals in tinycents
    pub total: u64,
    /// An array of strings for any caveats
    pub notes: Vec<String>,
}

#[cfg(feature = "serde")]
fn default_mode_string() -> String {
    "STATE".to_string()
}

/// Network fee component
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NetworkFee {
    /// The multiplier for the network fee
    pub multiplier: u64,
    /// The subtotal in tinycents
    pub subtotal: u64,
}

/// Fee estimate for a component
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FeeEstimate {
    /// The base fee price, in tinycents
    pub base: u64,
    /// The extra fees that apply for this fee component
    pub extras: Vec<FeeExtra>,
}

impl FeeEstimate {
    /// Calculate the subtotal (base + sum of extras)
    pub fn subtotal(&self) -> u64 {
        self.base + self.extras.iter().map(|e| e.subtotal).sum::<u64>()
    }
}

/// Extra fee charged for the transaction
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FeeExtra {
    /// The charged count of items as calculated by `max(0, count - included)`
    #[cfg_attr(feature = "serde", serde(default))]
    pub charged: u32,
    /// The actual count of items received
    pub count: u32,
    /// The fee price per unit in tinycents
    pub fee_per_unit: u64,
    /// The count of this "extra" that is included for free
    #[cfg_attr(feature = "serde", serde(default))]
    pub included: u32,
    /// The unique name of this extra fee as defined in the fee schedule
    pub name: String,
    /// The subtotal in tinycents for this extra fee. Calculated by multiplying the charged count by the fee_per_unit
    pub subtotal: u64,
}

const DEFAULT_MAX_ATTEMPTS: u64 = 10;

/// FeeEstimateQuery allows users to query expected transaction fees without submitting transactions to the network
pub struct FeeEstimateQuery<D> {
    mode: FeeEstimateMode,
    transaction: Option<Transaction<D>>,
    max_attempts: u64,
}

impl<D> Default for FeeEstimateQuery<D> {
    fn default() -> Self {
        Self { mode: FeeEstimateMode::State, transaction: None, max_attempts: DEFAULT_MAX_ATTEMPTS }
    }
}

impl<D> FeeEstimateQuery<D> {
    /// Create a new FeeEstimateQuery
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the estimation mode (optional, defaults to STATE)
    pub fn mode(mut self, mode: FeeEstimateMode) -> Self {
        self.mode = mode;
        self
    }

    /// Get the current estimation mode
    pub fn get_mode(&self) -> FeeEstimateMode {
        self.mode
    }

    /// Set the transaction to estimate (required)
    pub fn transaction(mut self, transaction: Transaction<D>) -> Self {
        self.transaction = Some(transaction);
        self
    }

    /// Get the current transaction
    pub fn get_transaction(&self) -> Option<&Transaction<D>> {
        self.transaction.as_ref()
    }

    /// Set the maximum number of retry attempts
    pub fn max_attempts(mut self, max_attempts: u64) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    /// Get the maximum number of retry attempts
    pub fn get_max_attempts(&self) -> u64 {
        self.max_attempts
    }
}

impl<D> FeeEstimateQuery<D>
where
    D: TransactionData + TransactionExecute,
{
    /// Execute the fee estimation query with the provided client
    pub async fn execute(&mut self, client: &Client) -> crate::Result<FeeEstimateResponse> {
        let transaction = self
            .transaction
            .as_mut()
            .ok_or_else(|| Error::basic_parse("transaction is required"))?;

        // Freeze the transaction if not already frozen
        if !transaction.is_frozen() {
            transaction.freeze_with(Some(client))?;
        }

        // Use make_sources to get the protobuf transactions
        let sources = transaction.make_sources()?;
        let transactions = sources.transactions();

        // Check if this is a chunked transaction
        let chunk_data = transaction.data().maybe_chunk_data();
        if let Some(chunk_data) = chunk_data {
            return execute_chunked_transaction(
                self.mode,
                self.max_attempts,
                client,
                transactions,
                chunk_data,
            )
            .await;
        }

        // Handle single transaction - use the first transaction from sources
        if let Some(proto_tx) = transactions.first() {
            call_get_fee_estimate(self.mode, self.max_attempts, client, proto_tx).await
        } else {
            Err(Error::basic_parse("no transactions found"))
        }
    }
}

/// Handle fee estimation for chunked transactions
async fn execute_chunked_transaction(
    mode: FeeEstimateMode,
    max_attempts: u64,
    client: &Client,
    transactions: &[services::Transaction],
    chunk_data: &ChunkData,
) -> crate::Result<FeeEstimateResponse> {
    let num_chunks = chunk_data.used_chunks();
    if num_chunks == 0 {
        return Err(Error::basic_parse("transaction has no chunks"));
    }

    // Get transactions for the first node (all chunks should have the same structure)
    // We need one transaction per chunk
    let node_count = transactions.len() / num_chunks;
    if node_count == 0 {
        return Err(Error::basic_parse("no transactions found for chunks"));
    }

    let mut aggregated_response = FeeEstimateResponse {
        mode: mode.as_str().to_string(),
        node: FeeEstimate { base: 0, extras: vec![] },
        service: FeeEstimate { base: 0, extras: vec![] },
        network: NetworkFee { multiplier: 0, subtotal: 0 },
        total: 0,
        notes: vec![],
    };

    let mut total_node_subtotal = 0u64;
    let mut total_service_subtotal = 0u64;

    // Estimate fees for each chunk (use first node's transactions)
    for chunk_index in 0..num_chunks {
        let tx_index = chunk_index * node_count;
        if tx_index >= transactions.len() {
            return Err(Error::basic_parse("insufficient transactions for chunks"));
        }

        let chunk_tx = &transactions[tx_index];
        let chunk_response = call_get_fee_estimate(mode, max_attempts, client, chunk_tx).await?;

        total_node_subtotal += chunk_response.node.subtotal();
        total_service_subtotal += chunk_response.service.subtotal();

        if chunk_index == 0 {
            aggregated_response.network.multiplier = chunk_response.network.multiplier;
        }

        aggregated_response.notes.extend(chunk_response.notes);
    }

    aggregated_response.node.base = total_node_subtotal;
    aggregated_response.service.base = total_service_subtotal;
    aggregated_response.network.subtotal =
        total_node_subtotal * aggregated_response.network.multiplier;
    aggregated_response.total =
        aggregated_response.network.subtotal + total_node_subtotal + total_service_subtotal;

    Ok(aggregated_response)
}

/// Call the fee estimate REST API endpoint
async fn call_get_fee_estimate(
    mode: FeeEstimateMode,
    max_attempts: u64,
    client: &Client,
    proto_tx: &services::Transaction,
) -> crate::Result<FeeEstimateResponse> {
    let mirror_network = client.mirror_network();
    if mirror_network.is_empty() {
        return Err(Error::basic_parse("mirror node is not set"));
    }

    let mirror_url = get_mirror_rest_api_base_url(&mirror_network[0])?;

    let tx_bytes = proto_tx.encode_to_vec();

    let url = format!("{}/api/v1/network/fees?mode={}", mirror_url, mode.as_str());

    let mut attempt = 0u64;
    let mut last_err = None;

    while attempt < max_attempts {
        let response = reqwest::Client::new()
            .post(&url)
            .header("Content-Type", "application/protobuf")
            .body(tx_bytes.clone())
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().as_u16() == 200 => {
                let body = resp.bytes().await.map_err(|e| {
                    Error::basic_parse(format!("failed to read response body: {}", e))
                })?;

                #[cfg(feature = "serde")]
                let fee_response: FeeEstimateResponse =
                    serde_json::from_slice(&body).map_err(|e| {
                        Error::basic_parse(format!("failed to unmarshal response: {}", e))
                    })?;
                #[cfg(not(feature = "serde"))]
                let fee_response: FeeEstimateResponse = {
                    return Err(Error::basic_parse("serde feature is required for fee estimation"));
                };

                return Ok(fee_response);
            }
            Ok(resp) => {
                let status = resp.status();
                let body_text = resp.text().await.unwrap_or_default();
                last_err = Some(format!(
                    "received non-200 response: {}, details: {}",
                    status.as_u16(),
                    body_text
                ));

                if !should_retry(None, Some(status.as_u16())) {
                    return Err(Error::basic_parse(last_err.unwrap()));
                }
            }
            Err(e) => {
                last_err = Some(format!("HTTP request failed: {}", e));
                if !should_retry(Some(&e), None) {
                    return Err(Error::basic_parse(format!(
                        "failed to call fee estimate API: {}",
                        last_err.unwrap()
                    )));
                }
            }
        }

        // Calculate delay with exponential backoff
        let delay_ms = (250.0 * (1u64 << attempt) as f64).min(8000.0);
        sleep(Duration::from_millis(delay_ms as u64)).await;

        attempt += 1;
    }

    Err(Error::basic_parse(format!(
        "failed to call fee estimate API after {} attempts: {}",
        max_attempts,
        last_err.unwrap_or_else(|| "unknown error".to_string())
    )))
}

/// Get the mirror REST API base URL
fn get_mirror_rest_api_base_url(mirror_address: &str) -> crate::Result<String> {
    let is_localhost = mirror_address.contains("localhost") || mirror_address.contains("127.0.0.1");

    if is_localhost {
        return Ok("http://localhost:8084".to_string());
    }

    // Extract host and port from address (format: "host:port")
    let parts: Vec<&str> = mirror_address.split(':').collect();
    if parts.len() != 2 {
        return Err(Error::basic_parse(format!(
            "invalid mirror address format: {}",
            mirror_address
        )));
    }

    let host = parts[0];
    let port = parts[1];

    // Determine protocol based on port
    let protocol = if port == "443" { "https" } else { "http" };

    Ok(format!("{}://{}:{}", protocol, host, port))
}

/// Determine if an error should be retried
fn should_retry(err: Option<&reqwest::Error>, status_code: Option<u16>) -> bool {
    if let Some(status) = status_code {
        // Retry on server errors (5xx) or rate limiting (429)
        if status >= 500 || status == 429 {
            return true;
        }
        // Don't retry on client errors (4xx) except 429
        if status >= 400 && status < 500 {
            return false;
        }
    }

    // Retry on network errors
    err.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee_estimate_mode_default() {
        assert_eq!(FeeEstimateMode::default(), FeeEstimateMode::State);
    }

    #[test]
    fn test_fee_estimate_mode_as_str() {
        assert_eq!(FeeEstimateMode::State.as_str(), "STATE");
        assert_eq!(FeeEstimateMode::Intrinsic.as_str(), "INTRINSIC");
    }

    #[test]
    fn test_fee_estimate_subtotal() {
        let fee = FeeEstimate {
            base: 100,
            extras: vec![
                FeeExtra {
                    charged: 1,
                    count: 1,
                    fee_per_unit: 10,
                    included: 0,
                    name: "extra1".to_string(),
                    subtotal: 10,
                },
                FeeExtra {
                    charged: 2,
                    count: 2,
                    fee_per_unit: 10,
                    included: 0,
                    name: "extra2".to_string(),
                    subtotal: 20,
                },
            ],
        };
        assert_eq!(fee.subtotal(), 130);
    }
}
