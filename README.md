# Hiero™ Rust SDK

> The SDK for interacting with Hiero: the official distributed
> consensus platform built using the Hashgraph consensus algorithm for fast,
> fair and secure transactions. Hiero enables and empowers developers to
> build an entirely new class of decentralized applications.

<sub>Maintained with ❤️ by <a href="https://launchbadge.com" target="_blank">LaunchBadge</a>, <a href="https://www.swirlds.com/" target="_blank">Swirlds Labs</a>, and the Hiero community</sub>

## Requirements

- [Rust](https://rustup.rs)
- [protoc](https://grpc.io/docs/protoc-installation)
- [OpenSSL](https://www.openssl.org/)

Clone this repository and its submodules:

```bash
git clone --recursive https://github.com/hiero-ledger/hiero-sdk-rust.git

```

Update [`\protobufs`](https://github.com/hiero-ledger/hiero-consensus-node.git) submodule to recent version (if necessary):

```bash
git submodule update --recursive --remote
```

To check dependencies validity, run the following command in the root directory:

```bash
cargo check
```

## API Docs

Check out the [Hiero Rust SDK API reference docs](http://docs.rs/hedera/latest/hedera/index.html).

## Community and Support

If you have any questions on the Hiero SDK or Hiero more generally, you can join our team and hundreds of other developers using Hedera in our community Discord:

<a href="https://hedera.com/discord" target="_blank">
  <img alt="" src="https://user-images.githubusercontent.com/753919/167244200-b95cd3a6-6256-4eaf-b9b4-f1f192341485.png" height="60">
</a>

### Examples

To the run an example, set an operator id and operator key in the `.env` file. By default, these run against testnet.

```bash
# Account that will pay query and transaction fees.
OPERATOR_ID=
# Default private key to use to sign for all transactions and queries.
OPERATOR_KEY=
```

Cargo Command:
```bash
cargo run --release --example <EXAMPLE_FILENAME>
```

Create Account Example: 
```bash
cargo run --release --example create_account
```


### Tests

Before running the integration tests, an operator key, operator id, and a network name must be set in an `.env` file.

```bash
# Account that will pay query and transaction fees.
TEST_OPERATOR_ID=
# Default private key to use to sign for all transactions and queries.
TEST_OPERATOR_KEY=
# Network names: "localhost", "testnet", "previewnet", "mainnet".
TEST_NETWORK_NAME=
# Running on-chain tests if this value is set to 1.
TEST_RUN_NONFREE=
```

Run the tests using `cargo test`

```bash
# Run all tests (unit and integration tests)
cargo test

# Run specific tests in a module
cargo test <MODULE_NAME>::<FILE_NAME>::<FUNCTION_NAME>
```

e.g.
```bash
# Run account module tests
cargo test account
# Run account module tests in file
cargo test account::create
# Run a single test
cargo test account::create::initial_balance_and_key
```

#### Local Environment Testing

Hiero offers a way to run tests through your localhost using the `hiero-local-node` service.

For instructions on how to set up and run local node, follow the steps in the git repository:
<https://github.com/hiero-ledger/hiero-local-node>

Once the local node is running in Docker, use these environment variables in the `.env`.

```bash
# Account that will pay query and transaction fees.
TEST_OPERATOR_ID=0.0.2
# Default private key to use to sign for all transactions and queries.
TEST_OPERATOR_KEY=302e020100300506032b65700422042091132178e72057a1d7528025956fe39b0b847f200ab59b2fdd367017f3087137
# Network names: "localhost", "testnet", "previewnet", "mainnet".
TEST_NETWORK_NAME=localhost
# Running on-chain tests if this value is set to 1.
TEST_RUN_NONFREE=1
```

Lastly, run the tests using `cargo test`

## Contributing

Contributions are welcome. Please see the [contributing guide](https://github.com/hashgraph/.github/blob/main/CONTRIBUTING.md) to see how you can get involved.

## Code of Conduct

This project is governed by the [Contributor Covenant Code of Conduct](https://github.com/hashgraph/.github/blob/main/CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code of conduct. Please report unacceptable behavior to [oss@hedera.com](mailto:oss@hedera.com).

## License

[Apache License 2.0](LICENSE)
