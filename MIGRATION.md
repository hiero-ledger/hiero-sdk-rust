# Migration Guide: hedera → hiero

As part of the transition from the Hedera SDK to the new Hiero ecosystem, this Rust SDK is now published under a new crate name:

-   ✅ **hiero** is the new crate name.
-   ⚠️ **hedera** is still maintained temporarily for backward compatibility, but will be deprecated in the future.

We encourage all users to migrate to **hiero** to receive future updates, features, and bug fixes.

## 🛠 How to Migrate

### 1. Update `Cargo.toml`

Change your dependency from:

```toml
hedera = "0.14"
```

To:

```toml
hiero = "0.14"
```

Make sure to run `cargo build` or `cargo update` to apply the change.

### 2. Update Imports

If you're using:

```rust
use hedera::{
    Client, AccountId, PrivateKey, TopicCreateTransaction, TopicMessageQuery, TopicMessageSubmitTransaction,
};
```

Change it to:

```rust
use hiero::{
    Client, AccountId, PrivateKey, TopicCreateTransaction, TopicMessageQuery, TopicMessageSubmitTransaction,
};
```

Most APIs and types remain identical between `hedera` and `hiero`.

## 📦 Crate Availability

| Crate  | Status    | Description                                |
| ------ | --------- | ------------------------------------------ |
| hedera | ✅ Legacy | Still works, but will be deprecated soon   |
| hiero  | ✅ Active | New name, actively maintained and improved |

We are currently dual-publishing new versions to both crates, but this will change in a future release where `hedera` will be marked as deprecated.

## ❓ Why the Change?

`hiero` represents the next generation of decentralized, open, and extensible infrastructure, building on the lessons and foundations of the Hedera SDK while evolving toward greater community ownership and flexibility.

The crate rename reflects this philosophical and architectural shift.

## 📅 Deprecation Timeline

We will continue to publish updates to both `hedera` and `hiero` for a short transition period. After that:

-   Final `hedera` version will be published
-   Crate will be marked as deprecated on crates.io
-   All new development will occur under `hiero`

## 🗣 Support

If you have questions or issues migrating:

-   Open an issue
-   Start a discussion
-   Visit our community Discord (link coming soon)

Thanks for growing with us, and welcome to Hiero. 🪐
