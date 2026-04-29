// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use bytes::Bytes;
use http_body_util::BodyExt as _;
use hyper::body::Incoming;
use hyper::{
    Method,
    Request,
    Response,
};
use hyper_openssl::client::legacy::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client as HttpClient;
use hyper_util::rt::TokioExecutor;
use openssl::ssl::{
    SslConnector,
    SslMethod,
    SslVerifyMode,
};

use crate::fee_estimate_types::{
    FeeEstimate,
    FeeEstimateResponse,
    FeeExtra,
    NetworkFee,
};
use crate::{
    Client,
    Error,
    FeeEstimateMode,
};

/// Default maximum number of retry attempts.
const DEFAULT_MAX_ATTEMPTS: usize = 10;

/// Default maximum backoff duration.
const DEFAULT_MAX_BACKOFF: Duration = Duration::from_secs(8);

/// Initial backoff delay.
const INITIAL_BACKOFF: Duration = Duration::from_millis(500);

/// Fee estimation query that communicates with the mirror node REST API.
///
/// This query estimates the expected transaction fees without submitting the transaction
/// to the network, per HIP-1261.
///
/// # Examples
/// ```no_run
/// # async fn example() -> hiero_sdk::Result<()> {
/// use hiero_sdk::{Client, TransferTransaction, FeeEstimateQuery, FeeEstimateMode};
///
/// let client = Client::for_testnet();
/// client.set_operator(/* ... */);
///
/// let mut tx = TransferTransaction::new();
/// // ... configure transaction ...
/// tx.freeze_with(&client)?;
///
/// let response = FeeEstimateQuery::new()
///     .set_transaction_bytes(tx.to_bytes()?)
///     .set_mode(FeeEstimateMode::Intrinsic)
///     .execute(&client)
///     .await?;
///
/// println!("Estimated fee: {} tinycents", response.total);
/// # Ok(())
/// # }
/// ```
pub struct FeeEstimateQuery {
    mode: FeeEstimateMode,
    transaction_bytes: Option<Vec<u8>>,
    high_volume_throttle: Option<u32>,
    max_attempts: usize,
    max_backoff: Duration,
}

impl Default for FeeEstimateQuery {
    fn default() -> Self {
        Self::new()
    }
}

impl FeeEstimateQuery {
    /// Creates a new fee estimate query with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            mode: FeeEstimateMode::default(),
            transaction_bytes: None,
            high_volume_throttle: None,
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            max_backoff: DEFAULT_MAX_BACKOFF,
        }
    }

    /// Sets the fee estimation mode.
    #[must_use]
    pub fn set_mode(mut self, mode: FeeEstimateMode) -> Self {
        self.mode = mode;
        self
    }

    /// Returns the fee estimation mode.
    #[must_use]
    pub fn get_mode(&self) -> FeeEstimateMode {
        self.mode
    }

    /// Sets the protobuf-encoded transaction bytes to estimate fees for.
    ///
    /// The transaction must be frozen before calling `to_bytes()`.
    #[must_use]
    pub fn set_transaction_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.transaction_bytes = Some(bytes);
        self
    }

    /// Returns the transaction bytes, if set.
    #[must_use]
    pub fn get_transaction_bytes(&self) -> Option<&[u8]> {
        self.transaction_bytes.as_deref()
    }

    /// Sets the high-volume throttle utilization in basis points (0–10000).
    ///
    /// This simulates throttle utilization for high-volume pricing per HIP-1313
    /// without impacting the actual network.
    ///
    /// # Panics
    /// Panics if `basis_points` is greater than 10000.
    #[must_use]
    pub fn set_high_volume_throttle(mut self, basis_points: u32) -> Self {
        assert!(basis_points <= 10000, "high_volume_throttle must be between 0 and 10000");
        self.high_volume_throttle = Some(basis_points);
        self
    }

    /// Returns the high-volume throttle, if set.
    #[must_use]
    pub fn get_high_volume_throttle(&self) -> Option<u32> {
        self.high_volume_throttle
    }

    /// Sets the maximum number of retry attempts.
    #[must_use]
    pub fn set_max_attempts(mut self, max_attempts: usize) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    /// Returns the maximum number of retry attempts.
    #[must_use]
    pub fn get_max_attempts(&self) -> usize {
        self.max_attempts
    }

    /// Sets the maximum backoff duration between retries.
    ///
    /// # Panics
    /// Panics if `max_backoff` is less than 500ms.
    #[must_use]
    pub fn set_max_backoff(mut self, max_backoff: Duration) -> Self {
        assert!(
            max_backoff >= Duration::from_millis(500),
            "max_backoff must be at least 500ms"
        );
        self.max_backoff = max_backoff;
        self
    }

    /// Returns the maximum backoff duration.
    #[must_use]
    pub fn get_max_backoff(&self) -> Duration {
        self.max_backoff
    }

    /// Executes the fee estimate query against the mirror node REST API.
    ///
    /// # Errors
    /// - If no transaction bytes have been set.
    /// - If the mirror node returns an unrecoverable error.
    /// - If all retry attempts are exhausted.
    pub async fn execute(&self, client: &Client) -> crate::Result<FeeEstimateResponse> {
        let transaction_bytes = self
            .transaction_bytes
            .as_ref()
            .ok_or_else(|| Error::basic_parse("transaction bytes must be set on FeeEstimateQuery"))?;

        let base_url = mirror_rest_base_url(client);
        let url = self.build_url(&base_url);

        let http_client = build_http_client(&base_url);

        let mut attempt = 0;
        loop {
            let request = Request::builder()
                .method(Method::POST)
                .uri(&url)
                .header("Content-Type", "application/x-protobuf")
                .header("Accept", "application/json")
                .body(http_body_util::Full::new(Bytes::from(transaction_bytes.clone())))
                .map_err(|e| Error::basic_parse(e.to_string()))?;

            let result = http_client.request(request).await;

            match result {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        return parse_response(response).await;
                    }

                    if should_retry_status(status) && attempt < self.max_attempts {
                        attempt += 1;
                        let delay = compute_backoff(attempt, self.max_backoff);
                        tokio::time::sleep(delay).await;
                        continue;
                    }

                    let body = read_body(response).await.unwrap_or_default();
                    return Err(Error::basic_parse(format!(
                        "fee estimate query failed with HTTP {status}: {body}"
                    )));
                }
                Err(e) => {
                    if attempt < self.max_attempts {
                        attempt += 1;
                        let delay = compute_backoff(attempt, self.max_backoff);
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    return Err(Error::basic_parse(format!(
                        "fee estimate query failed after {attempt} attempts: {e}"
                    )));
                }
            }
        }
    }

    fn build_url(&self, base_url: &str) -> String {
        let mut url = format!("{base_url}/network/fees?mode={}", self.mode.as_str());
        if let Some(throttle) = self.high_volume_throttle {
            url.push_str(&format!("&highVolumeThrottle={throttle}"));
        }
        url
    }
}

/// Determines whether a response status code is retryable.
fn should_retry_status(status: hyper::StatusCode) -> bool {
    matches!(
        status.as_u16(),
        408 | 429 | 500 | 502 | 503 | 504
    )
}

/// Computes exponential backoff delay: `min(500ms * 2^attempt, max_backoff)`.
fn compute_backoff(attempt: usize, max_backoff: Duration) -> Duration {
    let delay = INITIAL_BACKOFF.saturating_mul(1u32.wrapping_shl(attempt as u32));
    delay.min(max_backoff)
}

/// Constructs the mirror node REST base URL from the client's mirror network configuration.
fn mirror_rest_base_url(client: &Client) -> String {
    let addresses = client.mirror_network();
    let address = addresses.first().expect("mirror network must have at least one address");

    // Parse host and port
    let (host, port_str) = if let Some(idx) = address.rfind(':') {
        (&address[..idx], &address[idx + 1..])
    } else {
        (address.as_str(), "443")
    };

    let port: u16 = port_str.parse().unwrap_or(443);

    let is_localhost = host.contains("localhost") || host.contains("127.0.0.1");

    if is_localhost {
        // For local development, map gRPC port 5600 to REST port 5551
        let rest_port = if port == 5600 { 5551 } else { port };
        format!("http://{host}:{rest_port}/api/v1")
    } else {
        let scheme = if port == 80 { "http" } else { "https" };
        if (scheme == "https" && port == 443) || (scheme == "http" && port == 80) {
            format!("{scheme}://{host}/api/v1")
        } else {
            format!("{scheme}://{host}:{port}/api/v1")
        }
    }
}

/// Builds an HTTP client appropriate for the given base URL.
fn build_http_client(
    base_url: &str,
) -> HttpClient<HttpsConnector<HttpConnector>, http_body_util::Full<Bytes>> {
    let mut http = HttpConnector::new();
    http.enforce_http(false);

    if base_url.starts_with("https") {
        let mut ssl_builder = SslConnector::builder(SslMethod::tls()).unwrap();
        ssl_builder.set_verify(SslVerifyMode::PEER);
        let https = HttpsConnector::with_connector(http, ssl_builder).unwrap();
        HttpClient::builder(TokioExecutor::new()).build(https)
    } else {
        // For HTTP (localhost), still use HttpsConnector but with permissive settings
        let mut ssl_builder = SslConnector::builder(SslMethod::tls()).unwrap();
        ssl_builder.set_verify(SslVerifyMode::NONE);
        let https = HttpsConnector::with_connector(http, ssl_builder).unwrap();
        HttpClient::builder(TokioExecutor::new()).build(https)
    }
}

/// Reads the full body of an HTTP response as a string.
async fn read_body(response: Response<Incoming>) -> Result<String, String> {
    let body_bytes = response
        .into_body()
        .collect()
        .await
        .map_err(|e| e.to_string())?
        .to_bytes();
    String::from_utf8(body_bytes.to_vec()).map_err(|e| e.to_string())
}

/// Parses a successful fee estimate response from JSON.
async fn parse_response(response: Response<Incoming>) -> crate::Result<FeeEstimateResponse> {
    let body = read_body(response)
        .await
        .map_err(|e| Error::basic_parse(format!("failed to read response body: {e}")))?;

    parse_fee_estimate_response_json(&body)
}

fn parse_fee_estimate_response_json(json: &str) -> crate::Result<FeeEstimateResponse> {
    let value: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| Error::basic_parse(format!("failed to parse fee estimate JSON: {e}")))?;

    let high_volume_multiplier = value
        .get("high_volume_multiplier")
        .or_else(|| value.get("highVolumeMultiplier"))
        .and_then(|v| v.as_u64())
        .unwrap_or(1);

    let network = value
        .get("network")
        .and_then(|v| parse_network_fee(v));

    let node = value
        .get("node")
        .and_then(|v| parse_fee_estimate(v));

    let service = value
        .get("service")
        .and_then(|v| parse_fee_estimate(v));

    let total = value
        .get("total")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    Ok(FeeEstimateResponse {
        high_volume_multiplier,
        network,
        node,
        service,
        total,
    })
}

fn parse_network_fee(value: &serde_json::Value) -> Option<NetworkFee> {
    if value.is_null() {
        return None;
    }
    Some(NetworkFee {
        multiplier: value.get("multiplier").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        subtotal: value.get("subtotal").and_then(|v| v.as_u64()).unwrap_or(0),
    })
}

fn parse_fee_estimate(value: &serde_json::Value) -> Option<FeeEstimate> {
    if value.is_null() {
        return None;
    }
    let base = value.get("base").and_then(|v| v.as_u64()).unwrap_or(0);
    let extras = value
        .get("extras")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(parse_fee_extra).collect())
        .unwrap_or_default();

    Some(FeeEstimate { base, extras })
}

fn parse_fee_extra(value: &serde_json::Value) -> Option<FeeExtra> {
    if !value.is_object() {
        return None;
    }
    Some(FeeExtra {
        name: value.get("name").and_then(|v| v.as_str()).map(String::from),
        included: value.get("included").and_then(|v| v.as_u64()).unwrap_or(0),
        count: value.get("count").and_then(|v| v.as_u64()).unwrap_or(0),
        charged: value.get("charged").and_then(|v| v.as_u64()).unwrap_or(0),
        fee_per_unit: value
            .get("fee_per_unit")
            .or_else(|| value.get("feePerUnit"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        subtotal: value.get("subtotal").and_then(|v| v.as_u64()).unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_fee_estimate_response() {
        let json = r#"{
            "high_volume_multiplier": 1,
            "network": {
                "multiplier": 2,
                "subtotal": 1000
            },
            "node": {
                "base": 500,
                "extras": [
                    {
                        "name": "SIGNATURE_VERIFICATION",
                        "included": 1,
                        "count": 2,
                        "charged": 1,
                        "fee_per_unit": 100,
                        "subtotal": 100
                    }
                ]
            },
            "service": {
                "base": 300,
                "extras": []
            },
            "total": 1800
        }"#;

        let response = parse_fee_estimate_response_json(json).unwrap();
        assert_eq!(response.high_volume_multiplier, 1);
        assert_eq!(response.total, 1800);

        let network = response.network.unwrap();
        assert_eq!(network.multiplier, 2);
        assert_eq!(network.subtotal, 1000);

        let node = response.node.unwrap();
        assert_eq!(node.base, 500);
        assert_eq!(node.extras.len(), 1);
        assert_eq!(node.extras[0].name.as_deref(), Some("SIGNATURE_VERIFICATION"));
        assert_eq!(node.extras[0].charged, 1);
        assert_eq!(node.extras[0].fee_per_unit, 100);

        let service = response.service.unwrap();
        assert_eq!(service.base, 300);
        assert!(service.extras.is_empty());
    }

    #[test]
    fn parse_camel_case_response() {
        let json = r#"{
            "highVolumeMultiplier": 2,
            "network": { "multiplier": 1, "subtotal": 500 },
            "node": { "base": 100, "extras": [{ "name": "SIG", "included": 0, "count": 1, "charged": 1, "feePerUnit": 50, "subtotal": 50 }] },
            "service": { "base": 200, "extras": [] },
            "total": 750
        }"#;

        let response = parse_fee_estimate_response_json(json).unwrap();
        assert_eq!(response.high_volume_multiplier, 2);
        assert_eq!(response.total, 750);
        assert_eq!(response.node.unwrap().extras[0].fee_per_unit, 50);
    }

    #[test]
    fn fee_estimate_subtotal() {
        let estimate = FeeEstimate {
            base: 100,
            extras: vec![
                FeeExtra {
                    name: Some("a".into()),
                    included: 0,
                    count: 1,
                    charged: 1,
                    fee_per_unit: 50,
                    subtotal: 50,
                },
                FeeExtra {
                    name: None,
                    included: 0,
                    count: 2,
                    charged: 2,
                    fee_per_unit: 25,
                    subtotal: 50,
                },
            ],
        };
        assert_eq!(estimate.subtotal(), 200);
    }

    #[test]
    fn build_url_intrinsic() {
        let query = FeeEstimateQuery::new();
        let url = query.build_url("https://mirror.example.com/api/v1");
        assert_eq!(url, "https://mirror.example.com/api/v1/network/fees?mode=INTRINSIC");
    }

    #[test]
    fn build_url_state_with_throttle() {
        let query = FeeEstimateQuery::new()
            .set_mode(FeeEstimateMode::State)
            .set_high_volume_throttle(5000);
        let url = query.build_url("https://mirror.example.com/api/v1");
        assert_eq!(
            url,
            "https://mirror.example.com/api/v1/network/fees?mode=STATE&highVolumeThrottle=5000"
        );
    }

    #[test]
    fn mirror_rest_url_mainnet() {
        let url = mirror_rest_base_url_from_address("mainnet-public.mirrornode.hedera.com:443");
        assert_eq!(url, "https://mainnet-public.mirrornode.hedera.com/api/v1");
    }

    #[test]
    fn mirror_rest_url_localhost() {
        let url = mirror_rest_base_url_from_address("127.0.0.1:5600");
        assert_eq!(url, "http://127.0.0.1:5551/api/v1");
    }

    #[test]
    fn mirror_rest_url_localhost_custom_port() {
        let url = mirror_rest_base_url_from_address("localhost:8080");
        assert_eq!(url, "http://localhost:8080/api/v1");
    }

    /// Helper for tests that works on a single address string.
    fn mirror_rest_base_url_from_address(address: &str) -> String {
        let (host, port_str) = if let Some(idx) = address.rfind(':') {
            (&address[..idx], &address[idx + 1..])
        } else {
            (address, "443")
        };

        let port: u16 = port_str.parse().unwrap_or(443);
        let is_localhost = host.contains("localhost") || host.contains("127.0.0.1");

        if is_localhost {
            let rest_port = if port == 5600 { 5551 } else { port };
            format!("http://{host}:{rest_port}/api/v1")
        } else {
            let scheme = if port == 80 { "http" } else { "https" };
            if (scheme == "https" && port == 443) || (scheme == "http" && port == 80) {
                format!("{scheme}://{host}/api/v1")
            } else {
                format!("{scheme}://{host}:{port}/api/v1")
            }
        }
    }

    #[test]
    fn default_settings() {
        let query = FeeEstimateQuery::new();
        assert_eq!(query.get_mode(), FeeEstimateMode::Intrinsic);
        assert_eq!(query.get_max_attempts(), 10);
        assert!(query.get_transaction_bytes().is_none());
        assert!(query.get_high_volume_throttle().is_none());
    }

    #[test]
    #[should_panic(expected = "high_volume_throttle must be between 0 and 10000")]
    fn high_volume_throttle_out_of_range() {
        let _ = FeeEstimateQuery::new().set_high_volume_throttle(10001);
    }

    #[test]
    fn backoff_computation() {
        assert_eq!(compute_backoff(0, Duration::from_secs(8)), Duration::from_millis(500));
        assert_eq!(compute_backoff(1, Duration::from_secs(8)), Duration::from_millis(1000));
        assert_eq!(compute_backoff(2, Duration::from_secs(8)), Duration::from_millis(2000));
        assert_eq!(compute_backoff(3, Duration::from_secs(8)), Duration::from_millis(4000));
        assert_eq!(compute_backoff(4, Duration::from_secs(8)), Duration::from_millis(8000));
        assert_eq!(compute_backoff(5, Duration::from_secs(8)), Duration::from_millis(8000));
    }
}
