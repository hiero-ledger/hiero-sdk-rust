// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::crypto_service_client::CryptoServiceClient;
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
    Transaction,
    ValidateChecksums,
};

/// Mark an account as deleted, moving all its current hbars to another account.
///
/// It will remain in the ledger, marked as deleted, until it expires.
/// Transfers into it a deleted account will fail.
///
pub type AccountDeleteTransaction = Transaction<AccountDeleteTransactionData>;

#[derive(Debug, Clone, Default)]
pub struct AccountDeleteTransactionData {
    /// The account ID which will receive all remaining hbars.
    transfer_account_id: Option<AccountId>,

    /// The account ID which should be deleted.
    account_id: Option<AccountId>,
}

impl AccountDeleteTransaction {
    /// Get the account ID which should be deleted.
    #[must_use]
    pub fn get_account_id(&self) -> Option<AccountId> {
        self.data().account_id
    }

    /// Sets the account ID which should be deleted.
    pub fn account_id(&mut self, id: AccountId) -> &mut Self {
        self.data_mut().account_id = Some(id);
        self
    }

    /// Get the account ID which will receive all remaining hbars.
    #[must_use]
    pub fn get_transfer_account_id(&self) -> Option<AccountId> {
        self.data().transfer_account_id
    }

    /// Sets the account ID which will receive all remaining hbars.
    pub fn transfer_account_id(&mut self, id: AccountId) -> &mut Self {
        self.data_mut().transfer_account_id = Some(id);
        self
    }
}

impl TransactionData for AccountDeleteTransactionData {}

impl TransactionExecute for AccountDeleteTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { CryptoServiceClient::new(channel).crypto_delete(request).await })
    }
}

impl ValidateChecksums for AccountDeleteTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.transfer_account_id.validate_checksums(ledger_id)?;
        self.account_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for AccountDeleteTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::CryptoDelete(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for AccountDeleteTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::CryptoDelete(self.to_protobuf())
    }
}

impl From<AccountDeleteTransactionData> for AnyTransactionData {
    fn from(transaction: AccountDeleteTransactionData) -> Self {
        Self::AccountDelete(transaction)
    }
}

impl FromProtobuf<services::CryptoDeleteTransactionBody> for AccountDeleteTransactionData {
    fn from_protobuf(pb: services::CryptoDeleteTransactionBody) -> crate::Result<Self> {
        Ok(Self {
            transfer_account_id: Option::from_protobuf(pb.transfer_account_id)?,
            account_id: Option::from_protobuf(pb.delete_account_id)?,
        })
    }
}

impl ToProtobuf for AccountDeleteTransactionData {
    type Protobuf = services::CryptoDeleteTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        let account_id = self.account_id.to_protobuf();
        let transfer_account_id = self.transfer_account_id.to_protobuf();

        services::CryptoDeleteTransactionBody { transfer_account_id, delete_account_id: account_id }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hedera_proto::services;

    use crate::account::AccountDeleteTransactionData;
    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
    };
    use crate::{
        AccountDeleteTransaction,
        AccountId,
        AnyTransaction,
    };

    const ACCOUNT_ID: AccountId = AccountId::new(0, 0, 5007);
    const TRANSFER_ACCOUNT_ID: AccountId = AccountId::new(0, 0, 9);

    fn make_transaction() -> AccountDeleteTransaction {
        let mut tx = AccountDeleteTransaction::new_for_tests();

        tx.account_id(ACCOUNT_ID).transfer_account_id(TRANSFER_ACCOUNT_ID).freeze().unwrap();

        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            CryptoDelete(
                CryptoDeleteTransactionBody {
                    transfer_account_id: Some(
                        AccountId {
                            shard_num: 0,
                            realm_num: 0,
                            account: Some(
                                AccountNum(
                                    9,
                                ),
                            ),
                        },
                    ),
                    delete_account_id: Some(
                        AccountId {
                            shard_num: 0,
                            realm_num: 0,
                            account: Some(
                                AccountNum(
                                    5007,
                                ),
                            ),
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
        let tx = services::CryptoDeleteTransactionBody {
            delete_account_id: Some(ACCOUNT_ID.to_protobuf()),
            transfer_account_id: Some(TRANSFER_ACCOUNT_ID.to_protobuf()),
        };

        let tx = AccountDeleteTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(tx.account_id, Some(ACCOUNT_ID));
        assert_eq!(tx.transfer_account_id, Some(TRANSFER_ACCOUNT_ID));
    }

    #[test]
    fn get_set_account_id() {
        let mut tx = AccountDeleteTransaction::new();
        tx.account_id(ACCOUNT_ID);

        assert_eq!(tx.get_account_id(), Some(ACCOUNT_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_account_id_frozen_panics() {
        let mut tx = make_transaction();

        tx.account_id(ACCOUNT_ID);
    }

    #[test]
    fn get_set_transfer_account_id() {
        let mut tx = AccountDeleteTransaction::new();
        tx.transfer_account_id(TRANSFER_ACCOUNT_ID);

        assert_eq!(tx.get_transfer_account_id(), Some(TRANSFER_ACCOUNT_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_transfer_account_id_frozen_panics() {
        let mut tx = make_transaction();

        tx.transfer_account_id(TRANSFER_ACCOUNT_ID);
    }
}
