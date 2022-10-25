/*
 * ‌
 * Hedera Rust SDK
 * ​
 * Copyright (C) 2022 - 2023 Hedera Hashgraph, LLC
 * ​
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * ‍
 */

use hedera_proto::services;
use prost::Message;
use time::{
    Duration,
    OffsetDateTime,
};

use crate::protobuf::ToProtobuf;
use crate::{
    AccountId,
    FromProtobuf,
    Hbar,
    Key,
    LedgerId,
    PublicKey,
    StakingInfo,
    Tinybar,
};

/// Response from [`AccountInfoQuery`][crate::AccountInfoQuery].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    /// The account that is being referenced.
    pub account_id: AccountId,

    /// The Contract Account ID comprising of both the contract instance and the cryptocurrency
    /// account owned by the contract instance, in the format used by Solidity.
    pub contract_account_id: String,

    /// If true, then this account has been deleted, it will disappear when it expires, and all
    /// transactions for it will fail except the transaction to extend its expiration date.
    pub is_deleted: bool,

    /// The total number of hbars proxy staked to this account.
    pub proxy_received: Hbar,

    /// The key for the account, which must sign in order to transfer out, or to modify the
    /// account in any way other than extending its expiration date.
    pub key: Key,

    /// Current balance of the referenced account.
    pub balance: Hbar,

    /// If true, no transaction can transfer to this account unless signed by
    /// this account's key.
    pub is_receiver_signature_required: bool,

    /// The TimeStamp time at which this account is set to expire.
    pub expiration_time: Option<OffsetDateTime>,

    /// The duration for expiration time will extend every this many seconds.
    pub auto_renew_period: Option<Duration>,

    /// The memo associated with the account.
    pub account_memo: String,

    /// The number of NFTs owned by this account
    pub owned_nfts: u64,

    /// The maximum number of tokens that an Account can be implicitly associated with.
    pub max_automatic_token_associations: u32,

    /// The alias of this account.
    pub alias_key: Option<PublicKey>,

    /// The ethereum transaction nonce associated with this account.
    pub ethereum_nonce: u64,

    /// The ledger ID the response was returned from.
    pub ledger_id: LedgerId,

    /// Staking metadata for this account.
    pub staking: Option<StakingInfo>,
}

impl AccountInfo {
    /// Create a new `AccountInfo` from protobuf-encoded `bytes`.
    ///
    /// # Errors
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the bytes fails to produce a valid protobuf.
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the protobuf fails.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        FromProtobuf::<services::crypto_get_info_response::AccountInfo>::from_bytes(bytes)
    }

    /// Convert `self` to a protobuf-encoded [`Vec<u8>`].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        services::crypto_get_info_response::AccountInfo {
            account_id: Some(self.account_id.to_protobuf()),
            contract_account_id: self.contract_account_id.clone(),
            deleted: self.is_deleted,
            proxy_received: self.proxy_received.to_tinybars(),
            key: Some(self.key.to_protobuf()),
            balance: self.balance.to_tinybars() as u64,
            receiver_sig_required: self.is_receiver_signature_required,
            expiration_time: self.expiration_time.as_ref().map(ToProtobuf::to_protobuf),
            auto_renew_period: self.auto_renew_period.as_ref().map(ToProtobuf::to_protobuf),
            memo: self.account_memo.clone(),
            owned_nfts: self.owned_nfts as i64,
            max_automatic_token_associations: self.max_automatic_token_associations as i32,
            alias: self.alias_key.as_ref().map(ToProtobuf::to_bytes).unwrap_or_default(),
            ledger_id: self.ledger_id.to_bytes(),
            ethereum_nonce: self.ethereum_nonce as i64,
            staking_info: self.staking.as_ref().map(ToProtobuf::to_protobuf),

            // unimplemented fields
            live_hashes: Vec::default(),
            token_relationships: Vec::default(),

            // deprecated fields
            ..Default::default()
        }
        .encode_to_vec()
    }
}

impl FromProtobuf<services::response::Response> for AccountInfo {
    fn from_protobuf(pb: services::response::Response) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let response = pb_getv!(pb, CryptoGetInfo, services::response::Response);
        let info = pb_getf!(response, account_info)?;
        Self::from_protobuf(info)
    }
}

impl FromProtobuf<services::crypto_get_info_response::AccountInfo> for AccountInfo {
    fn from_protobuf(pb: services::crypto_get_info_response::AccountInfo) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let key = pb_getf!(pb, key)?;
        let account_id = pb_getf!(pb, account_id)?;
        let alias_key = PublicKey::from_alias_bytes(&pb.alias)?;
        let ledger_id = LedgerId::from_bytes(pb.ledger_id);
        let staking = pb.staking_info.map(StakingInfo::from_protobuf).transpose()?;

        Ok(Self {
            ledger_id,
            staking,
            account_id: AccountId::from_protobuf(account_id)?,
            contract_account_id: pb.contract_account_id,
            is_deleted: pb.deleted,
            proxy_received: Hbar::from_tinybars(pb.proxy_received),
            key: Key::from_protobuf(key)?,
            balance: Hbar::from_tinybars(pb.balance as Tinybar),
            expiration_time: pb.expiration_time.map(Into::into),
            auto_renew_period: pb.auto_renew_period.map(Into::into),
            account_memo: pb.memo,
            owned_nfts: pb.owned_nfts as u64,
            max_automatic_token_associations: pb.max_automatic_token_associations as u32,
            alias_key,
            ethereum_nonce: pb.ethereum_nonce as u64,
            is_receiver_signature_required: pb.receiver_sig_required,
        })
    }
}
