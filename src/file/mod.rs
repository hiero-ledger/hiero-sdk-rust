// SPDX-License-Identifier: Apache-2.0

mod file_append_transaction;
#[cfg(not(target_arch = "wasm32"))] // Contents query requires networking
mod file_contents_query;
mod file_contents_response;
mod file_create_transaction;
mod file_delete_transaction;
mod file_id;
mod file_info;
#[cfg(not(target_arch = "wasm32"))] // Info query requires networking
mod file_info_query;
mod file_update_transaction;

pub use file_append_transaction::FileAppendTransaction;
pub(crate) use file_append_transaction::FileAppendTransactionData;
#[cfg(not(target_arch = "wasm32"))]
pub use file_contents_query::FileContentsQuery;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use file_contents_query::FileContentsQueryData;
pub use file_contents_response::FileContentsResponse;
pub use file_create_transaction::FileCreateTransaction;
pub(crate) use file_create_transaction::FileCreateTransactionData;
pub use file_delete_transaction::FileDeleteTransaction;
pub(crate) use file_delete_transaction::FileDeleteTransactionData;
pub use file_id::FileId;
pub use file_info::FileInfo;
#[cfg(not(target_arch = "wasm32"))]
pub use file_info_query::FileInfoQuery;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use file_info_query::FileInfoQueryData;
pub use file_update_transaction::FileUpdateTransaction;
pub(crate) use file_update_transaction::FileUpdateTransactionData;
