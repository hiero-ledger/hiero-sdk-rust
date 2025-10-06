// SPDX-License-Identifier: Apache-2.0

#[cfg(not(target_arch = "wasm32"))] // Bytecode query requires networking
mod contract_bytecode_query;
#[cfg(not(target_arch = "wasm32"))] // Call query requires networking
mod contract_call_query;
#[cfg(not(target_arch = "wasm32"))] // Contract create flow requires client networking
mod contract_create_flow;
mod contract_create_transaction;
mod contract_delete_transaction;
mod contract_execute_transaction;
mod contract_function_parameters;
mod contract_function_result;
mod contract_function_selector;
mod contract_id;
mod contract_info;
#[cfg(not(target_arch = "wasm32"))] // Info query requires networking
mod contract_info_query;
mod contract_log_info;
mod contract_nonce_info;
mod contract_update_transaction;
mod delegate_contract_id;

#[cfg(not(target_arch = "wasm32"))]
pub use contract_bytecode_query::ContractBytecodeQuery;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use contract_bytecode_query::ContractBytecodeQueryData;
#[cfg(not(target_arch = "wasm32"))]
pub use contract_call_query::ContractCallQuery;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use contract_call_query::ContractCallQueryData;
#[cfg(not(target_arch = "wasm32"))]
pub use contract_create_flow::ContractCreateFlow;
pub use contract_create_transaction::ContractCreateTransaction;
pub(crate) use contract_create_transaction::ContractCreateTransactionData;
pub use contract_delete_transaction::ContractDeleteTransaction;
pub(crate) use contract_delete_transaction::ContractDeleteTransactionData;
pub use contract_execute_transaction::ContractExecuteTransaction;
pub(crate) use contract_execute_transaction::ContractExecuteTransactionData;
pub use contract_function_parameters::ContractFunctionParameters;
pub use contract_function_result::ContractFunctionResult;
pub use contract_id::ContractId;
pub use contract_info::ContractInfo;
#[cfg(not(target_arch = "wasm32"))]
pub use contract_info_query::ContractInfoQuery;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use contract_info_query::ContractInfoQueryData;
pub use contract_log_info::ContractLogInfo;
pub use contract_nonce_info::ContractNonceInfo;
pub use contract_update_transaction::ContractUpdateTransaction;
pub(crate) use contract_update_transaction::ContractUpdateTransactionData;
pub use delegate_contract_id::DelegateContractId;
