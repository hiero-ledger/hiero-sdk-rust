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

/// Marks a token as deleted, though it will remain in the ledger.
///
/// The operation must be signed by the specified Admin Key of the Token.
///
/// Once deleted update, mint, burn, wipe, freeze, unfreeze, grant kyc, revoke
/// kyc and token transfer transactions will resolve to `TOKEN_WAS_DELETED`.
///
/// - If admin key is not set, Transaction will result in `TOKEN_IS_IMMUTABlE`.
/// - If invalid token is specified, transaction will result in `INVALID_TOKEN_ID`
pub type TokenDeleteTransaction = Transaction<TokenDeleteTransactionData>;

#[derive(Debug, Clone, Default)]
pub struct TokenDeleteTransactionData {
    /// The token to be deleted.
    token_id: Option<TokenId>,
}

impl TokenDeleteTransaction {
    /// Returns the token to be deleted.
    #[must_use]
    pub fn get_token_id(&self) -> Option<TokenId> {
        self.data().token_id
    }

    /// Sets the token to be deleted.
    pub fn token_id(&mut self, token_id: impl Into<TokenId>) -> &mut Self {
        self.data_mut().token_id = Some(token_id.into());
        self
    }
}

impl TransactionData for TokenDeleteTransactionData {}

impl TransactionExecute for TokenDeleteTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { TokenServiceClient::new(channel).delete_token(request).await })
    }
}

impl ValidateChecksums for TokenDeleteTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.token_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for TokenDeleteTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::TokenDeletion(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for TokenDeleteTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::TokenDeletion(self.to_protobuf())
    }
}

impl From<TokenDeleteTransactionData> for AnyTransactionData {
    fn from(transaction: TokenDeleteTransactionData) -> Self {
        Self::TokenDelete(transaction)
    }
}

impl FromProtobuf<services::TokenDeleteTransactionBody> for TokenDeleteTransactionData {
    fn from_protobuf(pb: services::TokenDeleteTransactionBody) -> crate::Result<Self> {
        Ok(Self { token_id: Option::from_protobuf(pb.token)? })
    }
}

impl ToProtobuf for TokenDeleteTransactionData {
    type Protobuf = services::TokenDeleteTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::TokenDeleteTransactionBody { token: self.token_id.to_protobuf() }
    }
}

#[cfg(test)]
mod tests {

    use expect_test::expect_file;
    use hedera_proto::services;

    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::token::TokenDeleteTransactionData;
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
        TEST_TOKEN_ID,
    };
    use crate::{
        AnyTransaction,
        TokenDeleteTransaction,
    };

    fn make_transaction() -> TokenDeleteTransaction {
        let mut tx = TokenDeleteTransaction::new_for_tests();

        tx.token_id(TEST_TOKEN_ID).freeze().unwrap();

        tx
    }

    #[test]
    fn seriralize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect_file!["./snapshots/token_delete_transaction/serialize.txt"].assert_debug_eq(&tx);
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
        let tx = services::TokenDeleteTransactionBody { token: Some(TEST_TOKEN_ID.to_protobuf()) };

        let data = TokenDeleteTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(data.token_id, Some(TEST_TOKEN_ID));
    }

    #[test]
    fn get_set_token_id() {
        let mut tx = TokenDeleteTransaction::new();

        let tx2 = tx.token_id(TEST_TOKEN_ID);

        assert_eq!(tx2.get_token_id(), Some(TEST_TOKEN_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_token_id_frozen_panic() {
        let mut tx = make_transaction();

        tx.token_id(TEST_TOKEN_ID);
    }
}
