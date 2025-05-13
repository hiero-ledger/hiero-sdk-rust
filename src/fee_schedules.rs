use hedera_proto::services;
use time::OffsetDateTime;

use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};

/// Contains the current and next [`FeeSchedule`]s.
///
/// See the [Hiero documentation]
///
/// [Hiero documentation]: https://docs.hedera.com/guides/docs/hedera-api/basic-types/currentandnextfeeschedule
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FeeSchedules {
    /// The current fee schedule.
    pub current: Option<FeeSchedule>,

    /// The next fee schedule.
    pub next: Option<FeeSchedule>,
}

impl FeeSchedules {
    /// Create a new `FeeSchedules` from protobuf-encoded `bytes`.
    ///
    /// # Errors
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the bytes fails to produce a valid protobuf.
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the protobuf fails.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        FromProtobuf::from_bytes(bytes)
    }

    /// Convert `self` to a protobuf-encoded [`Vec<u8>`].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        ToProtobuf::to_bytes(self)
    }
}

impl FromProtobuf<services::CurrentAndNextFeeSchedule> for FeeSchedules {
    fn from_protobuf(pb: services::CurrentAndNextFeeSchedule) -> crate::Result<Self> {
        Ok(Self {
            current: Option::from_protobuf(pb.current_fee_schedule)?,
            next: Option::from_protobuf(pb.next_fee_schedule)?,
        })
    }
}

impl ToProtobuf for FeeSchedules {
    type Protobuf = services::CurrentAndNextFeeSchedule;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::CurrentAndNextFeeSchedule {
            current_fee_schedule: self.current.to_protobuf(),
            next_fee_schedule: self.next.to_protobuf(),
        }
    }
}

/// The fee schedules for hedera functionality and the time at which this fee schedule will expire.
///
/// See the [Hiero documentation].
///
/// [Hiero documentation]: https://docs.hedera.com/guides/docs/hedera-api/basic-types/feeschedule
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FeeSchedule {
    /// The fee schedules per specific piece of functionality.
    pub transaction_fee_schedules: Vec<TransactionFeeSchedule>,

    /// The time this fee schedule will expire at.
    pub expiration_time: OffsetDateTime,
}

impl FeeSchedule {
    /// Create a new `FeeSchedule` from protobuf-encoded `bytes`.
    ///
    /// # Errors
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the bytes fails to produce a valid protobuf.
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the protobuf fails.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        FromProtobuf::from_bytes(bytes)
    }

    /// Convert `self` to a protobuf-encoded [`Vec<u8>`].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        ToProtobuf::to_bytes(self)
    }
}

impl FromProtobuf<services::FeeSchedule> for FeeSchedule {
    fn from_protobuf(pb: services::FeeSchedule) -> crate::Result<Self> {
        Ok(Self {
            transaction_fee_schedules: Vec::from_protobuf(pb.transaction_fee_schedule)?,
            expiration_time: pb_getf!(pb, expiry_time)?.into(),
        })
    }
}

impl ToProtobuf for FeeSchedule {
    type Protobuf = services::FeeSchedule;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::FeeSchedule {
            transaction_fee_schedule: self.transaction_fee_schedules.to_protobuf(),
            expiry_time: Some(self.expiration_time.into()),
        }
    }
}

/// The fees for a specific transaction or query based on the fee data.
///
/// See the [Hiero documentation].
///
/// [Hiero documentation]: https://docs.hedera.com/guides/docs/hedera-api/basic-types/transactionfeeschedule
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TransactionFeeSchedule {
    /// The request type that this fee schedule applies to.
    pub request_type: RequestType,

    /// Resource price coefficients.
    #[deprecated]
    pub fee_data: Option<Box<FeeData>>,

    /// Resource price coefficients.
    ///
    /// Supports subtype definition.
    pub fees: Vec<FeeData>,
}

impl TransactionFeeSchedule {
    /// Create a new `TransactionFeeSchedule` from protobuf-encoded `bytes`.
    ///
    /// # Errors
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the bytes fails to produce a valid protobuf.
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the protobuf fails.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        FromProtobuf::from_bytes(bytes)
    }

    /// Convert `self` to a protobuf-encoded [`Vec<u8>`].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        ToProtobuf::to_bytes(self)
    }
}

impl FromProtobuf<services::TransactionFeeSchedule> for TransactionFeeSchedule {
    #[allow(deprecated)]
    fn from_protobuf(pb: services::TransactionFeeSchedule) -> crate::Result<Self> {
        Ok(Self {
            request_type: RequestType::from_protobuf(pb.hedera_functionality())?,
            fee_data: Option::from_protobuf(pb.fee_data)?.map(Box::new),
            fees: Vec::from_protobuf(pb.fees)?,
        })
    }
}

impl ToProtobuf for TransactionFeeSchedule {
    type Protobuf = services::TransactionFeeSchedule;

    #[allow(deprecated)]
    fn to_protobuf(&self) -> Self::Protobuf {
        services::TransactionFeeSchedule {
            hedera_functionality: self.request_type.to_protobuf() as i32,
            fee_data: self.fee_data.to_protobuf(),
            fees: self.fees.to_protobuf(),
        }
    }
}

/// The functionality provided by Hiero.
#[derive(Debug, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum RequestType {
    /// UNSPECIFIED - Need to keep first value as unspecified because first element is ignored and not parsed (0 is ignored by parser)
    None,

    /// Transfer from one account to another.
    ///
    /// [`TransferTransaction`](crate::TransferTransaction).
    CryptoTransfer,

    /// Update an account.
    ///
    /// [`AccountUpdateTransaction`](crate::AccountUpdateTransaction).
    CryptoUpdate,

    /// Delete an account.
    ///
    /// [`AccountDeleteTransaction`](crate::AccountDeleteTransaction).
    CryptoDelete,

    /// Add a live hash to an account (not currently supported).
    CryptoAddLiveHash,

    /// Remove a live hash from an account (not currently supported).
    CryptoDeleteLiveHash,

    /// Execute a contract.
    ///
    /// [`ContractExecuteTransaction`](crate::ContractExecuteTransaction)
    ContractCall,

    /// Create a contract.
    ///
    /// [`ContractCreateTransaction`](crate::ContractCreateTransaction)
    ContractCreate,

    /// Update a contract.
    ///
    /// [`ContractUpdateTransaction`](crate::ContractUpdateTransaction)
    ContractUpdate,

    /// Create a file.
    FileCreate,

    /// Append data to a file.
    FileAppend,

    /// Update a file.
    FileUpdate,

    /// Delete a file.
    FileDelete,

    /// Query the balance for an account.
    CryptoGetAccountBalance,

    /// Query the records for an account.
    CryptoGetAccountRecords,

    /// Query the info for an account.
    CryptoGetInfo,

    /// Execute a contract locally on a node.
    ContractCallLocal,

    /// Query the info for a contract.
    ContractGetInfo,

    /// Query the bytecode for a contract.
    ContractGetBytecode,

    /// Lookup a contract by its solidity ID.
    GetBySolidityId,

    /// Lookup a contract by key.
    GetByKey,

    /// Query the live hashes for a account (not currently supported).
    CryptoGetLiveHash,

    /// Query the stakers for an account.
    CryptoGetStakers,

    /// Query the contents of a file.
    FileGetContents,

    /// Query the info for a file.
    FileGetInfo,

    /// Query the record for a transaction.
    TransactionGetRecord,

    /// Query the records for a contract.
    ContractGetRecords,

    /// Create an account.
    CryptoCreate,

    /// System delete a file or contract.
    SystemDelete,

    /// System undelete a file or contract.
    SystemUndelete,

    /// Delete a contract.
    ContractDelete,

    /// Freeze the network.
    Freeze,

    /// Creation of a transaction record..
    CreateTransactionRecord,

    /// Auto renewal of an account.
    CryptoAccountAutoRenew,

    /// Auto renewal of a contract
    ContractAutoRenew,

    /// Query the version info of the network.
    GetVersionInfo,

    /// Query the receipt for a transaction.
    TransactionGetReceipt,

    /// Create a topic.
    ConsensusCreateTopic,

    /// Update a topic.
    ConsensusUpdateTopic,

    /// Delete a topic.
    ConsensusDeleteTopic,

    /// Query the info for a topic.
    ConsensusGetTopicInfo,

    /// Submit a message to a topic.
    ConsensusSubmitMessage,

    /// Submit a transaction without validation.
    UncheckedSubmit,

    /// Create a topic.
    TokenCreate,

    /// Query the info for a token.
    TokenGetInfo,

    /// Freeze an account's balance of a token.
    TokenFreezeAccount,

    /// Unfreeze an account's balance of a token.
    TokenUnfreezeAccount,

    /// Grant KYC to an account for a token.
    TokenGrantKycToAccount,

    /// Revoke KYC from an account for a token.
    TokenRevokeKycFromAccount,

    /// Delete a token.
    TokenDelete,

    /// Update a token.
    TokenUpdate,

    /// Mint items on a token.
    TokenMint,

    /// Burn items from a token.
    TokenBurn,

    /// Wipe an account's balance of a token.
    TokenAccountWipe,

    /// Associate tokens to an account.
    TokenAssociateToAccount,

    /// Dissociate tokens from an account.
    TokenDissociateFromAccount,

    /// Create a schedule.
    ScheduleCreate,

    /// Delete a schedule.
    ScheduleDelete,

    /// Sign a schedule.
    ScheduleSign,

    /// Query the info for a schedule.
    ScheduleGetInfo,

    /// Query the info of held NFTs for an account.
    TokenGetAccountNftInfos,

    /// Query the info of an NFT for a token.
    TokenGetNftInfo,

    /// Query the info of NFT for a token.
    TokenGetNftInfos,

    /// Update the fee schedule for a token.
    TokenFeeScheduleUpdate,

    /// Query the execution time of a transaction.
    NetworkGetExecutionTime,

    /// Pause usage of a token.
    TokenPause,

    /// Unpause usage of a token.
    TokenUnpause,

    /// Approve an account spending another account's currency.
    CryptoApproveAllowance,

    /// Unapprove an account spending another account's currency.
    CryptoDeleteAllowance,

    /// Query the details for an account.
    GetAccountDetails,

    /// Execute an ethereum style transaction.
    EthereumTransaction,

    /// Update an account/contract's staked node.
    NodeStakeUpdate,

    /// Execute a PRNG transaction.
    UtilPrng,

    /// Get a record for a transaction.
    TransactionGetFastRecord,

    /// Update the metadata of one or more NFT's of a specific token type.
    TokenUpdateNfts,

    /// Create a new node.
    NodeCreate,

    /// Update an existing node.
    NodeUpdate,

    /// Delete a node.
    NodeDelete,

    /// Reject tokens.
    TokenReject,

    /// Airdrop tokens.
    TokenAirdrop,

    /// Claim airdrop tokens.
    TokenClaimAirdrop,

    /// Cancel airdrop tokens.
    TokenCancelAirdrop,
}

impl FromProtobuf<services::HederaFunctionality> for RequestType {
    fn from_protobuf(pb: services::HederaFunctionality) -> crate::Result<Self> {
        use services::HederaFunctionality;
        let value = match pb {
            HederaFunctionality::None => Self::None,
            HederaFunctionality::CryptoTransfer => Self::CryptoTransfer,
            HederaFunctionality::CryptoUpdate => Self::CryptoUpdate,
            HederaFunctionality::CryptoDelete => Self::CryptoDelete,
            HederaFunctionality::CryptoAddLiveHash => Self::CryptoAddLiveHash,
            HederaFunctionality::CryptoDeleteLiveHash => Self::CryptoDeleteLiveHash,
            HederaFunctionality::ContractCall => Self::ContractCall,
            HederaFunctionality::ContractCreate => Self::ContractCreate,
            HederaFunctionality::ContractUpdate => Self::ContractUpdate,
            HederaFunctionality::FileCreate => Self::FileCreate,
            HederaFunctionality::FileAppend => Self::FileAppend,
            HederaFunctionality::FileUpdate => Self::FileUpdate,
            HederaFunctionality::FileDelete => Self::FileDelete,
            HederaFunctionality::CryptoGetAccountBalance => Self::CryptoGetAccountBalance,
            HederaFunctionality::CryptoGetAccountRecords => Self::CryptoGetAccountRecords,
            HederaFunctionality::CryptoGetInfo => Self::CryptoGetInfo,
            HederaFunctionality::ContractCallLocal => Self::ContractCallLocal,
            HederaFunctionality::ContractGetInfo => Self::ContractGetInfo,
            HederaFunctionality::ContractGetBytecode => Self::ContractGetBytecode,
            HederaFunctionality::GetBySolidityId => Self::GetBySolidityId,
            HederaFunctionality::GetByKey => Self::GetByKey,
            HederaFunctionality::CryptoGetLiveHash => Self::CryptoGetLiveHash,
            HederaFunctionality::CryptoGetStakers => Self::CryptoGetStakers,
            HederaFunctionality::FileGetContents => Self::FileGetContents,
            HederaFunctionality::FileGetInfo => Self::FileGetInfo,
            HederaFunctionality::TransactionGetRecord => Self::TransactionGetRecord,
            HederaFunctionality::ContractGetRecords => Self::ContractGetRecords,
            HederaFunctionality::CryptoCreate => Self::CryptoCreate,
            HederaFunctionality::SystemDelete => Self::SystemDelete,
            HederaFunctionality::SystemUndelete => Self::SystemUndelete,
            HederaFunctionality::ContractDelete => Self::ContractDelete,
            HederaFunctionality::Freeze => Self::Freeze,
            HederaFunctionality::CreateTransactionRecord => Self::CreateTransactionRecord,
            HederaFunctionality::CryptoAccountAutoRenew => Self::CryptoAccountAutoRenew,
            HederaFunctionality::ContractAutoRenew => Self::ContractAutoRenew,
            HederaFunctionality::GetVersionInfo => Self::GetVersionInfo,
            HederaFunctionality::TransactionGetReceipt => Self::TransactionGetReceipt,
            HederaFunctionality::ConsensusCreateTopic => Self::ConsensusCreateTopic,
            HederaFunctionality::ConsensusUpdateTopic => Self::ConsensusUpdateTopic,
            HederaFunctionality::ConsensusDeleteTopic => Self::ConsensusDeleteTopic,
            HederaFunctionality::ConsensusGetTopicInfo => Self::ConsensusGetTopicInfo,
            HederaFunctionality::ConsensusSubmitMessage => Self::ConsensusSubmitMessage,
            HederaFunctionality::UncheckedSubmit => Self::UncheckedSubmit,
            HederaFunctionality::TokenCreate => Self::TokenCreate,
            HederaFunctionality::TokenGetInfo => Self::TokenGetInfo,
            HederaFunctionality::TokenFreezeAccount => Self::TokenFreezeAccount,
            HederaFunctionality::TokenUnfreezeAccount => Self::TokenUnfreezeAccount,
            HederaFunctionality::TokenGrantKycToAccount => Self::TokenGrantKycToAccount,
            HederaFunctionality::TokenRevokeKycFromAccount => Self::TokenRevokeKycFromAccount,
            HederaFunctionality::TokenDelete => Self::TokenDelete,
            HederaFunctionality::TokenUpdate => Self::TokenUpdate,
            HederaFunctionality::TokenMint => Self::TokenMint,
            HederaFunctionality::TokenBurn => Self::TokenBurn,
            HederaFunctionality::TokenAccountWipe => Self::TokenAccountWipe,
            HederaFunctionality::TokenAssociateToAccount => Self::TokenAssociateToAccount,
            HederaFunctionality::TokenDissociateFromAccount => Self::TokenDissociateFromAccount,
            HederaFunctionality::ScheduleCreate => Self::ScheduleCreate,
            HederaFunctionality::ScheduleDelete => Self::ScheduleDelete,
            HederaFunctionality::ScheduleSign => Self::ScheduleSign,
            HederaFunctionality::ScheduleGetInfo => Self::ScheduleGetInfo,
            HederaFunctionality::TokenGetAccountNftInfos => Self::TokenGetAccountNftInfos,
            HederaFunctionality::TokenGetNftInfo => Self::TokenGetNftInfo,
            HederaFunctionality::TokenGetNftInfos => Self::TokenGetNftInfos,
            HederaFunctionality::TokenFeeScheduleUpdate => Self::TokenFeeScheduleUpdate,
            HederaFunctionality::NetworkGetExecutionTime => Self::NetworkGetExecutionTime,
            HederaFunctionality::TokenPause => Self::TokenPause,
            HederaFunctionality::TokenUnpause => Self::TokenUnpause,
            HederaFunctionality::CryptoApproveAllowance => Self::CryptoApproveAllowance,
            HederaFunctionality::CryptoDeleteAllowance => Self::CryptoDeleteAllowance,
            HederaFunctionality::GetAccountDetails => Self::GetAccountDetails,
            HederaFunctionality::EthereumTransaction => Self::EthereumTransaction,
            HederaFunctionality::NodeStakeUpdate => Self::NodeStakeUpdate,
            HederaFunctionality::UtilPrng => Self::UtilPrng,
            HederaFunctionality::TransactionGetFastRecord => Self::TransactionGetFastRecord,
            HederaFunctionality::TokenUpdateNfts => Self::TokenUpdateNfts,
            HederaFunctionality::NodeCreate => Self::NodeCreate,
            HederaFunctionality::NodeUpdate => Self::NodeUpdate,
            HederaFunctionality::NodeDelete => Self::NodeDelete,
            HederaFunctionality::TokenReject => Self::TokenReject,
            HederaFunctionality::TokenAirdrop => Self::TokenAirdrop,
            HederaFunctionality::TokenClaimAirdrop => Self::TokenClaimAirdrop,
            HederaFunctionality::TokenCancelAirdrop => Self::TokenCancelAirdrop,
            _ => todo!(),
        };

        Ok(value)
    }
}

impl ToProtobuf for RequestType {
    type Protobuf = services::HederaFunctionality;

    fn to_protobuf(&self) -> Self::Protobuf {
        use services::HederaFunctionality;
        match self {
            Self::None => HederaFunctionality::None,
            Self::CryptoTransfer => HederaFunctionality::CryptoTransfer,
            Self::CryptoUpdate => HederaFunctionality::CryptoUpdate,
            Self::CryptoDelete => HederaFunctionality::CryptoDelete,
            Self::CryptoAddLiveHash => HederaFunctionality::CryptoAddLiveHash,
            Self::CryptoDeleteLiveHash => HederaFunctionality::CryptoDeleteLiveHash,
            Self::ContractCall => HederaFunctionality::ContractCall,
            Self::ContractCreate => HederaFunctionality::ContractCreate,
            Self::ContractUpdate => HederaFunctionality::ContractUpdate,
            Self::FileCreate => HederaFunctionality::FileCreate,
            Self::FileAppend => HederaFunctionality::FileAppend,
            Self::FileUpdate => HederaFunctionality::FileUpdate,
            Self::FileDelete => HederaFunctionality::FileDelete,
            Self::CryptoGetAccountBalance => HederaFunctionality::CryptoGetAccountBalance,
            Self::CryptoGetAccountRecords => HederaFunctionality::CryptoGetAccountRecords,
            Self::CryptoGetInfo => HederaFunctionality::CryptoGetInfo,
            Self::ContractCallLocal => HederaFunctionality::ContractCallLocal,
            Self::ContractGetInfo => HederaFunctionality::ContractGetInfo,
            Self::ContractGetBytecode => HederaFunctionality::ContractGetBytecode,
            Self::GetBySolidityId => HederaFunctionality::GetBySolidityId,
            Self::GetByKey => HederaFunctionality::GetByKey,
            Self::CryptoGetLiveHash => HederaFunctionality::CryptoGetLiveHash,
            Self::CryptoGetStakers => HederaFunctionality::CryptoGetStakers,
            Self::FileGetContents => HederaFunctionality::FileGetContents,
            Self::FileGetInfo => HederaFunctionality::FileGetInfo,
            Self::TransactionGetRecord => HederaFunctionality::TransactionGetRecord,
            Self::ContractGetRecords => HederaFunctionality::ContractGetRecords,
            Self::CryptoCreate => HederaFunctionality::CryptoCreate,
            Self::SystemDelete => HederaFunctionality::SystemDelete,
            Self::SystemUndelete => HederaFunctionality::SystemUndelete,
            Self::ContractDelete => HederaFunctionality::ContractDelete,
            Self::Freeze => HederaFunctionality::Freeze,
            Self::CreateTransactionRecord => HederaFunctionality::CreateTransactionRecord,
            Self::CryptoAccountAutoRenew => HederaFunctionality::CryptoAccountAutoRenew,
            Self::ContractAutoRenew => HederaFunctionality::ContractAutoRenew,
            Self::GetVersionInfo => HederaFunctionality::GetVersionInfo,
            Self::TransactionGetReceipt => HederaFunctionality::TransactionGetReceipt,
            Self::ConsensusCreateTopic => HederaFunctionality::ConsensusCreateTopic,
            Self::ConsensusUpdateTopic => HederaFunctionality::ConsensusUpdateTopic,
            Self::ConsensusDeleteTopic => HederaFunctionality::ConsensusDeleteTopic,
            Self::ConsensusGetTopicInfo => HederaFunctionality::ConsensusGetTopicInfo,
            Self::ConsensusSubmitMessage => HederaFunctionality::ConsensusSubmitMessage,
            Self::UncheckedSubmit => HederaFunctionality::UncheckedSubmit,
            Self::TokenCreate => HederaFunctionality::TokenCreate,
            Self::TokenGetInfo => HederaFunctionality::TokenGetInfo,
            Self::TokenFreezeAccount => HederaFunctionality::TokenFreezeAccount,
            Self::TokenUnfreezeAccount => HederaFunctionality::TokenUnfreezeAccount,
            Self::TokenGrantKycToAccount => HederaFunctionality::TokenGrantKycToAccount,
            Self::TokenRevokeKycFromAccount => HederaFunctionality::TokenRevokeKycFromAccount,
            Self::TokenDelete => HederaFunctionality::TokenDelete,
            Self::TokenUpdate => HederaFunctionality::TokenUpdate,
            Self::TokenMint => HederaFunctionality::TokenMint,
            Self::TokenBurn => HederaFunctionality::TokenBurn,
            Self::TokenAccountWipe => HederaFunctionality::TokenAccountWipe,
            Self::TokenAssociateToAccount => HederaFunctionality::TokenAssociateToAccount,
            Self::TokenDissociateFromAccount => HederaFunctionality::TokenDissociateFromAccount,
            Self::ScheduleCreate => HederaFunctionality::ScheduleCreate,
            Self::ScheduleDelete => HederaFunctionality::ScheduleDelete,
            Self::ScheduleSign => HederaFunctionality::ScheduleSign,
            Self::ScheduleGetInfo => HederaFunctionality::ScheduleGetInfo,
            Self::TokenGetAccountNftInfos => HederaFunctionality::TokenGetAccountNftInfos,
            Self::TokenGetNftInfo => HederaFunctionality::TokenGetNftInfo,
            Self::TokenGetNftInfos => HederaFunctionality::TokenGetNftInfos,
            Self::TokenFeeScheduleUpdate => HederaFunctionality::TokenFeeScheduleUpdate,
            Self::NetworkGetExecutionTime => HederaFunctionality::NetworkGetExecutionTime,
            Self::TokenPause => HederaFunctionality::TokenPause,
            Self::TokenUnpause => HederaFunctionality::TokenUnpause,
            Self::CryptoApproveAllowance => HederaFunctionality::CryptoApproveAllowance,
            Self::CryptoDeleteAllowance => HederaFunctionality::CryptoDeleteAllowance,
            Self::GetAccountDetails => HederaFunctionality::GetAccountDetails,
            Self::EthereumTransaction => HederaFunctionality::EthereumTransaction,
            Self::NodeStakeUpdate => HederaFunctionality::NodeStakeUpdate,
            Self::UtilPrng => HederaFunctionality::UtilPrng,
            Self::TransactionGetFastRecord => HederaFunctionality::TransactionGetFastRecord,
            Self::TokenUpdateNfts => HederaFunctionality::TokenUpdateNfts,
            Self::NodeCreate => HederaFunctionality::NodeCreate,
            Self::NodeUpdate => HederaFunctionality::NodeUpdate,
            Self::NodeDelete => HederaFunctionality::NodeDelete,
            Self::TokenReject => HederaFunctionality::TokenReject,
            Self::TokenAirdrop => HederaFunctionality::TokenAirdrop,
            Self::TokenClaimAirdrop => HederaFunctionality::TokenClaimAirdrop,
            Self::TokenCancelAirdrop => HederaFunctionality::TokenCancelAirdrop,
        }
    }
}

/// The total fees charged for a transaction, consisting of 3 parts:
/// The node fee, the network fee, and the service fee.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FeeData {
    /// Fee charged by the node for this functionality.
    pub node: FeeComponents,

    /// Fee charged by Hiero for network operations.
    pub network: FeeComponents,

    /// Fee charged by Hiero for providing the service.
    pub service: FeeComponents,

    /// A subtype distinguishing between different types of fee data
    /// correlating to the same hedera functionality.
    pub kind: FeeDataType,
}

impl FeeData {
    /// Create a new `FeeData` from protobuf-encoded `bytes`.
    ///
    /// # Errors
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the bytes fails to produce a valid protobuf.
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the protobuf fails.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        FromProtobuf::from_bytes(bytes)
    }

    /// Convert `self` to a protobuf-encoded [`Vec<u8>`].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        ToProtobuf::to_bytes(self)
    }
}

impl FromProtobuf<services::FeeData> for FeeData {
    fn from_protobuf(pb: services::FeeData) -> crate::Result<Self> {
        Ok(Self {
            node: FeeComponents::from_protobuf(pb_getf!(pb, nodedata)?)?,
            network: FeeComponents::from_protobuf(pb_getf!(pb, networkdata)?)?,
            service: FeeComponents::from_protobuf(pb_getf!(pb, servicedata)?)?,
            kind: FeeDataType::from_protobuf(pb.sub_type())?,
        })
    }
}

impl ToProtobuf for FeeData {
    type Protobuf = services::FeeData;
    fn to_protobuf(&self) -> Self::Protobuf {
        services::FeeData {
            nodedata: Some(self.node.to_protobuf()),
            networkdata: Some(self.network.to_protobuf()),
            servicedata: Some(self.service.to_protobuf()),
            sub_type: self.kind.to_protobuf() as i32,
        }
    }
}

/// The different components used for fee calculation.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FeeComponents {
    /// The minimum fee that needs to be paid.
    pub min: u64,

    /// The maximum fee that can be submitted.
    pub max: u64,

    /// A constant determined by the business to calculate the fee.
    pub constant: u64,

    /// The cost of each byte in a transaction.
    pub bandwidth_byte: u64,

    /// The cost of each signature in a transaction.
    pub verification: u64,

    /// Cost of storage measured in byte-hours.
    pub storage_byte_hour: u64,

    /// Cost of memory measured in byte-hours.
    pub ram_byte_hour: u64,

    /// Price of gas.
    pub contract_transaction_gas: u64,

    /// Cost per hbar transfered.
    ///
    /// fee = `floor(transfer_value in tinybars / (transfer_volume_hbar / 1000))`
    pub transfer_volume_hbar: u64,

    /// The price per byte of bandwidth spent for data retrieved from memory for a response.
    pub response_memory_byte: u64,

    /// The price per byte of bandwidth spent for data retrieved from disk for a response.
    pub response_disk_byte: u64,
}

impl FeeComponents {
    /// Create a new `FeeComponents` from protobuf-encoded `bytes`.
    ///
    /// # Errors
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the bytes fails to produce a valid protobuf.
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the protobuf fails.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        FromProtobuf::from_bytes(bytes)
    }

    /// Convert `self` to a protobuf-encoded [`Vec<u8>`].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        ToProtobuf::to_bytes(self)
    }
}

impl FromProtobuf<services::FeeComponents> for FeeComponents {
    fn from_protobuf(pb: services::FeeComponents) -> crate::Result<Self> {
        Ok(Self {
            min: pb.min as u64,
            max: pb.max as u64,
            constant: pb.constant as u64,
            bandwidth_byte: pb.bpt as u64,
            verification: pb.vpt as u64,
            storage_byte_hour: pb.sbh as u64,
            ram_byte_hour: pb.rbh as u64,
            contract_transaction_gas: pb.gas as u64,
            transfer_volume_hbar: pb.tv as u64,
            response_memory_byte: pb.bpr as u64,
            response_disk_byte: pb.sbpr as u64,
        })
    }
}

impl ToProtobuf for FeeComponents {
    type Protobuf = services::FeeComponents;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::FeeComponents {
            min: self.min as i64,
            max: self.max as i64,
            constant: self.constant as i64,
            bpt: self.bandwidth_byte as i64,
            vpt: self.verification as i64,
            rbh: self.ram_byte_hour as i64,
            sbh: self.storage_byte_hour as i64,
            gas: self.contract_transaction_gas as i64,
            tv: self.transfer_volume_hbar as i64,
            bpr: self.response_memory_byte as i64,
            sbpr: self.storage_byte_hour as i64,
        }
    }
}

/// Possible [`FeeData`] subtypes.
#[derive(Debug, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum FeeDataType {
    /// The resource prices have no special scope.
    Default,

    /// The resource prices are scoped to an operation on a fungible token.
    TokenFungibleCommon,

    /// The resource prices are scoped to an operation on a non-fungible token.
    TokenNonFungibleUnique,

    /// The resource prices are scoped to an operation on a fungible token with a custom fee schedule.
    TokenFungibleCommonWithCustomFees,

    /// The resource prices are scoped to an operation on a non-fungible token with a custom fee schedule.
    TokenNonFungibleUniqueWithCustomFees,

    /// The resource prices are scoped to a [`ScheduleCreateTransaction`](crate::ScheduleCreateTransaction)
    /// containing a [`ContractExecuteTransaction`](crate::ContractExecuteTransaction).
    ScheduleCreateContractCall,

    /// The resource prices are scoped to a [`TopicCreateTransaction`](crate::TopicCreateTransaction)
    /// with a custom fee schedule.
    TopicCreateWithCustomFees,
}

impl FromProtobuf<services::SubType> for FeeDataType {
    fn from_protobuf(pb: services::SubType) -> crate::Result<Self> {
        use services::SubType;
        let value = match pb {
            SubType::Default => Self::Default,
            SubType::TokenFungibleCommon => Self::TokenFungibleCommon,
            SubType::TokenNonFungibleUnique => Self::TokenNonFungibleUnique,
            SubType::TokenFungibleCommonWithCustomFees => Self::TokenFungibleCommonWithCustomFees,
            SubType::TokenNonFungibleUniqueWithCustomFees => {
                Self::TokenNonFungibleUniqueWithCustomFees
            }
            SubType::ScheduleCreateContractCall => Self::ScheduleCreateContractCall,
            SubType::TopicCreateWithCustomFees => Self::TopicCreateWithCustomFees,
        };

        Ok(value)
    }
}

impl ToProtobuf for FeeDataType {
    type Protobuf = services::SubType;

    fn to_protobuf(&self) -> Self::Protobuf {
        use services::SubType;
        match self {
            Self::Default => SubType::Default,
            Self::TokenFungibleCommon => SubType::TokenFungibleCommon,
            Self::TokenNonFungibleUnique => SubType::TokenNonFungibleUnique,
            Self::TokenFungibleCommonWithCustomFees => SubType::TokenFungibleCommonWithCustomFees,
            Self::TokenNonFungibleUniqueWithCustomFees => {
                SubType::TokenNonFungibleUniqueWithCustomFees
            }
            Self::ScheduleCreateContractCall => SubType::ScheduleCreateContractCall,
            Self::TopicCreateWithCustomFees => SubType::TopicCreateWithCustomFees,
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use time::OffsetDateTime;

    use crate::protobuf::ToProtobuf;
    use crate::{
        FeeComponents,
        FeeData,
        FeeSchedule,
        FeeSchedules,
        TransactionFeeSchedule,
    };

    const ZERO_FEES: FeeComponents = FeeComponents {
        min: 0,
        max: 0,
        constant: 0,
        bandwidth_byte: 0,
        verification: 0,
        storage_byte_hour: 0,
        ram_byte_hour: 0,
        contract_transaction_gas: 0,
        transfer_volume_hbar: 0,
        response_memory_byte: 0,
        response_disk_byte: 0,
    };

    fn make_fee_schedules() -> FeeSchedules {
        #[allow(deprecated)]
        FeeSchedules {
            current: Some(FeeSchedule {
                transaction_fee_schedules: Vec::from([TransactionFeeSchedule {
                    request_type: crate::RequestType::None,
                    fee_data: None,
                    fees: Vec::from([FeeData {
                        node: ZERO_FEES,
                        network: FeeComponents { min: 2, max: 5, ..ZERO_FEES },
                        service: ZERO_FEES,
                        kind: crate::FeeDataType::Default,
                    }]),
                }]),
                expiration_time: OffsetDateTime::from_unix_timestamp(1554158542).unwrap(),
            }),
            next: Some(FeeSchedule {
                transaction_fee_schedules: Vec::from([TransactionFeeSchedule {
                    request_type: crate::RequestType::None,
                    fee_data: None,
                    fees: Vec::from([FeeData {
                        node: FeeComponents { min: 1, max: 2, ..ZERO_FEES },
                        network: ZERO_FEES,
                        service: ZERO_FEES,
                        kind: crate::FeeDataType::Default,
                    }]),
                }]),
                expiration_time: OffsetDateTime::from_unix_timestamp(1554158222).unwrap(),
            }),
        }
    }

    #[test]
    fn serialize() {
        let schedules = make_fee_schedules();

        expect![[r#"
            CurrentAndNextFeeSchedule {
                current_fee_schedule: Some(
                    FeeSchedule {
                        transaction_fee_schedule: [
                            TransactionFeeSchedule {
                                hedera_functionality: None,
                                fee_data: None,
                                fees: [
                                    FeeData {
                                        nodedata: Some(
                                            FeeComponents {
                                                min: 0,
                                                max: 0,
                                                constant: 0,
                                                bpt: 0,
                                                vpt: 0,
                                                rbh: 0,
                                                sbh: 0,
                                                gas: 0,
                                                tv: 0,
                                                bpr: 0,
                                                sbpr: 0,
                                            },
                                        ),
                                        networkdata: Some(
                                            FeeComponents {
                                                min: 2,
                                                max: 5,
                                                constant: 0,
                                                bpt: 0,
                                                vpt: 0,
                                                rbh: 0,
                                                sbh: 0,
                                                gas: 0,
                                                tv: 0,
                                                bpr: 0,
                                                sbpr: 0,
                                            },
                                        ),
                                        servicedata: Some(
                                            FeeComponents {
                                                min: 0,
                                                max: 0,
                                                constant: 0,
                                                bpt: 0,
                                                vpt: 0,
                                                rbh: 0,
                                                sbh: 0,
                                                gas: 0,
                                                tv: 0,
                                                bpr: 0,
                                                sbpr: 0,
                                            },
                                        ),
                                        sub_type: Default,
                                    },
                                ],
                            },
                        ],
                        expiry_time: Some(
                            TimestampSeconds {
                                seconds: 1554158542,
                            },
                        ),
                    },
                ),
                next_fee_schedule: Some(
                    FeeSchedule {
                        transaction_fee_schedule: [
                            TransactionFeeSchedule {
                                hedera_functionality: None,
                                fee_data: None,
                                fees: [
                                    FeeData {
                                        nodedata: Some(
                                            FeeComponents {
                                                min: 1,
                                                max: 2,
                                                constant: 0,
                                                bpt: 0,
                                                vpt: 0,
                                                rbh: 0,
                                                sbh: 0,
                                                gas: 0,
                                                tv: 0,
                                                bpr: 0,
                                                sbpr: 0,
                                            },
                                        ),
                                        networkdata: Some(
                                            FeeComponents {
                                                min: 0,
                                                max: 0,
                                                constant: 0,
                                                bpt: 0,
                                                vpt: 0,
                                                rbh: 0,
                                                sbh: 0,
                                                gas: 0,
                                                tv: 0,
                                                bpr: 0,
                                                sbpr: 0,
                                            },
                                        ),
                                        servicedata: Some(
                                            FeeComponents {
                                                min: 0,
                                                max: 0,
                                                constant: 0,
                                                bpt: 0,
                                                vpt: 0,
                                                rbh: 0,
                                                sbh: 0,
                                                gas: 0,
                                                tv: 0,
                                                bpr: 0,
                                                sbpr: 0,
                                            },
                                        ),
                                        sub_type: Default,
                                    },
                                ],
                            },
                        ],
                        expiry_time: Some(
                            TimestampSeconds {
                                seconds: 1554158222,
                            },
                        ),
                    },
                ),
            }
        "#]]
        .assert_debug_eq(&schedules.to_protobuf());
    }

    #[test]
    fn to_from_bytes() {
        let a = make_fee_schedules();
        let b = FeeSchedules::from_bytes(&a.to_bytes()).unwrap();

        assert_eq!(a, b);
    }

    #[test]
    fn serialize_default() {
        let schedules = FeeSchedules { current: None, next: None };
        expect![[r#"
            CurrentAndNextFeeSchedule {
                current_fee_schedule: None,
                next_fee_schedule: None,
            }
        "#]]
        .assert_debug_eq(&schedules.to_protobuf());
    }

    #[test]
    fn to_from_bytes_default() {
        let a = FeeSchedules { current: None, next: None };
        let b = FeeSchedules::from_bytes(&a.to_bytes()).unwrap();

        assert_eq!(a, b);
    }
}
