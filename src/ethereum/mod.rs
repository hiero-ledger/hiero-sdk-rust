// SPDX-License-Identifier: Apache-2.0

mod ethereum_data;
#[cfg(not(target_arch = "wasm32"))] // Ethereum flows require client networking
mod ethereum_flow;
mod ethereum_transaction;
mod evm_address;

pub use ethereum_data::{
    Eip1559EthereumData,
    EthereumData,
    LegacyEthereumData,
};
#[cfg(not(target_arch = "wasm32"))]
pub use ethereum_flow::EthereumFlow;
pub use ethereum_transaction::EthereumTransaction;
pub(crate) use ethereum_transaction::EthereumTransactionData;
pub use evm_address::EvmAddress;
pub(crate) use evm_address::SolidityAddress;
