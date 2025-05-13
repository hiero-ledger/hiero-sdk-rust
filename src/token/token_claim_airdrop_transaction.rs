// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::token_service_client::TokenServiceClient;
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::pending_airdrop_id::PendingAirdropId;
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
    Transaction,
    ValidateChecksums,
};

/// Token claim airdrop
/// Complete one or more pending transfers on behalf of the
/// recipient(s) for an airdrop.
///
/// The sender MUST have sufficient balance to fulfill the airdrop at the
/// time of claim. If the sender does not have sufficient balance, the
/// claim SHALL fail.
/// Each pending airdrop successfully claimed SHALL be removed from state and
/// SHALL NOT be available to claim again.
/// Each claim SHALL be represented in the transaction body and
/// SHALL NOT be restated in the record file.
/// All claims MUST succeed for this transaction to succeed.
///
/// ### Record Stream Effects
/// The completed transfers SHALL be present in the transfer list.
///
pub type TokenClaimAirdropTransaction = Transaction<TokenClaimAirdropTransactionData>;

#[derive(Debug, Clone, Default)]
pub struct TokenClaimAirdropTransactionData {
    /// A list of one or more pending airdrop identifiers.
    ///
    /// This transaction MUST be signed by the account identified by
    /// the `receiver_id` for each entry in this list.
    /// This list MUST contain between 1 and 10 entries, inclusive.
    /// This list MUST NOT have any duplicate entries.
    pending_airdrop_ids: Vec<PendingAirdropId>,
}

impl TokenClaimAirdropTransaction {
    /// Adds the list of pending airdrop identifiers to claim.
    pub fn pending_airdrop_ids(
        &mut self,
        pending_airdrop_ids: impl IntoIterator<Item = PendingAirdropId>,
    ) -> &mut Self {
        self.data_mut().pending_airdrop_ids = pending_airdrop_ids.into_iter().collect();
        self
    }

    /// Returns the list of pending airdrop identifiers to claim.
    #[must_use]
    pub fn get_pending_airdrop_ids(&self) -> Vec<PendingAirdropId> {
        self.data().pending_airdrop_ids.clone()
    }

    /// Adds a pending airdrop identifier to the list of pending airdrop identifiers.
    pub fn add_pending_airdrop_id(&mut self, pending_airdrop_id: PendingAirdropId) -> &mut Self {
        self.data_mut().pending_airdrop_ids.push(pending_airdrop_id);
        self
    }
}

impl TransactionData for TokenClaimAirdropTransactionData {}

impl TransactionExecute for TokenClaimAirdropTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { TokenServiceClient::new(channel).claim_airdrop(request).await })
    }
}

impl ValidateChecksums for TokenClaimAirdropTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.pending_airdrop_ids
            .iter()
            .try_for_each(|pending_airdrop_id| pending_airdrop_id.validate_checksums(ledger_id))?;
        Ok(())
    }
}

impl ToTransactionDataProtobuf for TokenClaimAirdropTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::TokenClaimAirdrop(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for TokenClaimAirdropTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::TokenClaimAirdrop(self.to_protobuf())
    }
}

impl From<TokenClaimAirdropTransactionData> for AnyTransactionData {
    fn from(transaction: TokenClaimAirdropTransactionData) -> Self {
        Self::TokenClaimAirdrop(transaction)
    }
}

impl ToProtobuf for TokenClaimAirdropTransactionData {
    type Protobuf = services::TokenClaimAirdropTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::TokenClaimAirdropTransactionBody {
            pending_airdrops: self.pending_airdrop_ids.iter().map(|id| id.to_protobuf()).collect(),
        }
    }
}

impl FromProtobuf<services::TokenClaimAirdropTransactionBody> for TokenClaimAirdropTransactionData {
    fn from_protobuf(pb: services::TokenClaimAirdropTransactionBody) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let pending_airdrop_ids = pb
            .pending_airdrops
            .into_iter()
            .map(|id: services::PendingAirdropId| PendingAirdropId::from_protobuf(id))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { pending_airdrop_ids })
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect_file;
    use hedera_proto::services;

    use crate::pending_airdrop_id::PendingAirdropId;
    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::token::TokenClaimAirdropTransactionData;
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
        unused_private_key,
    };
    use crate::{
        AccountId,
        AnyTransaction,
        TokenClaimAirdropTransaction,
        TokenId,
    };

    fn make_transaction() -> TokenClaimAirdropTransaction {
        let pending_airdrop_ids: Vec<PendingAirdropId> = vec![
            PendingAirdropId::new_token_id(
                AccountId::new(0, 2, 134),
                AccountId::new(0, 2, 6),
                TokenId::new(0, 0, 312),
            ),
            PendingAirdropId::new_nft_id(
                AccountId::new(0, 2, 134),
                AccountId::new(0, 2, 6),
                TokenId::new(1, 3, 5).nft(2),
            ),
        ]
        .into_iter()
        .collect();
        let mut tx = TokenClaimAirdropTransaction::new_for_tests();

        tx.pending_airdrop_ids(pending_airdrop_ids).freeze().unwrap().sign(unused_private_key());
        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect_file!["./snapshots/token_claim_airdrop_transaction/serialize.txt"]
            .assert_debug_eq(&tx);
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
        let tx = services::TokenClaimAirdropTransactionBody {
            pending_airdrops: vec![
                PendingAirdropId::new_token_id(
                    AccountId::new(0, 0, 415),
                    AccountId::new(0, 0, 6),
                    TokenId::new(0, 0, 312),
                )
                .to_protobuf(),
                PendingAirdropId::new_nft_id(
                    AccountId::new(0, 2, 134),
                    AccountId::new(0, 2, 6),
                    TokenId::new(0, 0, 123).nft(1),
                )
                .to_protobuf(),
            ],
        };

        let data = TokenClaimAirdropTransactionData::from_protobuf(tx).unwrap();

        let nft_ids: Vec<_> =
            data.pending_airdrop_ids.clone().into_iter().filter_map(|id| id.nft_id).collect();
        let token_ids: Vec<_> =
            data.pending_airdrop_ids.into_iter().filter_map(|id| id.token_id).collect();

        assert_eq!(nft_ids.len(), 1);
        assert_eq!(token_ids.len(), 1);
        assert!(token_ids.contains(&TokenId::new(0, 0, 312)));
        assert!(nft_ids.contains(&TokenId::new(0, 0, 123).nft(1)));
    }

    #[test]
    fn get_set_pending_airdrop_ids() {
        let pending_airdrop_ids = [
            PendingAirdropId::new_token_id(
                AccountId::new(0, 0, 134),
                AccountId::new(0, 0, 6),
                TokenId::new(0, 0, 420),
            ),
            PendingAirdropId::new_nft_id(
                AccountId::new(0, 2, 134),
                AccountId::new(0, 2, 6),
                TokenId::new(0, 0, 112).nft(1),
            ),
        ];
        let mut tx = TokenClaimAirdropTransaction::new();
        tx.pending_airdrop_ids(pending_airdrop_ids);

        let pending_airdrop_ids = tx.get_pending_airdrop_ids();

        let nft_ids: Vec<_> =
            pending_airdrop_ids.clone().into_iter().filter_map(|id| id.nft_id).collect();
        let token_ids: Vec<_> =
            pending_airdrop_ids.into_iter().filter_map(|id| id.token_id).collect();

        assert_eq!(nft_ids.len(), 1);
        assert_eq!(token_ids.len(), 1);

        assert!(token_ids.contains(&TokenId::new(0, 0, 420)));
        assert!(nft_ids.contains(&TokenId::new(0, 0, 112).nft(1)));
    }

    #[test]
    #[should_panic]
    fn set_pending_airdrop_ids_frozen_panic() {
        make_transaction().pending_airdrop_ids([PendingAirdropId::new_token_id(
            AccountId::new(0, 0, 134),
            AccountId::new(0, 0, 6),
            TokenId::new(0, 0, 420),
        )]);
    }

    #[test]
    fn get_set_add_pending_airdrop_id() {
        let mut tx = TokenClaimAirdropTransaction::new();
        tx.add_pending_airdrop_id(PendingAirdropId::new_token_id(
            AccountId::new(0, 0, 134),
            AccountId::new(0, 0, 6),
            TokenId::new(0, 0, 420),
        ));

        let pending_airdrop_ids = tx.get_pending_airdrop_ids();

        let token_ids: Vec<_> =
            pending_airdrop_ids.into_iter().filter_map(|id| id.token_id).collect();

        assert!(token_ids.contains(&TokenId::new(0, 0, 420)));
    }
}
