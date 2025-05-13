// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use prost::Message;
use time::OffsetDateTime;

use crate::protobuf::ToProtobuf;
use crate::{
    AccountId,
    FromProtobuf,
    LedgerId,
    NftId,
};

/// Response from [`TokenNftInfoQuery`][crate::TokenNftInfoQuery].

#[derive(Debug, Clone)]
pub struct TokenNftInfo {
    /// The ID of the NFT.
    pub nft_id: NftId,

    /// The current owner of the NFT.
    pub account_id: AccountId,

    /// Effective consensus timestamp at which the NFT was minted.
    pub creation_time: OffsetDateTime,

    /// The unique metadata of the NFT.
    pub metadata: Vec<u8>,

    /// If an allowance is granted for the NFT, its corresponding spender account.
    pub spender_id: Option<AccountId>,

    /// The ledger ID the response was returned from.
    pub ledger_id: LedgerId,
}

impl TokenNftInfo {
    /// Create a new `TokenInfo` from protobuf-encoded `bytes`.
    ///
    /// # Errors
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the bytes fails to produce a valid protobuf.
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the protobuf fails.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        FromProtobuf::<services::TokenNftInfo>::from_bytes(bytes)
    }

    /// Convert `self` to a protobuf-encoded [`Vec<u8>`].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        services::TokenNftInfo {
            nft_id: Some(self.nft_id.to_protobuf()),
            account_id: Some(self.account_id.to_protobuf()),
            creation_time: Some(self.creation_time.to_protobuf()),
            metadata: self.metadata.clone(),
            ledger_id: self.ledger_id.to_bytes(),
            spender_id: self.spender_id.to_protobuf(),
        }
        .encode_to_vec()
    }
}

impl FromProtobuf<services::response::Response> for TokenNftInfo {
    fn from_protobuf(pb: services::response::Response) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let pb = pb_getv!(pb, TokenGetNftInfo, services::response::Response);
        let nft = pb_getf!(pb, nft)?;
        Self::from_protobuf(nft)
    }
}

impl FromProtobuf<services::TokenNftInfo> for TokenNftInfo {
    fn from_protobuf(pb: services::TokenNftInfo) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let nft_id = pb_getf!(pb, nft_id)?;
        let account_id = pb_getf!(pb, account_id)?;
        let creation_time = pb.creation_time.unwrap();
        let metadata = pb.metadata;
        let spender_account_id = Option::from_protobuf(pb.spender_id)?;

        Ok(Self {
            nft_id: NftId::from_protobuf(nft_id)?,
            account_id: AccountId::from_protobuf(account_id)?,
            creation_time: OffsetDateTime::from(creation_time),
            metadata,
            spender_id: spender_account_id,
            ledger_id: LedgerId::from_bytes(pb.ledger_id),
        })
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hex_literal::hex;

    use crate::transaction::test_helpers::VALID_START;
    use crate::{
        AccountId,
        LedgerId,
        TokenId,
        TokenNftInfo,
    };

    fn make_info(spender_account_id: Option<AccountId>) -> TokenNftInfo {
        TokenNftInfo {
            nft_id: TokenId::new(1, 2, 3).nft(4),
            account_id: "5.6.7".parse().unwrap(),
            creation_time: VALID_START,
            metadata: hex!("deadbeef").into(),
            spender_id: spender_account_id,
            ledger_id: LedgerId::mainnet(),
        }
    }

    #[test]
    fn serialize() {
        let info = make_info(Some("8.9.10".parse().unwrap()));
        expect![[r#"
            Ok(
                TokenNftInfo {
                    nft_id: "1.2.3/4",
                    account_id: "5.6.7",
                    creation_time: 2019-04-01 22:42:22.0 +00:00:00,
                    metadata: [
                        222,
                        173,
                        190,
                        239,
                    ],
                    spender_id: Some(
                        "8.9.10",
                    ),
                    ledger_id: "mainnet",
                },
            )
        "#]]
        .assert_debug_eq(&TokenNftInfo::from_bytes(&info.to_bytes()));
    }

    #[test]
    fn serialize_no_spender() {
        let info = make_info(None);
        expect![[r#"
            Ok(
                TokenNftInfo {
                    nft_id: "1.2.3/4",
                    account_id: "5.6.7",
                    creation_time: 2019-04-01 22:42:22.0 +00:00:00,
                    metadata: [
                        222,
                        173,
                        190,
                        239,
                    ],
                    spender_id: None,
                    ledger_id: "mainnet",
                },
            )
        "#]]
        .assert_debug_eq(&TokenNftInfo::from_bytes(&info.to_bytes()));
    }
}
