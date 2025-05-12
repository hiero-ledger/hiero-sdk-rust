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
    TokenId,
    Transaction,
    ValidateChecksums,
};

/// Unpauses the Token. Must be signed with the Token's pause key.
///
/// Once executed the Token is marked as Unpaused and can be used in Transactions.
///
/// The operation is idempotent - becomes a no-op if the Token is already unpaused.
///
/// - If the provided token is not found, the transaction will resolve to `INVALID_TOKEN_ID`.
/// - If the provided token has been deleted, the transaction will resolve to `TOKEN_WAS_DELETED`.
/// - If no Pause Key is defined, the transaction will resolve to `TOKEN_HAS_NO_PAUSE_KEY`.
pub type TokenUnpauseTransaction = Transaction<TokenUnpauseTransactionData>;

#[derive(Debug, Clone, Default)]
pub struct TokenUnpauseTransactionData {
    /// The token to be unpaused.
    token_id: Option<TokenId>,
}

impl TokenUnpauseTransaction {
    /// Returns the token to be unpaused.
    #[must_use]
    pub fn get_token_id(&self) -> Option<TokenId> {
        self.data().token_id
    }

    /// Sets the token to be unpaused.
    pub fn token_id(&mut self, token_id: impl Into<TokenId>) -> &mut Self {
        self.data_mut().token_id = Some(token_id.into());
        self
    }
}

impl TransactionData for TokenUnpauseTransactionData {}

impl TransactionExecute for TokenUnpauseTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { TokenServiceClient::new(channel).unpause_token(request).await })
    }
}

impl ValidateChecksums for TokenUnpauseTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> crate::Result<()> {
        self.token_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for TokenUnpauseTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::TokenUnpause(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for TokenUnpauseTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::TokenUnpause(self.to_protobuf())
    }
}

impl From<TokenUnpauseTransactionData> for AnyTransactionData {
    fn from(transaction: TokenUnpauseTransactionData) -> Self {
        Self::TokenUnpause(transaction)
    }
}

impl FromProtobuf<services::TokenUnpauseTransactionBody> for TokenUnpauseTransactionData {
    fn from_protobuf(pb: services::TokenUnpauseTransactionBody) -> crate::Result<Self> {
        Ok(Self { token_id: Option::from_protobuf(pb.token)? })
    }
}

impl ToProtobuf for TokenUnpauseTransactionData {
    type Protobuf = services::TokenUnpauseTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::TokenUnpauseTransactionBody { token: self.token_id.to_protobuf() }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hedera_proto::services;

    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::token::TokenUnpauseTransactionData;
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
    };
    use crate::{
        AnyTransaction,
        TokenId,
        TokenUnpauseTransaction,
    };

    const TEST_TOKEN_ID: TokenId = TokenId::new(4, 2, 0);

    fn make_transaction() -> TokenUnpauseTransaction {
        let mut tx = TokenUnpauseTransaction::new_for_tests();

        tx.token_id(TEST_TOKEN_ID).freeze().unwrap();

        tx
    }

    #[test]
    fn seriralize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            TokenUnpause(
                TokenUnpauseTransactionBody {
                    token: Some(
                        TokenId {
                            shard_num: 4,
                            realm_num: 2,
                            token_num: 0,
                        },
                    ),
                },
            )
        "#]]
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
        let tx = services::TokenUnpauseTransactionBody { token: Some(TEST_TOKEN_ID.to_protobuf()) };

        let tx = TokenUnpauseTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(tx.token_id, Some(TEST_TOKEN_ID));
    }

    #[test]
    fn get_set_token_id() {
        let mut tx = TokenUnpauseTransaction::new();
        tx.token_id(TEST_TOKEN_ID);

        assert_eq!(tx.get_token_id(), Some(TEST_TOKEN_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_token_id_frozen_panic() {
        let mut tx = make_transaction();
        tx.token_id(TEST_TOKEN_ID);
    }
}
