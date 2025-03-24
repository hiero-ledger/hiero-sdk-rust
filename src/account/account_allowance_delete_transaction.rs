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
    NftId,
    TokenId,
    Transaction,
    ValidateChecksums,
};

/// Deletes one or more non-fungible approved allowances from an owner's account. This operation
/// will remove the allowances granted to one or more specific non-fungible token serial numbers. Each owner account
/// listed as wiping an allowance must sign the transaction. Hbar and fungible token allowances
/// can be removed by setting the amount to zero in
/// [`AccountAllowanceApproveTransaction`](crate::AccountAllowanceApproveTransaction).
pub type AccountAllowanceDeleteTransaction = Transaction<AccountAllowanceDeleteTransactionData>;

#[derive(Debug, Clone, Default)]
pub struct AccountAllowanceDeleteTransactionData {
    nft_allowances: Vec<NftRemoveAllowance>,
}

#[derive(Debug, Clone)]
pub struct NftRemoveAllowance {
    /// token that the allowance pertains to
    pub token_id: TokenId,

    /// The account ID that owns token.
    pub owner_account_id: AccountId,

    /// The list of serial numbers to remove allowances from.
    pub serials: Vec<i64>,
}

impl AccountAllowanceDeleteTransaction {
    /// Get the nft allowances that will be removed.
    #[must_use]
    pub fn get_nft_allowances(&self) -> &[NftRemoveAllowance] {
        &self.data().nft_allowances
    }

    /// Remove all nft token allowances.
    pub fn delete_all_token_nft_allowances(
        &mut self,
        nft_id: NftId,
        owner_account_id: AccountId,
    ) -> &mut Self {
        let data = self.data_mut();

        if let Some(allowance) = data.nft_allowances.iter_mut().find(|allowance| {
            allowance.token_id == nft_id.token_id && allowance.owner_account_id == owner_account_id
        }) {
            allowance.serials.push(nft_id.serial as i64);
        } else {
            data.nft_allowances.push(NftRemoveAllowance {
                token_id: nft_id.token_id,
                serials: vec![nft_id.serial as i64],
                owner_account_id,
            });
        }

        self
    }
}

impl TransactionData for AccountAllowanceDeleteTransactionData {}

impl TransactionExecute for AccountAllowanceDeleteTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { CryptoServiceClient::new(channel).delete_allowances(request).await })
    }
}

impl ValidateChecksums for AccountAllowanceDeleteTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        for allowance in &self.nft_allowances {
            allowance.token_id.validate_checksums(ledger_id)?;
            allowance.owner_account_id.validate_checksums(ledger_id)?;
        }
        Ok(())
    }
}

impl ToTransactionDataProtobuf for AccountAllowanceDeleteTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::CryptoDeleteAllowance(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for AccountAllowanceDeleteTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::CryptoDeleteAllowance(self.to_protobuf())
    }
}

impl From<AccountAllowanceDeleteTransactionData> for AnyTransactionData {
    fn from(transaction: AccountAllowanceDeleteTransactionData) -> Self {
        Self::AccountAllowanceDelete(transaction)
    }
}

impl FromProtobuf<services::CryptoDeleteAllowanceTransactionBody>
    for AccountAllowanceDeleteTransactionData
{
    fn from_protobuf(pb: services::CryptoDeleteAllowanceTransactionBody) -> crate::Result<Self> {
        Ok(Self { nft_allowances: Vec::from_protobuf(pb.nft_allowances)? })
    }
}

impl ToProtobuf for AccountAllowanceDeleteTransactionData {
    type Protobuf = services::CryptoDeleteAllowanceTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::CryptoDeleteAllowanceTransactionBody {
            nft_allowances: self.nft_allowances.to_protobuf(),
        }
    }
}

impl FromProtobuf<services::NftRemoveAllowance> for NftRemoveAllowance {
    fn from_protobuf(pb: services::NftRemoveAllowance) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            token_id: TokenId::from_protobuf(pb_getf!(pb, token_id)?)?,
            owner_account_id: AccountId::from_protobuf(pb_getf!(pb, owner)?)?,
            serials: pb.serial_numbers,
        })
    }
}

impl ToProtobuf for NftRemoveAllowance {
    type Protobuf = services::NftRemoveAllowance;

    fn to_protobuf(&self) -> Self::Protobuf {
        Self::Protobuf {
            token_id: Some(self.token_id.to_protobuf()),
            owner: Some(self.owner_account_id.to_protobuf()),
            serial_numbers: self.serials.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
    };
    use crate::{
        AccountAllowanceDeleteTransaction,
        AccountId,
        AnyTransaction,
        TokenId,
    };

    fn make_transaction() -> AccountAllowanceDeleteTransaction {
        let owner_id: AccountId = AccountId::new(5, 6, 7);

        let invalid_token_ids: [TokenId; 2] = [TokenId::new(4, 4, 4), TokenId::new(8, 8, 8)];

        let mut tx = AccountAllowanceDeleteTransaction::new_for_tests();

        tx.delete_all_token_nft_allowances(invalid_token_ids[0].nft(123), owner_id)
            .delete_all_token_nft_allowances(invalid_token_ids[0].nft(456), owner_id)
            .delete_all_token_nft_allowances(invalid_token_ids[1].nft(456), owner_id)
            .delete_all_token_nft_allowances(invalid_token_ids[0].nft(789), owner_id)
            .freeze()
            .unwrap();
        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            CryptoDeleteAllowance(
                CryptoDeleteAllowanceTransactionBody {
                    nft_allowances: [
                        NftRemoveAllowance {
                            token_id: Some(
                                TokenId {
                                    shard_num: 4,
                                    realm_num: 4,
                                    token_num: 4,
                                },
                            ),
                            owner: Some(
                                AccountId {
                                    shard_num: 5,
                                    realm_num: 6,
                                    account: Some(
                                        AccountNum(
                                            7,
                                        ),
                                    ),
                                },
                            ),
                            serial_numbers: [
                                123,
                                456,
                                789,
                            ],
                        },
                        NftRemoveAllowance {
                            token_id: Some(
                                TokenId {
                                    shard_num: 8,
                                    realm_num: 8,
                                    token_num: 8,
                                },
                            ),
                            owner: Some(
                                AccountId {
                                    shard_num: 5,
                                    realm_num: 6,
                                    account: Some(
                                        AccountNum(
                                            7,
                                        ),
                                    ),
                                },
                            ),
                            serial_numbers: [
                                456,
                            ],
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
}
