use std::ops::Not;

use hedera_proto::services;
use serde_with::base64::Base64;
use serde_with::{
    serde_as,
    skip_serializing_none,
};

use crate::{
    AccountId,
    ContractId,
    Error,
    FileId,
    FromProtobuf,
    ScheduleId,
    Status,
    TokenId,
    TopicId,
    TransactionId,
};

/// The summary of a transaction's result so far, if the transaction has reached consensus.
/// Response from [`TransactionReceiptQuery`][crate::TransactionReceiptQuery].
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReceipt {
    /// The consensus status of the transaction; is UNKNOWN if consensus has not been reached, or if
    /// the associated transaction did not have a valid payer signature.
    pub status: Status,

    /// In the receipt for an `AccountCreateTransaction`, the id of the newly created account.
    pub account_id: Option<AccountId>,

    /// In the receipt for a `FileCreateTransaction`, the id of the newly created file.
    pub file_id: Option<FileId>,

    /// In the receipt for a `ContractCreateTransaction`, the id of the newly created contract.
    pub contract_id: Option<ContractId>,

    // The exchange rates in effect when the transaction reached consensus.
    // TODO: pub exchange_rate: ExchangeRate,
    /// In the receipt for a `TopicCreateTransaction`, the id of the newly created topic.
    pub topic_id: Option<TopicId>,

    /// In the receipt for a `TopicMessageSubmitTransaction`, the new sequence number of the topic
    /// that received the message.
    pub topic_sequence_number: u64,

    // TODO: use a hash type (for display/debug/serialize purposes)
    /// In the receipt for a `TopicMessageSubmitTransaction`, the new running hash of the
    /// topic that received the message.
    #[serde_as(as = "Option<Base64>")]
    pub topic_running_hash: Option<Vec<u8>>,

    /// In the receipt of a `TopicMessageSubmitTransaction`, the version of the SHA-384
    /// digest used to update the running hash.
    pub topic_running_hash_version: u64,

    /// In the receipt for a `TokenCreateTransaction`, the id of the newly created token.
    pub token_id: Option<TokenId>,

    /// Populated in the receipt of `TokenMint`, `TokenWipe`, and `TokenBurn` transactions.
    ///
    /// For fungible tokens, the current total supply of this token.
    /// For non-fungible tokens, the total number of NFTs issued for a given token id.
    ///
    pub new_total_supply: u64,

    /// In the receipt for a `ScheduleCreateTransaction`, the id of the newly created schedule.
    pub schedule_id: Option<ScheduleId>,

    /// In the receipt of a `ScheduleCreateTransaction` or `ScheduleSignTransaction` that resolves
    /// to `Success`, the `TransactionId` that should be used to query for the receipt or
    /// record of the relevant scheduled transaction.
    pub scheduled_transaction_id: Option<TransactionId>,

    /// In the receipt of a `TokenMintTransaction` for tokens of type `NonFungibleUnique`,
    /// the serial numbers of the newly created NFTs.
    pub serial_numbers: Vec<i64>,

    /// The receipts of processing all transactions with the given id, in consensus time order.
    pub duplicates: Vec<TransactionReceipt>,

    /// The receipts (if any) of all child transactions spawned by the transaction with the
    /// given top-level id, in consensus order.
    pub children: Vec<TransactionReceipt>,
}

impl TransactionReceipt {
    fn from_protobuf(
        receipt: services::TransactionReceipt,
        duplicates: Vec<Self>,
        children: Vec<Self>,
    ) -> crate::Result<Self> {
        let status = if let Some(status) = Status::from_i32(receipt.status) {
            status
        } else {
            return Err(Error::ResponseStatusUnrecognized(receipt.status));
        };

        let account_id = receipt.account_id.map(AccountId::from_protobuf).transpose()?;
        let file_id = receipt.file_id.map(FileId::from_protobuf).transpose()?;
        let contract_id = receipt.contract_id.map(ContractId::from_protobuf).transpose()?;
        let topic_id = receipt.topic_id.map(TopicId::from_protobuf).transpose()?;
        let token_id = receipt.token_id.map(TokenId::from_protobuf).transpose()?;
        let schedule_id = receipt.schedule_id.map(ScheduleId::from_protobuf).transpose()?;

        let scheduled_transaction_id =
            receipt.scheduled_transaction_id.map(TransactionId::from_protobuf).transpose()?;

        Ok(Self {
            status,
            new_total_supply: receipt.new_total_supply,
            serial_numbers: receipt.serial_numbers,
            topic_running_hash_version: receipt.topic_running_hash_version,
            topic_sequence_number: receipt.topic_sequence_number,
            topic_running_hash: receipt
                .topic_running_hash
                .is_empty()
                .not()
                .then(|| receipt.topic_running_hash),
            scheduled_transaction_id,
            account_id,
            file_id,
            contract_id,
            topic_id,
            token_id,
            schedule_id,
            duplicates,
            children,
        })
    }
}

impl FromProtobuf<services::response::Response> for TransactionReceipt {
    fn from_protobuf(pb: services::response::Response) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let pb = pb_getv!(pb, TransactionGetReceipt, services::response::Response);

        let receipt = pb_getf!(pb, receipt)?;

        let duplicates = pb
            .duplicate_transaction_receipts
            .into_iter()
            .map(<TransactionReceipt as FromProtobuf<_>>::from_protobuf)
            .collect::<crate::Result<_>>()?;

        let children = pb
            .child_transaction_receipts
            .into_iter()
            .map(<TransactionReceipt as FromProtobuf<_>>::from_protobuf)
            .collect::<crate::Result<_>>()?;

        Self::from_protobuf(receipt, duplicates, children)
    }
}

impl FromProtobuf<services::TransactionReceipt> for TransactionReceipt {
    fn from_protobuf(receipt: services::TransactionReceipt) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Self::from_protobuf(receipt, Vec::new(), Vec::new())
    }
}
