// SPDX-License-Identifier: Apache-2.0

// todo: more indepth documentation
//! Hiero Rust SDK.

#![forbid(unsafe_op_in_unsafe_fn)]
#![warn(
    absolute_paths_not_starting_with_crate,
    deprecated_in_future,
    future_incompatible,
    missing_docs,
    clippy::cargo_common_metadata,
    clippy::future_not_send,
    clippy::missing_errors_doc
)]
// useful pedantic clippy lints
// This is an opt-in list instead of opt-out because sometimes clippy has weird lints.
#![warn(
    clippy::bool_to_int_with_if,
    clippy::checked_conversions,
    clippy::cloned_instead_of_copied,
    clippy::copy_iterator,
    clippy::default_trait_access,
    clippy::doc_link_with_quotes,
    clippy::doc_markdown,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::explicit_iter_loop,
    clippy::filter_map_next,
    clippy::flat_map_option,
    clippy::fn_params_excessive_bools,
    clippy::from_iter_instead_of_collect,
    clippy::if_not_else,
    clippy::implicit_clone,
    clippy::implicit_hasher,
    clippy::inconsistent_struct_constructor,
    clippy::index_refutable_slice,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::items_after_statements,
    clippy::iter_not_returning_iterator,
    clippy::large_digit_groups,
    clippy::large_stack_arrays,
    clippy::large_futures,
    clippy::large_types_passed_by_value,
    clippy::linkedlist,
    clippy::macro_use_imports,
    clippy::manual_assert,
    clippy::manual_instant_elapsed,
    clippy::manual_let_else,
    clippy::manual_ok_or,
    clippy::manual_string_new,
    clippy::many_single_char_names,
    clippy::map_unwrap_or,
    clippy::match_same_arms,
    clippy::match_wild_err_arm,
    clippy::match_wildcard_for_single_variants,
    clippy::maybe_infinite_iter,
    clippy::mismatching_type_param_order,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::mut_mut,
    clippy::naive_bytecount,
    clippy::needless_bitwise_bool,
    clippy::needless_continue,
    clippy::needless_for_each,
    clippy::needless_pass_by_value,
    clippy::no_effect_underscore_binding,
    clippy::option_option,
    clippy::range_minus_one,
    clippy::range_plus_one,
    clippy::redundant_closure_for_method_calls,
    clippy::redundant_else,
    clippy::ref_binding_to_reference,
    clippy::ref_option_ref,
    clippy::return_self_not_must_use,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::similar_names,
    clippy::stable_sort_primitive,
    clippy::string_add_assign,
    clippy::struct_excessive_bools,
    clippy::trivially_copy_pass_by_ref,
    clippy::unchecked_duration_subtraction,
    clippy::uninlined_format_args,
    clippy::unnecessary_join,
    clippy::unnecessary_wraps,
    clippy::unnested_or_patterns,
    clippy::unreadable_literal,
    clippy::unsafe_derive_deserialize,
    clippy::unused_async,
    clippy::unused_self,
    clippy::used_underscore_binding,
    clippy::zero_sized_map_values
)]
#![allow(clippy::enum_glob_use, clippy::enum_variant_names)]
#[macro_use]
mod protobuf;

mod account;
mod address_book;
mod client;
mod contract;
mod custom_fee_limit;
mod custom_fixed_fee;
mod downcast;
mod entity_id;
mod error;
mod ethereum;
mod exchange_rates;
mod execute;
mod fee_schedules;
mod file;
mod hbar;
mod key;
mod ledger_id;
mod mirror_query;
#[cfg(feature = "mnemonic")]
mod mnemonic;
mod network_version_info;
mod network_version_info_query;
mod node_address;
mod node_address_book;
mod node_address_book_query;
mod pending_airdrop_id;
mod pending_airdrop_record;
mod ping_query;
mod prng_transaction;
mod query;
mod retry;
mod schedule;
mod semantic_version;
mod service_endpoint;
mod signer;
mod staked_id;
mod staking_info;
mod system;
mod token;
mod topic;
mod transaction;
mod transaction_hash;
mod transaction_id;
mod transaction_receipt;
mod transaction_receipt_query;
mod transaction_record;
mod transaction_record_query;
mod transaction_response;
mod transfer;
mod transfer_transaction;

pub use account::{
    account_info_flow,
    AccountAllowanceApproveTransaction,
    AccountAllowanceDeleteTransaction,
    AccountBalance,
    AccountBalanceQuery,
    AccountCreateTransaction,
    AccountDeleteTransaction,
    AccountId,
    AccountInfo,
    AccountInfoQuery,
    AccountRecordsQuery,
    AccountUpdateTransaction,
    AllProxyStakers,
    ProxyStaker,
};
pub use address_book::{
    NodeCreateTransaction,
    NodeDeleteTransaction,
    NodeUpdateTransaction,
};
pub use client::Client;
pub(crate) use client::Operator;
pub use contract::{
    ContractBytecodeQuery,
    ContractCallQuery,
    ContractCreateFlow,
    ContractCreateTransaction,
    ContractDeleteTransaction,
    ContractExecuteTransaction,
    ContractFunctionParameters,
    ContractFunctionResult,
    ContractId,
    ContractInfo,
    ContractInfoQuery,
    ContractLogInfo,
    ContractNonceInfo,
    ContractUpdateTransaction,
    DelegateContractId,
};
pub use custom_fixed_fee::CustomFixedFee;
pub use entity_id::EntityId;
pub(crate) use entity_id::ValidateChecksums;
pub use error::{
    Error,
    Result,
};
#[cfg(feature = "mnemonic")]
pub use error::{
    MnemonicEntropyError,
    MnemonicParseError,
};
pub use ethereum::{
    Eip1559EthereumData,
    EthereumData,
    EthereumFlow,
    EthereumTransaction,
    EvmAddress,
    LegacyEthereumData,
};
pub use exchange_rates::{
    ExchangeRate,
    ExchangeRates,
};
pub use fee_schedules::{
    FeeComponents,
    FeeData,
    FeeDataType,
    FeeSchedule,
    FeeSchedules,
    RequestType,
    TransactionFeeSchedule,
};
pub use file::{
    FileAppendTransaction,
    FileContentsQuery,
    FileContentsResponse,
    FileCreateTransaction,
    FileDeleteTransaction,
    FileId,
    FileInfo,
    FileInfoQuery,
    FileUpdateTransaction,
};
pub use hbar::{
    Hbar,
    HbarUnit,
    Tinybar,
};
pub use hedera_proto::services::ResponseCodeEnum as Status;
pub use key::{
    Key,
    KeyList,
    PrivateKey,
    PublicKey,
};
pub use ledger_id::LedgerId;
pub use mirror_query::{
    AnyMirrorQuery,
    AnyMirrorQueryResponse,
    MirrorQuery,
};
#[cfg(feature = "mnemonic")]
pub use mnemonic::Mnemonic;
pub use network_version_info::NetworkVersionInfo;
pub use network_version_info_query::NetworkVersionInfoQuery;
pub(crate) use network_version_info_query::NetworkVersionInfoQueryData;
pub use node_address::NodeAddress;
pub use node_address_book::NodeAddressBook;
pub use node_address_book_query::NodeAddressBookQuery;
pub(crate) use node_address_book_query::NodeAddressBookQueryData;
pub use pending_airdrop_record::PendingAirdropRecord;
pub use prng_transaction::PrngTransaction;
pub(crate) use protobuf::{
    FromProtobuf,
    ToProtobuf,
};
pub use query::{
    AnyQuery,
    AnyQueryResponse,
    Query,
};
pub(crate) use retry::retry;
pub use schedule::{
    ScheduleCreateTransaction,
    ScheduleDeleteTransaction,
    ScheduleId,
    ScheduleInfo,
    ScheduleInfoQuery,
    ScheduleSignTransaction,
};
pub use semantic_version::SemanticVersion;
pub use service_endpoint::ServiceEndpoint;
pub use staking_info::StakingInfo;
pub use system::{
    FreezeTransaction,
    FreezeType,
    SystemDeleteTransaction,
    SystemUndeleteTransaction,
};
pub use token::{
    AnyCustomFee,
    AssessedCustomFee,
    CustomFee,
    Fee,
    FeeAssessmentMethod,
    FixedFee,
    FixedFeeData,
    FractionalFee,
    FractionalFeeData,
    NftId,
    RoyaltyFee,
    RoyaltyFeeData,
    TokenAirdropTransaction,
    TokenAssociateTransaction,
    TokenAssociation,
    TokenBurnTransaction,
    TokenCancelAirdropTransaction,
    TokenClaimAirdropTransaction,
    TokenCreateTransaction,
    TokenDeleteTransaction,
    TokenDissociateTransaction,
    TokenFeeScheduleUpdateTransaction,
    TokenFreezeTransaction,
    TokenGrantKycTransaction,
    TokenId,
    TokenInfo,
    TokenInfoQuery,
    TokenKeyValidation,
    TokenMintTransaction,
    TokenNftInfo,
    TokenNftInfoQuery,
    TokenNftTransfer,
    TokenPauseTransaction,
    TokenRejectFlow,
    TokenRejectTransaction,
    TokenRevokeKycTransaction,
    TokenSupplyType,
    TokenType,
    TokenUnfreezeTransaction,
    TokenUnpauseTransaction,
    TokenUpdateNftsTransaction,
    TokenUpdateTransaction,
    TokenWipeTransaction,
};
pub use topic::{
    TopicCreateTransaction,
    TopicDeleteTransaction,
    TopicId,
    TopicInfo,
    TopicInfoQuery,
    TopicMessage,
    TopicMessageQuery,
    TopicMessageSubmitTransaction,
    TopicUpdateTransaction,
};
pub use transaction::{
    AnyTransaction,
    Transaction,
};
pub use transaction_hash::TransactionHash;
pub use transaction_id::TransactionId;
pub use transaction_receipt::TransactionReceipt;
pub use transaction_receipt_query::TransactionReceiptQuery;
pub use transaction_record::TransactionRecord;
pub use transaction_record_query::TransactionRecordQuery;
pub(crate) use transaction_record_query::TransactionRecordQueryData;
pub use transaction_response::TransactionResponse;
pub use transfer::Transfer;
pub use transfer_transaction::TransferTransaction;

/// Like [`arc_swap::ArcSwapOption`] but with a [`triomphe::Arc`].
pub(crate) type ArcSwapOption<T> = arc_swap::ArcSwapAny<Option<triomphe::Arc<T>>>;

/// Like [`arc_swap::ArcSwap`] but with a [`triomphe::Arc`].
pub(crate) type ArcSwap<T> = arc_swap::ArcSwapAny<triomphe::Arc<T>>;

/// Boxed future for GRPC calls.
pub(crate) type BoxGrpcFuture<'a, T> =
    futures_core::future::BoxFuture<'a, tonic::Result<tonic::Response<T>>>;
