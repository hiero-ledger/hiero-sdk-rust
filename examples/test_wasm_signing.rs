use hedera::{AccountId, Hbar, PrivateKey, TransactionId, TransferTransaction};

fn main() {
    println!("=== Testing WASM Freeze & Sign ===\n");

    // Create a transaction
    let mut tx = TransferTransaction::new();
    tx.hbar_transfer(AccountId::new(0, 0, 1001), Hbar::new(-10))
        .hbar_transfer(AccountId::new(0, 0, 1002), Hbar::new(10))
        .transaction_id(TransactionId::generate(AccountId::new(0, 0, 1001)))
        .node_account_ids([AccountId::new(0, 0, 3)]);

    println!("1. Testing freeze():");
    match tx.freeze() {
        Ok(_) => println!("   ✓ Freeze works!"),
        Err(e) => println!("   ✗ Freeze failed: {:?}", e),
    }

    println!("\n2. Testing sign():");
    let private_key = PrivateKey::generate_ed25519();
    tx.sign(private_key);
    println!("   ✓ Sign works!");

    println!("\n3. Testing serialization after signing:");
    match tx.to_bytes() {
        Ok(bytes) => println!("   ✓ Signed transaction serialized: {} bytes", bytes.len()),
        Err(e) => println!("   ✗ Serialization failed: {:?}", e),
    }
}
