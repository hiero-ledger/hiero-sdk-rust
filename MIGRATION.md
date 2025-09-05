# Migration Guide: hedera → hiero-sdk

As part of the transition from the Hedera SDK to the new Hiero ecosystem, this Rust SDK will now be published under a new crate name:

-   ✅ **hiero-sdk** will be the new crate name.
-   ⚠️ **hedera** is still maintained temporarily for backward compatibility, but will be deprecated in the future.
-   ✅ **hiero-sdk-proto** will be the new crate name for the protobufs crate.
-   ⚠️ **hedera-proto** is still maintained temporarily for backward compatibility, but will be deprecated in the future.

We encourage all users to migrate to **hiero-sdk** and **hiero-sdk-proto** to receive future updates, features, and bug fixes.

## 🛠 How to Migrate

### 1. Update `Cargo.toml` (Manual)

Change your dependency from:

```toml
name = "hedera"
```

```toml
name = "hedera-proto"
```

To:

```toml
name = "hiero-sdk"
```

```toml
name = "hiero-sdk-proto"
```

Make sure to run `cargo build` or `cargo update` to apply the change. Keep in mind that right after the dual publishing
starts the namespace will still remain `hedera`. We will start publishing the `hiero-sdk` new SDK crates with the updated `.toml`
files.

### Import changes and other examples

If you're using the hedera crate:

```rust
use hedera::{
    Client, AccountId, PrivateKey, TopicCreateTransaction, TopicMessageQuery, TopicMessageSubmitTransaction,
};
```

Change it to:

```rust
use hiero_sdk::{
    Client, AccountId, PrivateKey, TopicCreateTransaction, TopicMessageQuery, TopicMessageSubmitTransaction,
};
```

If you're using the hedera-proto crate:
```rust
use hedera_proto::services::{
    self,
};
```

Change it to:

```rust
use hiero_sdk_proto::services::{
    self,
};
```

Most APIs and types will remain identical between `hedera` and `hiero-sdk` and between `hedera-proto` and `hiero-sdk-proto`. Keep in mind that right after the dual publishing
starts the namespace will still remain `hedera`. We will put up a notice when the namespace change will be mandatory.

### Update Transaction and Query Usage

If you previously used transactions or queries like this:

```rust
use hedera::{Client, AccountId, TransferTransaction};

let client = Client::for_testnet();
let account_id = AccountId::from_string("0.0.1234");
let tx = TransferTransaction::new()
    .add_hbar_transfer(account_id, 100)
    .add_hbar_transfer(AccountId::from_string("0.0.5678"), -100)
    .execute(&client)
    .await?;
```

Change to:

```rust
use hiero_sdk::{Client, AccountId, TransferTransaction};

let client = Client::for_testnet();
let account_id = AccountId::from_string("0.0.1234");
let tx = TransferTransaction::new()
    .add_hbar_transfer(account_id, 100)
    .add_hbar_transfer(AccountId::from_string("0.0.5678"), -100)
    .execute(&client)
    .await?;
```

### Updating Key Generation

Old:

```rust
use hedera::PrivateKey;

let private_key = PrivateKey::generate();
```

New:

```rust
use hiero_sdk::PrivateKey;

let private_key = PrivateKey::generate();
```

### Working with Topics (HCS)

Old:

```rust
use hedera::{TopicCreateTransaction, TopicMessageSubmitTransaction};

let topic = TopicCreateTransaction::new()
    .memo("My Topic")
    .execute(&client)
    .await?;
```

New:

```rust
use hiero_sdk::{TopicCreateTransaction, TopicMessageSubmitTransaction};

let topic = TopicCreateTransaction::new()
    .memo("My Topic")
    .execute(&client)
    .await?;
```

### Querying Account Balance

Old:

```rust
use hedera::{AccountBalanceQuery, AccountId};

let balance = AccountBalanceQuery::new()
    .account_id(AccountId::from_string("0.0.1234"))
    .execute(&client)
    .await?;
```

New:

```rust
use hiero_sdk::{AccountBalanceQuery, AccountId};

let balance = AccountBalanceQuery::new()
    .account_id(AccountId::from_string("0.0.1234"))
    .execute(&client)
    .await?;
```

### Error Handling

If you previously matched errors from `hedera`:

```rust
match result {
    Ok(value) => { /* ... */ }
    Err(hedera::Error::SomeError) => { /* ... */ }
    Err(e) => { /* ... */ }
}
```

Update to:

```rust
match result {
    Ok(value) => { /* ... */ }
    Err(hiero_sdk::Error::SomeError) => { /* ... */ }
    Err(e) => { /* ... */ }
}
```

## 📦 Crate Availability

| Crate           | Status   | Description                                 |
|-----------------|----------|---------------------------------------------|
| hedera          | ⌛ Legacy | Still works, but will be deprecated soon    |
| hiero-sdk       | ✅ Active | New name, actively maintained and improved  |
| hedera-proto    | ⌛ Legacy | Still works, but will be deprecated soon    |
| hiero-sdk-proto | ✅ Active | New name for protobufs, actively maintained |

We are currently starting dual-publishing new versions to both crates, but this will change in a future releases where `hedera` will be marked as deprecated.

## 📅 Deprecation Timeline

We will continue to publish updates to both `hedera` and `hiero-sdk` as well as to `hedera-proto` and `hiero-sdk-proto` for a short transition period. After that:

-   Final `hedera` version will be published
-   Crate will be marked as deprecated on crates.io
-   All new development will occur under `hiero-sdk`

## 🗣 Support

If you have questions or issues migrating:

-   Open an issue
-   Start a discussion
-   Visit our community Discord (link coming soon)

Thanks for growing with us, and welcome to Hiero. 🪐
