// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::token_service_client::TokenServiceClient;
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
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
    BoxGrpcFuture,
    Error,
    TokenId,
    Transaction,
    ValidateChecksums,
};

/// At consensus, updates an already created Non Fungible Token to the given values.
///
/// If no value is given for a field, that field is left unchanged.
/// Only certain fields such as metadata can be updated.
/// Updating the metadata of an NFT does not affect its ownership or transferability.
/// This operation is intended for updating attributes of individual NFTs in a collection./
/// --- Signing Requirements ---
/// 1. To update metadata of an NFT, the metadata_key of the token should sign the transaction.
pub type TokenUpdateNftsTransaction = Transaction<TokenUpdateNftsTransactionData>;

#[derive(Debug, Clone, Default)]
pub struct TokenUpdateNftsTransactionData {
    /// The token to be updated.
    token_id: Option<TokenId>,

    /// The list of serial numbers to be updated.
    serials: Vec<i64>,

    /// Metadata of the created token definition.
    metadata: Vec<u8>,
}

impl TokenUpdateNftsTransaction {
    /// Returns the token for which to update NFTs.
    #[must_use]
    pub fn get_token_id(&self) -> Option<TokenId> {
        self.data().token_id
    }

    /// Sets the token for which to update NFTs.
    pub fn token_id(&mut self, token_id: impl Into<TokenId>) -> &mut Self {
        self.data_mut().token_id = Some(token_id.into());
        self
    }

    /// Returns the list of serial numbers to be updated.
    #[must_use]
    pub fn get_serials(&self) -> Vec<i64> {
        self.data().serials.clone()
    }

    /// Sets the list of serial numbers to be updated.
    pub fn serials(&mut self, serials: Vec<i64>) -> &mut Self {
        self.data_mut().serials = serials;
        self
    }

    /// Returns the new metadata of the NFT(s).
    #[must_use]
    pub fn get_metadata(&self) -> Vec<u8> {
        self.data().metadata.clone()
    }

    /// Sets the new metadata of the NFT(s).
    pub fn metadata(&mut self, metadata: Vec<u8>) -> &mut Self {
        self.data_mut().metadata = metadata;
        self
    }
}

impl TransactionData for TokenUpdateNftsTransactionData {}

impl TransactionExecute for TokenUpdateNftsTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { TokenServiceClient::new(channel).update_token(request).await })
    }
}

impl ValidateChecksums for TokenUpdateNftsTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.token_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for TokenUpdateNftsTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::TokenUpdateNfts(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for TokenUpdateNftsTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::TokenUpdateNfts(self.to_protobuf())
    }
}

impl From<TokenUpdateNftsTransactionData> for AnyTransactionData {
    fn from(transaction: TokenUpdateNftsTransactionData) -> Self {
        Self::TokenUpdateNfts(transaction)
    }
}

impl FromProtobuf<services::TokenUpdateNftsTransactionBody> for TokenUpdateNftsTransactionData {
    fn from_protobuf(pb: services::TokenUpdateNftsTransactionBody) -> crate::Result<Self> {
        Ok(Self {
            token_id: Option::from_protobuf(pb.token)?,
            serials: pb.serial_numbers,
            metadata: pb.metadata.unwrap_or_default(),
        })
    }
}

impl ToProtobuf for TokenUpdateNftsTransactionData {
    type Protobuf = services::TokenUpdateNftsTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::TokenUpdateNftsTransactionBody {
            token: self.token_id.to_protobuf(),
            serial_numbers: self.serials.clone(),
            metadata: Some(self.metadata.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect_file;
    use hedera_proto::services;

    use super::TokenUpdateNftsTransactionData;
    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
    };
    use crate::{
        AnyTransaction,
        TokenId,
        TokenUpdateNftsTransaction,
    };

    const TEST_TOKEN_ID: TokenId = TokenId::new(0, 0, 12);
    const TEST_METADATA: &str = "Token Metadata";

    fn test_serials() -> Vec<i64> {
        vec![1, 2, 3]
    }

    fn make_transaction() -> TokenUpdateNftsTransaction {
        let mut tx = TokenUpdateNftsTransaction::new_for_tests();

        tx.token_id(TEST_TOKEN_ID)
            .serials(test_serials())
            .metadata(TEST_METADATA.as_bytes().to_vec())
            .freeze()
            .unwrap();

        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect_file!["./snapshots/token_update_nfts_transaction/serialize.txt"]
            .assert_debug_eq(&tx);
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
        let tx = services::TokenUpdateNftsTransactionBody {
            token: Some(TEST_TOKEN_ID.to_protobuf()),
            serial_numbers: test_serials(),
            metadata: Some(TEST_METADATA.to_owned().into()),
        };

        let tx = TokenUpdateNftsTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(tx.token_id, Some(TEST_TOKEN_ID));
        assert_eq!(tx.metadata, TEST_METADATA.as_bytes().to_vec());
        assert_eq!(tx.serials, test_serials())
    }

    #[test]
    fn get_set_token_id() {
        let mut tx = TokenUpdateNftsTransaction::new();
        tx.token_id(TEST_TOKEN_ID);
        assert_eq!(tx.get_token_id(), Some(TEST_TOKEN_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_token_id_frozen_panic() {
        let mut tx = make_transaction();
        tx.token_id(TEST_TOKEN_ID);
    }

    #[test]
    fn get_set_metadata() {
        let mut tx = TokenUpdateNftsTransaction::new();
        tx.metadata(TEST_METADATA.as_bytes().to_vec());
        assert_eq!(tx.get_metadata(), TEST_METADATA.as_bytes().to_vec());
    }

    #[test]
    #[should_panic]
    fn get_set_metadata_frozen_panic() {
        let mut tx = make_transaction();
        tx.metadata(TEST_METADATA.as_bytes().to_vec());
    }

    #[test]
    fn get_set_serials() {
        let mut tx = TokenUpdateNftsTransaction::new();
        tx.serials(test_serials());
        assert_eq!(tx.get_serials(), test_serials());
    }

    #[test]
    #[should_panic]
    fn get_set_serials_frozen_panic() {
        let mut tx = make_transaction();
        tx.serials(test_serials());
    }
}
