// Complete Hedera protobuf definitions for WASM
// Generated from ALL .proto files using prost-build

#![recursion_limit = "2048"]

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

// Include ALL generated protobuf definitions
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/hedera_protos.rs"));
}

// Time conversions for WASM builds
mod time_0_3;
mod fraction;

// Re-export commonly used types for convenience
// Re-export nested modules
pub use proto::proto::transaction_body;
pub use proto::proto::{
    signature_pair,
    AccountAmount,
    AccountId,
    CryptoTransferTransactionBody,
    Duration,
    SignatureMap,
    SignaturePair,
    Timestamp,
    Transaction,
    TransactionBody,
    TransactionId,
    TransferList,
};

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
    pub fn new(
        payer_shard: i64,
        payer_realm: i64,
        payer_account: i64,
        node_shard: i64,
        node_realm: i64,
        node_account: i64,
        transaction_fee: u64,
    ) -> HederaTransactionBuilder {
        console_log!("Creating Hedera transaction builder");
        HederaTransactionBuilder {
            payer_account: AccountId {
                shard_num: payer_shard,
                realm_num: payer_realm,
                account: Some(proto::proto::account_id::Account::AccountNum(payer_account)),
            },
            node_account: AccountId {
                shard_num: node_shard,
                realm_num: node_realm,
                account: Some(proto::proto::account_id::Account::AccountNum(node_account)),
            },
            transaction_fee,
        }
    }

    /// Create a crypto transfer transaction and return the bytes to sign
    #[wasm_bindgen]
    pub fn create_crypto_transfer(
        &self,
        receiver_shard: i64,
        receiver_realm: i64,
        receiver_account: i64,
        amount: i64,
    ) -> Vec<u8> {
        console_log!("Creating crypto transfer for {} tinybars", amount);

        let receiver = AccountId {
            shard_num: receiver_shard,
            realm_num: receiver_realm,
            account: Some(proto::proto::account_id::Account::AccountNum(receiver_account)),
        };

        let now = js_sys::Date::now();
        let transaction_id = TransactionId {
            account_id: Some(self.payer_account.clone()),
            transaction_valid_start: Some(Timestamp {
                seconds: (now / 1000.0) as i64,
                nanos: ((now % 1000.0) * 1_000_000.0) as i32,
            }),
            scheduled: false,
            nonce: 0,
        };

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

        let transaction_body = TransactionBody {
            transaction_id: Some(transaction_id),
            node_account_id: Some(self.node_account.clone()),
            transaction_fee: self.transaction_fee,
            transaction_valid_duration: Some(Duration { seconds: 180 }), // 3 minutes
            generate_record: false,
            memo: String::new(),
            batch_key: None,
            max_custom_fees: vec![],
            data: Some(transaction_body::Data::CryptoTransfer(CryptoTransferTransactionBody {
                transfers: Some(TransferList { account_amounts: transfers }),
                token_transfers: vec![], // Empty for crypto transfers
            })),
        };

        let bytes = transaction_body.encode_to_vec();
        console_log!("Generated transaction bytes: {} bytes", bytes.len());
        bytes
    }

    /// Create a complete signed transaction from body bytes and signature
    #[wasm_bindgen]
    pub fn create_signed_transaction(
        &self,
        body_bytes: Vec<u8>,
        signature: Vec<u8>,
        public_key_prefix: Vec<u8>,
    ) -> Vec<u8> {
        console_log!("Creating signed transaction");

        let signature_pair = SignaturePair {
            pub_key_prefix: public_key_prefix,
            signature: Some(signature_pair::Signature::Ed25519(signature)),
        };

        let signature_map = SignatureMap { sig_pair: vec![signature_pair] };

        let transaction = Transaction {
            body: Some(proto::proto::TransactionBody::decode(&body_bytes[..]).unwrap()),
            signed_transaction_bytes: vec![],
            sigs: Some(proto::proto::SignatureList {
                sigs: signature_map
                    .sig_pair
                    .iter()
                    .map(|pair| {
                        match &pair.signature {
                            Some(signature_pair::Signature::Ed25519(sig)) => {
                                proto::proto::Signature {
                                    signature: Some(proto::proto::signature::Signature::Ed25519(
                                        sig.clone(),
                                    )),
                                }
                            }
                            Some(signature_pair::Signature::Contract(sig)) => {
                                proto::proto::Signature {
                                    signature: Some(proto::proto::signature::Signature::Contract(
                                        sig.clone(),
                                    )),
                                }
                            }
                            Some(signature_pair::Signature::Rsa3072(sig)) => {
                                proto::proto::Signature {
                                    signature: Some(proto::proto::signature::Signature::Rsa3072(
                                        sig.clone(),
                                    )),
                                }
                            }
                            Some(signature_pair::Signature::Ecdsa384(sig)) => {
                                proto::proto::Signature {
                                    signature: Some(proto::proto::signature::Signature::Ecdsa384(
                                        sig.clone(),
                                    )),
                                }
                            }
                            Some(_) => {
                                // Handle any other signature types with default empty signature
                                proto::proto::Signature { signature: None }
                            }
                            None => proto::proto::Signature { signature: None },
                        }
                    })
                    .collect(),
            }),
            body_bytes,
            sig_map: Some(signature_map),
        };

        let bytes = transaction.encode_to_vec();

        console_log!("Generated signed transaction: {} bytes", bytes.len());
        bytes
    }
}

// Utility functions exposed to JavaScript
#[wasm_bindgen]
pub fn get_current_timestamp_seconds() -> i64 {
    (js_sys::Date::now() / 1000.0) as i64
}
