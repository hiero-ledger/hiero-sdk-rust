// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use tonic::transport::Channel;

use super::chunked::ChunkInfo;
use super::{
    TransactionData,
    TransactionExecuteChunked,
};
use crate::custom_fee_limit::CustomFeeLimit;
use crate::downcast::DowncastOwned;
use crate::entity_id::ValidateChecksums;
use crate::ledger_id::RefLedgerId;
use crate::protobuf::FromProtobuf;
use crate::transaction::{
    ToTransactionDataProtobuf,
    TransactionBody,
    TransactionExecute,
};
use crate::{
    AccountId,
    BoxGrpcFuture,
    Error,
    Hbar,
    Transaction,
    TransactionId,
};

mod data {
    pub(super) use crate::account::{
        AccountAllowanceApproveTransactionData as AccountAllowanceApprove,
        AccountAllowanceDeleteTransactionData as AccountAllowanceDelete,
        AccountCreateTransactionData as AccountCreate,
        AccountDeleteTransactionData as AccountDelete,
        AccountUpdateTransactionData as AccountUpdate,
    };
    pub(super) use crate::address_book::{
        NodeCreateTransactionData as NodeCreate,
        NodeDeleteTransactionData as NodeDelete,
        NodeUpdateTransactionData as NodeUpdate,
    };
    pub(super) use crate::contract::{
        ContractCreateTransactionData as ContractCreate,
        ContractDeleteTransactionData as ContractDelete,
        ContractExecuteTransactionData as ContractExecute,
        ContractUpdateTransactionData as ContractUpdate,
    };
    pub(super) use crate::ethereum::EthereumTransactionData as Ethereum;
    pub(super) use crate::file::{
        FileAppendTransactionData as FileAppend,
        FileCreateTransactionData as FileCreate,
        FileDeleteTransactionData as FileDelete,
        FileUpdateTransactionData as FileUpdate,
    };
    pub(super) use crate::prng_transaction::PrngTransactionData as Prng;
    pub(super) use crate::schedule::{
        ScheduleCreateTransactionData as ScheduleCreate,
        ScheduleDeleteTransactionData as ScheduleDelete,
        ScheduleSignTransactionData as ScheduleSign,
    };
    pub(super) use crate::system::{
        FreezeTransactionData as Freeze,
        SystemDeleteTransactionData as SystemDelete,
        SystemUndeleteTransactionData as SystemUndelete,
    };
    pub(super) use crate::token::{
        TokenAirdropTransactionData as TokenAirdrop,
        TokenAssociateTransactionData as TokenAssociate,
        TokenBurnTransactionData as TokenBurn,
        TokenCancelAirdropTransactionData as TokenCancelAirdrop,
        TokenClaimAirdropTransactionData as TokenClaimAirdrop,
        TokenCreateTransactionData as TokenCreate,
        TokenDeleteTransactionData as TokenDelete,
        TokenDissociateTransactionData as TokenDissociate,
        TokenFeeScheduleUpdateTransactionData as TokenFeeScheduleUpdate,
        TokenFreezeTransactionData as TokenFreeze,
        TokenGrantKycTransactionData as TokenGrantKyc,
        TokenMintTransactionData as TokenMint,
        TokenPauseTransactionData as TokenPause,
        TokenRejectTransactionData as TokenReject,
        TokenRevokeKycTransactionData as TokenRevokeKyc,
        TokenUnfreezeTransactionData as TokenUnfreeze,
        TokenUnpauseTransactionData as TokenUnpause,
        TokenUpdateNftsTransactionData as TokenUpdateNfts,
        TokenUpdateTransactionData as TokenUpdate,
        TokenWipeTransactionData as TokenWipe,
    };
    pub(super) use crate::topic::{
        TopicCreateTransactionData as TopicCreate,
        TopicDeleteTransactionData as TopicDelete,
        TopicMessageSubmitTransactionData as TopicMessageSubmit,
        TopicUpdateTransactionData as TopicUpdate,
    };
    pub(super) use crate::transfer_transaction::TransferTransactionData as Transfer;
}

/// Any possible transaction that may be executed on the Hiero network.
pub type AnyTransaction = Transaction<AnyTransactionData>;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum AnyTransactionData {
    AccountCreate(data::AccountCreate),
    AccountUpdate(data::AccountUpdate),
    AccountDelete(data::AccountDelete),
    AccountAllowanceApprove(data::AccountAllowanceApprove),
    AccountAllowanceDelete(data::AccountAllowanceDelete),
    ContractCreate(data::ContractCreate),
    ContractUpdate(data::ContractUpdate),
    ContractDelete(data::ContractDelete),
    ContractExecute(data::ContractExecute),
    Transfer(data::Transfer),
    TopicCreate(data::TopicCreate),
    TopicUpdate(data::TopicUpdate),
    TopicDelete(data::TopicDelete),
    TopicMessageSubmit(data::TopicMessageSubmit),
    FileAppend(data::FileAppend),
    FileCreate(data::FileCreate),
    FileUpdate(data::FileUpdate),
    FileDelete(data::FileDelete),
    Prng(data::Prng),
    ScheduleCreate(data::ScheduleCreate),
    ScheduleSign(data::ScheduleSign),
    ScheduleDelete(data::ScheduleDelete),
    TokenAssociate(data::TokenAssociate),
    TokenBurn(data::TokenBurn),
    TokenCreate(data::TokenCreate),
    TokenDelete(data::TokenDelete),
    TokenDissociate(data::TokenDissociate),
    TokenFeeScheduleUpdate(data::TokenFeeScheduleUpdate),
    TokenFreeze(data::TokenFreeze),
    TokenGrantKyc(data::TokenGrantKyc),
    TokenMint(data::TokenMint),
    TokenPause(data::TokenPause),
    TokenRevokeKyc(data::TokenRevokeKyc),
    TokenUnfreeze(data::TokenUnfreeze),
    TokenUnpause(data::TokenUnpause),
    TokenUpdate(data::TokenUpdate),
    TokenWipe(data::TokenWipe),
    SystemDelete(data::SystemDelete),
    SystemUndelete(data::SystemUndelete),
    Freeze(data::Freeze),
    Ethereum(data::Ethereum),
    TokenUpdateNfts(data::TokenUpdateNfts),
    NodeCreate(data::NodeCreate),
    NodeUpdate(data::NodeUpdate),
    NodeDelete(data::NodeDelete),
    TokenReject(data::TokenReject),
    TokenAirdrop(data::TokenAirdrop),
    TokenClaimAirdrop(data::TokenClaimAirdrop),
    TokenCancelAirdrop(data::TokenCancelAirdrop),
}

impl ToTransactionDataProtobuf for AnyTransactionData {
    // not really anything I can do about this
    #[allow(clippy::too_many_lines)]
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        match self {
            Self::Transfer(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::AccountCreate(transaction) => {
                transaction.to_transaction_data_protobuf(chunk_info)
            }

            Self::AccountUpdate(transaction) => {
                transaction.to_transaction_data_protobuf(chunk_info)
            }

            Self::AccountDelete(transaction) => {
                transaction.to_transaction_data_protobuf(chunk_info)
            }

            Self::AccountAllowanceApprove(transaction) => {
                transaction.to_transaction_data_protobuf(chunk_info)
            }

            Self::AccountAllowanceDelete(transaction) => {
                transaction.to_transaction_data_protobuf(chunk_info)
            }

            Self::ContractCreate(transaction) => {
                transaction.to_transaction_data_protobuf(chunk_info)
            }

            Self::ContractUpdate(transaction) => {
                transaction.to_transaction_data_protobuf(chunk_info)
            }

            Self::ContractDelete(transaction) => {
                transaction.to_transaction_data_protobuf(chunk_info)
            }

            Self::ContractExecute(transaction) => {
                transaction.to_transaction_data_protobuf(chunk_info)
            }

            Self::FileAppend(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::FileCreate(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::FileUpdate(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::FileDelete(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::Prng(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::TokenAssociate(transaction) => {
                transaction.to_transaction_data_protobuf(chunk_info)
            }

            Self::TokenBurn(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::TokenCreate(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::TokenDelete(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::TokenDissociate(transaction) => {
                transaction.to_transaction_data_protobuf(chunk_info)
            }

            Self::TokenFeeScheduleUpdate(transaction) => {
                transaction.to_transaction_data_protobuf(chunk_info)
            }

            Self::TokenFreeze(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::TokenGrantKyc(transaction) => {
                transaction.to_transaction_data_protobuf(chunk_info)
            }

            Self::TokenMint(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::TokenPause(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::TokenRevokeKyc(transaction) => {
                transaction.to_transaction_data_protobuf(chunk_info)
            }

            Self::TokenUnfreeze(transaction) => {
                transaction.to_transaction_data_protobuf(chunk_info)
            }

            Self::TokenUnpause(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::TokenUpdate(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::TokenWipe(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::TopicCreate(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::TopicUpdate(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::TopicDelete(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::TokenReject(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::TopicMessageSubmit(transaction) => {
                transaction.to_transaction_data_protobuf(chunk_info)
            }

            Self::SystemDelete(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::SystemUndelete(transaction) => {
                transaction.to_transaction_data_protobuf(chunk_info)
            }

            Self::Freeze(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::ScheduleCreate(transaction) => {
                transaction.to_transaction_data_protobuf(chunk_info)
            }

            Self::ScheduleSign(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::ScheduleDelete(transaction) => {
                transaction.to_transaction_data_protobuf(chunk_info)
            }

            Self::Ethereum(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::TokenUpdateNfts(transaction) => {
                transaction.to_transaction_data_protobuf(chunk_info)
            }

            Self::NodeCreate(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::NodeUpdate(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::NodeDelete(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::TokenAirdrop(transaction) => transaction.to_transaction_data_protobuf(chunk_info),

            Self::TokenClaimAirdrop(transaction) => {
                transaction.to_transaction_data_protobuf(chunk_info)
            }
            Self::TokenCancelAirdrop(transaction) => {
                transaction.to_transaction_data_protobuf(chunk_info)
            }
        }
    }
}

impl TransactionData for AnyTransactionData {
    fn default_max_transaction_fee(&self) -> Hbar {
        match self {
            Self::Transfer(transaction) => transaction.default_max_transaction_fee(),
            Self::AccountCreate(transaction) => transaction.default_max_transaction_fee(),
            Self::AccountUpdate(transaction) => transaction.default_max_transaction_fee(),
            Self::AccountDelete(transaction) => transaction.default_max_transaction_fee(),
            Self::AccountAllowanceApprove(transaction) => transaction.default_max_transaction_fee(),
            Self::AccountAllowanceDelete(transaction) => transaction.default_max_transaction_fee(),
            Self::ContractCreate(transaction) => transaction.default_max_transaction_fee(),
            Self::ContractUpdate(transaction) => transaction.default_max_transaction_fee(),
            Self::ContractDelete(transaction) => transaction.default_max_transaction_fee(),
            Self::ContractExecute(transaction) => transaction.default_max_transaction_fee(),
            Self::FileAppend(transaction) => transaction.default_max_transaction_fee(),
            Self::FileCreate(transaction) => transaction.default_max_transaction_fee(),
            Self::FileUpdate(transaction) => transaction.default_max_transaction_fee(),
            Self::FileDelete(transaction) => transaction.default_max_transaction_fee(),
            Self::Prng(transaction) => transaction.default_max_transaction_fee(),
            Self::TokenAssociate(transaction) => transaction.default_max_transaction_fee(),
            Self::TokenBurn(transaction) => transaction.default_max_transaction_fee(),
            Self::TokenCreate(transaction) => transaction.default_max_transaction_fee(),
            Self::TokenDelete(transaction) => transaction.default_max_transaction_fee(),
            Self::TokenDissociate(transaction) => transaction.default_max_transaction_fee(),
            Self::TokenFeeScheduleUpdate(transaction) => transaction.default_max_transaction_fee(),
            Self::TokenFreeze(transaction) => transaction.default_max_transaction_fee(),
            Self::TokenGrantKyc(transaction) => transaction.default_max_transaction_fee(),
            Self::TokenMint(transaction) => transaction.default_max_transaction_fee(),
            Self::TokenPause(transaction) => transaction.default_max_transaction_fee(),
            Self::TokenRevokeKyc(transaction) => transaction.default_max_transaction_fee(),
            Self::TokenUnfreeze(transaction) => transaction.default_max_transaction_fee(),
            Self::TokenUnpause(transaction) => transaction.default_max_transaction_fee(),
            Self::TokenUpdate(transaction) => transaction.default_max_transaction_fee(),
            Self::TokenWipe(transaction) => transaction.default_max_transaction_fee(),
            Self::TopicCreate(transaction) => transaction.default_max_transaction_fee(),
            Self::TopicUpdate(transaction) => transaction.default_max_transaction_fee(),
            Self::TopicDelete(transaction) => transaction.default_max_transaction_fee(),
            Self::TopicMessageSubmit(transaction) => transaction.default_max_transaction_fee(),
            Self::SystemDelete(transaction) => transaction.default_max_transaction_fee(),
            Self::SystemUndelete(transaction) => transaction.default_max_transaction_fee(),
            Self::Freeze(transaction) => transaction.default_max_transaction_fee(),
            Self::ScheduleCreate(transaction) => transaction.default_max_transaction_fee(),
            Self::ScheduleSign(transaction) => transaction.default_max_transaction_fee(),
            Self::ScheduleDelete(transaction) => transaction.default_max_transaction_fee(),
            Self::Ethereum(transaction) => transaction.default_max_transaction_fee(),
            Self::TokenUpdateNfts(transaction) => transaction.default_max_transaction_fee(),
            Self::NodeCreate(transaction) => transaction.default_max_transaction_fee(),
            Self::NodeUpdate(transaction) => transaction.default_max_transaction_fee(),
            Self::NodeDelete(transaction) => transaction.default_max_transaction_fee(),
            Self::TokenReject(transaction) => transaction.default_max_transaction_fee(),
            Self::TokenAirdrop(transaction) => transaction.default_max_transaction_fee(),
            Self::TokenClaimAirdrop(transaction) => transaction.default_max_transaction_fee(),
            Self::TokenCancelAirdrop(transaction) => transaction.default_max_transaction_fee(),
        }
    }

    fn maybe_chunk_data(&self) -> Option<&super::ChunkData> {
        match self {
            Self::AccountCreate(it) => it.maybe_chunk_data(),
            Self::AccountUpdate(it) => it.maybe_chunk_data(),
            Self::AccountDelete(it) => it.maybe_chunk_data(),
            Self::AccountAllowanceApprove(it) => it.maybe_chunk_data(),
            Self::AccountAllowanceDelete(it) => it.maybe_chunk_data(),
            Self::ContractCreate(it) => it.maybe_chunk_data(),
            Self::ContractUpdate(it) => it.maybe_chunk_data(),
            Self::ContractDelete(it) => it.maybe_chunk_data(),
            Self::ContractExecute(it) => it.maybe_chunk_data(),
            Self::Transfer(it) => it.maybe_chunk_data(),
            Self::TopicCreate(it) => it.maybe_chunk_data(),
            Self::TopicUpdate(it) => it.maybe_chunk_data(),
            Self::TopicDelete(it) => it.maybe_chunk_data(),
            Self::TopicMessageSubmit(it) => it.maybe_chunk_data(),
            Self::FileAppend(it) => it.maybe_chunk_data(),
            Self::FileCreate(it) => it.maybe_chunk_data(),
            Self::FileUpdate(it) => it.maybe_chunk_data(),
            Self::FileDelete(it) => it.maybe_chunk_data(),
            Self::Prng(it) => it.maybe_chunk_data(),
            Self::TokenAssociate(it) => it.maybe_chunk_data(),
            Self::TokenBurn(it) => it.maybe_chunk_data(),
            Self::TokenCreate(it) => it.maybe_chunk_data(),
            Self::TokenDelete(it) => it.maybe_chunk_data(),
            Self::TokenDissociate(it) => it.maybe_chunk_data(),
            Self::TokenFeeScheduleUpdate(it) => it.maybe_chunk_data(),
            Self::TokenFreeze(it) => it.maybe_chunk_data(),
            Self::TokenGrantKyc(it) => it.maybe_chunk_data(),
            Self::TokenMint(it) => it.maybe_chunk_data(),
            Self::TokenPause(it) => it.maybe_chunk_data(),
            Self::TokenRevokeKyc(it) => it.maybe_chunk_data(),
            Self::TokenUnfreeze(it) => it.maybe_chunk_data(),
            Self::TokenUnpause(it) => it.maybe_chunk_data(),
            Self::TokenUpdate(it) => it.maybe_chunk_data(),
            Self::TokenWipe(it) => it.maybe_chunk_data(),
            Self::SystemDelete(it) => it.maybe_chunk_data(),
            Self::SystemUndelete(it) => it.maybe_chunk_data(),
            Self::Freeze(it) => it.maybe_chunk_data(),
            Self::ScheduleCreate(it) => it.maybe_chunk_data(),
            Self::ScheduleSign(it) => it.maybe_chunk_data(),
            Self::ScheduleDelete(it) => it.maybe_chunk_data(),
            Self::Ethereum(it) => it.maybe_chunk_data(),
            Self::TokenUpdateNfts(it) => it.maybe_chunk_data(),
            Self::NodeCreate(it) => it.maybe_chunk_data(),
            Self::NodeUpdate(it) => it.maybe_chunk_data(),
            Self::NodeDelete(it) => it.maybe_chunk_data(),
            Self::TokenReject(it) => it.maybe_chunk_data(),
            Self::TokenAirdrop(it) => it.maybe_chunk_data(),
            Self::TokenClaimAirdrop(it) => it.maybe_chunk_data(),
            Self::TokenCancelAirdrop(it) => it.maybe_chunk_data(),
        }
    }

    fn wait_for_receipt(&self) -> bool {
        match self {
            Self::AccountCreate(it) => it.wait_for_receipt(),
            Self::AccountUpdate(it) => it.wait_for_receipt(),
            Self::AccountDelete(it) => it.wait_for_receipt(),
            Self::AccountAllowanceApprove(it) => it.wait_for_receipt(),
            Self::AccountAllowanceDelete(it) => it.wait_for_receipt(),
            Self::ContractCreate(it) => it.wait_for_receipt(),
            Self::ContractUpdate(it) => it.wait_for_receipt(),
            Self::ContractDelete(it) => it.wait_for_receipt(),
            Self::ContractExecute(it) => it.wait_for_receipt(),
            Self::Transfer(it) => it.wait_for_receipt(),
            Self::TopicCreate(it) => it.wait_for_receipt(),
            Self::TopicUpdate(it) => it.wait_for_receipt(),
            Self::TopicDelete(it) => it.wait_for_receipt(),
            Self::TopicMessageSubmit(it) => it.wait_for_receipt(),
            Self::FileAppend(it) => it.wait_for_receipt(),
            Self::FileCreate(it) => it.wait_for_receipt(),
            Self::FileUpdate(it) => it.wait_for_receipt(),
            Self::FileDelete(it) => it.wait_for_receipt(),
            Self::Prng(it) => it.wait_for_receipt(),
            Self::TokenAssociate(it) => it.wait_for_receipt(),
            Self::TokenBurn(it) => it.wait_for_receipt(),
            Self::TokenCreate(it) => it.wait_for_receipt(),
            Self::TokenDelete(it) => it.wait_for_receipt(),
            Self::TokenDissociate(it) => it.wait_for_receipt(),
            Self::TokenFeeScheduleUpdate(it) => it.wait_for_receipt(),
            Self::TokenFreeze(it) => it.wait_for_receipt(),
            Self::TokenGrantKyc(it) => it.wait_for_receipt(),
            Self::TokenMint(it) => it.wait_for_receipt(),
            Self::TokenPause(it) => it.wait_for_receipt(),
            Self::TokenRevokeKyc(it) => it.wait_for_receipt(),
            Self::TokenUnfreeze(it) => it.wait_for_receipt(),
            Self::TokenUnpause(it) => it.wait_for_receipt(),
            Self::TokenUpdate(it) => it.wait_for_receipt(),
            Self::TokenWipe(it) => it.wait_for_receipt(),
            Self::SystemDelete(it) => it.wait_for_receipt(),
            Self::SystemUndelete(it) => it.wait_for_receipt(),
            Self::Freeze(it) => it.wait_for_receipt(),
            Self::ScheduleCreate(it) => it.wait_for_receipt(),
            Self::ScheduleSign(it) => it.wait_for_receipt(),
            Self::ScheduleDelete(it) => it.wait_for_receipt(),
            Self::Ethereum(it) => it.wait_for_receipt(),
            Self::TokenUpdateNfts(it) => it.wait_for_receipt(),
            Self::NodeCreate(it) => it.wait_for_receipt(),
            Self::NodeUpdate(it) => it.wait_for_receipt(),
            Self::NodeDelete(it) => it.wait_for_receipt(),
            Self::TokenReject(it) => it.wait_for_receipt(),
            Self::TokenAirdrop(it) => it.wait_for_receipt(),
            Self::TokenClaimAirdrop(it) => it.wait_for_receipt(),
            Self::TokenCancelAirdrop(it) => it.wait_for_receipt(),
        }
    }
}

impl TransactionExecute for AnyTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        match self {
            Self::Transfer(transaction) => transaction.execute(channel, request),
            Self::AccountCreate(transaction) => transaction.execute(channel, request),
            Self::AccountUpdate(transaction) => transaction.execute(channel, request),
            Self::AccountDelete(transaction) => transaction.execute(channel, request),
            Self::AccountAllowanceApprove(transaction) => transaction.execute(channel, request),
            Self::AccountAllowanceDelete(transaction) => transaction.execute(channel, request),
            Self::ContractCreate(transaction) => transaction.execute(channel, request),
            Self::ContractUpdate(transaction) => transaction.execute(channel, request),
            Self::ContractDelete(transaction) => transaction.execute(channel, request),
            Self::ContractExecute(transaction) => transaction.execute(channel, request),
            Self::FileAppend(transaction) => transaction.execute(channel, request),
            Self::FileCreate(transaction) => transaction.execute(channel, request),
            Self::FileUpdate(transaction) => transaction.execute(channel, request),
            Self::FileDelete(transaction) => transaction.execute(channel, request),
            Self::Prng(transaction) => transaction.execute(channel, request),
            Self::TokenAssociate(transaction) => transaction.execute(channel, request),
            Self::TokenBurn(transaction) => transaction.execute(channel, request),
            Self::TokenCreate(transaction) => transaction.execute(channel, request),
            Self::TokenDelete(transaction) => transaction.execute(channel, request),
            Self::TokenDissociate(transaction) => transaction.execute(channel, request),
            Self::TokenFeeScheduleUpdate(transaction) => transaction.execute(channel, request),
            Self::TokenFreeze(transaction) => transaction.execute(channel, request),
            Self::TokenGrantKyc(transaction) => transaction.execute(channel, request),
            Self::TokenMint(transaction) => transaction.execute(channel, request),
            Self::TokenPause(transaction) => transaction.execute(channel, request),
            Self::TokenRevokeKyc(transaction) => transaction.execute(channel, request),
            Self::TokenUnfreeze(transaction) => transaction.execute(channel, request),
            Self::TokenUnpause(transaction) => transaction.execute(channel, request),
            Self::TokenUpdate(transaction) => transaction.execute(channel, request),
            Self::TokenWipe(transaction) => transaction.execute(channel, request),
            Self::TopicCreate(transaction) => transaction.execute(channel, request),
            Self::TopicUpdate(transaction) => transaction.execute(channel, request),
            Self::TopicDelete(transaction) => transaction.execute(channel, request),
            Self::TopicMessageSubmit(transaction) => transaction.execute(channel, request),
            Self::SystemDelete(transaction) => transaction.execute(channel, request),
            Self::SystemUndelete(transaction) => transaction.execute(channel, request),
            Self::Freeze(transaction) => transaction.execute(channel, request),
            Self::ScheduleCreate(transaction) => transaction.execute(channel, request),
            Self::ScheduleSign(transaction) => transaction.execute(channel, request),
            Self::ScheduleDelete(transaction) => transaction.execute(channel, request),
            Self::Ethereum(transaction) => transaction.execute(channel, request),
            Self::TokenUpdateNfts(transaction) => transaction.execute(channel, request),
            Self::NodeCreate(transaction) => transaction.execute(channel, request),
            Self::NodeUpdate(transaction) => transaction.execute(channel, request),
            Self::NodeDelete(transaction) => transaction.execute(channel, request),
            Self::TokenReject(transaction) => transaction.execute(channel, request),
            Self::TokenAirdrop(transaction) => transaction.execute(channel, request),
            Self::TokenClaimAirdrop(transaction) => transaction.execute(channel, request),
            Self::TokenCancelAirdrop(transaction) => transaction.execute(channel, request),
        }
    }
}

impl TransactionExecuteChunked for AnyTransactionData {}

impl ValidateChecksums for AnyTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        match self {
            Self::AccountCreate(transaction) => transaction.validate_checksums(ledger_id),
            Self::AccountUpdate(transaction) => transaction.validate_checksums(ledger_id),
            Self::AccountDelete(transaction) => transaction.validate_checksums(ledger_id),
            Self::AccountAllowanceApprove(transaction) => transaction.validate_checksums(ledger_id),
            Self::AccountAllowanceDelete(transaction) => transaction.validate_checksums(ledger_id),
            Self::ContractCreate(transaction) => transaction.validate_checksums(ledger_id),
            Self::ContractUpdate(transaction) => transaction.validate_checksums(ledger_id),
            Self::ContractDelete(transaction) => transaction.validate_checksums(ledger_id),
            Self::ContractExecute(transaction) => transaction.validate_checksums(ledger_id),
            Self::Transfer(transaction) => transaction.validate_checksums(ledger_id),
            Self::TopicCreate(transaction) => transaction.validate_checksums(ledger_id),
            Self::TopicUpdate(transaction) => transaction.validate_checksums(ledger_id),
            Self::TopicDelete(transaction) => transaction.validate_checksums(ledger_id),
            Self::TopicMessageSubmit(transaction) => transaction.validate_checksums(ledger_id),
            Self::FileAppend(transaction) => transaction.validate_checksums(ledger_id),
            Self::FileCreate(transaction) => transaction.validate_checksums(ledger_id),
            Self::FileUpdate(transaction) => transaction.validate_checksums(ledger_id),
            Self::FileDelete(transaction) => transaction.validate_checksums(ledger_id),
            Self::Prng(transaction) => transaction.validate_checksums(ledger_id),
            Self::ScheduleCreate(transaction) => transaction.validate_checksums(ledger_id),
            Self::ScheduleSign(transaction) => transaction.validate_checksums(ledger_id),
            Self::ScheduleDelete(transaction) => transaction.validate_checksums(ledger_id),
            Self::TokenAssociate(transaction) => transaction.validate_checksums(ledger_id),
            Self::TokenBurn(transaction) => transaction.validate_checksums(ledger_id),
            Self::TokenCreate(transaction) => transaction.validate_checksums(ledger_id),
            Self::TokenDelete(transaction) => transaction.validate_checksums(ledger_id),
            Self::TokenDissociate(transaction) => transaction.validate_checksums(ledger_id),
            Self::TokenFeeScheduleUpdate(transaction) => transaction.validate_checksums(ledger_id),
            Self::TokenFreeze(transaction) => transaction.validate_checksums(ledger_id),
            Self::TokenGrantKyc(transaction) => transaction.validate_checksums(ledger_id),
            Self::TokenMint(transaction) => transaction.validate_checksums(ledger_id),
            Self::TokenPause(transaction) => transaction.validate_checksums(ledger_id),
            Self::TokenRevokeKyc(transaction) => transaction.validate_checksums(ledger_id),
            Self::TokenUnfreeze(transaction) => transaction.validate_checksums(ledger_id),
            Self::TokenUnpause(transaction) => transaction.validate_checksums(ledger_id),
            Self::TokenUpdate(transaction) => transaction.validate_checksums(ledger_id),
            Self::TokenWipe(transaction) => transaction.validate_checksums(ledger_id),
            Self::SystemDelete(transaction) => transaction.validate_checksums(ledger_id),
            Self::SystemUndelete(transaction) => transaction.validate_checksums(ledger_id),
            Self::Freeze(transaction) => transaction.validate_checksums(ledger_id),
            Self::Ethereum(transaction) => transaction.validate_checksums(ledger_id),
            Self::TokenUpdateNfts(transaction) => transaction.validate_checksums(ledger_id),
            Self::NodeCreate(transaction) => transaction.validate_checksums(ledger_id),
            Self::NodeUpdate(transaction) => transaction.validate_checksums(ledger_id),
            Self::NodeDelete(transaction) => transaction.validate_checksums(ledger_id),
            Self::TokenReject(transaction) => transaction.validate_checksums(ledger_id),
            Self::TokenAirdrop(transaction) => transaction.validate_checksums(ledger_id),
            Self::TokenClaimAirdrop(transaction) => transaction.validate_checksums(ledger_id),
            Self::TokenCancelAirdrop(transaction) => transaction.validate_checksums(ledger_id),
        }
    }
}

impl FromProtobuf<services::transaction_body::Data> for AnyTransactionData {
    fn from_protobuf(pb: services::transaction_body::Data) -> crate::Result<Self>
    where
        Self: Sized,
    {
        use services::transaction_body::Data;
        let data = match pb {
            Data::ContractCall(pb) => data::ContractExecute::from_protobuf(pb)?.into(),
            Data::ContractCreateInstance(pb) => data::ContractCreate::from_protobuf(pb)?.into(),
            Data::ContractUpdateInstance(pb) => data::ContractUpdate::from_protobuf(pb)?.into(),
            Data::ContractDeleteInstance(pb) => data::ContractDelete::from_protobuf(pb)?.into(),
            Data::EthereumTransaction(pb) => data::Ethereum::from_protobuf(pb)?.into(),
            Data::CryptoApproveAllowance(pb) => {
                data::AccountAllowanceApprove::from_protobuf(pb)?.into()
            }
            Data::CryptoDeleteAllowance(pb) => {
                data::AccountAllowanceDelete::from_protobuf(pb)?.into()
            }
            Data::CryptoCreateAccount(pb) => data::AccountCreate::from_protobuf(pb)?.into(),
            Data::CryptoDelete(pb) => data::AccountDelete::from_protobuf(pb)?.into(),
            Data::CryptoTransfer(pb) => data::Transfer::from_protobuf(pb)?.into(),
            Data::CryptoUpdateAccount(pb) => data::AccountUpdate::from_protobuf(pb)?.into(),
            Data::FileAppend(pb) => data::FileAppend::from_protobuf(pb)?.into(),
            Data::FileCreate(pb) => data::FileCreate::from_protobuf(pb)?.into(),
            Data::FileDelete(pb) => data::FileDelete::from_protobuf(pb)?.into(),
            Data::FileUpdate(pb) => data::FileUpdate::from_protobuf(pb)?.into(),
            Data::UtilPrng(pb) => data::Prng::from_protobuf(pb)?.into(),
            Data::SystemDelete(pb) => data::SystemDelete::from_protobuf(pb)?.into(),
            Data::SystemUndelete(pb) => data::SystemUndelete::from_protobuf(pb)?.into(),
            Data::Freeze(pb) => data::Freeze::from_protobuf(pb)?.into(),
            Data::ConsensusCreateTopic(pb) => data::TopicCreate::from_protobuf(pb)?.into(),
            Data::ConsensusUpdateTopic(pb) => data::TopicUpdate::from_protobuf(pb)?.into(),
            Data::ConsensusDeleteTopic(pb) => data::TopicDelete::from_protobuf(pb)?.into(),
            Data::ConsensusSubmitMessage(pb) => data::TopicMessageSubmit::from_protobuf(pb)?.into(),
            Data::TokenCreation(pb) => data::TokenCreate::from_protobuf(pb)?.into(),
            Data::TokenFreeze(pb) => data::TokenFreeze::from_protobuf(pb)?.into(),
            Data::TokenUnfreeze(pb) => data::TokenUnfreeze::from_protobuf(pb)?.into(),
            Data::TokenGrantKyc(pb) => data::TokenGrantKyc::from_protobuf(pb)?.into(),
            Data::TokenRevokeKyc(pb) => data::TokenRevokeKyc::from_protobuf(pb)?.into(),
            Data::TokenDeletion(pb) => data::TokenDelete::from_protobuf(pb)?.into(),
            Data::TokenUpdate(pb) => data::TokenUpdate::from_protobuf(pb)?.into(),
            Data::TokenMint(pb) => data::TokenMint::from_protobuf(pb)?.into(),
            Data::TokenBurn(pb) => data::TokenBurn::from_protobuf(pb)?.into(),
            Data::TokenWipe(pb) => data::TokenWipe::from_protobuf(pb)?.into(),
            Data::TokenAssociate(pb) => data::TokenAssociate::from_protobuf(pb)?.into(),
            Data::TokenDissociate(pb) => data::TokenDissociate::from_protobuf(pb)?.into(),
            Data::TokenFeeScheduleUpdate(pb) => {
                data::TokenFeeScheduleUpdate::from_protobuf(pb)?.into()
            }
            Data::TokenPause(pb) => data::TokenPause::from_protobuf(pb)?.into(),
            Data::TokenUnpause(pb) => data::TokenUnpause::from_protobuf(pb)?.into(),
            Data::TokenReject(pb) => data::TokenReject::from_protobuf(pb)?.into(),
            Data::ScheduleCreate(pb) => data::ScheduleCreate::from_protobuf(pb)?.into(),
            Data::ScheduleDelete(pb) => data::ScheduleDelete::from_protobuf(pb)?.into(),
            Data::ScheduleSign(pb) => data::ScheduleSign::from_protobuf(pb)?.into(),
            Data::TokenUpdateNfts(pb) => data::TokenUpdateNfts::from_protobuf(pb)?.into(),
            Data::NodeCreate(pb) => data::NodeCreate::from_protobuf(pb)?.into(),
            Data::NodeUpdate(pb) => data::NodeUpdate::from_protobuf(pb)?.into(),
            Data::NodeDelete(pb) => data::NodeDelete::from_protobuf(pb)?.into(),
            Data::TokenAirdrop(pb) => data::TokenAirdrop::from_protobuf(pb)?.into(),
            Data::TokenClaimAirdrop(pb) => data::TokenClaimAirdrop::from_protobuf(pb)?.into(),
            Data::TokenCancelAirdrop(pb) => data::TokenCancelAirdrop::from_protobuf(pb)?.into(),
            Data::CryptoAddLiveHash(_) => {
                return Err(Error::from_protobuf(
                    "unsupported transaction `AddLiveHashTransaction`",
                ))
            }
            Data::CryptoDeleteLiveHash(_) => {
                return Err(Error::from_protobuf(
                    "unsupported transaction `DeleteLiveHashTransaction`",
                ))
            }
            Data::UncheckedSubmit(_) => {
                return Err(Error::from_protobuf(
                    "unsupported transaction `UncheckedSubmitTransaction`",
                ))
            }
            Data::NodeStakeUpdate(_) => {
                return Err(Error::from_protobuf(
                    "unsupported transaction `NodeStakeUpdateTransaction`",
                ))
            }
        };

        Ok(data)
    }
}

impl AnyTransactionData {
    // can't do anything about the # of lines, since this function just delegates to `data::_::from_protobuf`.
    #[allow(clippy::too_many_lines)]
    fn from_protobuf(data_chunks: ServicesTransactionDataList) -> crate::Result<Self> {
        fn try_into_only_element<T>(v: Vec<T>) -> crate::Result<T> {
            match <[T; 1]>::try_from(v) {
                Ok([element]) => Ok(element),
                _ => Err(Error::from_protobuf("chunks in non chunkable transaction")),
            }
        }

        let data = match data_chunks {
            ServicesTransactionDataList::AccountCreate(v) => {
                data::AccountCreate::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::AccountUpdate(v) => {
                data::AccountUpdate::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::AccountDelete(v) => {
                data::AccountDelete::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::AccountAllowanceApprove(v) => {
                data::AccountAllowanceApprove::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::AccountAllowanceDelete(v) => {
                data::AccountAllowanceDelete::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::ContractCreate(v) => {
                data::ContractCreate::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::ContractUpdate(v) => {
                data::ContractUpdate::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::ContractDelete(v) => {
                data::ContractDelete::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::ContractExecute(v) => {
                data::ContractExecute::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::Transfer(v) => {
                data::Transfer::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::TopicCreate(v) => {
                data::TopicCreate::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::TopicUpdate(v) => {
                data::TopicUpdate::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::TopicDelete(v) => {
                data::TopicDelete::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::TopicMessageSubmit(v) => {
                data::TopicMessageSubmit::from_protobuf(v)?.into()
            }
            ServicesTransactionDataList::FileAppend(v) => {
                data::FileAppend::from_protobuf(v)?.into()
            }
            ServicesTransactionDataList::FileCreate(v) => {
                data::FileCreate::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::FileUpdate(v) => {
                data::FileUpdate::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::FileDelete(v) => {
                data::FileDelete::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::TokenAssociate(v) => {
                data::TokenAssociate::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::TokenBurn(v) => {
                data::TokenBurn::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::TokenCreate(v) => {
                data::TokenCreate::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::TokenDelete(v) => {
                data::TokenDelete::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::TokenDissociate(v) => {
                data::TokenDissociate::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::TokenFeeScheduleUpdate(v) => {
                data::TokenFeeScheduleUpdate::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::TokenFreeze(v) => {
                data::TokenFreeze::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::TokenGrantKyc(v) => {
                data::TokenGrantKyc::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::TokenMint(v) => {
                data::TokenMint::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::TokenPause(v) => {
                data::TokenPause::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::TokenRevokeKyc(v) => {
                data::TokenRevokeKyc::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::TokenUnfreeze(v) => {
                data::TokenUnfreeze::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::TokenUnpause(v) => {
                data::TokenUnpause::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::TokenUpdate(v) => {
                data::TokenUpdate::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::TokenWipe(v) => {
                data::TokenWipe::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::SystemDelete(v) => {
                data::SystemDelete::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::SystemUndelete(v) => {
                data::SystemUndelete::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::Freeze(v) => {
                data::Freeze::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::ScheduleCreate(v) => {
                data::ScheduleCreate::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::ScheduleSign(v) => {
                data::ScheduleSign::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::ScheduleDelete(v) => {
                data::ScheduleDelete::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::Ethereum(v) => {
                data::Ethereum::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::UtilPrng(v) => {
                data::Prng::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::TokenUpdateNfts(v) => {
                data::TokenUpdateNfts::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::TokenReject(v) => {
                data::TokenReject::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::NodeCreate(v) => {
                data::NodeCreate::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::NodeUpdate(v) => {
                data::NodeUpdate::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::NodeDelete(v) => {
                data::NodeDelete::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::TokenAirdrop(v) => {
                data::TokenAirdrop::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::TokenClaimAirdrop(v) => {
                data::TokenClaimAirdrop::from_protobuf(try_into_only_element(v)?)?.into()
            }
            ServicesTransactionDataList::TokenCancelAirdrop(v) => {
                data::TokenCancelAirdrop::from_protobuf(try_into_only_element(v)?)?.into()
            }
        };

        Ok(data)
    }
}

impl AnyTransaction {
    pub(super) fn from_protobuf(
        first_body: services::TransactionBody,
        data_chunks: Vec<services::transaction_body::Data>,
    ) -> crate::Result<Self> {
        let transaction_id: Option<TransactionId> = match first_body.transaction_id {
            Some(id) => match TransactionId::from_protobuf(id) {
                Ok(id) => Some(id),
                Err(_) => None, // Or handle the error differently
            },
            None => None,
        };

        let node_account_ids = match first_body.node_account_id {
            Some(id) => match AccountId::from_protobuf(id) {
                Ok(id) => Some(vec![id]),
                Err(_) => None, // Handle error gracefully
            },
            None => None,
        };

        // Default to 0 tinybars if transaction_fee is missing
        let transaction_fee = Hbar::from_tinybars(first_body.transaction_fee as i64);

        // Check if data_chunks is empty and handle gracefully
        if data_chunks.is_empty() {
            return Err(Error::from_protobuf("Transaction data is missing"));
        }

        let transaction_data = match AnyTransactionData::from_protobuf(
            ServicesTransactionDataList::from_protobuf(data_chunks)?,
        ) {
            Ok(data) => data,
            Err(e) => return Err(e),
        };

        Ok(Transaction {
            body: TransactionBody {
                data: transaction_data,
                transaction_memo: first_body.memo,
                node_account_ids,
                transaction_valid_duration: first_body.transaction_valid_duration.map(Into::into),
                max_transaction_fee: Some(transaction_fee),
                transaction_id,
                operator: None,
                is_frozen: false,
                regenerate_transaction_id: Some(false),
                custom_fee_limits: first_body
                    .max_custom_fees
                    .into_iter()
                    .map(CustomFeeLimit::from_protobuf)
                    .collect::<Result<Vec<_>, _>>()?,
            },
            signers: Vec::new(),
            sources: None,
        })
    }
}

// sigh.
#[derive(Debug)]
enum ServicesTransactionDataList {
    AccountCreate(Vec<services::CryptoCreateTransactionBody>),
    AccountUpdate(Vec<services::CryptoUpdateTransactionBody>),
    AccountDelete(Vec<services::CryptoDeleteTransactionBody>),
    AccountAllowanceApprove(Vec<services::CryptoApproveAllowanceTransactionBody>),
    AccountAllowanceDelete(Vec<services::CryptoDeleteAllowanceTransactionBody>),
    ContractCreate(Vec<services::ContractCreateTransactionBody>),
    ContractUpdate(Vec<services::ContractUpdateTransactionBody>),
    ContractDelete(Vec<services::ContractDeleteTransactionBody>),
    ContractExecute(Vec<services::ContractCallTransactionBody>),
    Transfer(Vec<services::CryptoTransferTransactionBody>),
    TopicCreate(Vec<services::ConsensusCreateTopicTransactionBody>),
    TopicUpdate(Vec<services::ConsensusUpdateTopicTransactionBody>),
    TopicDelete(Vec<services::ConsensusDeleteTopicTransactionBody>),
    TopicMessageSubmit(Vec<services::ConsensusSubmitMessageTransactionBody>),
    FileAppend(Vec<services::FileAppendTransactionBody>),
    FileCreate(Vec<services::FileCreateTransactionBody>),
    FileUpdate(Vec<services::FileUpdateTransactionBody>),
    FileDelete(Vec<services::FileDeleteTransactionBody>),
    TokenAssociate(Vec<services::TokenAssociateTransactionBody>),
    TokenBurn(Vec<services::TokenBurnTransactionBody>),
    TokenCreate(Vec<services::TokenCreateTransactionBody>),
    TokenDelete(Vec<services::TokenDeleteTransactionBody>),
    TokenDissociate(Vec<services::TokenDissociateTransactionBody>),
    TokenFeeScheduleUpdate(Vec<services::TokenFeeScheduleUpdateTransactionBody>),
    TokenFreeze(Vec<services::TokenFreezeAccountTransactionBody>),
    TokenGrantKyc(Vec<services::TokenGrantKycTransactionBody>),
    TokenMint(Vec<services::TokenMintTransactionBody>),
    TokenPause(Vec<services::TokenPauseTransactionBody>),
    TokenRevokeKyc(Vec<services::TokenRevokeKycTransactionBody>),
    TokenUnfreeze(Vec<services::TokenUnfreezeAccountTransactionBody>),
    TokenUnpause(Vec<services::TokenUnpauseTransactionBody>),
    TokenUpdate(Vec<services::TokenUpdateTransactionBody>),
    TokenWipe(Vec<services::TokenWipeAccountTransactionBody>),
    TokenUpdateNfts(Vec<services::TokenUpdateNftsTransactionBody>),
    TokenReject(Vec<services::TokenRejectTransactionBody>),
    SystemDelete(Vec<services::SystemDeleteTransactionBody>),
    SystemUndelete(Vec<services::SystemUndeleteTransactionBody>),
    Freeze(Vec<services::FreezeTransactionBody>),
    ScheduleCreate(Vec<services::ScheduleCreateTransactionBody>),
    ScheduleSign(Vec<services::ScheduleSignTransactionBody>),
    ScheduleDelete(Vec<services::ScheduleDeleteTransactionBody>),
    Ethereum(Vec<services::EthereumTransactionBody>),
    UtilPrng(Vec<services::UtilPrngTransactionBody>),
    NodeCreate(Vec<services::NodeCreateTransactionBody>),
    NodeUpdate(Vec<services::NodeUpdateTransactionBody>),
    NodeDelete(Vec<services::NodeDeleteTransactionBody>),
    TokenAirdrop(Vec<services::TokenAirdropTransactionBody>),
    TokenClaimAirdrop(Vec<services::TokenClaimAirdropTransactionBody>),
    TokenCancelAirdrop(Vec<services::TokenCancelAirdropTransactionBody>),
}

impl FromProtobuf<Vec<services::transaction_body::Data>> for ServicesTransactionDataList {
    fn from_protobuf(pb: Vec<services::transaction_body::Data>) -> crate::Result<Self> {
        use services::transaction_body::Data;

        fn make_vec<T>(first: T, cap: usize) -> Vec<T> {
            let mut v = Vec::with_capacity(cap);
            v.push(first);
            v
        }

        let len = pb.len();

        let mut iter = pb.into_iter();

        let first = iter
            .next()
            .expect("empty transaction data list (should be handled earlier up the pipeline)");

        // note: this impl is what I (srr) believe to be the "best" impl
        let mut value = match first {
            Data::ContractCall(it) => Self::ContractExecute(make_vec(it, len)),
            Data::ContractCreateInstance(it) => Self::ContractCreate(make_vec(it, len)),
            Data::ContractUpdateInstance(it) => Self::ContractUpdate(make_vec(it, len)),
            Data::ContractDeleteInstance(it) => Self::ContractDelete(make_vec(it, len)),
            Data::EthereumTransaction(it) => Self::Ethereum(make_vec(it, len)),
            Data::CryptoApproveAllowance(it) => Self::AccountAllowanceApprove(make_vec(it, len)),
            Data::CryptoDeleteAllowance(it) => Self::AccountAllowanceDelete(make_vec(it, len)),
            Data::CryptoCreateAccount(it) => Self::AccountCreate(make_vec(it, len)),
            Data::CryptoDelete(it) => Self::AccountDelete(make_vec(it, len)),
            Data::CryptoTransfer(it) => Self::Transfer(make_vec(it, len)),
            Data::CryptoUpdateAccount(it) => Self::AccountUpdate(make_vec(it, len)),
            Data::FileAppend(it) => Self::FileAppend(make_vec(it, len)),
            Data::FileCreate(it) => Self::FileCreate(make_vec(it, len)),
            Data::FileDelete(it) => Self::FileDelete(make_vec(it, len)),
            Data::FileUpdate(it) => Self::FileUpdate(make_vec(it, len)),
            Data::SystemDelete(it) => Self::SystemDelete(make_vec(it, len)),
            Data::SystemUndelete(it) => Self::SystemUndelete(make_vec(it, len)),
            Data::Freeze(it) => Self::Freeze(make_vec(it, len)),
            Data::ConsensusCreateTopic(it) => Self::TopicCreate(make_vec(it, len)),
            Data::ConsensusUpdateTopic(it) => Self::TopicUpdate(make_vec(it, len)),
            Data::ConsensusDeleteTopic(it) => Self::TopicDelete(make_vec(it, len)),
            Data::ConsensusSubmitMessage(it) => Self::TopicMessageSubmit(make_vec(it, len)),
            Data::TokenCreation(it) => Self::TokenCreate(make_vec(it, len)),
            Data::TokenFreeze(it) => Self::TokenFreeze(make_vec(it, len)),
            Data::TokenUnfreeze(it) => Self::TokenUnfreeze(make_vec(it, len)),
            Data::TokenGrantKyc(it) => Self::TokenGrantKyc(make_vec(it, len)),
            Data::TokenRevokeKyc(it) => Self::TokenRevokeKyc(make_vec(it, len)),
            Data::TokenDeletion(it) => Self::TokenDelete(make_vec(it, len)),
            Data::TokenUpdate(it) => Self::TokenUpdate(make_vec(it, len)),
            Data::TokenMint(it) => Self::TokenMint(make_vec(it, len)),
            Data::TokenBurn(it) => Self::TokenBurn(make_vec(it, len)),
            Data::TokenWipe(it) => Self::TokenWipe(make_vec(it, len)),
            Data::TokenAssociate(it) => Self::TokenAssociate(make_vec(it, len)),
            Data::TokenDissociate(it) => Self::TokenDissociate(make_vec(it, len)),
            Data::TokenFeeScheduleUpdate(it) => Self::TokenFeeScheduleUpdate(make_vec(it, len)),
            Data::TokenPause(it) => Self::TokenPause(make_vec(it, len)),
            Data::TokenUnpause(it) => Self::TokenUnpause(make_vec(it, len)),
            Data::TokenReject(it) => Self::TokenReject(make_vec(it, len)),
            Data::ScheduleCreate(it) => Self::ScheduleCreate(make_vec(it, len)),
            Data::ScheduleDelete(it) => Self::ScheduleDelete(make_vec(it, len)),
            Data::ScheduleSign(it) => Self::ScheduleSign(make_vec(it, len)),
            Data::UtilPrng(it) => Self::UtilPrng(make_vec(it, len)),
            Data::TokenUpdateNfts(it) => Self::TokenUpdateNfts(make_vec(it, len)),
            Data::NodeCreate(it) => Self::NodeCreate(make_vec(it, len)),
            Data::NodeUpdate(it) => Self::NodeUpdate(make_vec(it, len)),
            Data::NodeDelete(it) => Self::NodeDelete(make_vec(it, len)),
            Data::TokenAirdrop(it) => Self::TokenAirdrop(make_vec(it, len)),
            Data::TokenClaimAirdrop(it) => Self::TokenClaimAirdrop(make_vec(it, len)),
            Data::TokenCancelAirdrop(it) => Self::TokenCancelAirdrop(make_vec(it, len)),
            Data::CryptoAddLiveHash(_) => {
                return Err(Error::from_protobuf(
                    "unsupported transaction `AddLiveHashTransaction`",
                ))
            }
            Data::CryptoDeleteLiveHash(_) => {
                return Err(Error::from_protobuf(
                    "unsupported transaction `DeleteLiveHashTransaction`",
                ))
            }

            Data::UncheckedSubmit(_) => {
                return Err(Error::from_protobuf(
                    "unsupported transaction `UncheckedSubmitTransaction`",
                ))
            }
            Data::NodeStakeUpdate(_) => {
                return Err(Error::from_protobuf(
                    "unsupported transaction `NodeStakeUpdateTransaction`",
                ))
            }
        };

        for transaction in iter {
            match (&mut value, transaction) {
                (Self::AccountCreate(v), Data::CryptoCreateAccount(element)) => v.push(element),
                (Self::AccountUpdate(v), Data::CryptoUpdateAccount(element)) => v.push(element),
                (Self::AccountDelete(v), Data::CryptoDelete(element)) => v.push(element),
                (Self::AccountAllowanceApprove(v), Data::CryptoApproveAllowance(element)) => {
                    v.push(element);
                }
                (Self::AccountAllowanceDelete(v), Data::CryptoDeleteAllowance(element)) => {
                    v.push(element);
                }
                (Self::ContractCreate(v), Data::ContractCreateInstance(element)) => v.push(element),
                (Self::ContractUpdate(v), Data::ContractUpdateInstance(element)) => v.push(element),
                (Self::ContractDelete(v), Data::ContractDeleteInstance(element)) => v.push(element),
                (Self::ContractExecute(v), Data::ContractCall(element)) => v.push(element),
                (Self::Transfer(v), Data::CryptoTransfer(element)) => v.push(element),
                (Self::TopicCreate(v), Data::ConsensusCreateTopic(element)) => v.push(element),
                (Self::TopicUpdate(v), Data::ConsensusUpdateTopic(element)) => v.push(element),
                (Self::TopicDelete(v), Data::ConsensusDeleteTopic(element)) => v.push(element),
                (Self::TopicMessageSubmit(v), Data::ConsensusSubmitMessage(element)) => {
                    v.push(element);
                }
                (Self::FileAppend(v), Data::FileAppend(element)) => v.push(element),
                (Self::FileCreate(v), Data::FileCreate(element)) => v.push(element),
                (Self::FileUpdate(v), Data::FileUpdate(element)) => v.push(element),
                (Self::FileDelete(v), Data::FileDelete(element)) => v.push(element),
                (Self::TokenAssociate(v), Data::TokenAssociate(element)) => v.push(element),
                (Self::TokenBurn(v), Data::TokenBurn(element)) => v.push(element),
                (Self::TokenCreate(v), Data::TokenCreation(element)) => v.push(element),
                (Self::TokenDelete(v), Data::TokenDeletion(element)) => v.push(element),
                (Self::TokenDissociate(v), Data::TokenDissociate(element)) => v.push(element),
                (Self::TokenFeeScheduleUpdate(v), Data::TokenFeeScheduleUpdate(element)) => {
                    v.push(element);
                }
                (Self::TokenFreeze(v), Data::TokenFreeze(element)) => v.push(element),
                (Self::TokenGrantKyc(v), Data::TokenGrantKyc(element)) => v.push(element),
                (Self::TokenMint(v), Data::TokenMint(element)) => v.push(element),
                (Self::TokenPause(v), Data::TokenPause(element)) => v.push(element),
                (Self::TokenRevokeKyc(v), Data::TokenRevokeKyc(element)) => v.push(element),
                (Self::TokenUnfreeze(v), Data::TokenUnfreeze(element)) => v.push(element),
                (Self::TokenUnpause(v), Data::TokenUnpause(element)) => v.push(element),
                (Self::TokenUpdate(v), Data::TokenUpdate(element)) => v.push(element),
                (Self::TokenWipe(v), Data::TokenWipe(element)) => v.push(element),
                (Self::SystemDelete(v), Data::SystemDelete(element)) => v.push(element),
                (Self::SystemUndelete(v), Data::SystemUndelete(element)) => v.push(element),
                (Self::Freeze(v), Data::Freeze(element)) => v.push(element),
                (Self::ScheduleCreate(v), Data::ScheduleCreate(element)) => v.push(element),
                (Self::ScheduleSign(v), Data::ScheduleSign(element)) => v.push(element),
                (Self::ScheduleDelete(v), Data::ScheduleDelete(element)) => v.push(element),
                (Self::Ethereum(v), Data::EthereumTransaction(element)) => v.push(element),
                (Self::UtilPrng(v), Data::UtilPrng(element)) => v.push(element),
                (Self::TokenAirdrop(v), Data::TokenAirdrop(element)) => v.push(element),
                (Self::TokenClaimAirdrop(v), Data::TokenClaimAirdrop(element)) => v.push(element),
                (Self::TokenCancelAirdrop(v), Data::TokenCancelAirdrop(element)) => v.push(element),

                _ => return Err(Error::from_protobuf("mismatched transaction types")),
            }
        }

        Ok(value)
    }
}

impl AnyTransaction {
    /// Attempt to downcast from any transaction to the given transaction kind.
    ///
    /// # Errors
    /// - If self doesn't match the given transaction type, the transaction is returned as-is.
    pub fn downcast<D>(self) -> Result<D, Self>
    where
        Self: DowncastOwned<D>,
    {
        self.downcast_owned()
    }
}

// this is macro worthy (there's like 40 transactions that all do this the exact same way)
/// Impl `DowncastOwned` for `AnyTransactionData`.
///
/// This macro will ensure you get all variants via a pattern match, if something changes (say, another transaction type is added), you'll get a `Missing match arm` compiler error.
macro_rules! impl_cast_any {
    ($($id:ident),+$(,)?) => {
        $(
            impl $crate::downcast::DowncastOwned<data::$id> for AnyTransactionData {
                fn downcast_owned(self) -> Result<data::$id, Self> {
                    let Self::$id(data) = self else {
                        return Err(self)
                    };

                    Ok(data)
                }
            }

            impl From<Transaction<data::$id>> for AnyTransaction {
                fn from(transaction: Transaction<data::$id>) -> Self {
                    Self {
                        body: TransactionBody {
                            data: transaction.body.data.into(),
                            node_account_ids: transaction.body.node_account_ids,
                            transaction_valid_duration: transaction.body.transaction_valid_duration,
                            max_transaction_fee: transaction.body.max_transaction_fee,
                            transaction_memo: transaction.body.transaction_memo,
                            transaction_id: transaction.body.transaction_id,
                            operator: transaction.body.operator,
                            is_frozen: transaction.body.is_frozen,
                            regenerate_transaction_id: transaction.body.regenerate_transaction_id,
                            custom_fee_limits: transaction.body.custom_fee_limits,
                        },
                        signers: transaction.signers,
                        sources: transaction.sources,
                    }
                }
            }
        )*

        #[allow(non_snake_case)]
        mod ___private_impl_cast_any {
            use super::AnyTransactionData;
            // ensure the what we were given is actually everything.
            fn _assert_exhaustive(d: AnyTransactionData)
            {
                match d {
                    $(AnyTransactionData::$id(_) => {},)+
                }
            }
        }
    };
}

impl_cast_any! {
    AccountCreate,
    AccountUpdate,
    AccountDelete,
    AccountAllowanceApprove,
    AccountAllowanceDelete,
    ContractCreate,
    ContractUpdate,
    ContractDelete,
    ContractExecute,
    Transfer,
    TopicCreate,
    TopicUpdate,
    TopicDelete,
    TopicMessageSubmit,
    FileAppend,
    FileCreate,
    FileUpdate,
    FileDelete,
    Prng,
    ScheduleCreate,
    ScheduleSign,
    ScheduleDelete,
    TokenAssociate,
    TokenBurn,
    TokenCreate,
    TokenDelete,
    TokenDissociate,
    TokenFeeScheduleUpdate,
    TokenFreeze,
    TokenGrantKyc,
    TokenMint,
    TokenPause,
    TokenRevokeKyc,
    TokenUnfreeze,
    TokenUnpause,
    TokenUpdate,
    TokenWipe,
    SystemDelete,
    SystemUndelete,
    Freeze,
    Ethereum,
    TokenUpdateNfts,
    NodeCreate,
    NodeUpdate,
    NodeDelete,
    TokenReject,
    TokenAirdrop,
    TokenClaimAirdrop,
    TokenCancelAirdrop
}
