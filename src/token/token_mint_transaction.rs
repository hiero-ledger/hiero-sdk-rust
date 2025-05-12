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

/// Mints tokens to the Token's treasury Account.
///
/// The operation increases the Total Supply of the Token. The maximum total supply a token can have
/// is 2^63-1.
///
/// The amount provided must be in the lowest denomination possible. Example: Token A has 2 decimals.
/// In order to mint 100 tokens, one must provide amount of 10000. In order to mint 100.55 tokens,
/// one must provide amount of 10055.
///
/// - If no Supply Key is defined, the transaction will resolve to `TokenHasNoSupplyKey`.
/// - If both amount and metadata list get filled, a `InvalidTransactionBody` response code will be
/// returned.
/// - If the metadata list contains metadata which is too large, a `MetadataTooLong` response code will
/// be returned.
/// - If neither the amount nor the metadata list get filled, a `InvalidTokenMintAmount` response code
/// will be returned.
/// - If the metadata list count is greater than the batch size limit global dynamic property, a
/// `BatchSizeLimitExceeded` response code will be returned.
pub type TokenMintTransaction = Transaction<TokenMintTransactionData>;

#[derive(Debug, Clone, Default)]
pub struct TokenMintTransactionData {
    /// The token for which to mint tokens.
    token_id: Option<TokenId>,

    /// The amount of a fungible token to mint to the treasury account.
    amount: u64,

    /// The list of metadata for a non-fungible token to mint to the treasury account.
    metadata: Vec<Vec<u8>>,
}

impl TokenMintTransaction {
    /// Returns the token for which to mint tokens.
    #[must_use]
    pub fn get_token_id(&self) -> Option<TokenId> {
        self.data().token_id
    }

    /// Sets the token for which to mint tokens.
    pub fn token_id(&mut self, token_id: impl Into<TokenId>) -> &mut Self {
        self.data_mut().token_id = Some(token_id.into());
        self
    }

    /// Returns the amount of a fungible token to mint to the treasury account.
    #[must_use]
    pub fn get_amount(&self) -> u64 {
        self.data().amount
    }

    /// Sets the amount of a fungible token to mint to the treasury account.
    pub fn amount(&mut self, amount: u64) -> &mut Self {
        self.data_mut().amount = amount;
        self
    }

    /// Returns the list of metadata for a non-fungible token to mint to the treasury account.
    #[must_use]
    pub fn get_metadata(&self) -> &[Vec<u8>] {
        &self.data().metadata
    }

    /// Sets the list of metadata for a non-fungible token to mint to the treasury account.
    pub fn metadata<Bytes>(&mut self, metadata: impl IntoIterator<Item = Bytes>) -> &mut Self
    where
        Bytes: AsRef<[u8]>,
    {
        self.data_mut().metadata =
            metadata.into_iter().map(|bytes| bytes.as_ref().to_vec()).collect();

        self
    }
}

impl TransactionData for TokenMintTransactionData {}

impl TransactionExecute for TokenMintTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { TokenServiceClient::new(channel).mint_token(request).await })
    }
}

impl ValidateChecksums for TokenMintTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.token_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for TokenMintTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::TokenMint(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for TokenMintTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::TokenMint(self.to_protobuf())
    }
}

impl From<TokenMintTransactionData> for AnyTransactionData {
    fn from(transaction: TokenMintTransactionData) -> Self {
        Self::TokenMint(transaction)
    }
}

impl FromProtobuf<services::TokenMintTransactionBody> for TokenMintTransactionData {
    fn from_protobuf(pb: services::TokenMintTransactionBody) -> crate::Result<Self> {
        Ok(Self {
            token_id: Option::from_protobuf(pb.token)?,
            amount: pb.amount,
            metadata: pb.metadata,
        })
    }
}

impl ToProtobuf for TokenMintTransactionData {
    type Protobuf = services::TokenMintTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::TokenMintTransactionBody {
            token: self.token_id.to_protobuf(),
            amount: self.amount,
            metadata: self.metadata.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hedera_proto::services::TokenMintTransactionBody;

    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::token::TokenMintTransactionData;
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
    };
    use crate::{
        AnyTransaction,
        TokenId,
        TokenMintTransaction,
    };

    const TEST_TOKEN_ID: TokenId = TokenId::new(4, 2, 0);
    const TEST_AMOUNT: u64 = 10;

    fn metadata() -> Vec<Vec<u8>> {
        [[1, 2, 3, 4, 5].into()].into()
    }

    fn make_transaction() -> TokenMintTransaction {
        let mut tx = TokenMintTransaction::new_for_tests();

        tx.token_id(TEST_TOKEN_ID).amount(TEST_AMOUNT).freeze().unwrap();

        tx
    }

    fn make_metadata_transaction() -> TokenMintTransaction {
        let mut tx = TokenMintTransaction::new_for_tests();

        tx.token_id(TEST_TOKEN_ID).metadata(metadata()).freeze().unwrap();

        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            TokenMint(
                TokenMintTransactionBody {
                    token: Some(
                        TokenId {
                            shard_num: 4,
                            realm_num: 2,
                            token_num: 0,
                        },
                    ),
                    amount: 10,
                    metadata: [],
                },
            )
        "#]]
        .assert_debug_eq(&tx)
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
    fn serialize_metadata() {
        let tx = make_metadata_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            TokenMint(
                TokenMintTransactionBody {
                    token: Some(
                        TokenId {
                            shard_num: 4,
                            realm_num: 2,
                            token_num: 0,
                        },
                    ),
                    amount: 0,
                    metadata: [
                        [
                            1,
                            2,
                            3,
                            4,
                            5,
                        ],
                    ],
                },
            )
        "#]]
        .assert_debug_eq(&tx)
    }

    #[test]
    fn to_from_bytes_metadata() {
        let tx = make_metadata_transaction();

        let tx2 = AnyTransaction::from_bytes(&tx.to_bytes().unwrap()).unwrap();

        let tx = transaction_body(tx);

        let tx2 = transaction_body(tx2);

        assert_eq!(tx, tx2);
    }

    #[test]
    fn from_proto_body() {
        let tx = TokenMintTransactionBody {
            token: Some(TEST_TOKEN_ID.to_protobuf()),
            amount: TEST_AMOUNT,
            metadata: metadata(),
        };

        let data = TokenMintTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(data.token_id, Some(TEST_TOKEN_ID));
        assert_eq!(data.amount, TEST_AMOUNT);
        assert_eq!(data.metadata, metadata());
    }

    #[test]
    fn get_set_token_id() {
        let mut tx = TokenMintTransaction::new();
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
    fn get_set_amount() {
        let mut tx = TokenMintTransaction::new();
        tx.amount(TEST_AMOUNT);

        assert_eq!(tx.get_amount(), TEST_AMOUNT);
    }

    #[test]
    #[should_panic]
    fn get_set_amount_frozen_panic() {
        let mut tx = make_transaction();
        tx.amount(TEST_AMOUNT);
    }

    #[test]
    fn get_set_metadata() {
        let mut tx = TokenMintTransaction::new();
        tx.metadata(metadata());

        assert_eq!(tx.get_metadata(), &metadata());
    }

    #[test]
    #[should_panic]
    fn get_set_metadata_frozen_panic() {
        let mut tx = make_transaction();
        tx.metadata(metadata());
    }
}
