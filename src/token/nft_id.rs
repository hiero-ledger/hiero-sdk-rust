// SPDX-License-Identifier: Apache-2.0

use std::fmt::{
    self,
    Debug,
    Display,
    Formatter,
};
use std::str::FromStr;

use hedera_proto::services;

use crate::entity_id::ValidateChecksums;
use crate::ledger_id::RefLedgerId;
use crate::{
    Client,
    Error,
    FromProtobuf,
    ToProtobuf,
    TokenId,
};

/// The unique identifier for a token on Hiero.
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct NftId {
    /// The (non-fungible) token of which this NFT is an instance.
    pub token_id: TokenId,

    /// The unique identifier for this instance.
    pub serial: u64,
}

impl NftId {
    /// Create a new `NftId` from protobuf-encoded `bytes`.
    ///
    /// # Errors
    /// - [`Error::FromProtobuf`] if decoding the bytes fails to produce a valid protobuf.
    /// - [`Error::FromProtobuf`] if decoding the protobuf fails.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        FromProtobuf::from_bytes(bytes)
    }

    /// Convert `self` to a protobuf-encoded [`Vec<u8>`].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        ToProtobuf::to_bytes(self)
    }

    /// Convert `self` to a string with a valid checksum.
    #[must_use]
    pub fn to_string_with_checksum(&self, client: &Client) -> String {
        format!("{}/{}", self.token_id.to_string_with_checksum(client), self.serial)
    }
}

impl Debug for NftId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl Display for NftId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.token_id, self.serial)
    }
}

impl FromProtobuf<services::NftId> for NftId {
    fn from_protobuf(pb: services::NftId) -> crate::Result<Self> {
        Ok(Self {
            token_id: TokenId::from_protobuf(pb_getf!(pb, token_id)?)?,
            serial: pb.serial_number as u64,
        })
    }
}

impl ToProtobuf for NftId {
    type Protobuf = services::NftId;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::NftId {
            token_id: Some(self.token_id.to_protobuf()),
            serial_number: self.serial as i64,
        }
    }
}

impl FromStr for NftId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (token_id, serial) = s
            .rsplit_once('/')
            .or_else(|| s.rsplit_once('@'))
            .ok_or_else(|| Error::basic_parse("unexpected NftId format - expected [token_id]/[serial_number] or [token_id]@[serial_number]"))?;

        let serial = serial.parse().map_err(|_| Error::basic_parse("invalid serial number"))?;

        Ok(Self { token_id: TokenId::from_str(token_id)?, serial })
    }
}

impl From<(TokenId, u64)> for NftId {
    fn from(tuple: (TokenId, u64)) -> Self {
        Self { token_id: tuple.0, serial: tuple.1 }
    }
}

impl ValidateChecksums for NftId {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.token_id.validate_checksums(ledger_id)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use hedera_proto::services;

    use crate::ledger_id::RefLedgerId;
    use crate::token::nft_id::NftId;
    use crate::{
        FromProtobuf,
        ToProtobuf,
        TokenId,
        ValidateChecksums,
    };

    #[test]
    fn it_can_convert_to_protobuf() -> anyhow::Result<()> {
        let nft_id = NftId { token_id: TokenId::from(1), serial: 1 };

        let nft_id_proto = nft_id.to_protobuf();

        assert_eq!(nft_id.serial, nft_id_proto.serial_number as u64);
        assert_eq!(nft_id.token_id.to_protobuf(), nft_id_proto.token_id.unwrap());

        Ok(())
    }

    #[test]
    fn it_can_create_from_protobuf() -> anyhow::Result<()> {
        let nft_id_proto =
            services::NftId { token_id: Some(TokenId::from(1).to_protobuf()), serial_number: 1 };

        let nft_id = NftId::from_protobuf(nft_id_proto.clone())?;

        assert_eq!(nft_id.serial, nft_id_proto.serial_number as u64);
        assert_eq!(nft_id.token_id, TokenId::from_protobuf(nft_id_proto.token_id.unwrap())?);

        Ok(())
    }

    #[test]
    fn from_str() -> anyhow::Result<()> {
        // Test '/' format parsing
        let nft_id_slash_str = "0.0.123/456";

        let nft_id_from_slash_str = NftId::from_str(nft_id_slash_str)?;

        assert_eq!(nft_id_from_slash_str.serial, 456);
        assert_eq!(nft_id_from_slash_str.token_id.num, 123);

        // Test '@' format parsing
        let nft_id_at_str = "0.0.123@456";

        let nft_id_from_at_str = NftId::from_str(nft_id_at_str)?;

        assert_eq!(nft_id_from_at_str.serial, 456);
        assert_eq!(nft_id_from_at_str.token_id.num, 123);

        Ok(())
    }

    #[test]
    fn to_string() {
        assert_eq!(TokenId::new(0, 0, 123).nft(456).to_string(), "0.0.123/456");
    }

    #[test]
    fn parse_with_checksum_on_mainnet() {
        let nft_id = NftId::from_str("0.0.123-vfmkw/7584").unwrap();

        nft_id.validate_checksums(RefLedgerId::MAINNET).unwrap();

        assert_eq!(nft_id.to_string(), TokenId::new(0, 0, 123).nft(7584).to_string());
    }

    #[test]
    fn parse_with_checksum_on_testnet() {
        let nft_id = NftId::from_str("0.0.123-esxsf@584903").unwrap();

        nft_id.validate_checksums(RefLedgerId::TESTNET).unwrap();

        assert_eq!(nft_id.to_string(), TokenId::new(0, 0, 123).nft(584903).to_string());
    }

    #[test]
    fn parse_with_checksum_on_previewnet() {
        let nft_id = NftId::from_str("0.0.123-ogizo/487302").unwrap();

        nft_id.validate_checksums(RefLedgerId::PREVIEWNET).unwrap();

        assert_eq!(nft_id.to_string(), TokenId::new(0, 0, 123).nft(487302).to_string());
    }

    #[test]
    fn it_can_create_from_a_tuple() -> anyhow::Result<()> {
        let tuple = (TokenId::from(1), 123);

        let nft_id_from_tuple = NftId::from(tuple);

        assert_eq!(tuple.0, nft_id_from_tuple.token_id);
        assert_eq!(tuple.1, nft_id_from_tuple.serial);

        Ok(())
    }

    #[test]
    fn it_can_create_by_using_into_on_tuple() -> anyhow::Result<()> {
        let tuple = (TokenId::from(1), 123);

        let nft_id_from_tuple: NftId = tuple.into();

        assert_eq!(tuple.0, nft_id_from_tuple.token_id);
        assert_eq!(tuple.1, nft_id_from_tuple.serial);

        Ok(())
    }
}
