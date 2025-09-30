// SPDX-License-Identifier: Apache-2.0

mod topic_create_transaction;
mod topic_delete_transaction;
mod topic_id;
mod topic_info;
#[cfg(not(target_arch = "wasm32"))] // Info query requires networking
mod topic_info_query;
mod topic_message;
#[cfg(not(target_arch = "wasm32"))] // Message query requires networking
mod topic_message_query;
mod topic_message_submit_transaction;
mod topic_update_transaction;

pub use topic_create_transaction::TopicCreateTransaction;
pub(crate) use topic_create_transaction::TopicCreateTransactionData;
pub use topic_delete_transaction::TopicDeleteTransaction;
pub(crate) use topic_delete_transaction::TopicDeleteTransactionData;
pub use topic_id::TopicId;
pub use topic_info::TopicInfo;
#[cfg(not(target_arch = "wasm32"))]
pub use topic_info_query::TopicInfoQuery;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use topic_info_query::TopicInfoQueryData;
pub use topic_message::TopicMessage;
#[cfg(not(target_arch = "wasm32"))]
pub use topic_message_query::TopicMessageQuery;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use topic_message_query::TopicMessageQueryData;
pub use topic_message_submit_transaction::TopicMessageSubmitTransaction;
pub(crate) use topic_message_submit_transaction::TopicMessageSubmitTransactionData;
pub use topic_update_transaction::TopicUpdateTransaction;
pub(crate) use topic_update_transaction::TopicUpdateTransactionData;
