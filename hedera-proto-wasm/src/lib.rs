// Minimal Hedera protobuf definitions for WASM transaction serialization
// This crate provides only the essential protobuf structures needed to create and serialize Hedera transactions

use prost::Message;
use wasm_bindgen::prelude::*;

// Import the `console.log` function from the browser for debugging
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Macro for easy console logging
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

/// Account identifier on the Hedera network
#[derive(Clone, PartialEq, Message)]
pub struct AccountId {
    #[prost(int64, tag = "1")]
    pub shard_num: i64,
    #[prost(int64, tag = "2")]
    pub realm_num: i64,
    #[prost(int64, tag = "3")]
    pub account_num: i64,
}

/// Timestamp for transactions
#[derive(Clone, PartialEq, Message)]
pub struct Timestamp {
    #[prost(int64, tag = "1")]
    pub seconds: i64,
    #[prost(int32, tag = "2")]
    pub nanos: i32,
}

/// Transaction identifier
#[derive(Clone, PartialEq, Message)]
pub struct TransactionId {
    #[prost(message, optional, tag = "1")]
    pub account_id: Option<AccountId>,
    #[prost(message, optional, tag = "2")]
    pub transaction_valid_start: Option<Timestamp>,
    #[prost(bool, tag = "3")]
    pub scheduled: bool,
    #[prost(int32, tag = "4")]
    pub nonce: i32,
}

/// Duration for transaction validity
#[derive(Clone, PartialEq, Message)]
pub struct Duration {
    #[prost(int64, tag = "1")]
    pub seconds: i64,
}

/// Account amount for transfers
#[derive(Clone, PartialEq, Message)]
pub struct AccountAmount {
    #[prost(message, optional, tag = "1")]
    pub account_id: Option<AccountId>,
    #[prost(int64, tag = "2")]
    pub amount: i64,
    #[prost(bool, tag = "3")]
    pub is_approval: bool,
}

/// Transfer list for crypto transfers
#[derive(Clone, PartialEq, Message)]
pub struct TransferList {
    #[prost(message, repeated, tag = "1")]
    pub account_amounts: Vec<AccountAmount>,
}

/// Crypto transfer transaction body
#[derive(Clone, PartialEq, Message)]
pub struct CryptoTransferTransactionBody {
    #[prost(message, optional, tag = "1")]
    pub transfers: Option<TransferList>,
}

/// Transaction body - the core transaction data that gets signed
#[derive(Clone, PartialEq, Message)]
pub struct TransactionBody {
    #[prost(message, optional, tag = "1")]
    pub transaction_id: Option<TransactionId>,
    #[prost(message, optional, tag = "2")]
    pub node_account_id: Option<AccountId>,
    #[prost(uint64, tag = "3")]
    pub transaction_fee: u64,
    #[prost(message, optional, tag = "4")]
    pub transaction_valid_duration: Option<Duration>,
    #[prost(bool, tag = "5")]
    pub generate_record: bool,
    #[prost(string, tag = "6")]
    pub memo: String,
    #[prost(oneof = "transaction_body::Data", tags = "14")]
    pub data: Option<transaction_body::Data>,
}

pub mod transaction_body {
    #[derive(Clone, PartialEq, prost::Oneof)]
    pub enum Data {
        #[prost(message, tag = "14")]
        CryptoTransfer(super::CryptoTransferTransactionBody),
    }
}

/// Signature pair for transaction signing
#[derive(Clone, PartialEq, Message)]
pub struct SignaturePair {
    #[prost(bytes = "vec", tag = "1")]
    pub pub_key_prefix: Vec<u8>,
    #[prost(oneof = "signature_pair::Signature", tags = "4")]
    pub signature: Option<signature_pair::Signature>,
}

pub mod signature_pair {
    #[derive(Clone, PartialEq, prost::Oneof)]
    pub enum Signature {
        #[prost(bytes, tag = "4")]
        Ed25519(Vec<u8>),
    }
}

/// Signature map containing all signatures for a transaction
#[derive(Clone, PartialEq, Message)]
pub struct SignatureMap {
    #[prost(message, repeated, tag = "1")]
    pub sig_pair: Vec<SignaturePair>,
}

/// Complete signed transaction ready for submission
#[derive(Clone, PartialEq, Message)]
pub struct Transaction {
    #[prost(bytes = "vec", tag = "1")]
    pub body_bytes: Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub sig_map: Option<SignatureMap>,
}

// Utility functions for WASM and native use
impl TransactionBody {
    /// Serialize this transaction body to bytes for signing
    /// This is what you would sign with your private key
    pub fn to_bytes(&self) -> Vec<u8> {
        self.encode_to_vec()
    }
    
    /// Create a simple crypto transfer transaction
    pub fn new_crypto_transfer(
        transaction_id: TransactionId,
        node_account_id: AccountId,
        transaction_fee: u64,
        transfers: Vec<AccountAmount>,
    ) -> Self {
        Self {
            transaction_id: Some(transaction_id),
            node_account_id: Some(node_account_id),
            transaction_fee,
            transaction_valid_duration: Some(Duration { seconds: 180 }), // 3 minutes
            generate_record: false,
            memo: String::new(),
            data: Some(transaction_body::Data::CryptoTransfer(
                CryptoTransferTransactionBody {
                    transfers: Some(TransferList {
                        account_amounts: transfers,
                    }),
                },
            )),
        }
    }
}

impl Transaction {
    /// Create a new transaction from body bytes and signatures
    pub fn new(body_bytes: Vec<u8>, sig_map: SignatureMap) -> Self {
        Self {
            body_bytes,
            sig_map: Some(sig_map),
        }
    }
    
    /// Serialize this transaction to bytes for submission to Hedera
    pub fn to_bytes(&self) -> Vec<u8> {
        self.encode_to_vec()
    }
}

impl AccountId {
    /// Create a new account ID
    pub fn new(shard: i64, realm: i64, account: i64) -> Self {
        Self {
            shard_num: shard,
            realm_num: realm,
            account_num: account,
        }
    }
}

impl Timestamp {
    /// Create a timestamp from seconds and nanoseconds
    pub fn new(seconds: i64, nanos: i32) -> Self {
        Self { seconds, nanos }
    }
    
    /// Create a timestamp for "now" using JavaScript Date
    pub fn now() -> Self {
        let now = js_sys::Date::now();
        let seconds = (now / 1000.0) as i64;
        let nanos = ((now % 1000.0) * 1_000_000.0) as i32;
        Self { seconds, nanos }
    }
}

impl TransactionId {
    /// Create a new transaction ID
    pub fn new(account_id: AccountId) -> Self {
        Self {
            account_id: Some(account_id),
            transaction_valid_start: Some(Timestamp::now()),
            scheduled: false,
            nonce: 0,
        }
    }
}

// JavaScript-friendly wrapper functions using wasm-bindgen
#[wasm_bindgen]
pub struct HederaTransactionBuilder {
    payer_account: AccountId,
    node_account: AccountId,
    transaction_fee: u64,
}

#[wasm_bindgen]
impl HederaTransactionBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new(payer_shard: i64, payer_realm: i64, payer_account: i64,
               node_shard: i64, node_realm: i64, node_account: i64,
               transaction_fee: u64) -> HederaTransactionBuilder {
        console_log!("Creating Hedera transaction builder");
        HederaTransactionBuilder {
            payer_account: AccountId::new(payer_shard, payer_realm, payer_account),
            node_account: AccountId::new(node_shard, node_realm, node_account),
            transaction_fee,
        }
    }

    /// Create a crypto transfer transaction and return the bytes to sign
    #[wasm_bindgen]
    pub fn create_crypto_transfer(&self, 
                                  receiver_shard: i64, 
                                  receiver_realm: i64, 
                                  receiver_account: i64,
                                  amount: i64) -> Vec<u8> {
        console_log!("Creating crypto transfer for {} tinybars", amount);
        
        let receiver = AccountId::new(receiver_shard, receiver_realm, receiver_account);
        let transaction_id = TransactionId::new(self.payer_account.clone());
        
        let transfers = vec![
            AccountAmount {
                account_id: Some(self.payer_account.clone()),
                amount: -amount, // negative = sending
                is_approval: false,
            },
            AccountAmount {
                account_id: Some(receiver),
                amount, // positive = receiving
                is_approval: false,
            },
        ];
        
        let transaction_body = TransactionBody::new_crypto_transfer(
            transaction_id,
            self.node_account.clone(),
            self.transaction_fee,
            transfers,
        );
        
        let bytes = transaction_body.to_bytes();
        console_log!("Generated transaction bytes: {} bytes", bytes.len());
        bytes
    }

    /// Create a complete signed transaction from body bytes and signature
    #[wasm_bindgen]
    pub fn create_signed_transaction(&self, body_bytes: Vec<u8>, signature: Vec<u8>, public_key_prefix: Vec<u8>) -> Vec<u8> {
        console_log!("Creating signed transaction");
        
        let signature_pair = SignaturePair {
            pub_key_prefix: public_key_prefix,
            signature: Some(signature_pair::Signature::Ed25519(signature)),
        };
        
        let signature_map = SignatureMap {
            sig_pair: vec![signature_pair],
        };
        
        let transaction = Transaction::new(body_bytes, signature_map);
        let bytes = transaction.to_bytes();
        
        console_log!("Generated signed transaction: {} bytes", bytes.len());
        bytes
    }
}

// Utility functions exposed to JavaScript
#[wasm_bindgen]
pub fn get_current_timestamp_seconds() -> i64 {
    Timestamp::now().seconds
}

#[wasm_bindgen]
pub fn log_to_console(message: &str) {
    console_log!("{}", message);
}
