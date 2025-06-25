# Release process for Rust SDK

## Setup
1. Login to your *crates.io* account: `cargo login`
>- Prior authorization must be given by a maintainer of the crate

## Release

In the main directory, use the the following steps to publish to [hedera](https://crates.io/crates/hedera): 

1. Create a new git branch: `release/vX.Y.Z`.
2. Run all tests against [hiero-local-node](https://github.com/hiero-ledger/hiero-local-node). Stop local-node once the tests are completed.
>- `cargo test`
3. Change the version number in *Cargo.toml*.
>- `version = major.minor.patch`
>- Follows [semver 2.0](https://semver.org/spec/v2.0.0.html)
4. Before publishing, run `--dry-run` to check for warnings or errors.
>- `cargo publish --dry-run`
5. If all warnings and error are resolved, publish the newest version to *crates.io*.
>- `cargo publish`
6. Once branch has been approved and merged to main, document added features pertaining to the newest release.
7. Create the new tag in Github `vX.Y.Z`
>- [Tags and Releases for Rust SDK](https://github.com/hiero-ledger/hiero-sdk-rust/releases)

**Note** 
- If there are new local changes to [`hedera-proto`](https://crates.io/crates/hedera-proto) crate, this must be published before `hedera` crate is published for each release. 
Cargo will prevent publishing if this is the case.