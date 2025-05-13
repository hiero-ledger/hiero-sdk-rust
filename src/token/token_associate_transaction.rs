// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::token_service_client::TokenServiceClient;
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::protobuf::FromProtobuf;
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
    ToProtobuf,
    TokenId,
    Transaction,
    ValidateChecksums,
};

/// Associates the provided account with the provided tokens. Must be signed by the provided Account's key.
///
/// - If the provided account is not found, the transaction will resolve to `INVALID_ACCOUNT_ID`.
/// - If the provided account has been deleted, the transaction will resolve to `ACCOUNT_DELETED`.
/// - If any of the provided tokens are not found, the transaction will resolve to `INVALID_TOKEN_REF`.
/// - If any of the provided tokens have been deleted, the transaction will resolve to
/// `TOKEN_WAS_DELETED`.
/// - If an association between the provided account and any of the tokens already exists, the
/// transaction will resolve to `TOKEN_ALREADY_ASSOCIATED_TO_ACCOUNT`.
/// - If the provided account's associations count exceed the constraint of maximum token associations
/// per account, the transaction will resolve to `TOKENS_PER_ACCOUNT_LIMIT_EXCEEDED`.
/// - On success, associations between the provided account and tokens are made and the account is
/// ready to interact with the tokens.
pub type TokenAssociateTransaction = Transaction<TokenAssociateTransactionData>;

#[derive(Debug, Clone, Default)]
pub struct TokenAssociateTransactionData {
    /// The account to be associated with the provided tokens.
    account_id: Option<AccountId>,

    /// The tokens to be associated with the provided account.
    token_ids: Vec<TokenId>,
}

impl TokenAssociateTransaction {
    /// Returns the account to be associated with the provided tokens.
    #[must_use]
    pub fn get_account_id(&self) -> Option<AccountId> {
        self.data().account_id
    }

    /// Sets the account to be associated with the provided tokens.
    pub fn account_id(&mut self, account_id: AccountId) -> &mut Self {
        self.data_mut().account_id = Some(account_id);
        self
    }

    /// Returns the tokens to be associated with the provided account.
    #[must_use]
    pub fn get_token_ids(&self) -> &[TokenId] {
        &self.data().token_ids
    }

    /// Sets the tokens to be associated with the provided account.
    pub fn token_ids(&mut self, token_ids: impl IntoIterator<Item = TokenId>) -> &mut Self {
        self.data_mut().token_ids = token_ids.into_iter().collect();
        self
    }
}

impl TransactionData for TokenAssociateTransactionData {}

impl TransactionExecute for TokenAssociateTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { TokenServiceClient::new(channel).associate_tokens(request).await })
    }
}

impl ValidateChecksums for TokenAssociateTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.account_id.validate_checksums(ledger_id)?;
        for token_id in &self.token_ids {
            token_id.validate_checksums(ledger_id)?;
        }
        Ok(())
    }
}

impl ToTransactionDataProtobuf for TokenAssociateTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::TokenAssociate(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for TokenAssociateTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::TokenAssociate(self.to_protobuf())
    }
}

impl From<TokenAssociateTransactionData> for AnyTransactionData {
    fn from(transaction: TokenAssociateTransactionData) -> Self {
        Self::TokenAssociate(transaction)
    }
}

impl FromProtobuf<services::TokenAssociateTransactionBody> for TokenAssociateTransactionData {
    fn from_protobuf(pb: services::TokenAssociateTransactionBody) -> crate::Result<Self> {
        Ok(Self {
            account_id: Option::from_protobuf(pb.account)?,
            token_ids: Vec::from_protobuf(pb.tokens)?,
        })
    }
}

impl ToProtobuf for TokenAssociateTransactionData {
    type Protobuf = services::TokenAssociateTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        let account = self.account_id.to_protobuf();
        let tokens = self.token_ids.to_protobuf();

        services::TokenAssociateTransactionBody { account, tokens }
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
    use crate::token::TokenAssociateTransactionData;
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
        TEST_ACCOUNT_ID,
        TEST_TOKEN_ID,
    };
    use crate::{
        AnyTransaction,
        TokenAssociateTransaction,
    };

    fn make_transaction() -> TokenAssociateTransaction {
        let mut tx = TokenAssociateTransaction::new_for_tests();

        tx.account_id(TEST_ACCOUNT_ID).token_ids([TEST_TOKEN_ID]).freeze().unwrap();

        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect_file!["./snapshots/token_associate_transaction/serialize.txt"].assert_debug_eq(&tx);
    }

    #[test]
    fn to_from_bytes() {
        let tx = make_transaction();

        let tx2 = AnyTransaction::from_bytes(&tx.to_bytes().unwrap()).unwrap();

        let tx = transaction_body(tx);
        let tx2 = transaction_body(tx2);

        assert_eq!(tx, tx2)
    }

    #[test]
    fn from_proto_body() {
        let tx = services::TokenAssociateTransactionBody {
            account: Some(TEST_ACCOUNT_ID.to_protobuf()),
            tokens: Vec::from([TEST_TOKEN_ID.to_protobuf()]),
        };

        let data = TokenAssociateTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(data.account_id, Some(TEST_ACCOUNT_ID));
        assert_eq!(data.token_ids, &[TEST_TOKEN_ID]);
    }

    #[test]
    fn get_set_token_ids() {
        let token_ids = [TEST_TOKEN_ID];
        let mut tx = TokenAssociateTransaction::new();
        tx.token_ids(token_ids.to_owned());

        assert_eq!(tx.get_token_ids(), &token_ids[..]);
    }

    #[test]
    #[should_panic]
    fn get_set_token_ids_frozen_panic() {
        make_transaction().token_ids([TEST_TOKEN_ID]);
    }

    #[test]
    fn get_set_account_id() {
        let mut tx = TokenAssociateTransaction::new();
        tx.account_id(TEST_ACCOUNT_ID);

        assert_eq!(tx.get_account_id(), Some(TEST_ACCOUNT_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_account_id_frozen_panic() {
        make_transaction().account_id(TEST_ACCOUNT_ID);
    }
}
