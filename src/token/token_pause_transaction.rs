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

/// Pauses the Token from being involved in any kind of Transaction until it is unpaused.
///
/// Must be signed with the Token's pause key.
///
/// Once executed the Token is marked as paused and will be not able to be a part of any transaction.
/// The operation is idempotent - becomes a no-op if the Token is already Paused.
///
/// - If the provided token is not found, the transaction will resolve to `INVALID_TOKEN_ID`.
/// - If the provided token has been deleted, the transaction will resolve to `TOKEN_WAS_DELETED`.
/// - If no Pause Key is defined, the transaction will resolve to `TOKEN_HAS_NO_PAUSE_KEY`.
pub type TokenPauseTransaction = Transaction<TokenPauseTransactionData>;

#[derive(Debug, Clone, Default)]
pub struct TokenPauseTransactionData {
    /// The token to be paused.
    token_id: Option<TokenId>,
}

impl TokenPauseTransaction {
    /// Returns the token to be paused.
    #[must_use]
    pub fn get_token_id(&self) -> Option<TokenId> {
        self.data().token_id
    }

    /// Sets the token to be paused.
    pub fn token_id(&mut self, token_id: impl Into<TokenId>) -> &mut Self {
        self.data_mut().token_id = Some(token_id.into());
        self
    }
}

impl TransactionData for TokenPauseTransactionData {}

impl TransactionExecute for TokenPauseTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { TokenServiceClient::new(channel).pause_token(request).await })
    }
}

impl ValidateChecksums for TokenPauseTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> crate::Result<()> {
        self.token_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for TokenPauseTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::TokenPause(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for TokenPauseTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::TokenPause(self.to_protobuf())
    }
}

impl From<TokenPauseTransactionData> for AnyTransactionData {
    fn from(transaction: TokenPauseTransactionData) -> Self {
        Self::TokenPause(transaction)
    }
}

impl FromProtobuf<services::TokenPauseTransactionBody> for TokenPauseTransactionData {
    fn from_protobuf(pb: services::TokenPauseTransactionBody) -> crate::Result<Self> {
        Ok(Self { token_id: Option::from_protobuf(pb.token)? })
    }
}

impl ToProtobuf for TokenPauseTransactionData {
    type Protobuf = services::TokenPauseTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::TokenPauseTransactionBody { token: self.token_id.to_protobuf() }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hedera_proto::services::TokenPauseTransactionBody;

    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::token::TokenPauseTransactionData;
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
    };
    use crate::{
        AnyTransaction,
        TokenId,
        TokenPauseTransaction,
    };

    const TEST_TOKEN_ID: TokenId = TokenId::new(4, 2, 0);

    fn make_transaction() -> TokenPauseTransaction {
        let mut tx = TokenPauseTransaction::new_for_tests();

        tx.token_id(TEST_TOKEN_ID).freeze().unwrap();

        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            TokenPause(
                TokenPauseTransactionBody {
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
    fn from_proto_body() {
        let tx = TokenPauseTransactionBody { token: Some(TEST_TOKEN_ID.to_protobuf()) };

        let data = TokenPauseTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(data.token_id, Some(TEST_TOKEN_ID));
    }

    #[test]
    fn get_set_token_id() {
        let mut tx = TokenPauseTransaction::new();
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
