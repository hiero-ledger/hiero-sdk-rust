// SPDX-License-Identifier: Apache-2.0

mod account_allowance_approve_transaction;
mod account_allowance_delete_transaction;
mod account_balance;
#[cfg(not(target_arch = "wasm32"))] // Balance query requires networking
mod account_balance_query;
mod account_create_transaction;
mod account_delete_transaction;
mod account_id;
mod account_info;
// note(sr): there's absolutely no way I'm going to write an enum or struct for namespacing here.
/// Flow for verifying signatures via account info.
pub mod account_info_flow;
#[cfg(not(target_arch = "wasm32"))] // Info query requires networking
mod account_info_query;
#[cfg(not(target_arch = "wasm32"))] // Records query requires networking
mod account_records_query;
mod account_update_transaction;
mod proxy_staker;

pub use account_allowance_approve_transaction::AccountAllowanceApproveTransaction;
pub(crate) use account_allowance_approve_transaction::AccountAllowanceApproveTransactionData;
pub use account_allowance_delete_transaction::AccountAllowanceDeleteTransaction;
pub(crate) use account_allowance_delete_transaction::AccountAllowanceDeleteTransactionData;
pub use account_balance::AccountBalance;
#[cfg(not(target_arch = "wasm32"))]
pub use account_balance_query::AccountBalanceQuery;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use account_balance_query::AccountBalanceQueryData;
pub use account_create_transaction::AccountCreateTransaction;
pub(crate) use account_create_transaction::AccountCreateTransactionData;
pub use account_delete_transaction::AccountDeleteTransaction;
pub(crate) use account_delete_transaction::AccountDeleteTransactionData;
pub use account_id::AccountId;
pub use account_info::AccountInfo;
#[cfg(not(target_arch = "wasm32"))]
pub use account_info_query::AccountInfoQuery;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use account_info_query::AccountInfoQueryData;
#[cfg(not(target_arch = "wasm32"))]
pub use account_records_query::AccountRecordsQuery;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use account_records_query::AccountRecordsQueryData;
pub use account_update_transaction::AccountUpdateTransaction;
pub(crate) use account_update_transaction::AccountUpdateTransactionData;
pub use proxy_staker::{
    AllProxyStakers,
    ProxyStaker,
};
