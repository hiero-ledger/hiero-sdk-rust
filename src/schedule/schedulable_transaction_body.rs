use crate::proto::services;

use crate::protobuf::FromProtobuf;
use crate::transaction::{
    AnyTransactionData,
    ToSchedulableTransactionDataProtobuf,
    TransactionData,
};
use crate::Hbar;

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
    pub(super) use crate::file::{
        FileAppendTransactionData as FileAppend,
        FileCreateTransactionData as FileCreate,
        FileDeleteTransactionData as FileDelete,
        FileUpdateTransactionData as FileUpdate,
    };
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) use crate::prng_transaction::PrngTransactionData as Prng;
    pub(super) use crate::schedule::ScheduleDeleteTransactionData as ScheduleDelete;
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

// Hack: In rust this is this, but in swift this is just a less densely populatable `AnyTransaction`.
// As a result, the sources stuff doesn't carry over... That can be fixed after SDK level FFI is removed.
#[derive(Debug, Clone)]
pub(super) struct SchedulableTransactionBody {
    pub(super) data: Box<AnySchedulableTransactionData>,

    pub(super) max_transaction_fee: Option<Hbar>,

    pub(super) transaction_memo: String,
}

impl FromProtobuf<services::SchedulableTransactionBody> for SchedulableTransactionBody {
    fn from_protobuf(pb: services::SchedulableTransactionBody) -> crate::Result<Self> {
        Ok(Self {
            data: Box::new(AnySchedulableTransactionData::from_protobuf(pb_getf!(pb, data)?)?),
            max_transaction_fee: Some(Hbar::from_tinybars(pb.transaction_fee as i64)),
            transaction_memo: pb.memo,
        })
    }
}

impl SchedulableTransactionBody {
    pub(super) fn to_scheduled_body_protobuf(&self) -> services::SchedulableTransactionBody {
        services::SchedulableTransactionBody {
            data: Some(self.data.to_schedulable_transaction_data_protobuf()),
            memo: self.transaction_memo.clone(),
            // FIXME: does not use the client to default the max transaction fee
            transaction_fee: self
                .max_transaction_fee
                .unwrap_or_else(|| self.data.default_max_transaction_fee())
                .to_tinybars() as u64,
            max_custom_fees: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub(super) enum AnySchedulableTransactionData {
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
    #[cfg(not(target_arch = "wasm32"))]
    Prng(data::Prng),
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
    TokenUpdateNfts(data::TokenUpdateNfts),
    TokenReject(data::TokenReject),
    SystemDelete(data::SystemDelete),
    SystemUndelete(data::SystemUndelete),
    Freeze(data::Freeze),
    ScheduleDelete(data::ScheduleDelete),
    NodeCreate(data::NodeCreate),
    NodeUpdate(data::NodeUpdate),
    NodeDelete(data::NodeDelete),
    TokenAirdrop(data::TokenAirdrop),
    TokenClaimAirdrop(data::TokenClaimAirdrop),
    TokenCancelAirdrop(data::TokenCancelAirdrop),
}

impl AnySchedulableTransactionData {
    pub(super) fn default_max_transaction_fee(&self) -> Hbar {
        match self {
            AnySchedulableTransactionData::AccountCreate(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::AccountUpdate(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::AccountDelete(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::AccountAllowanceApprove(it) => {
                it.default_max_transaction_fee()
            }
            AnySchedulableTransactionData::AccountAllowanceDelete(it) => {
                it.default_max_transaction_fee()
            }
            AnySchedulableTransactionData::ContractCreate(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::ContractUpdate(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::ContractDelete(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::ContractExecute(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::Transfer(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::TopicCreate(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::TopicUpdate(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::TopicDelete(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::TopicMessageSubmit(it) => {
                it.default_max_transaction_fee()
            }
            AnySchedulableTransactionData::FileAppend(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::FileCreate(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::FileUpdate(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::FileDelete(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::TokenAssociate(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::TokenBurn(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::TokenCreate(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::TokenDelete(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::TokenDissociate(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::TokenFeeScheduleUpdate(it) => {
                it.default_max_transaction_fee()
            }
            AnySchedulableTransactionData::TokenFreeze(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::TokenGrantKyc(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::TokenMint(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::TokenUpdateNfts(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::TokenPause(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::TokenRevokeKyc(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::TokenUnfreeze(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::TokenUnpause(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::TokenUpdate(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::TokenWipe(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::TokenReject(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::SystemDelete(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::SystemUndelete(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::Freeze(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::ScheduleDelete(it) => it.default_max_transaction_fee(),
            #[cfg(not(target_arch = "wasm32"))]
            AnySchedulableTransactionData::Prng(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::NodeCreate(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::NodeUpdate(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::NodeDelete(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::TokenAirdrop(it) => it.default_max_transaction_fee(),
            AnySchedulableTransactionData::TokenClaimAirdrop(it) => {
                it.default_max_transaction_fee()
            }
            AnySchedulableTransactionData::TokenCancelAirdrop(it) => {
                it.default_max_transaction_fee()
            }
        }
    }
}

impl FromProtobuf<services::schedulable_transaction_body::Data> for AnySchedulableTransactionData {
    // large function that just delegates...
    #[allow(clippy::too_many_lines)]
    fn from_protobuf(pb: services::schedulable_transaction_body::Data) -> crate::Result<Self> {
        use services::schedulable_transaction_body::Data;
        match pb {
            Data::ContractCall(it) => {
                Ok(Self::ContractExecute(data::ContractExecute::from_protobuf(it)?))
            }
            Data::ContractCreateInstance(it) => {
                Ok(Self::ContractCreate(data::ContractCreate::from_protobuf(it)?))
            }
            Data::ContractUpdateInstance(it) => {
                Ok(Self::ContractUpdate(data::ContractUpdate::from_protobuf(it)?))
            }
            Data::ContractDeleteInstance(it) => {
                Ok(Self::ContractDelete(data::ContractDelete::from_protobuf(it)?))
            }
            Data::CryptoApproveAllowance(it) => {
                Ok(Self::AccountAllowanceApprove(data::AccountAllowanceApprove::from_protobuf(it)?))
            }
            Data::CryptoDeleteAllowance(it) => {
                Ok(Self::AccountAllowanceDelete(data::AccountAllowanceDelete::from_protobuf(it)?))
            }
            Data::CryptoCreateAccount(it) => {
                Ok(Self::AccountCreate(data::AccountCreate::from_protobuf(it)?))
            }
            Data::CryptoDelete(it) => {
                Ok(Self::AccountDelete(data::AccountDelete::from_protobuf(it)?))
            }
            Data::CryptoTransfer(it) => Ok(Self::Transfer(data::Transfer::from_protobuf(it)?)),
            Data::CryptoUpdateAccount(it) => {
                Ok(Self::AccountUpdate(data::AccountUpdate::from_protobuf(it)?))
            }
            Data::FileAppend(it) => Ok(Self::FileAppend(data::FileAppend::from_protobuf(it)?)),
            Data::FileCreate(it) => Ok(Self::FileCreate(data::FileCreate::from_protobuf(it)?)),
            Data::FileDelete(it) => Ok(Self::FileDelete(data::FileDelete::from_protobuf(it)?)),
            Data::FileUpdate(it) => Ok(Self::FileUpdate(data::FileUpdate::from_protobuf(it)?)),
            Data::SystemDelete(it) => {
                Ok(Self::SystemDelete(data::SystemDelete::from_protobuf(it)?))
            }
            Data::SystemUndelete(it) => {
                Ok(Self::SystemUndelete(data::SystemUndelete::from_protobuf(it)?))
            }
            Data::Freeze(it) => Ok(Self::Freeze(data::Freeze::from_protobuf(it)?)),
            Data::ConsensusCreateTopic(it) => {
                Ok(Self::TopicCreate(data::TopicCreate::from_protobuf(it)?))
            }
            Data::ConsensusUpdateTopic(it) => {
                Ok(Self::TopicUpdate(data::TopicUpdate::from_protobuf(it)?))
            }
            Data::ConsensusDeleteTopic(it) => {
                Ok(Self::TopicDelete(data::TopicDelete::from_protobuf(it)?))
            }
            Data::ConsensusSubmitMessage(it) => {
                Ok(Self::TopicMessageSubmit(data::TopicMessageSubmit::from_protobuf(it)?))
            }
            Data::TokenCreation(it) => Ok(Self::TokenCreate(data::TokenCreate::from_protobuf(it)?)),
            Data::TokenFreeze(it) => Ok(Self::TokenFreeze(data::TokenFreeze::from_protobuf(it)?)),
            Data::TokenUnfreeze(it) => {
                Ok(Self::TokenUnfreeze(data::TokenUnfreeze::from_protobuf(it)?))
            }
            Data::TokenGrantKyc(it) => {
                Ok(Self::TokenGrantKyc(data::TokenGrantKyc::from_protobuf(it)?))
            }
            Data::TokenRevokeKyc(it) => {
                Ok(Self::TokenRevokeKyc(data::TokenRevokeKyc::from_protobuf(it)?))
            }
            Data::TokenDeletion(it) => Ok(Self::TokenDelete(data::TokenDelete::from_protobuf(it)?)),
            Data::TokenUpdate(it) => Ok(Self::TokenUpdate(data::TokenUpdate::from_protobuf(it)?)),
            Data::TokenMint(it) => Ok(Self::TokenMint(data::TokenMint::from_protobuf(it)?)),
            Data::TokenBurn(it) => Ok(Self::TokenBurn(data::TokenBurn::from_protobuf(it)?)),
            Data::TokenWipe(it) => Ok(Self::TokenWipe(data::TokenWipe::from_protobuf(it)?)),
            Data::TokenAssociate(it) => {
                Ok(Self::TokenAssociate(data::TokenAssociate::from_protobuf(it)?))
            }
            Data::TokenDissociate(it) => {
                Ok(Self::TokenDissociate(data::TokenDissociate::from_protobuf(it)?))
            }
            Data::TokenFeeScheduleUpdate(it) => {
                Ok(Self::TokenFeeScheduleUpdate(data::TokenFeeScheduleUpdate::from_protobuf(it)?))
            }
            Data::TokenPause(it) => Ok(Self::TokenPause(data::TokenPause::from_protobuf(it)?)),
            Data::TokenUnpause(it) => {
                Ok(Self::TokenUnpause(data::TokenUnpause::from_protobuf(it)?))
            }
            Data::TokenReject(it) => Ok(Self::TokenReject(data::TokenReject::from_protobuf(it)?)),
            Data::ScheduleDelete(it) => {
                Ok(Self::ScheduleDelete(data::ScheduleDelete::from_protobuf(it)?))
            }
            #[cfg(not(target_arch = "wasm32"))]
            Data::UtilPrng(it) => Ok(Self::Prng(data::Prng::from_protobuf(it)?)),
            #[cfg(target_arch = "wasm32")]
            Data::UtilPrng(_) => {
                Err(crate::Error::from_protobuf("UtilPrng is not supported for WASM"))
            }
            Data::TokenUpdateNfts(it) => {
                Ok(Self::TokenUpdateNfts(data::TokenUpdateNfts::from_protobuf(it)?))
            }
            Data::NodeCreate(it) => Ok(Self::NodeCreate(data::NodeCreate::from_protobuf(it)?)),
            Data::NodeUpdate(it) => Ok(Self::NodeUpdate(data::NodeUpdate::from_protobuf(it)?)),
            Data::NodeDelete(it) => Ok(Self::NodeDelete(data::NodeDelete::from_protobuf(it)?)),
            Data::TokenAirdrop(it) => {
                Ok(Self::TokenAirdrop(data::TokenAirdrop::from_protobuf(it)?))
            }
            Data::TokenClaimAirdrop(it) => {
                Ok(Self::TokenClaimAirdrop(data::TokenClaimAirdrop::from_protobuf(it)?))
            }
            Data::TokenCancelAirdrop(it) => {
                Ok(Self::TokenCancelAirdrop(data::TokenCancelAirdrop::from_protobuf(it)?))
            }
        }
    }
}

impl ToSchedulableTransactionDataProtobuf for AnySchedulableTransactionData {
    // large function that just delegates...
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        match self {
            AnySchedulableTransactionData::AccountCreate(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::AccountUpdate(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::AccountDelete(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::AccountAllowanceApprove(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::AccountAllowanceDelete(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::ContractCreate(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::ContractUpdate(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::ContractDelete(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::ContractExecute(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::Transfer(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::TopicCreate(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::TopicUpdate(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::TopicDelete(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::TopicMessageSubmit(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::FileAppend(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::FileCreate(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::FileUpdate(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::FileDelete(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::TokenAssociate(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::TokenBurn(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::TokenCreate(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::TokenDelete(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::TokenDissociate(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::TokenFeeScheduleUpdate(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::TokenFreeze(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::TokenGrantKyc(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::TokenMint(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::TokenPause(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::TokenRevokeKyc(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::TokenUnfreeze(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::TokenUnpause(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::TokenUpdate(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::TokenWipe(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::SystemDelete(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::SystemUndelete(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::Freeze(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::ScheduleDelete(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            #[cfg(not(target_arch = "wasm32"))]
            AnySchedulableTransactionData::Prng(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::TokenUpdateNfts(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::TokenReject(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::NodeCreate(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::NodeUpdate(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::NodeDelete(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::TokenAirdrop(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::TokenClaimAirdrop(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
            AnySchedulableTransactionData::TokenCancelAirdrop(it) => {
                it.to_schedulable_transaction_data_protobuf()
            }
        }
    }
}

impl TryFrom<AnyTransactionData> for AnySchedulableTransactionData {
    type Error = crate::Error;

    fn try_from(value: AnyTransactionData) -> Result<Self, Self::Error> {
        match value {
            AnyTransactionData::AccountCreate(it) => Ok(Self::AccountCreate(it)),
            AnyTransactionData::AccountUpdate(it) => Ok(Self::AccountUpdate(it)),
            AnyTransactionData::AccountDelete(it) => Ok(Self::AccountDelete(it)),
            AnyTransactionData::AccountAllowanceApprove(it) => {
                Ok(Self::AccountAllowanceApprove(it))
            }
            AnyTransactionData::AccountAllowanceDelete(it) => Ok(Self::AccountAllowanceDelete(it)),
            AnyTransactionData::ContractCreate(it) => Ok(Self::ContractCreate(it)),
            AnyTransactionData::ContractUpdate(it) => Ok(Self::ContractUpdate(it)),
            AnyTransactionData::ContractDelete(it) => Ok(Self::ContractDelete(it)),
            AnyTransactionData::ContractExecute(it) => Ok(Self::ContractExecute(it)),
            AnyTransactionData::Transfer(it) => Ok(Self::Transfer(it)),
            AnyTransactionData::TopicCreate(it) => Ok(Self::TopicCreate(it)),
            AnyTransactionData::TopicUpdate(it) => Ok(Self::TopicUpdate(it)),
            AnyTransactionData::TopicDelete(it) => Ok(Self::TopicDelete(it)),
            AnyTransactionData::TopicMessageSubmit(it) => Ok(Self::TopicMessageSubmit(it)),
            AnyTransactionData::FileAppend(it) => Ok(Self::FileAppend(it)),
            AnyTransactionData::FileCreate(it) => Ok(Self::FileCreate(it)),
            AnyTransactionData::FileUpdate(it) => Ok(Self::FileUpdate(it)),
            AnyTransactionData::FileDelete(it) => Ok(Self::FileDelete(it)),
            AnyTransactionData::TokenAssociate(it) => Ok(Self::TokenAssociate(it)),
            AnyTransactionData::TokenBurn(it) => Ok(Self::TokenBurn(it)),
            AnyTransactionData::TokenCreate(it) => Ok(Self::TokenCreate(it)),
            AnyTransactionData::TokenDelete(it) => Ok(Self::TokenDelete(it)),
            AnyTransactionData::TokenDissociate(it) => Ok(Self::TokenDissociate(it)),
            AnyTransactionData::TokenFeeScheduleUpdate(it) => Ok(Self::TokenFeeScheduleUpdate(it)),
            AnyTransactionData::TokenFreeze(it) => Ok(Self::TokenFreeze(it)),
            AnyTransactionData::TokenGrantKyc(it) => Ok(Self::TokenGrantKyc(it)),
            AnyTransactionData::TokenMint(it) => Ok(Self::TokenMint(it)),
            AnyTransactionData::TokenPause(it) => Ok(Self::TokenPause(it)),
            AnyTransactionData::TokenRevokeKyc(it) => Ok(Self::TokenRevokeKyc(it)),
            AnyTransactionData::TokenUnfreeze(it) => Ok(Self::TokenUnfreeze(it)),
            AnyTransactionData::TokenUnpause(it) => Ok(Self::TokenUnpause(it)),
            AnyTransactionData::TokenUpdate(it) => Ok(Self::TokenUpdate(it)),
            AnyTransactionData::TokenWipe(it) => Ok(Self::TokenWipe(it)),
            AnyTransactionData::TokenReject(it) => Ok(Self::TokenReject(it)),
            AnyTransactionData::SystemDelete(it) => Ok(Self::SystemDelete(it)),
            AnyTransactionData::SystemUndelete(it) => Ok(Self::SystemUndelete(it)),
            AnyTransactionData::Freeze(it) => Ok(Self::Freeze(it)),
            AnyTransactionData::ScheduleDelete(it) => Ok(Self::ScheduleDelete(it)),
            #[cfg(not(target_arch = "wasm32"))]
            AnyTransactionData::Prng(it) => Ok(Self::Prng(it)),
            AnyTransactionData::TokenUpdateNfts(it) => Ok(Self::TokenUpdateNfts(it)),
            AnyTransactionData::NodeCreate(it) => Ok(Self::NodeCreate(it)),
            AnyTransactionData::NodeUpdate(it) => Ok(Self::NodeUpdate(it)),
            AnyTransactionData::NodeDelete(it) => Ok(Self::NodeDelete(it)),
            AnyTransactionData::TokenAirdrop(it) => Ok(Self::TokenAirdrop(it)),
            AnyTransactionData::TokenClaimAirdrop(it) => Ok(Self::TokenClaimAirdrop(it)),
            AnyTransactionData::TokenCancelAirdrop(it) => Ok(Self::TokenCancelAirdrop(it)),

            // fixme: basic-parse isn't suitable for this.
            AnyTransactionData::ScheduleCreate(_) => {
                Err(crate::Error::basic_parse("Cannot schedule `ScheduleCreateTransaction`"))
            }
            AnyTransactionData::ScheduleSign(_) => {
                Err(crate::Error::basic_parse("Cannot schedule `ScheduleSignTransaction`"))
            }
            AnyTransactionData::Ethereum(_) => {
                Err(crate::Error::basic_parse("Cannot schedule `EthereumTransaction`"))
            }
            #[cfg(not(target_arch = "wasm32"))]
            AnyTransactionData::Batch(_) => {
                Err(crate::Error::basic_parse("Cannot schedule `BatchTransaction`"))
            }
        }
    }
}

impl From<AnySchedulableTransactionData> for AnyTransactionData {
    fn from(value: AnySchedulableTransactionData) -> Self {
        match value {
            AnySchedulableTransactionData::AccountCreate(it) => Self::AccountCreate(it),
            AnySchedulableTransactionData::AccountUpdate(it) => Self::AccountUpdate(it),
            AnySchedulableTransactionData::AccountDelete(it) => Self::AccountDelete(it),
            AnySchedulableTransactionData::AccountAllowanceApprove(it) => {
                Self::AccountAllowanceApprove(it)
            }
            AnySchedulableTransactionData::AccountAllowanceDelete(it) => {
                Self::AccountAllowanceDelete(it)
            }
            AnySchedulableTransactionData::ContractCreate(it) => Self::ContractCreate(it),
            AnySchedulableTransactionData::ContractUpdate(it) => Self::ContractUpdate(it),
            AnySchedulableTransactionData::ContractDelete(it) => Self::ContractDelete(it),
            AnySchedulableTransactionData::ContractExecute(it) => Self::ContractExecute(it),
            AnySchedulableTransactionData::Transfer(it) => Self::Transfer(it),
            AnySchedulableTransactionData::TopicCreate(it) => Self::TopicCreate(it),
            AnySchedulableTransactionData::TopicUpdate(it) => Self::TopicUpdate(it),
            AnySchedulableTransactionData::TopicDelete(it) => Self::TopicDelete(it),
            AnySchedulableTransactionData::TopicMessageSubmit(it) => Self::TopicMessageSubmit(it),
            AnySchedulableTransactionData::FileAppend(it) => Self::FileAppend(it),
            AnySchedulableTransactionData::FileCreate(it) => Self::FileCreate(it),
            AnySchedulableTransactionData::FileUpdate(it) => Self::FileUpdate(it),
            AnySchedulableTransactionData::FileDelete(it) => Self::FileDelete(it),
            AnySchedulableTransactionData::TokenAssociate(it) => Self::TokenAssociate(it),
            AnySchedulableTransactionData::TokenBurn(it) => Self::TokenBurn(it),
            AnySchedulableTransactionData::TokenCreate(it) => Self::TokenCreate(it),
            AnySchedulableTransactionData::TokenDelete(it) => Self::TokenDelete(it),
            AnySchedulableTransactionData::TokenDissociate(it) => Self::TokenDissociate(it),
            AnySchedulableTransactionData::TokenFeeScheduleUpdate(it) => {
                Self::TokenFeeScheduleUpdate(it)
            }
            AnySchedulableTransactionData::TokenFreeze(it) => Self::TokenFreeze(it),
            AnySchedulableTransactionData::TokenGrantKyc(it) => Self::TokenGrantKyc(it),
            AnySchedulableTransactionData::TokenMint(it) => Self::TokenMint(it),
            AnySchedulableTransactionData::TokenPause(it) => Self::TokenPause(it),
            AnySchedulableTransactionData::TokenRevokeKyc(it) => Self::TokenRevokeKyc(it),
            AnySchedulableTransactionData::TokenUnfreeze(it) => Self::TokenUnfreeze(it),
            AnySchedulableTransactionData::TokenUnpause(it) => Self::TokenUnpause(it),
            AnySchedulableTransactionData::TokenUpdate(it) => Self::TokenUpdate(it),
            AnySchedulableTransactionData::TokenWipe(it) => Self::TokenWipe(it),
            AnySchedulableTransactionData::SystemDelete(it) => Self::SystemDelete(it),
            AnySchedulableTransactionData::SystemUndelete(it) => Self::SystemUndelete(it),
            AnySchedulableTransactionData::Freeze(it) => Self::Freeze(it),
            AnySchedulableTransactionData::ScheduleDelete(it) => Self::ScheduleDelete(it),
            #[cfg(not(target_arch = "wasm32"))]
            AnySchedulableTransactionData::Prng(it) => Self::Prng(it),
            AnySchedulableTransactionData::TokenUpdateNfts(it) => Self::TokenUpdateNfts(it),
            AnySchedulableTransactionData::NodeCreate(it) => Self::NodeCreate(it),
            AnySchedulableTransactionData::NodeUpdate(it) => Self::NodeUpdate(it),
            AnySchedulableTransactionData::NodeDelete(it) => Self::NodeDelete(it),
            AnySchedulableTransactionData::TokenReject(it) => Self::TokenReject(it),
            AnySchedulableTransactionData::TokenAirdrop(it) => Self::TokenAirdrop(it),
            AnySchedulableTransactionData::TokenClaimAirdrop(it) => Self::TokenClaimAirdrop(it),
            AnySchedulableTransactionData::TokenCancelAirdrop(it) => Self::TokenCancelAirdrop(it),
        }
    }
}
