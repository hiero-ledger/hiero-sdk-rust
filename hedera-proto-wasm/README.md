# Hedera Proto WASM

Minimal Hedera protobuf definitions for WASM transaction serialization with JavaScript bindings. This crate provides the essential protobuf structures needed to create and serialize Hedera transactions in WebAssembly environments.

## Problem Solved

The main Hedera Rust SDK doesn't work with WASM due to `mio` and `tokio` dependencies that don't compile for WASM targets. This crate solves that by providing a minimal, WASM-compatible implementation focused solely on transaction byte generation.

## Features

- ✅ **WASM Compatible**: Compiles to `wasm32-unknown-unknown` without issues
- ✅ **JavaScript Bindings**: Easy-to-use JavaScript API via wasm-bindgen
- ✅ **Transaction Serialization**: Generate proper Hedera transaction bytes
- ✅ **Minimal Dependencies**: Only uses `prost` for protobuf serialization
- ✅ **Real Timestamps**: Uses JavaScript Date API for accurate timestamps

## JavaScript Usage

After building with `wasm-pack build --target web`, you can use it in JavaScript:

```javascript
import init, { HederaTransactionBuilder, get_current_timestamp_seconds } from './pkg/hedera_proto_wasm.js';

async function createHederaTransaction() {
    // Initialize the WASM module
    await init();
    
    // Create a transaction builder
    const builder = new HederaTransactionBuilder(
        0, 0, 12345,  // payer account (shard, realm, account)
        0, 0, 3,      // node account  
        100000        // transaction fee in tinybars
    );
    
    // Create a crypto transfer transaction (sending 100 tinybars)
    const bodyBytes = builder.create_crypto_transfer(
        0, 0, 67890,  // receiver account
        100           // amount in tinybars
    );
    
    console.log(`Transaction body bytes: ${bodyBytes.length} bytes`);
    
    // Sign the bodyBytes with your crypto library
    const signature = await signBytes(bodyBytes); // Your signing function
    const publicKeyPrefix = new Uint8Array([0xed, 0x01, 0x20]); // Ed25519 prefix
    
    // Create complete signed transaction
    const signedTransaction = builder.create_signed_transaction(
        bodyBytes,
        signature,
        publicKeyPrefix
    );
    
    console.log(`Signed transaction: ${signedTransaction.length} bytes`);
    
    // Submit to Hedera network via HTTP/gRPC-web
    return signedTransaction;
}
```

## Rust Usage

You can also use it directly in Rust:

```rust
use hedera_proto_wasm::*;

// Create account IDs
let payer_account = AccountId::new(0, 0, 12345);
let receiver_account = AccountId::new(0, 0, 67890);
let node_account = AccountId::new(0, 0, 3);

// Create a transaction ID
let transaction_id = TransactionId::new(payer_account.clone());

// Create transfer amounts (sending 100 tinybars)
let transfers = vec![
    AccountAmount {
        account_id: Some(payer_account.clone()),
        amount: -100, // negative = sending
        is_approval: false,
    },
    AccountAmount {
        account_id: Some(receiver_account),
        amount: 100, // positive = receiving
        is_approval: false,
    },
];

// Create transaction body
let transaction_body = TransactionBody::new_crypto_transfer(
    transaction_id,
    node_account,
    100_000, // transaction fee in tinybars
    transfers,
);

// Serialize transaction body to bytes for signing
let body_bytes = transaction_body.to_bytes();
```

## Building

### For JavaScript/Web

```bash
# Install wasm-pack if you haven't already
cargo install wasm-pack

# Build for web
wasm-pack build --target web

# The generated files will be in pkg/
```

### For Rust

```bash
# Regular build
cargo build

# WASM build
cargo build --target wasm32-unknown-unknown
```

## Workflow

This crate enables the following workflow for WASM-based Hedera applications:

1. **Create Transaction**: Use `HederaTransactionBuilder` in JavaScript or structs in Rust
2. **Get Signing Bytes**: Call `create_crypto_transfer()` to get bytes that need signing
3. **Sign Externally**: Use your preferred crypto library (JavaScript or WASM) to sign
4. **Create Complete Transaction**: Call `create_signed_transaction()` with body + signature
5. **Submit via HTTP**: Use JavaScript `fetch()` to submit to Hedera via HTTP/gRPC-web

## JavaScript API Reference

### `HederaTransactionBuilder`

Constructor:
```javascript
new HederaTransactionBuilder(
    payer_shard, payer_realm, payer_account,
    node_shard, node_realm, node_account,
    transaction_fee
)
```

Methods:
- `create_crypto_transfer(receiver_shard, receiver_realm, receiver_account, amount)` → `Uint8Array`
- `create_signed_transaction(body_bytes, signature, public_key_prefix)` → `Uint8Array`

### Utility Functions

- `get_current_timestamp_seconds()` → `number` - Get current timestamp
- `log_to_console(message)` - Log to browser console from WASM

## Example Integration

```html
<!DOCTYPE html>
<html>
<head>
    <title>Hedera WASM Demo</title>
</head>
<body>
    <script type="module">
        import init, { HederaTransactionBuilder } from './pkg/hedera_proto_wasm.js';
        
        async function demo() {
            await init();
            
            const builder = new HederaTransactionBuilder(0, 0, 12345, 0, 0, 3, 100000);
            const bytes = builder.create_crypto_transfer(0, 0, 67890, 100);
            
            console.log('Transaction bytes ready for signing:', bytes);
        }
        
        demo();
    </script>
</body>
</html>
```

## Use Cases

This crate unlocks several important WASM use cases:

- Browser-based Hedera wallets that generate transaction bytes client-side
- WASM modules that create transactions for JavaScript applications  
- Serverless functions that need to generate Hedera transactions
- Any web application that needs to interact with Hedera without a full SDK
- Crypto signing workflows where transaction generation happens in WASM

## Performance Benefits

- **Fast**: WASM execution is near-native performance
- **Type-safe**: Rust's type system prevents many runtime errors
- **Small**: Minimal dependencies keep bundle size down
- **Secure**: No networking code reduces attack surface 