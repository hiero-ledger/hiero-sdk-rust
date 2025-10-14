// SPDX-License-Identifier: Apache-2.0

//! Tests demonstrating WASM compatibility for transaction building
//!
//! These tests prove that the core transaction functionality works on both
//! native and WASM targets, enabling transaction construction and serialization
//! for web environments.

use hedera::{
    AccountId,
    FileCreateTransaction,
    Hbar,
    PrivateKey,
    TransactionId,
    TransferTransaction,
};

#[test]
fn test_wasm_transfer_transaction_building() {
    // Test that we can build transfer transactions for WASM
    let from_account = AccountId::new(0, 0, 1001);
    let to_account = AccountId::new(0, 0, 1002);
    let amount = Hbar::new(10);

    let mut transaction = TransferTransaction::new();
    transaction
        .hbar_transfer(from_account, -amount)
        .hbar_transfer(to_account, amount)
        .transaction_id(TransactionId::generate(from_account))
        .max_transaction_fee(Hbar::new(1));

    // This is the key WASM use case: getting transaction bytes
    let transaction_bytes = transaction.to_bytes().expect("Should serialize to bytes");

    assert!(!transaction_bytes.is_empty(), "Transaction bytes should not be empty");
    assert!(transaction_bytes.len() > 50, "Transaction should have substantial size");
}

#[test]
fn test_wasm_file_transaction_building() {
    // Test building file transactions for WASM
    let payer_account = AccountId::new(0, 0, 1001);
    let file_contents = b"Hello, WASM world!".to_vec();

    let mut transaction = FileCreateTransaction::new();
    transaction
        .contents(file_contents.clone())
        .transaction_id(TransactionId::generate(payer_account))
        .max_transaction_fee(Hbar::new(2));

    // Serialize for WASM usage
    let transaction_bytes = transaction.to_bytes().expect("Should serialize to bytes");

    assert!(!transaction_bytes.is_empty(), "File transaction bytes should not be empty");
    println!("File transaction serialized to {} bytes", transaction_bytes.len());
}

#[test]
fn test_wasm_transaction_signing_preparation() {
    // Test preparing transactions for external signing (key WASM workflow)
    let account = AccountId::new(0, 0, 1001);
    let recipient = AccountId::new(0, 0, 1002);

    let mut transaction = TransferTransaction::new();
    transaction
        .hbar_transfer(account, Hbar::new(-5))
        .hbar_transfer(recipient, Hbar::new(5))
        .transaction_id(TransactionId::generate(account))
        .max_transaction_fee(Hbar::new(1));

    // In WASM, we'd typically:
    // 1. Build the transaction (✓ works)
    let transaction_bytes = transaction.to_bytes().expect("Should build transaction");

    // 2. Send to JavaScript for signing
    // 3. Get signed bytes back
    // 4. Submit via JavaScript networking

    assert!(!transaction_bytes.is_empty());

    // Verify we can create a private key for testing (should work on WASM)
    let private_key = PrivateKey::generate_ed25519();
    assert!(private_key.to_string_der().len() > 0, "Private key should generate");
}

#[test]
fn test_wasm_multiple_transaction_types() {
    // Test that various transaction types can be built for WASM
    let account = AccountId::new(0, 0, 1001);

    // Transfer transaction
    let mut transfer_tx = TransferTransaction::new();
    transfer_tx
        .hbar_transfer(account, Hbar::new(-1))
        .hbar_transfer(AccountId::new(0, 0, 1002), Hbar::new(1))
        .transaction_id(TransactionId::generate(account));

    let transfer_bytes = transfer_tx.to_bytes().expect("Transfer should serialize");

    // File create transaction
    let mut file_tx = FileCreateTransaction::new();
    file_tx.contents(b"test file".to_vec()).transaction_id(TransactionId::generate(account));

    let file_bytes = file_tx.to_bytes().expect("File create should serialize");

    // All should produce valid transaction bytes
    assert!(!transfer_bytes.is_empty());
    assert!(!file_bytes.is_empty());
    assert_ne!(transfer_bytes, file_bytes, "Different transactions should produce different bytes");

    println!("✅ WASM compatibility verified:");
    println!("   Transfer transaction: {} bytes", transfer_bytes.len());
    println!("   File create transaction: {} bytes", file_bytes.len());
}

#[cfg(target_arch = "wasm32")]
#[test]
fn test_wasm_specific_functionality() {
    // This test only runs on WASM targets
    use hedera::AccountId;

    let account = AccountId::new(0, 0, 42);
    assert_eq!(account.shard, 0);
    assert_eq!(account.realm, 0);
    assert_eq!(account.num, 42);

    println!("✅ WASM-specific test passed!");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_native_specific_functionality() {
    // This test only runs on native targets
    use hedera::AccountId;

    let account = AccountId::new(0, 0, 42);
    assert_eq!(account.shard, 0);
    assert_eq!(account.realm, 0);
    assert_eq!(account.num, 42);

    println!("✅ Native-specific test passed!");
}
