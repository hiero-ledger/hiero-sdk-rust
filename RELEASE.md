# Release process for Rust SDK

## Setup
1. Login to your *crates.io* account: `cargo login`
>- Prior authorization must be given by a maintainer of the crate

## Release

In the main directory, use the following steps to publish to [hedera](https://crates.io/crates/hedera): 

1. Create a new git branch: `prepare-release-vX.Y.Z`.
2. Run all tests against [hiero-local-node](https://github.com/hiero-ledger/hiero-local-node). Stop local-node once the tests are completed.
>- `cargo test`
3. Change the version number in *Cargo.toml*.
>- `version = major.minor.patch`
>- Follows [semver 2.0](https://semver.org/spec/v2.0.0.html)
4. Generate the `Cargo.lock` file.
>- `cargo generate-lockfile`
5. Before merging to main, run a `--dry-run` publish to check for warnings or errors.
>- `cargo publish --dry-run`
6. Merge the `prepare-release-vX.Y.Z` branch into `main`.
>- *Pull Request* must be created and approved by a maintainer.
7. Check out the latest version of the `main` branch.
>- `git checkout main`
8. Tag the `main` branch with the new version number.
>- `git tag -s vX.Y.Z`
9. Push the tag to the remote repository.
>- `git push -u origin vX.Y.Z`
10. The `Publish Release` workflow will trigger on the push of the tag (`vX.Y.Z`).
>- This will publish the `hedera`, `hedera-proto`, `hiero-sdk`, and `hiero-sdk-proto` crates to *crates.io*. and create a release on GitHub.
>- [Tags and Releases for Rust SDK](https://github.com/hiero-ledger/hiero-sdk-rust/releases)

**Note** 
- The `Publish Release` workflow will only publish the crates if they have not been published before.