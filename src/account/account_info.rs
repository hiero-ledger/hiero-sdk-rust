// SPDX-License-Identifier: Apache-2.0

use hiero_sdk_proto::services;
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
    KeyList,
    LedgerId,
    PublicKey,
    StakingInfo,
    Tinybar,
    TokenId,
};

/// Response from [`AccountInfoQuery`][crate::AccountInfoQuery].
#[derive(Debug, Clone)]
pub struct AccountInfo {
    /// The account that is being referenced.
    pub account_id: AccountId,

    /// The Contract Account ID comprising of both the contract instance and the cryptocurrency
    /// account owned by the contract instance, in the format used by Solidity.
    pub contract_account_id: String,

    /// If true, then this account has been deleted, it will disappear when it expires, and all
    /// transactions for it will fail except the transaction to extend its expiration date.
    pub is_deleted: bool,

    /// The Account ID of the account to which this is proxy staked.
    ///
    /// If `proxy_account_id` is `None`, an invalid account, or an account that isn't a node,
    /// then this account is automatically proxy staked to a node chosen by the network,
    /// but without earning payments.
    ///
    /// If the `proxy_account_id` account refuses to accept proxy staking, or if it is not currently
    /// running a node, then it will behave as if `proxy_account_id` is `None`.
    #[deprecated]
    pub proxy_account_id: Option<AccountId>,

    /// The total number of hbars proxy staked to this account.
    pub proxy_received: Hbar,

    /// The key for the account, which must sign in order to transfer out, or to modify the
    /// account in any way other than extending its expiration date.
    pub key: Key,

    /// Current balance of the referenced account.
    pub balance: Hbar,

    /// The threshold amount for which an account record is created (and this account
    /// charged for them) for any send/withdraw transaction.
    #[deprecated]
    pub send_record_threshold: Hbar,

    /// The threshold amount for which an account record is created
    /// (and this account charged for them) for any transaction above this amount.
    #[deprecated]
    pub receive_record_threshold: Hbar,

    /// If true, no transaction can transfer to this account unless signed by
    /// this account's key.
    pub is_receiver_signature_required: bool,

    /// The time at which this account is set to expire.
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

    /// All of the livehashes attached to the account.
    pub live_hashes: Vec<LiveHash>,

    /// All tokens associated with the account.
    pub token_relationships: Vec<TokenRelationship>,
}

/// A live hash attached to an account.
#[derive(Debug, Clone)]
pub struct LiveHash {
    /// The account with which this live hash is associated.
    pub account_id: AccountId,
    /// The SHA-384 hash of the content.
    pub hash: Vec<u8>,
    /// The keys that can delete this live hash.
    pub keys: KeyList,
    /// The duration for which this live hash is valid.
    pub duration: Duration,
}

/// The relationship between an account and a token.
#[derive(Debug, Clone)]
pub struct TokenRelationship {
    /// The token ID.
    pub token_id: TokenId,
    /// The token symbol.
    pub symbol: Option<String>,
    /// The account's balance of this token.
    pub balance: u64,
    /// Whether KYC is granted for this token.
    pub is_kyc_granted: Option<bool>,
    /// Whether the account is frozen for this token.
    pub is_frozen: Option<bool>,
    /// Whether this token was automatically associated.
    pub automatic_association: Option<bool>,
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
        #[allow(deprecated)]
        services::crypto_get_info_response::AccountInfo {
            account_id: Some(self.account_id.to_protobuf()),
            contract_account_id: self.contract_account_id.clone(),
            deleted: self.is_deleted,
            proxy_received: self.proxy_received.to_tinybars(),
            key: Some(self.key.to_protobuf()),
            balance: self.balance.to_tinybars() as u64,
            receiver_sig_required: self.is_receiver_signature_required,
            expiration_time: self.expiration_time.to_protobuf(),
            auto_renew_period: self.auto_renew_period.to_protobuf(),
            memo: self.account_memo.clone(),
            owned_nfts: self.owned_nfts as i64,
            max_automatic_token_associations: self.max_automatic_token_associations as i32,
            alias: self.alias_key.as_ref().map(ToProtobuf::to_bytes).unwrap_or_default(),
            ledger_id: self.ledger_id.to_bytes(),
            ethereum_nonce: self.ethereum_nonce as i64,
            staking_info: self.staking.to_protobuf(),

            // implemented deprecated fields
            proxy_account_id: self.proxy_account_id.to_protobuf(),
            generate_receive_record_threshold: self.receive_record_threshold.to_tinybars() as u64,
            generate_send_record_threshold: self.send_record_threshold.to_tinybars() as u64,

            // additional fields
            live_hashes: self.live_hashes.to_protobuf(),
            token_relationships: self.token_relationships.to_protobuf(),
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
        let staking = Option::from_protobuf(pb.staking_info)?;
        let live_hashes = Vec::from_protobuf(pb.live_hashes)?;
        let token_relationships = Vec::from_protobuf(pb.token_relationships)?;

        #[allow(deprecated)]
        Ok(Self {
            ledger_id,
            staking,
            live_hashes,
            token_relationships,
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

            // deprecated fields
            proxy_account_id: Option::from_protobuf(pb.proxy_account_id)?,
            send_record_threshold: Hbar::from_tinybars(pb.generate_send_record_threshold as i64),
            receive_record_threshold: Hbar::from_tinybars(
                pb.generate_receive_record_threshold as i64,
            ),
        })
    }
}

impl FromProtobuf<services::LiveHash> for LiveHash {
    fn from_protobuf(pb: services::LiveHash) -> crate::Result<Self> {
        Ok(Self {
            account_id: AccountId::from_protobuf(pb_getf!(pb, account_id)?)?,
            hash: pb.hash,
            keys: KeyList::from_protobuf(pb_getf!(pb, keys)?)?,
            duration: pb.duration.map(Into::into).unwrap_or(Duration::ZERO),
        })
    }
}

impl ToProtobuf for LiveHash {
    type Protobuf = services::LiveHash;

    fn to_protobuf(&self) -> Self::Protobuf {
        Self::Protobuf {
            account_id: Some(self.account_id.to_protobuf()),
            hash: self.hash.clone(),
            keys: Some(self.keys.to_protobuf()),
            duration: Some(self.duration.into()),
        }
    }
}

impl FromProtobuf<services::TokenRelationship> for TokenRelationship {
    fn from_protobuf(pb: services::TokenRelationship) -> crate::Result<Self> {
        use services::{
            TokenFreezeStatus,
            TokenKycStatus,
        };

        let is_kyc_granted = match pb.kyc_status() {
            TokenKycStatus::Granted => Some(true),
            TokenKycStatus::Revoked => Some(false),
            _ => None,
        };

        let is_frozen = match pb.freeze_status() {
            TokenFreezeStatus::Frozen => Some(true),
            TokenFreezeStatus::Unfrozen => Some(false),
            _ => None,
        };

        Ok(Self {
            token_id: TokenId::from_protobuf(pb_getf!(pb, token_id)?)?,
            symbol: if pb.symbol.is_empty() { None } else { Some(pb.symbol) },
            balance: pb.balance,
            is_kyc_granted,
            is_frozen,
            automatic_association: if pb.automatic_association { Some(true) } else { None },
        })
    }
}

impl ToProtobuf for TokenRelationship {
    type Protobuf = services::TokenRelationship;

    fn to_protobuf(&self) -> Self::Protobuf {
        use services::{
            TokenFreezeStatus,
            TokenKycStatus,
        };

        let kyc_status = match self.is_kyc_granted {
            Some(true) => TokenKycStatus::Granted as i32,
            Some(false) => TokenKycStatus::Revoked as i32,
            None => TokenKycStatus::KycNotApplicable as i32,
        };

        let freeze_status = match self.is_frozen {
            Some(true) => TokenFreezeStatus::Frozen as i32,
            Some(false) => TokenFreezeStatus::Unfrozen as i32,
            None => TokenFreezeStatus::FreezeNotApplicable as i32,
        };

        Self::Protobuf {
            token_id: Some(self.token_id.to_protobuf()),
            symbol: self.symbol.clone().unwrap_or_default(),
            balance: self.balance,
            kyc_status,
            freeze_status,
            decimals: 0, // Not stored in our structure
            automatic_association: self.automatic_association.unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use hiero_sdk_proto::services;
    use time::Duration;

    use super::{
        LiveHash,
        TokenRelationship,
    };
    use crate::protobuf::ToProtobuf;
    use crate::{
        AccountId,
        FromProtobuf,
        Key,
        KeyList,
        PublicKey,
        TokenId,
    };

    #[test]
    fn live_hash_to_from_protobuf() {
        let account_id = AccountId::new(1, 2, 3);
        let hash = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let public_key = PublicKey::from_str_ed25519(
            "302a300506032b65700321001234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        )
        .unwrap();
        let key = Key::Single(public_key);
        let keys = KeyList { keys: vec![key.clone()], threshold: None };
        let duration = Duration::seconds(3600);

        let live_hash = LiveHash { account_id, hash: hash.clone(), keys: keys.clone(), duration };

        let protobuf = live_hash.to_protobuf();
        let restored = LiveHash::from_protobuf(protobuf).unwrap();

        assert_eq!(restored.account_id, account_id);
        assert_eq!(restored.hash, hash);
        assert_eq!(restored.keys.keys.len(), keys.keys.len());
        assert_eq!(restored.duration, duration);
    }

    #[test]
    fn live_hash_fields() {
        let account_id = AccountId::new(5, 6, 7);
        let hash = vec![0xff, 0xee, 0xdd, 0xcc];
        let public_key = PublicKey::from_str_ed25519(
            "302a300506032b65700321001234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        )
        .unwrap();
        let key = Key::Single(public_key);
        let keys = KeyList { keys: vec![key], threshold: Some(1) };
        let duration = Duration::hours(24);

        let live_hash = LiveHash { account_id, hash: hash.clone(), keys: keys.clone(), duration };

        assert_eq!(live_hash.account_id.shard, 5);
        assert_eq!(live_hash.account_id.realm, 6);
        assert_eq!(live_hash.account_id.num, 7);
        assert_eq!(live_hash.hash, hash);
        assert_eq!(live_hash.keys.keys.len(), 1);
        assert_eq!(live_hash.keys.threshold, Some(1));
        assert_eq!(live_hash.duration, duration);
    }

    #[test]
    fn token_relationship_to_from_protobuf_with_kyc_granted() {
        let token_id = TokenId::new(1, 2, 3);
        let symbol = Some("TEST".to_string());
        let balance = 1000u64;
        let is_kyc_granted = Some(true);
        let is_frozen = Some(false);
        let automatic_association = Some(true);

        let relationship = TokenRelationship {
            token_id,
            symbol: symbol.clone(),
            balance,
            is_kyc_granted,
            is_frozen,
            automatic_association,
        };

        let protobuf = relationship.to_protobuf();
        assert_eq!(protobuf.kyc_status, services::TokenKycStatus::Granted as i32);
        assert_eq!(protobuf.freeze_status, services::TokenFreezeStatus::Unfrozen as i32);
        assert_eq!(protobuf.automatic_association, true);

        let restored = TokenRelationship::from_protobuf(protobuf).unwrap();

        assert_eq!(restored.token_id, token_id);
        assert_eq!(restored.symbol, symbol);
        assert_eq!(restored.balance, balance);
        assert_eq!(restored.is_kyc_granted, is_kyc_granted);
        assert_eq!(restored.is_frozen, is_frozen);
        assert_eq!(restored.automatic_association, automatic_association);
    }

    #[test]
    fn token_relationship_to_from_protobuf_with_kyc_revoked() {
        let token_id = TokenId::new(4, 5, 6);
        let relationship = TokenRelationship {
            token_id,
            symbol: Some("REVOKED".to_string()),
            balance: 500,
            is_kyc_granted: Some(false),
            is_frozen: Some(true),
            automatic_association: None,
        };

        let protobuf = relationship.to_protobuf();
        assert_eq!(protobuf.kyc_status, services::TokenKycStatus::Revoked as i32);
        assert_eq!(protobuf.freeze_status, services::TokenFreezeStatus::Frozen as i32);
        assert_eq!(protobuf.automatic_association, false);

        let restored = TokenRelationship::from_protobuf(protobuf).unwrap();
        assert_eq!(restored.is_kyc_granted, Some(false));
        assert_eq!(restored.is_frozen, Some(true));
        assert_eq!(restored.automatic_association, None);
    }

    #[test]
    fn token_relationship_to_from_protobuf_with_none_values() {
        let token_id = TokenId::new(7, 8, 9);
        let relationship = TokenRelationship {
            token_id,
            symbol: None,
            balance: 0,
            is_kyc_granted: None,
            is_frozen: None,
            automatic_association: None,
        };

        let protobuf = relationship.to_protobuf();
        assert_eq!(protobuf.kyc_status, services::TokenKycStatus::KycNotApplicable as i32);
        assert_eq!(protobuf.freeze_status, services::TokenFreezeStatus::FreezeNotApplicable as i32);
        assert_eq!(protobuf.symbol, "");
        assert_eq!(protobuf.automatic_association, false);

        let restored = TokenRelationship::from_protobuf(protobuf).unwrap();
        assert_eq!(restored.symbol, None);
        assert_eq!(restored.is_kyc_granted, None);
        assert_eq!(restored.is_frozen, None);
        assert_eq!(restored.automatic_association, None);
    }

    #[test]
    fn token_relationship_fields() {
        let token_id = TokenId::new(10, 11, 12);
        let symbol = Some("SYMBOL".to_string());
        let balance = 5000u64;

        let relationship = TokenRelationship {
            token_id,
            symbol: symbol.clone(),
            balance,
            is_kyc_granted: Some(true),
            is_frozen: Some(false),
            automatic_association: Some(true),
        };

        assert_eq!(relationship.token_id.shard, 10);
        assert_eq!(relationship.token_id.realm, 11);
        assert_eq!(relationship.token_id.num, 12);
        assert_eq!(relationship.symbol, symbol);
        assert_eq!(relationship.balance, balance);
        assert_eq!(relationship.is_kyc_granted, Some(true));
        assert_eq!(relationship.is_frozen, Some(false));
        assert_eq!(relationship.automatic_association, Some(true));
    }

    #[test]
    fn token_relationship_empty_symbol_converts_to_none() {
        let protobuf = services::TokenRelationship {
            token_id: Some(services::TokenId { shard_num: 1, realm_num: 2, token_num: 3 }),
            symbol: "".to_string(),
            balance: 100,
            kyc_status: services::TokenKycStatus::KycNotApplicable as i32,
            freeze_status: services::TokenFreezeStatus::FreezeNotApplicable as i32,
            decimals: 0,
            automatic_association: false,
        };

        let relationship = TokenRelationship::from_protobuf(protobuf).unwrap();
        assert_eq!(relationship.symbol, None);
    }

    #[test]
    fn token_relationship_automatic_association_false_converts_to_none() {
        let protobuf = services::TokenRelationship {
            token_id: Some(services::TokenId { shard_num: 1, realm_num: 2, token_num: 3 }),
            symbol: "TEST".to_string(),
            balance: 100,
            kyc_status: services::TokenKycStatus::KycNotApplicable as i32,
            freeze_status: services::TokenFreezeStatus::FreezeNotApplicable as i32,
            decimals: 0,
            automatic_association: false,
        };

        let relationship = TokenRelationship::from_protobuf(protobuf).unwrap();
        assert_eq!(relationship.automatic_association, None);
    }

    #[test]
    fn token_relationship_automatic_association_true_converts_to_some_true() {
        let protobuf = services::TokenRelationship {
            token_id: Some(services::TokenId { shard_num: 1, realm_num: 2, token_num: 3 }),
            symbol: "TEST".to_string(),
            balance: 100,
            kyc_status: services::TokenKycStatus::KycNotApplicable as i32,
            freeze_status: services::TokenFreezeStatus::FreezeNotApplicable as i32,
            decimals: 0,
            automatic_association: true,
        };

        let relationship = TokenRelationship::from_protobuf(protobuf).unwrap();
        assert_eq!(relationship.automatic_association, Some(true));
    }

    #[test]
    fn live_hash_zero_duration() {
        let account_id = AccountId::new(1, 2, 3);
        let hash = vec![1, 2, 3];
        let public_key = PublicKey::from_str_ed25519(
            "302a300506032b65700321001234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        )
        .unwrap();
        let key = Key::Single(public_key);
        let keys = KeyList { keys: vec![key], threshold: None };

        let protobuf = services::LiveHash {
            account_id: Some(account_id.to_protobuf()),
            hash: hash.clone(),
            keys: Some(keys.to_protobuf()),
            duration: None, // Test None duration
        };

        let live_hash = LiveHash::from_protobuf(protobuf).unwrap();
        assert_eq!(live_hash.duration, Duration::ZERO);
    }

    #[test]
    fn live_hash_empty_hash() {
        let account_id = AccountId::new(1, 2, 3);
        let hash = vec![]; // Empty hash
        let public_key = PublicKey::from_str_ed25519(
            "302a300506032b65700321001234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        )
        .unwrap();
        let key = Key::Single(public_key);
        let keys = KeyList { keys: vec![key], threshold: None };
        let duration = Duration::seconds(60);

        let live_hash = LiveHash { account_id, hash: hash.clone(), keys, duration };

        assert!(live_hash.hash.is_empty());

        let protobuf = live_hash.to_protobuf();
        assert!(protobuf.hash.is_empty());

        let restored = LiveHash::from_protobuf(protobuf).unwrap();
        assert!(restored.hash.is_empty());
    }

    #[test]
    fn token_relationship_zero_balance() {
        let token_id = TokenId::new(1, 2, 3);
        let relationship = TokenRelationship {
            token_id,
            symbol: Some("ZERO".to_string()),
            balance: 0,
            is_kyc_granted: Some(true),
            is_frozen: Some(false),
            automatic_association: None,
        };

        assert_eq!(relationship.balance, 0);

        let protobuf = relationship.to_protobuf();
        assert_eq!(protobuf.balance, 0);

        let restored = TokenRelationship::from_protobuf(protobuf).unwrap();
        assert_eq!(restored.balance, 0);
    }

    #[test]
    fn token_relationship_large_balance() {
        let token_id = TokenId::new(1, 2, 3);
        let large_balance = u64::MAX;
        let relationship = TokenRelationship {
            token_id,
            symbol: Some("MAX".to_string()),
            balance: large_balance,
            is_kyc_granted: None,
            is_frozen: None,
            automatic_association: None,
        };

        assert_eq!(relationship.balance, large_balance);

        let protobuf = relationship.to_protobuf();
        assert_eq!(protobuf.balance, large_balance);

        let restored = TokenRelationship::from_protobuf(protobuf).unwrap();
        assert_eq!(restored.balance, large_balance);
    }
}
