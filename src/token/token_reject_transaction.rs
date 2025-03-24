// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::token_service_client::TokenServiceClient;
use tonic::transport::Channel;

use super::NftId;
use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};
use crate::transaction::{
    AnyTransactionData,
    ChunkInfo,
    ToSchedulableTransactionDataProtobuf,
    ToTransactionDataProtobuf,
    TransactionData,
    TransactionExecute,
};
use crate::{
    AccountId,
    BoxGrpcFuture,
    Error,
    TokenId,
    Transaction,
    ValidateChecksums,
};

/// Reject undesired token(s)
/// Transfer one or more token balances held by the requesting account to the treasury for each
/// token type.
/// Each transfer SHALL be one of the following
/// - A single non-fungible/unique token.
/// - The full balance held for a fungible/common token type.
///
/// A single tokenReject transaction SHALL support a maximum of 10 transfers.
///
/// ### Transaction Record Effects
/// - Each successful transfer from `payer` to `treasury` SHALL be recorded in `token_transfer_list` for the transaction record.
pub type TokenRejectTransaction = Transaction<TokenRejectTransactionData>;

#[derive(Debug, Clone, Default)]
pub struct TokenRejectTransactionData {
    /// An account holding the tokens to be rejected.
    owner: Option<AccountId>,

    /// The list of rejected Fungible tokens.
    token_ids: Vec<TokenId>,

    /// The list of rejected Non-fungible tokens.
    nft_ids: Vec<NftId>,
}

impl TokenRejectTransaction {
    /// Returns the owner id of the token to be rejected.
    #[must_use]
    pub fn get_owner(&self) -> Option<AccountId> {
        self.data().owner
    }

    /// Sets the owner id of the token to be rejected.
    pub fn owner(&mut self, owner: impl Into<AccountId>) -> &mut Self {
        self.data_mut().owner = Some(owner.into());
        self
    }

    /// Returns the list of Fungible tokens to be rejected.
    #[must_use]
    pub fn get_token_ids(&self) -> Vec<TokenId> {
        self.data().token_ids.clone()
    }

    /// Sets the list of Fungible tokens to be rejected.
    pub fn token_ids(&mut self, token_ids: impl IntoIterator<Item = TokenId>) -> &mut Self {
        self.data_mut().token_ids = token_ids.into_iter().collect();
        self
    }

    /// Appends a Fungible token to the list of rejected tokens.
    pub fn add_token_id(&mut self, token_id: TokenId) -> &mut Self {
        self.data_mut().token_ids.push(token_id);
        self
    }

    /// Returns the list of Non-fungible tokens to be rejected.
    #[must_use]
    pub fn get_nft_ids(&self) -> Vec<NftId> {
        self.data().nft_ids.clone()
    }

    /// Sets the list of Non-fungible tokens to be rejected.
    pub fn nft_ids(&mut self, nft_ids: impl IntoIterator<Item = NftId>) -> &mut Self {
        self.data_mut().nft_ids = nft_ids.into_iter().collect();
        self
    }

    /// Appends a Non-Fungible token to the list of rejected nfts.
    pub fn add_nft_id(&mut self, nft_id: NftId) -> &mut Self {
        self.data_mut().nft_ids.push(nft_id);
        self
    }
}

impl TransactionData for TokenRejectTransactionData {}

impl TransactionExecute for TokenRejectTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { TokenServiceClient::new(channel).reject_token(request).await })
    }
}

impl ValidateChecksums for TokenRejectTransactionData {
    fn validate_checksums(&self, ledger_id: &crate::ledger_id::RefLedgerId) -> Result<(), Error> {
        self.owner.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for TokenRejectTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::TokenReject(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for TokenRejectTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::TokenReject(self.to_protobuf())
    }
}

impl From<TokenRejectTransactionData> for AnyTransactionData {
    fn from(transaction: TokenRejectTransactionData) -> Self {
        Self::TokenReject(transaction)
    }
}

impl FromProtobuf<services::TokenRejectTransactionBody> for TokenRejectTransactionData {
    fn from_protobuf(pb: services::TokenRejectTransactionBody) -> crate::Result<Self> {
        let mut token_ids = Vec::new();
        let mut nft_ids = Vec::new();

        for reference in pb.rejections {
            match reference.token_identifier {
                Some(it) => match it {
                    services::token_reference::TokenIdentifier::FungibleToken(it) => {
                        token_ids.push(TokenId::from_protobuf(it)?);
                    }
                    services::token_reference::TokenIdentifier::Nft(it) => {
                        nft_ids.push(NftId::from_protobuf(it)?);
                    }
                },
                None => {
                    return Err(Error::from_protobuf("Invalid token identifier"));
                }
            }
        }

        Ok(Self { owner: Option::from_protobuf(pb.owner)?, token_ids, nft_ids })
    }
}

impl ToProtobuf for TokenRejectTransactionData {
    type Protobuf = services::TokenRejectTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        let owner = self.owner.to_protobuf();

        let rejections = self
            .token_ids
            .iter()
            .map(|token_id| services::TokenReference {
                token_identifier: Some(services::token_reference::TokenIdentifier::FungibleToken(
                    token_id.to_protobuf(),
                )),
            })
            .chain(self.nft_ids.iter().map(|nft_id| services::TokenReference {
                token_identifier: Some(services::token_reference::TokenIdentifier::Nft(
                    nft_id.to_protobuf(),
                )),
            }))
            .collect::<Vec<_>>();

        services::TokenRejectTransactionBody { owner, rejections }
    }
}

#[cfg(test)]
mod tests {

    use expect_test::expect_file;
    use hedera_proto::services::{
        token_reference,
        TokenReference,
        TokenRejectTransactionBody,
    };

    use super::TokenRejectTransaction;
    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::token::TokenRejectTransactionData;
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
        TEST_ACCOUNT_ID,
        TEST_NFT_IDS,
        TEST_TOKEN_IDS,
    };
    use crate::AnyTransaction;

    fn make_transaction() -> TokenRejectTransaction {
        let mut tx = TokenRejectTransaction::new_for_tests();
        tx.owner(TEST_ACCOUNT_ID).token_ids(TEST_TOKEN_IDS).nft_ids(TEST_NFT_IDS).freeze().unwrap();

        tx
    }

    #[test]
    fn seriralize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect_file!["./snapshots/token_reject_transaction/serialize.txt"].assert_debug_eq(&tx);
    }

    #[test]
    fn to_from_bytes() {
        let tx = make_transaction();

        let tx2 = AnyTransaction::from_bytes(&tx.to_bytes().unwrap()).unwrap();

        let tx = transaction_body(tx);
        let tx2 = transaction_body(tx2);

        assert_eq!(tx, tx2);
    }

    #[test]
    fn from_proto_body() {
        let mut references = Vec::new();

        for token_id in TEST_TOKEN_IDS {
            references.push(TokenReference {
                token_identifier: Some(token_reference::TokenIdentifier::FungibleToken(
                    token_id.to_protobuf(),
                )),
            });
        }

        for nft_id in TEST_NFT_IDS {
            references.push(TokenReference {
                token_identifier: Some(token_reference::TokenIdentifier::Nft(nft_id.to_protobuf())),
            });
        }

        let tx = TokenRejectTransactionBody {
            owner: Some(TEST_ACCOUNT_ID.to_protobuf()),
            rejections: references,
        };

        let data = TokenRejectTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(data.owner, Some(TEST_ACCOUNT_ID));
        assert_eq!(data.token_ids, TEST_TOKEN_IDS);
        assert_eq!(data.nft_ids, TEST_NFT_IDS);
    }

    #[test]
    fn get_set_owner() {
        let mut tx = TokenRejectTransaction::new();

        let tx2 = tx.owner(TEST_ACCOUNT_ID);

        assert_eq!(tx2.get_owner(), Some(TEST_ACCOUNT_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_owner_frozen_panic() {
        let mut tx = make_transaction();

        tx.owner(TEST_ACCOUNT_ID);
    }

    #[test]
    fn get_set_token_ids() {
        let mut tx = TokenRejectTransaction::new();

        let tx2 = tx.token_ids(TEST_TOKEN_IDS);

        assert_eq!(tx2.get_token_ids(), TEST_TOKEN_IDS);
    }

    #[test]
    #[should_panic]
    fn get_set_token_ids_frozen_panic() {
        let mut tx = make_transaction();

        tx.token_ids(TEST_TOKEN_IDS);
    }

    #[test]
    fn get_set_nft_ids() {
        let mut tx = TokenRejectTransaction::new();

        let tx2 = tx.nft_ids(TEST_NFT_IDS);

        assert_eq!(tx2.get_nft_ids(), TEST_NFT_IDS);
    }

    #[test]
    #[should_panic]
    fn get_set_nft_ids_frozen_panic() {
        let mut tx = make_transaction();

        tx.nft_ids(TEST_NFT_IDS);
    }

    #[test]
    fn get_set_add_token_ids() {
        let mut tx = TokenRejectTransaction::new();

        tx.add_token_id(TEST_TOKEN_IDS[0]);
        tx.add_token_id(TEST_TOKEN_IDS[2]);

        assert_eq!(tx.get_token_ids()[0], TEST_TOKEN_IDS[0]);
        assert_eq!(tx.get_token_ids()[1], TEST_TOKEN_IDS[2]);
    }

    #[test]
    fn get_set_add_nft_ids() {
        let mut tx = TokenRejectTransaction::new();

        tx.add_nft_id(TEST_NFT_IDS[0]);
        tx.add_nft_id(TEST_NFT_IDS[2]);

        assert_eq!(tx.get_nft_ids()[0], TEST_NFT_IDS[0]);
        assert_eq!(tx.get_nft_ids()[1], TEST_NFT_IDS[2]);
    }
}
