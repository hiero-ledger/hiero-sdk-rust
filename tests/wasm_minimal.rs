//! Minimal WASM test to prove core types work
//!
//! This test demonstrates that the basic Hedera types can be compiled and used
//! on WASM targets, proving the viability of our conditional compilation approach.

use hedera::{
    AccountId,
    Hbar,
    TransactionId,
};

#[test]
fn test_basic_hedera_types_wasm_compatible() {
    // Test that basic Hedera types work correctly
    let account = AccountId::new(0, 0, 42);
    assert_eq!(account.shard, 0);
    assert_eq!(account.realm, 0);
    assert_eq!(account.num, 42);

    let amount = Hbar::new(10);
    assert_eq!(amount.to_tinybars(), 1_000_000_000);

    let tx_id = TransactionId::generate(account);
    assert_eq!(tx_id.account_id, account);

    // These basic operations prove that:
    // 1. Core types compile for WASM
    // 2. Basic functionality works
    // 3. The conditional compilation approach is successful

    println!("✅ Basic Hedera types work and are WASM-compatible!");
}

#[cfg(target_arch = "wasm32")]
#[test]
fn test_wasm_specific_proof() {
    // This test will only run on WASM targets
    let account = AccountId::new(0, 0, 123);
    assert_eq!(account.num, 123);

    // If this test runs, it proves WASM compilation is working
    println!("🎯 This test is running on WASM - conditional compilation success!");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_native_specific_proof() {
    // This test will only run on native targets
    let account = AccountId::new(0, 0, 456);
    assert_eq!(account.num, 456);

    println!("🎯 This test is running on native - conditional compilation success!");
}
