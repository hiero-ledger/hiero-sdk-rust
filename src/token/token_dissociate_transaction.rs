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
    AccountId,
    BoxGrpcFuture,
    Error,
    TokenId,
    Transaction,
    ValidateChecksums,
};

/// Dissociates the provided account with the provided tokens. Must be signed by the provided
/// Account's key.
///
/// On success, associations between the provided account and tokens are removed.
///
/// - If the provided account is not found, the transaction will resolve to `INVALID_ACCOUNT_ID`.
/// - If the provided account has been deleted, the transaction will resolve to `ACCOUNT_DELETED`.
/// - If any of the provided tokens is not found, the transaction will resolve to `INVALID_TOKEN_REF`.
/// - If any of the provided tokens has been deleted, the transaction will resolve to `TOKEN_WAS_DELETED`.
/// - If an association between the provided account and any of the tokens does not exist, the
/// transaction will resolve to `TOKEN_NOT_ASSOCIATED_TO_ACCOUNT`.
/// - If a token has not been deleted and has not expired, and the user has a nonzero balance, the
/// transaction will resolve to `TRANSACTION_REQUIRES_ZERO_TOKEN_BALANCES`.
/// - If a <b>fungible token</b> has expired, the user can disassociate even if their token balance is
/// not zero.
/// - If a <b>non fungible token</b> has expired, the user can <b>not</b> disassociate if their token
/// balance is not zero. The transaction will resolve to `TRANSACTION_REQUIRED_ZERO_TOKEN_BALANCES`.
pub type TokenDissociateTransaction = Transaction<TokenDissociateTransactionData>;

#[derive(Debug, Clone, Default)]
pub struct TokenDissociateTransactionData {
    /// The account to be dissociated with the provided tokens.
    account_id: Option<AccountId>,

    /// The tokens to be dissociated with the provided account.
    token_ids: Vec<TokenId>,
}

impl TokenDissociateTransaction {
    /// Returns the account to be dissociated with the provided tokens.
    #[must_use]
    pub fn get_account_id(&self) -> Option<AccountId> {
        self.data().account_id
    }

    /// Sets the account to be dissociated with the provided tokens.
    pub fn account_id(&mut self, account_id: AccountId) -> &mut Self {
        self.data_mut().account_id = Some(account_id);
        self
    }

    /// Returns the tokens to be dissociated with the provided account.
    #[must_use]
    pub fn get_token_ids(&self) -> &[TokenId] {
        &self.data().token_ids
    }

    /// Sets the tokens to be dissociated with the provided account.
    pub fn token_ids(&mut self, token_ids: impl IntoIterator<Item = TokenId>) -> &mut Self {
        self.data_mut().token_ids = token_ids.into_iter().collect();
        self
    }
}

impl TransactionData for TokenDissociateTransactionData {}

impl TransactionExecute for TokenDissociateTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { TokenServiceClient::new(channel).dissociate_tokens(request).await })
    }
}

impl ValidateChecksums for TokenDissociateTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.account_id.validate_checksums(ledger_id)?;
        for token_id in &self.token_ids {
            token_id.validate_checksums(ledger_id)?;
        }
        Ok(())
    }
}

impl ToTransactionDataProtobuf for TokenDissociateTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::TokenDissociate(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for TokenDissociateTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::TokenDissociate(self.to_protobuf())
    }
}

impl From<TokenDissociateTransactionData> for AnyTransactionData {
    fn from(transaction: TokenDissociateTransactionData) -> Self {
        Self::TokenDissociate(transaction)
    }
}

impl FromProtobuf<services::TokenDissociateTransactionBody> for TokenDissociateTransactionData {
    fn from_protobuf(pb: services::TokenDissociateTransactionBody) -> crate::Result<Self> {
        Ok(Self {
            account_id: Option::from_protobuf(pb.account)?,
            token_ids: Vec::from_protobuf(pb.tokens)?,
        })
    }
}

impl ToProtobuf for TokenDissociateTransactionData {
    type Protobuf = services::TokenDissociateTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        let account = self.account_id.to_protobuf();
        let tokens = self.token_ids.to_protobuf();

        services::TokenDissociateTransactionBody { account, tokens }
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
    use crate::token::TokenDissociateTransactionData;
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
    };
    use crate::{
        AccountId,
        AnyTransaction,
        TokenDissociateTransaction,
        TokenId,
    };

    const TEST_ACCOUNT_ID: AccountId =
        AccountId { shard: 6, realm: 9, num: 0, alias: None, evm_address: None, checksum: None };

    const TEST_TOKEN_IDS: [TokenId; 3] =
        [TokenId::new(4, 2, 0), TokenId::new(4, 2, 1), TokenId::new(4, 2, 2)];

    fn make_transaction() -> TokenDissociateTransaction {
        let mut tx = TokenDissociateTransaction::new_for_tests();

        tx.account_id(TEST_ACCOUNT_ID).token_ids(TEST_TOKEN_IDS).freeze().unwrap();

        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            TokenDissociate(
                TokenDissociateTransactionBody {
                    account: Some(
                        AccountId {
                            shard_num: 6,
                            realm_num: 9,
                            account: Some(
                                AccountNum(
                                    0,
                                ),
                            ),
                        },
                    ),
                    tokens: [
                        TokenId {
                            shard_num: 4,
                            realm_num: 2,
                            token_num: 0,
                        },
                        TokenId {
                            shard_num: 4,
                            realm_num: 2,
                            token_num: 1,
                        },
                        TokenId {
                            shard_num: 4,
                            realm_num: 2,
                            token_num: 2,
                        },
                    ],
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
        let tx = services::TokenDissociateTransactionBody {
            account: Some(TEST_ACCOUNT_ID.to_protobuf()),
            tokens: TEST_TOKEN_IDS.iter().map(TokenId::to_protobuf).collect(),
        };

        let data = TokenDissociateTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(data.account_id, Some(TEST_ACCOUNT_ID));
        assert_eq!(data.token_ids, TEST_TOKEN_IDS);
    }

    #[test]
    fn get_set_account_id() {
        let mut tx = TokenDissociateTransaction::new();
        tx.account_id(TEST_ACCOUNT_ID);

        assert_eq!(tx.get_account_id(), Some(TEST_ACCOUNT_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_account_id_frozen_panic() {
        make_transaction().account_id(TEST_ACCOUNT_ID);
    }

    #[test]
    fn get_set_token_ids() {
        let mut tx = TokenDissociateTransaction::new();
        tx.token_ids(TEST_TOKEN_IDS);

        assert_eq!(tx.get_token_ids(), &TEST_TOKEN_IDS);
    }

    #[test]
    #[should_panic]
    fn get_set_token_ids_frozen_panic() {
        make_transaction().token_ids(TEST_TOKEN_IDS);
    }
}
