// SPDX-License-Identifier: Apache-2.0

//! Demonstration of WASM-compatible transaction building and serialization.
//!
//! This example shows how to build transactions and serialize them to bytes
//! in a WASM environment. These bytes can then be sent to a JavaScript client
//! for signing and submission to the Hedera network.
//!
//! Run with: cargo run --example wasm_transaction_demo

use hedera::{
    AccountId, AnyTransaction, FileCreateTransaction, Hbar, TokenCreateTransaction, TransactionId, TransferTransaction
};

fn main() {
    println!("=== Hedera WASM Transaction Building Demo ===\n");

    // Example 1: Simple Transfer Transaction
    println!("1. Building a Transfer Transaction:");
    let from_account = AccountId::new(0, 0, 1001);
    let to_account = AccountId::new(0, 0, 1002);
    let amount = Hbar::new(10);

    let mut transfer_tx = TransferTransaction::new();
    transfer_tx
        .hbar_transfer(from_account, -amount)
        .hbar_transfer(to_account, amount)
        .transaction_id(TransactionId::generate(from_account))
        .max_transaction_fee(Hbar::new(1))
        .transaction_memo("WASM Transfer Demo");

    match transfer_tx.to_bytes() {
        Ok(bytes) => {
            println!("   ✓ Transaction serialized successfully!");
            println!("   ✓ Byte length: {} bytes", bytes.len());
            println!("   ✓ First 20 bytes (hex): {}\n", hex_preview(&bytes, 20));
        }
        Err(e) => println!("   ✗ Error: {:?}\n", e),
    }

    // Example 2: File Create Transaction
    println!("2. Building a File Create Transaction:");
    let mut file_create_tx = FileCreateTransaction::new();
    file_create_tx
        .contents(b"Hello from WASM!")
        .transaction_id(TransactionId::generate(from_account))
        .max_transaction_fee(Hbar::new(2))
        .transaction_memo("WASM File Create");

    match file_create_tx.to_bytes() {
        Ok(bytes) => {
            println!("   ✓ Transaction serialized successfully!");
            println!("   ✓ Byte length: {} bytes", bytes.len());
            println!("   ✓ First 20 bytes (hex): {}\n", hex_preview(&bytes, 20));
        }
        Err(e) => println!("   ✗ Error: {:?}\n", e),
    }

    // Example 3: Token Create Transaction
    println!("3. Building a Token Create Transaction:");
    let mut token_create_tx = TokenCreateTransaction::new();
    token_create_tx
        .name("WASM Token")
        .symbol("WASM")
        .decimals(2)
        .initial_supply(1_000_000)
        .treasury_account_id(from_account)
        .transaction_id(TransactionId::generate(from_account))
        .max_transaction_fee(Hbar::new(40))
        .transaction_memo("WASM Token Create");

    match token_create_tx.to_bytes() {
        Ok(bytes) => {
            println!("   ✓ Transaction serialized successfully!");
            println!("   ✓ Byte length: {} bytes", bytes.len());
            println!("   ✓ First 20 bytes (hex): {}\n", hex_preview(&bytes, 20));
        }
        Err(e) => println!("   ✗ Error: {:?}\n", e),
    }

    // Example 4: Round-trip serialization/deserialization using AnyTransaction
    println!("4. Testing Round-trip Serialization (with AnyTransaction):");
    let original_tx_id = TransactionId::generate(from_account);
    let mut original_tx = TransferTransaction::new();
    original_tx
        .hbar_transfer(from_account, Hbar::from_tinybars(-100))
        .hbar_transfer(to_account, Hbar::from_tinybars(100))
        .transaction_id(original_tx_id)
        .max_transaction_fee(Hbar::new(1));

    match original_tx.to_bytes() {
        Ok(bytes) => {
            println!(
                "   ✓ Original transaction serialized: {} bytes",
                bytes.len()
            );

            // Deserialize it back as AnyTransaction
            match AnyTransaction::from_bytes(&bytes) {
                Ok(deserialized_tx) => {
                    println!("   ✓ Transaction deserialized successfully!");
                    println!(
                        "   ✓ Transaction ID present: {}",
                        deserialized_tx.get_transaction_id().is_some()
                    );

                    // Serialize again to verify
                    match deserialized_tx.to_bytes() {
                        Ok(bytes2) => {
                            println!("   ✓ Re-serialized: {} bytes", bytes2.len());
                            println!("   ✓ Bytes match original: {}", bytes == bytes2);
                        }
                        Err(e) => println!("   ✗ Re-serialization error: {:?}", e),
                    }
                }
                Err(e) => println!("   ✗ Deserialization error: {:?}", e),
            }
        }
        Err(e) => println!("   ✗ Serialization error: {:?}", e),
    }

    println!("\n=== Summary ===");
    println!("✓ WASM-compatible transaction building works!");
    println!("✓ Transactions can be serialized to bytes");
    println!("✓ Bytes can be sent to JavaScript for signing and submission");
    println!("\nNext steps:");
    println!("  1. Compile to WASM with: cargo build --target wasm32-unknown-unknown");
    println!("  2. Use wasm-bindgen to create JavaScript bindings");
    println!("  3. Send transaction bytes to hedera-sdk-js for signing");
    println!("  4. Submit signed transactions to the Hedera network");
}

/// Helper function to display hex preview of bytes
fn hex_preview(bytes: &[u8], max_len: usize) -> String {
    let preview_bytes = if bytes.len() > max_len {
        &bytes[..max_len]
    } else {
        bytes
    };

    preview_bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ")
        + if bytes.len() > max_len { " ..." } else { "" }
}
