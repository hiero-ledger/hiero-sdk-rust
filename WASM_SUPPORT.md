# WASM Support for Hedera Rust SDK

## Overview

The Hedera Rust SDK now supports **WebAssembly (WASM)** compilation for transaction building and serialization! This enables JavaScript applications to leverage Rust's type safety and performance for constructing Hedera transactions.

## ✅ What Works in WASM

### Transaction Building
- ✅ All transaction types can be built (Transfer, FileCreate, TokenCreate, etc.)
- ✅ Transaction IDs can be generated
- ✅ Transaction parameters can be set (memo, fees, node IDs, etc.)
- ✅ **Transactions can be frozen** (`freeze()`)
- ✅ **Transactions can be signed** (`sign()`, `sign_with()`)
- ✅ Transactions can be serialized to bytes (`to_bytes()`)
- ✅ Transactions can be deserialized from bytes (`from_bytes()` via `AnyTransaction`)

### Core Types
- ✅ `AccountId`, `FileId`, `TokenId`, `TopicId`, etc.
- ✅ `Hbar` and amount types
- ✅ `TransactionId` generation
- ✅ `PrivateKey` and `PublicKey` (generation and signing)
- ✅ `AnyTransaction` for type-erased transactions
- ✅ All transaction data types

### Protobuf Serialization
- ✅ Protobuf encoding/decoding using `prost` (via `hedera-proto-wasm`)
- ✅ Transaction body serialization
- ✅ Compatible with Hedera network format

## ❌ What's Not Available in WASM

These features require networking or execution context and are **native-only**:

- ❌ `Client` (network communication)
- ❌ Transaction execution (`.execute()`)
- ❌ Query execution
- ❌ Receipt/Record retrieval
- ❌ Transaction signing **with operator** (`.sign_with_operator()` - requires Client)
- ❌ Transaction freezing **with client** (`.freeze_with(client)` - requires Client)
- ❌ Chunked transaction execution (FileAppend, TopicMessageSubmit)
- ❌ Mirror node queries
- ❌ Flow operations (high-level orchestration)

## 🚀 Use Cases

### Use Case 1: Full Transaction Building & Signing in WASM

**WASM can now handle the complete transaction lifecycle except execution!**

```rust
// In WASM:
let mut tx = TransferTransaction::new();
tx.hbar_transfer(from, -amount)
  .hbar_transfer(to, amount)
  .transaction_id(TransactionId::generate(from))
  .node_account_ids([node_id])
  .freeze()?;  // ✅ Works in WASM!

let private_key = PrivateKey::from_str("...")?;
tx.sign(private_key);  // ✅ Works in WASM!

let signed_bytes = tx.to_bytes()?;
// Send to JavaScript for execution only
```

### Use Case 2: Transaction Building (JavaScript Signs)

Alternatively, **build in WASM and sign in JavaScript**:

```
┌──────────────────────────┐         ┌─────────────────┐
│      Rust/WASM           │         │  Hedera Network │
│                          │         │                 │
│ Build TX → Freeze → Sign │────────▶│ Execute TX      │
│ to_bytes()               │ bytes   │ (via JS client) │
└──────────────────────────┘         └─────────────────┘
```

Or separate signing:

```
┌─────────────────┐         ┌──────────────────┐         ┌─────────────────┐
│   Rust/WASM     │         │   JavaScript     │         │  Hedera Network │
│                 │         │                  │         │                 │
│ Build & Freeze  │────────▶│ Sign TX          │────────▶│ Execute TX      │
│ to_bytes()      │ bytes   │ (hedera-sdk-js)  │ signed  │                 │
└─────────────────┘         └──────────────────┘         └─────────────────┘
```

### Workflow Options

#### Option A: Full WASM (Build + Sign)
```rust
// WASM does everything except execution
let mut tx = TransferTransaction::new();
tx.hbar_transfer(from, -amount)
  .hbar_transfer(to, amount)
  .transaction_id(TransactionId::generate(from))
  .node_account_ids([node])
  .freeze()?;

let key = PrivateKey::from_str(private_key_hex)?;
tx.sign(key);

let signed_bytes = tx.to_bytes()?;
// Send to JavaScript for execution only
```

```javascript
// JavaScript just executes the pre-signed transaction
const signedTxBytes = wasmModule.buildAndSignTransaction(...);
const tx = Transaction.fromBytes(signedTxBytes);
const response = await tx.execute(client);
```

#### Option B: WASM Build, JavaScript Sign
```rust
// WASM builds and freezes
let mut tx = TransferTransaction::new();
tx.hbar_transfer(from, -amount)
  .hbar_transfer(to, amount)
  .transaction_id(TransactionId::generate(from))
  .node_account_ids([node])
  .freeze()?;
  
let bytes = tx.to_bytes()?;
```

```javascript
// JavaScript signs and executes
const txBytes = wasmModule.buildTransaction(...);
const tx = Transaction.fromBytes(txBytes);
const signedTx = await tx.sign(privateKey);
const response = await signedTx.execute(client);
```

## 📦 Building for WASM

### Prerequisites
```bash
rustup target add wasm32-unknown-unknown
```

### Build
```bash
cargo build --target wasm32-unknown-unknown --release
```

### With wasm-bindgen
```bash
cargo install wasm-bindgen-cli
wasm-bindgen target/wasm32-unknown-unknown/release/hedera.wasm \
  --out-dir ./pkg \
  --target web
```

## 🧪 Testing & Examples

### Run the Demos

#### Basic Transaction Building
```bash
cargo run --example wasm_transaction_demo
```

This demonstrates:
- Building various transaction types
- Serializing to bytes
- Round-trip serialization/deserialization
- Byte inspection

#### Freeze & Sign Demo
```bash
cargo run --example test_wasm_signing
```

This demonstrates:
- ✅ Transaction freezing without client
- ✅ Transaction signing with PrivateKey
- ✅ Serialization of signed transactions

### Expected Output
```
=== Hedera WASM Transaction Building Demo ===

1. Building a Transfer Transaction:
   ✓ Transaction serialized successfully!
   ✓ Byte length: 173 bytes
   ✓ First 20 bytes (hex): 0a aa 01 1a 00 22 50 ...

2. Building a File Create Transaction:
   ✓ Transaction serialized successfully!
   ✓ Byte length: 177 bytes
   ...

✓ WASM-compatible transaction building works!
✓ Transactions can be serialized to bytes
✓ Bytes can be sent to JavaScript for signing and submission
```

## 🔧 Technical Implementation

### Conditional Compilation
The SDK uses `#[cfg(target_arch = "wasm32")]` to conditionally compile:
- Native: Full SDK with networking (tonic, gRPC)
- WASM: Transaction building only (no networking)

### Protobuf Handling
- **Native**: Uses `hedera-proto` with `tonic` for gRPC
- **WASM**: Uses `hedera-proto-wasm` with `prost` only (no tonic dependency)

### Key Architecture Decisions
1. **`TransactionData` trait**: Available for both, defines transaction building
2. **`TransactionExecute` trait**: Native-only, defines networking behavior
3. **`AnyTransaction`**: Available for both, enables type erasure
4. **`ChunkInfo`**: Simplified for WASM (metadata only, no execution)
5. **Error handling**: Full error types available in both targets

## 📝 Known Limitations

1. **Chunking**: FileAppend and TopicMessageSubmit cannot execute multi-chunk transactions in WASM (serialization works for single chunks)
2. **Operator signing**: `.sign_with_operator()` requires Client, not available in WASM (but `.sign()` works!)
3. **Freeze with client**: `.freeze_with(client)` requires Client, not available in WASM (but `.freeze()` works!)
4. **Checksum validation**: Disabled in WASM (returns `Ok(())`)
5. **Prng transactions**: Not available in WASM
6. **Batch transactions**: Not available in WASM

## 🎯 Unlocked Use Cases

With WASM support, you can now:

1. **Complete transaction lifecycle in WASM** (build, freeze, sign, serialize) - only execution needs JavaScript
2. **Type-safe transaction building** in JavaScript applications
3. **Secure private key handling** entirely in WASM (keys never touch JavaScript)
4. **Leverage Rust's performance** for complex transaction construction
5. **Share transaction logic** between Rust backends and JavaScript frontends
6. **Build browser-based** Hedera applications with Rust
7. **Create mobile apps** using React Native + WASM
8. **Offline transaction signing** - prepare and sign transactions without network access

## 🔮 Future Improvements

Potential enhancements:
- [ ] wasm-bindgen bindings for direct JavaScript integration
- [ ] Web Crypto API integration for WASM signing
- [ ] IndexedDB storage for WASM environments
- [ ] More comprehensive WASM examples
- [ ] Performance benchmarks (WASM vs native)

## 📚 Related Resources

- [WebAssembly Official Site](https://webassembly.org/)
- [wasm-bindgen Documentation](https://rustwasm.github.io/wasm-bindgen/)
- [Hedera Documentation](https://docs.hedera.com/)
- [hedera-sdk-js](https://github.com/hashgraph/hedera-sdk-js)

## 🙏 Contributing

Contributions to improve WASM support are welcome! Areas of interest:
- JavaScript integration examples
- Browser-based demos
- Performance optimizations
- Additional transaction types
- Documentation improvements

---

**Note**: WASM support focuses on **transaction building and serialization**. For full Hedera network interaction (signing, execution, queries), use the native Rust SDK or integrate with hedera-sdk-js for the networking layer.

