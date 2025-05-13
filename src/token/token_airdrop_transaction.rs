// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use hedera_proto::services;
use hedera_proto::services::token_service_client::TokenServiceClient;
use tonic::transport::Channel;

use super::{
    NftId,
    TokenId,
    TokenNftTransfer,
};
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
use crate::transfer_transaction::{
    TokenTransfer,
    Transfer,
};
use crate::{
    AccountId,
    BoxGrpcFuture,
    Error,
    Transaction,
    ValidateChecksums,
};

///
/// Airdrop one or more tokens to one or more accounts.
///
///  ### Effects
///  This distributes tokens from the balance of one or more sending account(s) to the balance
///  of one or more recipient accounts. Accounts MAY receive the tokens in one of four ways.
///
///   - An account already associated to the token to be distributed SHALL receive the
///     airdropped tokens immediately to the recipient account balance.<br/>
///     The fee for this transfer SHALL include the transfer, the airdrop fee, and any custom fees.
///   - An account with available automatic association slots SHALL be automatically
///     associated to the token, and SHALL immediately receive the airdropped tokens to the
///     recipient account balance.<br/>
///     The fee for this transfer SHALL include the transfer, the association, the cost to renew
///     that association once, the airdrop fee, and any custom fees.
///   - An account with "receiver signature required" set SHALL have a "Pending Airdrop" created
///     and must claim that airdrop with a `claimAirdrop` transaction.<br/>
///     The fee for this transfer SHALL include the transfer, the association, the cost to renew
///     that association once, the airdrop fee, and any custom fees. If the pending airdrop is not
///     claimed immediately, the `sender` SHALL pay the cost to renew the token association, and
///     the cost to maintain the pending airdrop, until the pending airdrop is claimed or cancelled.
///   - An account with no available automatic association slots SHALL have a "Pending Airdrop"
///     created and must claim that airdrop with a `claimAirdrop` transaction.<br/>
///     The fee for this transfer SHALL include the transfer, the association, the cost to renew
///     that association once, the airdrop fee, and any custom fees. If the pending airdrop is not
///     claimed immediately, the `sender` SHALL pay the cost to renew the token association, and
///     the cost to maintain the pending airdrop, until the pending airdrop is claimed or cancelled.
///
///  If an airdrop would create a pending airdrop for a fungible/common token, and a pending airdrop
///  for the same sender, receiver, and token already exists, the existing pending airdrop
///  SHALL be updated to add the new amount to the existing airdrop, rather than creating a new
///  pending airdrop.
///
///  Any airdrop that completes immediately SHALL be irreversible. Any airdrop that results in a
///  "Pending Airdrop" MAY be canceled via a `cancelAirdrop` transaction.
///
///  All transfer fees (including custom fees and royalties), as well as the rent cost for the
///  first auto-renewal period for any automatic-association slot occupied by the airdropped
///  tokens, SHALL be charged to the account paying for this transaction.
///
///  ### Record Stream Effects
///  - Each successful transfer SHALL be recorded in `token_transfer_list` for the transaction record.
///  - Each successful transfer that consumes an automatic association slot SHALL populate the
///    `automatic_association` field for the record.
///  - Each pending transfer _created_ SHALL be added to the `pending_airdrops` field for the record.
///  - Each pending transfer _updated_ SHALL be added to the `pending_airdrops` field for the record.
///
pub type TokenAirdropTransaction = Transaction<TokenAirdropTransactionData>;

#[derive(Debug, Clone, Default)]
pub struct TokenAirdropTransactionData {
    /// A list of token transfers representing one or more airdrops.
    token_transfers: Vec<TokenTransfer>,
}

impl TokenAirdropTransaction {
    /// Add a non-approved token transfer.
    pub fn token_transfer(
        &mut self,
        token_id: TokenId,
        account_id: AccountId,
        value: i64,
    ) -> &mut Self {
        self._token_transfer(token_id, account_id, value, false)
    }

    /// Return a non-approved token transfer.
    pub fn get_token_transfers(&self) -> HashMap<TokenId, HashMap<AccountId, i64>> {
        use std::collections::hash_map::Entry;

        // note: using fold instead of nested collects on the off chance a token is in here twice.
        self.data().token_transfers.iter().fold(
            HashMap::with_capacity(self.data().token_transfers.len()),
            |mut map, transfer| {
                let iter = transfer.transfers.iter().map(|it| (it.account_id, it.amount));
                match map.entry(transfer.token_id) {
                    Entry::Occupied(mut it) => it.get_mut().extend(iter),
                    Entry::Vacant(it) => {
                        it.insert(iter.collect());
                    }
                }

                map
            },
        )
    }

    /// Add a non-approved nft transfer.
    pub fn nft_transfer(
        &mut self,
        nft_id: NftId,
        sender: AccountId,
        receiver: AccountId,
    ) -> &mut Self {
        self._nft_transfer(nft_id, sender, receiver, false);
        self
    }

    /// Extract the of token nft transfers.
    pub fn get_nft_transfers(&self) -> HashMap<TokenId, Vec<TokenNftTransfer>> {
        self.data().token_transfers.iter().map(|t| (t.token_id, t.nft_transfers.clone())).collect()
    }

    /// Add a non-approved token transfer with decimals.
    pub fn token_transfer_with_decimals(
        &mut self,
        token_id: TokenId,
        account_id: AccountId,
        amount: i64,
        decimals: u32,
    ) -> &mut Self {
        self._token_transfer_with_decimals(token_id, account_id, amount, false, Some(decimals));
        self
    }

    /// Extract the list of token id decimals.
    pub fn get_token_ids_with_decimals(&self) -> HashMap<TokenId, Option<u32>> {
        self.data().token_transfers.iter().map(|t| (t.token_id, t.expected_decimals)).collect()
    }

    /// Add an approved token transfer to the transaction.
    pub fn approved_token_transfer(
        &mut self,
        token_id: TokenId,
        account_id: AccountId,
        amount: i64,
    ) -> &mut Self {
        self._token_transfer(token_id, account_id, amount, true);
        self
    }

    /// Add an approved nft transfer.
    pub fn approved_nft_transfer(
        &mut self,
        nft_id: NftId,
        sender: AccountId,
        receiver: AccountId,
    ) -> &mut Self {
        self._nft_transfer(nft_id, sender, receiver, true);
        self
    }

    /// Add an approved token transfer with decimals.
    pub fn approved_token_transfer_with_decimals(
        &mut self,
        token_id: TokenId,
        account_id: AccountId,
        amount: i64,
        decimals: u32,
    ) -> &mut Self {
        self._token_transfer_with_decimals(token_id, account_id, amount, true, Some(decimals));
        self
    }

    fn _token_transfer(
        &mut self,
        token_id: TokenId,
        account_id: AccountId,
        amount: i64,
        is_approved: bool,
    ) -> &mut Self {
        let transfer = Transfer { account_id, amount, is_approval: is_approved };
        let data = self.data_mut();

        if let Some(tt) = data.token_transfers.iter_mut().find(|tt| tt.token_id == token_id) {
            if let Some(tt) = tt
                .transfers
                .iter_mut()
                .find(|t| t.account_id == account_id && t.is_approval == is_approved)
            {
                tt.amount += amount;
            } else {
                tt.transfers.push(transfer);
            }
        } else {
            data.token_transfers.push(TokenTransfer {
                token_id,
                expected_decimals: None,
                nft_transfers: Vec::new(),
                transfers: vec![transfer],
            });
        }
        self
    }

    fn _token_transfer_with_decimals(
        &mut self,
        token_id: TokenId,
        account_id: AccountId,
        amount: i64,
        approved: bool,
        expected_decimals: Option<u32>,
    ) -> &mut Self {
        let transfer = Transfer { account_id, amount, is_approval: approved };
        let data = self.data_mut();

        if let Some(tt) = data.token_transfers.iter_mut().find(|tt| tt.token_id == token_id) {
            if tt.expected_decimals.is_some() && tt.expected_decimals != expected_decimals {
                panic!("expected decimals mismatch");
            }

            tt.expected_decimals = expected_decimals;

            if let Some(tt) = tt
                .transfers
                .iter_mut()
                .find(|t| t.account_id == account_id && t.is_approval == approved)
            {
                tt.amount += amount;
            } else {
                tt.transfers.push(transfer);
            }
        } else {
            data.token_transfers.push(TokenTransfer {
                token_id,
                expected_decimals,
                nft_transfers: Vec::new(),
                transfers: vec![transfer],
            });
        }
        self
    }

    fn _nft_transfer(
        &mut self,
        nft_id: NftId,
        sender: AccountId,
        receiver: AccountId,
        is_approved: bool,
    ) -> &mut Self {
        let NftId { token_id, serial } = nft_id;
        let transfer = TokenNftTransfer { token_id, serial, sender, receiver, is_approved };

        let data = self.data_mut();

        if let Some(tt) = data.token_transfers.iter_mut().find(|tt| tt.token_id == token_id) {
            tt.nft_transfers.push(transfer);
        } else {
            data.token_transfers.push(TokenTransfer {
                token_id,
                expected_decimals: None,
                transfers: Vec::new(),
                nft_transfers: vec![transfer],
            });
        }

        self
    }
}

impl TransactionData for TokenAirdropTransactionData {}

impl TransactionExecute for TokenAirdropTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { TokenServiceClient::new(channel).airdrop_tokens(request).await })
    }
}

impl ValidateChecksums for TokenAirdropTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        for token_transfer in &self.token_transfers {
            token_transfer.token_id.validate_checksums(ledger_id)?;
            for transfer in &token_transfer.transfers {
                transfer.account_id.validate_checksums(ledger_id)?;
            }
            for nft_transfer in &token_transfer.nft_transfers {
                nft_transfer.sender.validate_checksums(ledger_id)?;
                nft_transfer.receiver.validate_checksums(ledger_id)?;
            }
        }
        Ok(())
    }
}

impl ToTransactionDataProtobuf for TokenAirdropTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::TokenAirdrop(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for TokenAirdropTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::TokenAirdrop(self.to_protobuf())
    }
}

impl From<TokenAirdropTransactionData> for AnyTransactionData {
    fn from(transaction: TokenAirdropTransactionData) -> Self {
        Self::TokenAirdrop(transaction)
    }
}

impl ToProtobuf for TokenAirdropTransactionData {
    type Protobuf = services::TokenAirdropTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        let mut token_transfers = self.token_transfers.clone();

        // Sort token transfers by token ID
        token_transfers.sort_by(|a, b| {
            a.token_id
                .shard
                .cmp(&b.token_id.shard)
                .then(a.token_id.realm.cmp(&b.token_id.realm))
                .then(a.token_id.num.cmp(&b.token_id.num))
        });

        // Sort transfers within each TokenTransfer
        for tt in &mut token_transfers {
            tt.transfers.sort_by(|a, b| {
                a.account_id
                    .shard
                    .cmp(&b.account_id.shard)
                    .then_with(|| a.account_id.realm.cmp(&b.account_id.realm))
                    .then_with(|| a.account_id.num.cmp(&b.account_id.num))
                    .then_with(|| a.is_approval.cmp(&b.is_approval))
            });

            tt.nft_transfers.sort_by(|a, b| a.serial.cmp(&b.serial));
        }

        services::TokenAirdropTransactionBody {
            token_transfers: token_transfers
                .into_iter()
                .map(|tt| services::TokenTransferList {
                    token: Some(tt.token_id.to_protobuf()),
                    transfers: tt.transfers.into_iter().map(|t| t.to_protobuf()).collect(),
                    nft_transfers: tt
                        .nft_transfers
                        .into_iter()
                        .map(|nt| nt.to_protobuf())
                        .collect(),
                    expected_decimals: tt.expected_decimals.map(|d| d as u32),
                })
                .collect(),
        }
    }
}

impl FromProtobuf<services::TokenAirdropTransactionBody> for TokenAirdropTransactionData {
    fn from_protobuf(pb: services::TokenAirdropTransactionBody) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            token_transfers: pb
                .token_transfers
                .into_iter()
                .map(|t| TokenTransfer::from_protobuf(t))
                .collect::<crate::Result<_>>()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use expect_test::expect_file;
    use hedera_proto::services::{
        self,
        AccountAmount,
        NftTransfer,
        TokenTransferList,
    };

    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::token::TokenAirdropTransactionData;
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
        unused_private_key,
        TEST_ACCOUNT_ID,
        TEST_TOKEN_ID,
    };
    use crate::{
        AccountId,
        AnyTransaction,
        TokenAirdropTransaction,
        TokenId,
    };

    fn make_transaction() -> TokenAirdropTransaction {
        let mut tx = TokenAirdropTransaction::new_for_tests();

        tx.token_transfer(TokenId::new(0, 0, 5005), AccountId::new(0, 0, 5006), 400)
            .token_transfer_with_decimals(
                TokenId::new(0, 0, 5),
                AccountId::new(0, 0, 5005),
                -800,
                3,
            )
            .token_transfer_with_decimals(
                TokenId::new(0, 0, 5),
                AccountId::new(0, 0, 5007),
                -400,
                3,
            )
            .token_transfer(TokenId::new(0, 0, 4), AccountId::new(0, 0, 5008), 1)
            .token_transfer(TokenId::new(0, 0, 4), AccountId::new(0, 0, 5006), -1)
            .nft_transfer(
                TokenId::new(0, 0, 3).nft(2),
                AccountId::new(0, 0, 5008),
                AccountId::new(0, 0, 5007),
            )
            .nft_transfer(
                TokenId::new(0, 0, 3).nft(1),
                AccountId::new(0, 0, 5008),
                AccountId::new(0, 0, 5007),
            )
            .nft_transfer(
                TokenId::new(0, 0, 3).nft(3),
                AccountId::new(0, 0, 5008),
                AccountId::new(0, 0, 5006),
            )
            .nft_transfer(
                TokenId::new(0, 0, 3).nft(4),
                AccountId::new(0, 0, 5007),
                AccountId::new(0, 0, 5006),
            )
            .nft_transfer(
                TokenId::new(0, 0, 2).nft(4),
                AccountId::new(0, 0, 5007),
                AccountId::new(0, 0, 5006),
            )
            .approved_token_transfer(TokenId::new(0, 0, 4), AccountId::new(0, 0, 5006), 123)
            .approved_nft_transfer(
                TokenId::new(0, 0, 4).nft(4),
                AccountId::new(0, 0, 5005),
                AccountId::new(0, 0, 5006),
            )
            .freeze()
            .unwrap()
            .sign(unused_private_key());
        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect_file!["./snapshots/token_airdrop_transaction/serialize.txt"].assert_debug_eq(&tx);
    }

    #[test]
    fn to_from_bytes() {
        let tx = make_transaction();

        let tx2 = AnyTransaction::from_bytes(&tx.to_bytes().unwrap()).unwrap();

        let tx = transaction_body(tx);
        let tx2 = transaction_body(tx2);

        println!("tx: {:?}", tx);
        println!("tx2: {:?}", tx2);

        assert_eq!(tx, tx2)
    }

    #[test]
    fn from_proto_body() {
        let tx = services::TokenAirdropTransactionBody {
            token_transfers: vec![TokenTransferList {
                token: Some(TEST_TOKEN_ID.to_protobuf()),
                transfers: vec![
                    AccountAmount {
                        account_id: Some(AccountId::from_str("0.0.5008").unwrap().to_protobuf()),
                        amount: 200,
                        is_approval: false,
                    },
                    AccountAmount {
                        account_id: Some(AccountId::from_str("0.0.5009").unwrap().to_protobuf()),
                        amount: -100,
                        is_approval: false,
                    },
                    AccountAmount {
                        account_id: Some(AccountId::from_str("0.0.5010").unwrap().to_protobuf()),
                        amount: 40,
                        is_approval: false,
                    },
                    AccountAmount {
                        account_id: Some(AccountId::from_str("0.0.5011").unwrap().to_protobuf()),
                        amount: 20,
                        is_approval: false,
                    },
                ],
                nft_transfers: vec![NftTransfer {
                    sender_account_id: Some(AccountId::from_str("0.0.5010").unwrap().to_protobuf()),
                    receiver_account_id: Some(
                        AccountId::from_str("0.0.5011").unwrap().to_protobuf(),
                    ),
                    serial_number: 1,
                    is_approval: true,
                }],
                expected_decimals: Some(3),
            }],
        };

        let data = TokenAirdropTransactionData::from_protobuf(tx).unwrap();

        let ft_transfers =
            data.token_transfers.iter().flat_map(|t| &t.transfers).collect::<Vec<_>>();
        let nft_transfers =
            data.token_transfers.iter().flat_map(|t| &t.nft_transfers).collect::<Vec<_>>();

        assert_eq!(ft_transfers.len(), 4);
        assert_eq!(nft_transfers.len(), 1);
    }

    #[test]
    fn get_set_token_transfers() {
        let token_id = TokenId::new(0, 0, 123);
        let account_id = AccountId::new(0, 0, 456);
        let value = 1000;
        let mut tx = TokenAirdropTransaction::new();
        tx.token_transfer(token_id, account_id, value);

        let token_transfers = tx.get_token_transfers();

        assert!(token_transfers.contains_key(&token_id));
        assert_eq!(token_transfers.len(), 1);
        assert_eq!(value, *token_transfers.get(&token_id).unwrap().get(&account_id).unwrap());
    }

    #[test]
    #[should_panic]
    fn get_set_token_transfers_frozen_panic() {
        make_transaction().token_transfer(TEST_TOKEN_ID, TEST_ACCOUNT_ID, 142);
    }

    #[test]
    fn get_set_nft_transfer() {
        let (nft_id, sender, receiver) =
            (TEST_TOKEN_ID.nft(1), TEST_ACCOUNT_ID, AccountId::new(0, 0, 5011));
        let mut tx = TokenAirdropTransaction::new();
        tx.nft_transfer(nft_id, sender, receiver);
        let nft_transfers = tx.get_nft_transfers();

        assert!(nft_transfers.contains_key(&nft_id.token_id));
        assert_eq!(1, nft_transfers.get(&nft_id.token_id).unwrap().len());
        assert_eq!(sender, nft_transfers.get(&nft_id.token_id).unwrap()[0].sender);
        assert_eq!(receiver, nft_transfers.get(&nft_id.token_id).unwrap()[0].receiver);
    }

    #[test]
    #[should_panic]
    fn get_set_nft_transfer_frozen_panic() {
        make_transaction().nft_transfer(
            TEST_TOKEN_ID.nft(1),
            TEST_ACCOUNT_ID,
            AccountId::new(0, 0, 156),
        );
    }

    #[test]
    fn get_set_approved_nft_transfer() {
        let (nft_id, sender, receiver) =
            (TEST_TOKEN_ID.nft(1), TEST_ACCOUNT_ID, AccountId::new(0, 0, 123));
        let mut tx = TokenAirdropTransaction::new();
        tx.approved_nft_transfer(nft_id, sender, receiver);
        let nft_transfers = tx.get_nft_transfers();

        assert!(nft_transfers.contains_key(&nft_id.token_id));
        assert_eq!(nft_transfers.get(&nft_id.token_id).unwrap().len(), 1);
        assert_eq!(sender, nft_transfers.get(&nft_id.token_id).unwrap()[0].sender);
        assert_eq!(receiver, nft_transfers.get(&nft_id.token_id).unwrap()[0].receiver);
    }

    #[test]
    fn get_set_approved_token_transfer() {
        let (token_id, account_id, value) =
            (TokenId::new(0, 0, 1420), AccountId::new(0, 0, 415), 1000);
        let mut tx = TokenAirdropTransaction::new();
        tx.approved_token_transfer(token_id, account_id, value);

        let token_transfers = tx.get_token_transfers();

        assert!(token_transfers.contains_key(&token_id));
        assert_eq!(token_transfers.len(), 1);
        assert_eq!(value, *token_transfers.get(&token_id).unwrap().get(&account_id).unwrap());
    }

    #[test]
    fn get_set_token_id_decimals() {
        let (nft_id, sender, receiver) =
            (TEST_TOKEN_ID.nft(1), TEST_ACCOUNT_ID, AccountId::new(0, 0, 123));
        let mut tx = TokenAirdropTransaction::new();
        tx.approved_nft_transfer(nft_id, sender, receiver);
        let nft_transfers = tx.get_nft_transfers();

        assert!(nft_transfers.contains_key(&nft_id.token_id));
        assert_eq!(nft_transfers.get(&nft_id.token_id).unwrap().len(), 1);
        assert_eq!(sender, nft_transfers.get(&nft_id.token_id).unwrap()[0].sender);
        assert_eq!(receiver, nft_transfers.get(&nft_id.token_id).unwrap()[0].receiver);
    }
}
