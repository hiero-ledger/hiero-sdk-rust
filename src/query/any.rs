// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use tonic::transport::Channel;

use super::ToQueryProtobuf;
use crate::account::{
    AccountBalanceQueryData,
    AccountInfoQueryData,
    AccountRecordsQueryData,
};
use crate::contract::{
    ContractBytecodeQueryData,
    ContractCallQueryData,
    ContractInfoQueryData,
};
use crate::entity_id::ValidateChecksums;
use crate::file::{
    FileContentsQueryData,
    FileInfoQueryData,
};
use crate::ledger_id::RefLedgerId;
use crate::query::QueryExecute;
use crate::schedule::ScheduleInfoQueryData;
use crate::token::{
    TokenInfoQueryData,
    TokenNftInfoQueryData,
};
use crate::topic::TopicInfoQueryData;
use crate::transaction_receipt_query::TransactionReceiptQueryData;
use crate::{
    AccountBalance,
    AccountInfo,
    AllProxyStakers,
    BoxGrpcFuture,
    ContractFunctionResult,
    ContractInfo,
    Error,
    FileContentsResponse,
    FileInfo,
    FromProtobuf,
    Hbar,
    NetworkVersionInfo,
    NetworkVersionInfoQueryData,
    Query,
    ScheduleInfo,
    TokenInfo,
    TokenNftInfo,
    TopicInfo,
    TransactionReceipt,
    TransactionRecord,
    TransactionRecordQueryData,
};

/// Any possible query that may be executed on the Hiero network.
pub type AnyQuery = Query<AnyQueryData>;

#[derive(Debug, Clone)]
pub enum AnyQueryData {
    AccountBalance(AccountBalanceQueryData),
    AccountInfo(AccountInfoQueryData),
    AccountRecords(AccountRecordsQueryData),
    TransactionReceipt(TransactionReceiptQueryData),
    TransactionRecord(TransactionRecordQueryData),
    FileContents(FileContentsQueryData),
    FileInfo(FileInfoQueryData),
    ContractBytecode(ContractBytecodeQueryData),
    ContractCall(ContractCallQueryData),
    TokenInfo(TokenInfoQueryData),
    ContractInfo(ContractInfoQueryData),
    TokenNftInfo(TokenNftInfoQueryData),
    TopicInfo(TopicInfoQueryData),
    ScheduleInfo(ScheduleInfoQueryData),
    NetworkVersionInfo(NetworkVersionInfoQueryData),
}

// todo: strategically box fields of variants, rather than the entire structs.
/// Common response type for *all* queries.
#[derive(Debug, Clone)]
pub enum AnyQueryResponse {
    /// Response from [`AccountBalanceQuery`](crate::AccountBalanceQuery).
    AccountBalance(AccountBalance),

    /// Response from [`AccountInfoQuery`](crate::AccountInfoQuery).
    AccountInfo(AccountInfo),

    /// Response from [`AccountStakersQuery`](crate::AccountStakersQuery).
    AccountStakers(AllProxyStakers),

    /// Response from [`AccountRecordsQuery`](crate::AccountRecordsQuery).
    AccountRecords(Vec<TransactionRecord>),

    /// Response from [`TransactionReceiptQuery`](crate::TransactionReceiptQuery).
    TransactionReceipt(TransactionReceipt),

    /// Response from [`TransactionRecordQuery`](crate::TransactionRecordQuery).
    TransactionRecord(Box<TransactionRecord>),

    /// Response from [`FileContentsQuery`](crate::FileContentsQuery).
    FileContents(FileContentsResponse),

    /// Response from [`FileInfoQuery`](crate::FileInfoQuery).
    FileInfo(FileInfo),

    /// Response from [`ContractBytecodeQuery`](crate::ContractBytecodeQuery).
    ContractBytecode(Vec<u8>),

    /// Response from [`ContractCallQuery`](crate::ContractCallQuery).
    ContractCall(ContractFunctionResult),

    /// Response from [`TokenInfoQuery`](crate::TokenInfoQuery).
    TokenInfo(Box<TokenInfo>),

    /// Response from [`TopicInfoQuery`](crate::TopicInfoQuery).
    TopicInfo(TopicInfo),

    /// Response from [`ContractInfoQuery`](crate::ContractInfoQuery).
    ContractInfo(ContractInfo),

    /// Response from [`TokenNftInfoQuery`](crate::TokenNftInfoQuery).
    TokenNftInfo(TokenNftInfo),

    /// Response from [`ScheduleInfoQuery`](crate::ScheduleInfoQuery).
    ScheduleInfo(ScheduleInfo),

    /// Response from [`NetworkVersionInfoQuery`](crate::NetworkVersionInfoQuery).
    NetworkVersionInfo(NetworkVersionInfo),
}

impl ToQueryProtobuf for AnyQueryData {
    fn to_query_protobuf(&self, header: services::QueryHeader) -> services::Query {
        match self {
            Self::AccountBalance(data) => data.to_query_protobuf(header),
            Self::AccountInfo(data) => data.to_query_protobuf(header),
            Self::AccountRecords(data) => data.to_query_protobuf(header),
            Self::TransactionReceipt(data) => data.to_query_protobuf(header),
            Self::TransactionRecord(data) => data.to_query_protobuf(header),
            Self::FileContents(data) => data.to_query_protobuf(header),
            Self::FileInfo(data) => data.to_query_protobuf(header),
            Self::ContractBytecode(data) => data.to_query_protobuf(header),
            Self::ContractCall(data) => data.to_query_protobuf(header),
            Self::ContractInfo(data) => data.to_query_protobuf(header),
            Self::TokenNftInfo(data) => data.to_query_protobuf(header),
            Self::TokenInfo(data) => data.to_query_protobuf(header),
            Self::TopicInfo(data) => data.to_query_protobuf(header),
            Self::ScheduleInfo(data) => data.to_query_protobuf(header),
            Self::NetworkVersionInfo(data) => data.to_query_protobuf(header),
        }
    }
}

impl QueryExecute for AnyQueryData {
    type Response = AnyQueryResponse;

    fn is_payment_required(&self) -> bool {
        match self {
            Self::AccountInfo(query) => query.is_payment_required(),
            Self::AccountBalance(query) => query.is_payment_required(),
            Self::AccountRecords(query) => query.is_payment_required(),
            Self::TransactionReceipt(query) => query.is_payment_required(),
            Self::TransactionRecord(query) => query.is_payment_required(),
            Self::FileContents(query) => query.is_payment_required(),
            Self::FileInfo(query) => query.is_payment_required(),
            Self::ContractBytecode(query) => query.is_payment_required(),
            Self::ContractCall(query) => query.is_payment_required(),
            Self::ContractInfo(query) => query.is_payment_required(),
            Self::TokenNftInfo(query) => query.is_payment_required(),
            Self::TokenInfo(query) => query.is_payment_required(),
            Self::TopicInfo(query) => query.is_payment_required(),
            Self::ScheduleInfo(query) => query.is_payment_required(),
            Self::NetworkVersionInfo(query) => query.is_payment_required(),
        }
    }

    fn map_cost(&self, cost: Hbar) -> Hbar {
        match self {
            Self::AccountInfo(query) => query.map_cost(cost),
            Self::AccountBalance(query) => query.map_cost(cost),
            Self::AccountRecords(query) => query.map_cost(cost),
            Self::TransactionReceipt(query) => query.map_cost(cost),
            Self::TransactionRecord(query) => query.map_cost(cost),
            Self::FileContents(query) => query.map_cost(cost),
            Self::FileInfo(query) => query.map_cost(cost),
            Self::ContractBytecode(query) => query.map_cost(cost),
            Self::ContractCall(query) => query.map_cost(cost),
            Self::ContractInfo(query) => query.map_cost(cost),
            Self::TokenNftInfo(query) => query.map_cost(cost),
            Self::TokenInfo(query) => query.map_cost(cost),
            Self::TopicInfo(query) => query.map_cost(cost),
            Self::ScheduleInfo(query) => query.map_cost(cost),
            Self::NetworkVersionInfo(query) => query.map_cost(cost),
        }
    }

    fn execute(
        &self,
        channel: Channel,
        request: services::Query,
    ) -> BoxGrpcFuture<'_, services::Response> {
        match self {
            Self::AccountInfo(query) => query.execute(channel, request),
            Self::AccountBalance(query) => query.execute(channel, request),
            Self::AccountRecords(query) => query.execute(channel, request),
            Self::TransactionReceipt(query) => query.execute(channel, request),
            Self::TransactionRecord(query) => query.execute(channel, request),
            Self::FileContents(query) => query.execute(channel, request),
            Self::FileInfo(query) => query.execute(channel, request),
            Self::ContractBytecode(query) => query.execute(channel, request),
            Self::ContractCall(query) => query.execute(channel, request),
            Self::ContractInfo(query) => query.execute(channel, request),
            Self::TokenNftInfo(query) => query.execute(channel, request),
            Self::TokenInfo(query) => query.execute(channel, request),
            Self::TopicInfo(query) => query.execute(channel, request),
            Self::ScheduleInfo(query) => query.execute(channel, request),
            Self::NetworkVersionInfo(query) => query.execute(channel, request),
        }
    }

    fn should_retry_pre_check(&self, status: crate::Status) -> bool {
        match self {
            Self::AccountInfo(query) => query.should_retry_pre_check(status),
            Self::AccountBalance(query) => query.should_retry_pre_check(status),
            Self::AccountRecords(query) => query.should_retry_pre_check(status),
            Self::TransactionReceipt(query) => query.should_retry_pre_check(status),
            Self::TransactionRecord(query) => query.should_retry_pre_check(status),
            Self::FileContents(query) => query.should_retry_pre_check(status),
            Self::FileInfo(query) => query.should_retry_pre_check(status),
            Self::ContractBytecode(query) => query.should_retry_pre_check(status),
            Self::ContractCall(query) => query.should_retry_pre_check(status),
            Self::ContractInfo(query) => query.should_retry_pre_check(status),
            Self::TokenNftInfo(query) => query.should_retry_pre_check(status),
            Self::TokenInfo(query) => query.should_retry_pre_check(status),
            Self::TopicInfo(query) => query.should_retry_pre_check(status),
            Self::ScheduleInfo(query) => query.should_retry_pre_check(status),
            Self::NetworkVersionInfo(query) => query.should_retry_pre_check(status),
        }
    }

    fn should_retry(&self, response: &services::Response) -> bool {
        match self {
            Self::AccountInfo(query) => query.should_retry(response),
            Self::AccountBalance(query) => query.should_retry(response),
            Self::AccountRecords(query) => query.should_retry(response),
            Self::TransactionReceipt(query) => query.should_retry(response),
            Self::TransactionRecord(query) => query.should_retry(response),
            Self::FileContents(query) => query.should_retry(response),
            Self::FileInfo(query) => query.should_retry(response),
            Self::ContractBytecode(query) => query.should_retry(response),
            Self::ContractCall(query) => query.should_retry(response),
            Self::ContractInfo(query) => query.should_retry(response),
            Self::TokenNftInfo(query) => query.should_retry(response),
            Self::TokenInfo(query) => query.should_retry(response),
            Self::TopicInfo(query) => query.should_retry(response),
            Self::ScheduleInfo(query) => query.should_retry(response),
            Self::NetworkVersionInfo(query) => query.should_retry(response),
        }
    }

    fn transaction_id(&self) -> Option<crate::TransactionId> {
        match self {
            Self::AccountBalance(query) => query.transaction_id(),
            Self::AccountInfo(query) => query.transaction_id(),
            Self::AccountRecords(query) => query.transaction_id(),
            Self::TransactionReceipt(query) => query.transaction_id(),
            Self::TransactionRecord(query) => query.transaction_id(),
            Self::FileContents(query) => query.transaction_id(),
            Self::FileInfo(query) => query.transaction_id(),
            Self::ContractBytecode(query) => query.transaction_id(),
            Self::ContractCall(query) => query.transaction_id(),
            Self::TokenInfo(query) => query.transaction_id(),
            Self::ContractInfo(query) => query.transaction_id(),
            Self::TokenNftInfo(query) => query.transaction_id(),
            Self::TopicInfo(query) => query.transaction_id(),
            Self::ScheduleInfo(query) => query.transaction_id(),
            Self::NetworkVersionInfo(query) => query.transaction_id(),
        }
    }

    fn make_response(
        &self,
        response: services::response::Response,
    ) -> crate::Result<Self::Response> {
        match self {
            Self::AccountBalance(query) => {
                query.make_response(response).map(AnyQueryResponse::AccountBalance)
            }
            Self::AccountInfo(query) => {
                query.make_response(response).map(AnyQueryResponse::AccountInfo)
            }
            Self::AccountRecords(query) => {
                query.make_response(response).map(AnyQueryResponse::AccountRecords)
            }
            Self::TransactionReceipt(query) => {
                query.make_response(response).map(AnyQueryResponse::TransactionReceipt)
            }
            Self::TransactionRecord(query) => {
                query.make_response(response).map(Box::new).map(AnyQueryResponse::TransactionRecord)
            }
            Self::FileContents(query) => {
                query.make_response(response).map(AnyQueryResponse::FileContents)
            }
            Self::FileInfo(query) => query.make_response(response).map(AnyQueryResponse::FileInfo),
            Self::ContractBytecode(query) => {
                query.make_response(response).map(AnyQueryResponse::ContractBytecode)
            }
            Self::ContractCall(query) => {
                query.make_response(response).map(AnyQueryResponse::ContractCall)
            }
            Self::TokenInfo(query) => {
                query.make_response(response).map(Box::new).map(AnyQueryResponse::TokenInfo)
            }
            Self::ContractInfo(query) => {
                query.make_response(response).map(AnyQueryResponse::ContractInfo)
            }
            Self::TokenNftInfo(query) => {
                query.make_response(response).map(AnyQueryResponse::TokenNftInfo)
            }
            Self::TopicInfo(query) => {
                query.make_response(response).map(AnyQueryResponse::TopicInfo)
            }
            Self::ScheduleInfo(query) => {
                query.make_response(response).map(AnyQueryResponse::ScheduleInfo)
            }
            Self::NetworkVersionInfo(query) => {
                query.make_response(response).map(AnyQueryResponse::NetworkVersionInfo)
            }
        }
    }
}

impl ValidateChecksums for AnyQueryData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        match self {
            Self::AccountBalance(query) => query.validate_checksums(ledger_id),
            Self::AccountInfo(query) => query.validate_checksums(ledger_id),
            Self::AccountRecords(query) => query.validate_checksums(ledger_id),
            Self::TransactionReceipt(query) => query.validate_checksums(ledger_id),
            Self::TransactionRecord(query) => query.validate_checksums(ledger_id),
            Self::FileContents(query) => query.validate_checksums(ledger_id),
            Self::FileInfo(query) => query.validate_checksums(ledger_id),
            Self::ContractBytecode(query) => query.validate_checksums(ledger_id),
            Self::ContractCall(query) => query.validate_checksums(ledger_id),
            Self::TokenInfo(query) => query.validate_checksums(ledger_id),
            Self::ContractInfo(query) => query.validate_checksums(ledger_id),
            Self::TokenNftInfo(query) => query.validate_checksums(ledger_id),
            Self::TopicInfo(query) => query.validate_checksums(ledger_id),
            Self::ScheduleInfo(query) => query.validate_checksums(ledger_id),
            Self::NetworkVersionInfo(query) => query.validate_checksums(ledger_id),
        }
    }
}

impl FromProtobuf<services::response::Response> for AnyQueryResponse {
    fn from_protobuf(response: services::response::Response) -> crate::Result<Self>
    where
        Self: Sized,
    {
        use services::response::Response::*;

        Ok(match response {
            TransactionGetReceipt(_) => {
                Self::TransactionReceipt(TransactionReceipt::from_protobuf(response)?)
            }
            CryptoGetInfo(_) => Self::AccountInfo(AccountInfo::from_protobuf(response)?),
            CryptogetAccountBalance(_) => {
                Self::AccountBalance(AccountBalance::from_protobuf(response)?)
            }
            FileGetContents(_) => {
                Self::FileContents(FileContentsResponse::from_protobuf(response)?)
            }
            ContractGetBytecodeResponse(_) => {
                Self::ContractBytecode(Vec::<u8>::from_protobuf(response)?)
            }
            ContractCallLocal(_) => {
                Self::ContractCall(ContractFunctionResult::from_protobuf(response)?)
            }
            ContractGetInfo(_) => Self::ContractInfo(ContractInfo::from_protobuf(response)?),
            ConsensusGetTopicInfo(_) => Self::TopicInfo(TopicInfo::from_protobuf(response)?),
            ScheduleGetInfo(_) => Self::ScheduleInfo(ScheduleInfo::from_protobuf(response)?),
            CryptoGetProxyStakers(_) => {
                Self::AccountStakers(AllProxyStakers::from_protobuf(response)?)
            }
            CryptoGetAccountRecords(_) => {
                Self::AccountRecords(Vec::<TransactionRecord>::from_protobuf(response)?)
            }
            TransactionGetRecord(_) => {
                Self::TransactionRecord(Box::new(TransactionRecord::from_protobuf(response)?))
            }
            NetworkGetVersionInfo(_) => {
                Self::NetworkVersionInfo(NetworkVersionInfo::from_protobuf(response)?)
            }
            FileGetInfo(_) => Self::FileInfo(FileInfo::from_protobuf(response)?),
            TokenGetInfo(_) => Self::TokenInfo(Box::new(TokenInfo::from_protobuf(response)?)),
            TokenGetNftInfo(_) | TokenGetNftInfos(_) => {
                Self::TokenNftInfo(TokenNftInfo::from_protobuf(response)?)
            }
            // Unimplemented on hedera services
            TransactionGetFastRecord(_)
            | CryptoGetLiveHash(_)
            | GetBySolidityId(_)
            | TokenGetAccountNftInfos(_)
            | NetworkGetExecutionTime(_)
            | ContractGetRecordsResponse(_)
            | AccountDetails(_)
            | GetByKey(_) => unreachable!(),
        })
    }
}
