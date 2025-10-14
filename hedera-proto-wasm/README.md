# Hedera Proto WASM

Complete Hedera protobuf definitions compiled for WebAssembly with JavaScript bindings. This crate provides **ALL** Hedera protobuf types and transaction capabilities for WASM environments, enabling full Hedera transaction construction and serialization in web browsers.

## What This Provides

🔥 **Complete Protobuf API**: All 194+ Hedera `.proto` files compiled with `prost-build`  
🌐 **JavaScript Ready**: Full `wasm-bindgen` integration for seamless web usage  
⚡ **Transaction Builder**: High-level API for common transaction types  
🎯 **Type Safety**: Full Rust type safety with protobuf validation  
📦 **Zero Network Dependencies**: Pure transaction construction, no `mio`/`tokio`  

## Architecture

```
┌─────────────────────┐    ┌─────────────────────┐
│   Native Hedera SDK │    │  hedera-proto-wasm  │
│                     │    │                     │
├─────────────────────┤    ├─────────────────────┤
│ ❌ mio/tokio deps   │    │ ✅ WASM compatible   │
│ ❌ No WASM support  │    │ ✅ All protobufs     │
│ ✅ Full gRPC client │    │ ✅ JavaScript API    │
│ ✅ Network features │    │ ✅ Pure serialization│
└─────────────────────┘    └─────────────────────┘
```

## Generated Protobuf Types

This crate includes **complete** protobuf definitions for:

- **Services**: All transaction types (crypto, token, contract, etc.)
- **Basic Types**: AccountId, TokenId, Timestamp, Duration, etc.  
- **Transaction Structure**: TransactionBody, Transaction, SignatureMap
- **Query Types**: All query request/response types
- **Stream Types**: Record stream and mirror node types
- **Custom Types**: All Hedera-specific enums and structures

## JavaScript Usage

### Installation & Setup

```bash
# Build for web
wasm-pack build --target web

# Use in your web project
```

```html
<!-- In your HTML -->
<script type="module">
import init, { HederaTransactionBuilder } from './pkg/hedera_proto_wasm.js';

async function main() {
    await init(); // Initialize WASM module
    
    // Ready to use!
    const builder = new HederaTransactionBuilder(
        0, 0, 123456,  // payer: shard.realm.account  
        0, 0, 3,       // node: shard.realm.account
        100000         // fee in tinybars
    );
    
    // Create a 500 tinybar transfer
    const bodyBytes = builder.create_crypto_transfer(
        0, 0, 789012,  // receiver account
        500            // amount in tinybars  
    );
    
    console.log(`Transaction body: ${bodyBytes.length} bytes`);
    
    // Sign with your preferred library
    const signature = await yourSigningFunction(bodyBytes);
    const publicKeyBytes = new Uint8Array([/* your public key */]);
    
    // Create final signed transaction
    const signedTx = builder.create_signed_transaction(
        bodyBytes, 
        signature, 
        publicKeyBytes
    );
    
    // Submit via HTTP to Hedera
    await submitToHedera(signedTx);
}

main();
</script>
```

### Transaction Builder API

```javascript
// Constructor
const builder = new HederaTransactionBuilder(
    payerShard, payerRealm, payerAccount,
    nodeShard, nodeRealm, nodeAccount, 
    transactionFee
);

// Methods
const bodyBytes = builder.create_crypto_transfer(
    receiverShard, receiverRealm, receiverAccount, 
    amountTinybars
);

const signedTx = builder.create_signed_transaction(
    bodyBytes,        // Uint8Array from above
    signature,        // Uint8Array signature  
    publicKeyPrefix   // Uint8Array public key
);

// Utilities
const timestamp = get_current_timestamp_seconds();
```

## Rust Usage

### Direct Protobuf Access

```rust
use hedera_proto_wasm::*;

// Create account ID using generated protobuf types
let account = AccountId {
    shard_num: 0,
    realm_num: 0, 
    account: Some(proto::proto::account_id::Account::AccountNum(123456)),
};

// Create transaction ID
let tx_id = TransactionId {
    account_id: Some(account.clone()),
    transaction_valid_start: Some(Timestamp {
        seconds: 1640995200,
        nanos: 0,
    }),
    scheduled: false,
    nonce: 0,
};

// Build complete transaction body
let tx_body = TransactionBody {
    transaction_id: Some(tx_id),
    node_account_id: Some(AccountId { /* node account */ }),
    transaction_fee: 100_000,
    transaction_valid_duration: Some(Duration { seconds: 180 }),
    generate_record: false,
    memo: String::new(),
    data: Some(transaction_body::Data::CryptoTransfer(
        CryptoTransferTransactionBody {
            transfers: Some(TransferList {
                account_amounts: vec![
                    AccountAmount {
                        account_id: Some(/* payer */),
                        amount: -500,  // sending
                        is_approval: false,
                    },
                    AccountAmount {
                        account_id: Some(/* receiver */),
                        amount: 500,   // receiving  
                        is_approval: false,
                    },
                ],
            }),
            token_transfers: vec![],
        }
    )),
    // ... other fields
};

// Serialize to bytes
use prost::Message;
let bytes = tx_body.encode_to_vec();
```

### High-Level Builder (WASM + Native)

```rust
#[cfg(target_arch = "wasm32")]
use hedera_proto_wasm::HederaTransactionBuilder;

let builder = HederaTransactionBuilder::new(
    0, 0, 123456,  // payer
    0, 0, 3,       // node  
    100_000        // fee
);

let body_bytes = builder.create_crypto_transfer(0, 0, 789012, 500);
```

## Building

### Prerequisites

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Ensure WASM target is installed
rustup target add wasm32-unknown-unknown
```

### Build Commands

```bash
# For JavaScript/Browser use
wasm-pack build --target web
# Outputs to pkg/ directory

# For Node.js use  
wasm-pack build --target nodejs

# For bundler use
wasm-pack build --target bundler

# Direct WASM build
cargo build --target wasm32-unknown-unknown

# Native Rust build (works too!)
cargo build
```

## Integration Examples

### React/Vue/Vanilla JS

```javascript
// transaction-utils.js
import init, { HederaTransactionBuilder } from './pkg/hedera_proto_wasm.js';

let wasmInitialized = false;

export async function initHedera() {
    if (!wasmInitialized) {
        await init();
        wasmInitialized = true;
    }
}

export function createTransfer(fromAccount, toAccount, amount, fee = 100000) {
    const builder = new HederaTransactionBuilder(
        ...fromAccount,  // [shard, realm, account]
        0, 0, 3,         // default node
        fee
    );
    
    return builder.create_crypto_transfer(...toAccount, amount);
}
```

### Webpack Integration

```javascript
// webpack.config.js
module.exports = {
    experiments: {
        asyncWebAssembly: true,
    },
    // ... other config
};
```

### Vite Integration

```javascript
// vite.config.js
export default {
    server: {
        fs: {
            allow: ['..']  // Allow loading WASM from pkg/
        }
    }
}
```

## Performance

- **Build Time**: ~12 seconds (194 protobuf files)
- **WASM Size**: ~1-2MB (depending on optimization)  
- **Runtime**: Near-native performance for transaction construction
- **Memory**: Minimal heap usage, stack-based operations

## Use Cases Unlocked

✅ **Browser Wallets**: Client-side transaction construction  
✅ **DApps**: Direct Hedera integration without server  
✅ **Mobile Apps**: Using WebView with WASM performance  
✅ **Serverless**: Edge functions with instant cold starts  
✅ **Security**: No network code, pure transaction building  
✅ **Offline**: Complete transaction creation without connectivity  

## Comparison

| Feature | Native SDK | hedera-proto-wasm |
|---------|------------|-------------------|
| WASM Support | ❌ No | ✅ Full |
| Protobuf Types | ✅ All | ✅ All |  
| Transaction Building | ✅ Yes | ✅ Yes |
| Network Requests | ✅ gRPC | ❌ Bring your own |
| Bundle Size | 🔴 Large | 🟢 Small |
| Dependencies | 🔴 Many | 🟢 Minimal |

## Limitations

- **No Network Layer**: You handle HTTP/gRPC-web submission
- **No Query Support**: Focus on transaction construction only
- **Signature External**: Use your preferred crypto library
- **WASM Bundle**: Adds ~1-2MB to your web app

## Development

```bash
# Run tests
cargo test

# Check formatting  
cargo fmt --check

# Lint
cargo clippy

# See generated protobufs
find target/wasm32-unknown-unknown/debug/build/*/out/ -name "*.rs" | head -5
```

## License

Apache-2.0 - Same as Hedera Rust SDK 