// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;

use crate::protobuf::ToProtobuf;
use crate::{
    AccountId,
    FromProtobuf,
    TokenId,
};

/// A token <-> account association.
#[derive(Debug, Clone)]
pub struct TokenAssociation {
    /// The token involved in the association.
    pub token_id: TokenId,

    /// The account involved in the association.
    pub account_id: AccountId,
}

impl TokenAssociation {
    /// Create a new `TokenAssociation` from protobuf-encoded `bytes`.
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

impl FromProtobuf<services::TokenAssociation> for TokenAssociation {
    fn from_protobuf(pb: services::TokenAssociation) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let token_id = pb_getf!(pb, token_id)?;
        let account_id = pb_getf!(pb, account_id)?;

        Ok(Self {
            token_id: TokenId::from_protobuf(token_id)?,
            account_id: AccountId::from_protobuf(account_id)?,
        })
    }
}

impl ToProtobuf for TokenAssociation {
    type Protobuf = services::TokenAssociation;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::TokenAssociation {
            token_id: Some(self.token_id.to_protobuf()),
            account_id: Some(self.account_id.to_protobuf()),
        }
    }
}

#[cfg(test)]
mod tests {

    use expect_test::expect;

    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::transaction::test_helpers::{
        TEST_ACCOUNT_ID,
        TEST_TOKEN_ID,
    };
    use crate::{
        AccountId,
        TokenAssociation,
    };

    fn make_association() -> TokenAssociation {
        TokenAssociation { token_id: TEST_TOKEN_ID, account_id: AccountId::new(1, 2, 3) }
    }

    #[test]
    fn serialize_account() {
        let tx = make_association();

        let tx_bytes = tx.to_bytes();

        let tx = TokenAssociation::from_bytes(&tx_bytes).unwrap();

        expect![[r#"
            TokenAssociation {
                token_id: "1.2.3",
                account_id: "1.2.3",
            }
        "#]]
        .assert_debug_eq(&tx);
    }

    #[test]
    fn from_protobuf() {
        let token_proto =
            TokenAssociation { token_id: TEST_TOKEN_ID, account_id: TEST_ACCOUNT_ID }.to_protobuf();

        let token = TokenAssociation::from_protobuf(token_proto).unwrap();

        assert_eq!(token.account_id, TEST_ACCOUNT_ID);
        assert_eq!(token.token_id, TEST_TOKEN_ID);
    }

    #[test]
    fn to_protobuf() {
        let proto =
            TokenAssociation { token_id: TEST_TOKEN_ID, account_id: TEST_ACCOUNT_ID }.to_protobuf();

        assert_eq!(proto.account_id, Some(TEST_ACCOUNT_ID.to_protobuf()));
        assert_eq!(proto.token_id, Some(TEST_TOKEN_ID.to_protobuf()));
    }

    #[test]
    fn from_bytes() {
        let token = TokenAssociation { token_id: TEST_TOKEN_ID, account_id: TEST_ACCOUNT_ID };

        let token = TokenAssociation::from_bytes(&token.to_bytes()).unwrap();

        assert_eq!(token.account_id, TEST_ACCOUNT_ID);
        assert_eq!(token.token_id, TEST_TOKEN_ID);
    }

    #[test]
    fn to_bytes() {
        let token = TokenAssociation { token_id: TEST_TOKEN_ID, account_id: TEST_ACCOUNT_ID };

        let token_bytes = token.to_bytes();

        assert_eq!(token_bytes, token.to_bytes());
    }
}
