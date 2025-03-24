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
    TokenId,
    Transaction,
    ValidateChecksums,
};

/// Revokes KYC from the account for the given token.
///
/// Must be signed by the Token's kycKey.
///
/// Once executed the Account is marked as KYC Revoked.
///
/// - If the provided account is not found, the transaction will resolve to `INVALID_ACCOUNT_ID`.
/// - If the provided account has been deleted, the transaction will resolve to `ACCOUNT_DELETED`.
/// - If the provided token is not found, the transaction will resolve to `INVALID_TOKEN_ID`.
/// - If the provided token has been deleted, the transaction will resolve to `TOKEN_WAS_DELETED`.
/// - If an Association between the provided token and account is not found, the transaction will
/// resolve to `TOKEN_NOT_ASSOCIATED_TO_ACCOUNT`.
/// - If no KYC Key is defined, the transaction will resolve to `TOKEN_HAS_NO_KYC_KEY`.
pub type TokenRevokeKycTransaction = Transaction<TokenRevokeKycTransactionData>;

#[derive(Debug, Clone, Default)]
pub struct TokenRevokeKycTransactionData {
    /// The account to have their KYC revoked.
    account_id: Option<AccountId>,

    /// The token for which this account will have their KYC revoked.
    token_id: Option<TokenId>,
}

impl TokenRevokeKycTransaction {
    /// Returns the account to have their KYC revoked.
    #[must_use]
    pub fn get_account_id(&self) -> Option<AccountId> {
        self.data().account_id
    }
    /// Sets the account to have their KYC revoked.
    pub fn account_id(&mut self, account_id: AccountId) -> &mut Self {
        self.data_mut().account_id = Some(account_id);
        self
    }

    /// Returns the token for which the account will have their KYC revoked.
    #[must_use]
    pub fn get_token_id(&self) -> Option<TokenId> {
        self.data().token_id
    }

    /// Sets the token for which this account will have their KYC revoked.
    pub fn token_id(&mut self, token_id: impl Into<TokenId>) -> &mut Self {
        self.data_mut().token_id = Some(token_id.into());
        self
    }
}

impl TransactionData for TokenRevokeKycTransactionData {}

impl TransactionExecute for TokenRevokeKycTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async {
            TokenServiceClient::new(channel).revoke_kyc_from_token_account(request).await
        })
    }
}

impl ValidateChecksums for TokenRevokeKycTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> crate::Result<()> {
        self.token_id.validate_checksums(ledger_id)?;
        self.account_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for TokenRevokeKycTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::TokenRevokeKyc(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for TokenRevokeKycTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::TokenRevokeKyc(self.to_protobuf())
    }
}

impl From<TokenRevokeKycTransactionData> for AnyTransactionData {
    fn from(transaction: TokenRevokeKycTransactionData) -> Self {
        Self::TokenRevokeKyc(transaction)
    }
}

impl FromProtobuf<services::TokenRevokeKycTransactionBody> for TokenRevokeKycTransactionData {
    fn from_protobuf(pb: services::TokenRevokeKycTransactionBody) -> crate::Result<Self> {
        Ok(Self {
            account_id: Option::from_protobuf(pb.account)?,
            token_id: Option::from_protobuf(pb.token)?,
        })
    }
}

impl ToProtobuf for TokenRevokeKycTransactionData {
    type Protobuf = services::TokenRevokeKycTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::TokenRevokeKycTransactionBody {
            token: self.token_id.to_protobuf(),
            account: self.account_id.to_protobuf(),
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hedera_proto::services;

    use super::TokenRevokeKycTransactionData;
    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
    };
    use crate::{
        AccountId,
        AnyTransaction,
        TokenId,
        TokenRevokeKycTransaction,
    };

    const TEST_TOKEN_ID: TokenId = TokenId::new(4, 2, 0);
    const TEST_ACCOUNT_ID: AccountId =
        AccountId { shard: 6, realm: 9, num: 0, alias: None, evm_address: None, checksum: None };

    fn make_transaction() -> TokenRevokeKycTransaction {
        let mut tx = TokenRevokeKycTransaction::new_for_tests();

        tx.token_id(TEST_TOKEN_ID).account_id(TEST_ACCOUNT_ID).freeze().unwrap();

        tx
    }

    #[test]
    fn seriralize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            TokenRevokeKyc(
                TokenRevokeKycTransactionBody {
                    token: Some(
                        TokenId {
                            shard_num: 4,
                            realm_num: 2,
                            token_num: 0,
                        },
                    ),
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
        let tx = services::TokenRevokeKycTransactionBody {
            token: Some(TEST_TOKEN_ID.to_protobuf()),
            account: Some(TEST_ACCOUNT_ID.to_protobuf()),
        };

        let tx = TokenRevokeKycTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(tx.account_id, Some(TEST_ACCOUNT_ID));
        assert_eq!(tx.token_id, Some(TEST_TOKEN_ID));
    }

    #[test]
    fn get_set_account_id() {
        let mut tx = TokenRevokeKycTransaction::new();
        tx.account_id(TEST_ACCOUNT_ID);

        assert_eq!(tx.get_account_id(), Some(TEST_ACCOUNT_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_account_id_frozen_panic() {
        let mut tx = make_transaction();
        tx.account_id(TEST_ACCOUNT_ID);
    }

    #[test]
    fn get_set_token_id() {
        let mut tx = TokenRevokeKycTransaction::new();
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
