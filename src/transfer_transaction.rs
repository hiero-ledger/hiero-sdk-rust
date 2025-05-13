// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::ops::Not;

use hedera_proto::services;
use hedera_proto::services::crypto_service_client::CryptoServiceClient;
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
    Hbar,
    NftId,
    ToProtobuf,
    TokenId,
    TokenNftTransfer,
    Transaction,
    ValidateChecksums,
};

/// Transfers cryptocurrency among two or more accounts by making the desired adjustments to their
/// balances.
///
/// Each transfer list can specify up to 10 adjustments. Each negative amount is withdrawn
/// from the corresponding account (a sender), and each positive one is added to the corresponding
/// account (a receiver). The amounts list must sum to zero.
///
/// All transfers are in the lowest denomination, for `Hbar` that is tinybars (although `Hbar` handles this itself).
///
/// As an example: for a fungible token with `3` decimals (and let's say the symbol is `ƒ`), transferring `1` _always_ transfers `0.001 ƒ`.
pub type TransferTransaction = Transaction<TransferTransactionData>;

#[derive(Debug, Clone, Default)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct TransferTransactionData {
    transfers: Vec<Transfer>,
    token_transfers: Vec<TokenTransfer>,
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub(crate) struct Transfer {
    /// The account involved in the transfer.
    pub account_id: AccountId,

    /// The value of the transfer.
    pub amount: i64,

    /// If this is an approved transfer.
    pub is_approval: bool,
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub(crate) struct TokenTransfer {
    pub token_id: TokenId,

    pub transfers: Vec<Transfer>,

    pub nft_transfers: Vec<TokenNftTransfer>,

    pub expected_decimals: Option<u32>,
}

impl TransferTransaction {
    fn _hbar_transfer(&mut self, account_id: AccountId, amount: Hbar, approved: bool) -> &mut Self {
        self.data_mut().transfers.push(Transfer {
            account_id,
            amount: amount.to_tinybars(),
            is_approval: approved,
        });

        self
    }

    /// Add a non-approved hbar transfer to the transaction.
    pub fn hbar_transfer(&mut self, account_id: AccountId, amount: Hbar) -> &mut Self {
        self._hbar_transfer(account_id, amount, false)
    }

    /// Add an approved hbar transfer to the transaction.
    pub fn approved_hbar_transfer(&mut self, account_id: AccountId, amount: Hbar) -> &mut Self {
        self._hbar_transfer(account_id, amount, true)
    }

    /// Returns all transfers associated with this transaction.
    pub fn get_hbar_transfers(&self) -> HashMap<AccountId, Hbar> {
        self.data()
            .transfers
            .iter()
            .map(|it| (it.account_id, Hbar::from_tinybars(it.amount)))
            .collect()
    }

    fn _token_transfer(
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
            tt.expected_decimals = expected_decimals;
            tt.transfers.push(transfer);
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

    /// Add a non-approved token transfer to the transaction.
    ///
    /// `amount` is in the lowest denomination for the token (if the token has `2` decimals this would be `0.01` tokens).
    pub fn token_transfer(
        &mut self,
        token_id: TokenId,
        account_id: AccountId,
        amount: i64,
    ) -> &mut Self {
        self._token_transfer(token_id, account_id, amount, false, None)
    }

    /// Add an approved token transfer to the transaction.
    ///
    /// `amount` is in the lowest denomination for the token (if the token has `2` decimals this would be `0.01` tokens).
    pub fn approved_token_transfer(
        &mut self,
        token_id: TokenId,
        account_id: AccountId,
        amount: i64,
    ) -> &mut Self {
        self._token_transfer(token_id, account_id, amount, true, None)
    }

    // todo: make the examples into code, just not sure how to do that.
    /// Add a non-approved token transfer to the transaction, ensuring that the token has `expected_decimals` decimals.
    ///
    /// `amount` is _still_ in the lowest denomination, however,
    /// you will get an error if the token has a different amount of decimals than `expected_decimals`.
    pub fn token_transfer_with_decimals(
        &mut self,
        token_id: TokenId,
        account_id: AccountId,
        amount: i64,
        expected_decimals: u32,
    ) -> &mut Self {
        self._token_transfer(token_id, account_id, amount, false, Some(expected_decimals))
    }

    /// Add an approved token transfer, ensuring that the token has `expected_decimals` decimals.
    ///
    /// `amount` is _still_ in the lowest denomination, however,
    /// you will get an error if the token has a different amount of decimals than `expected_decimals`.
    pub fn approved_token_transfer_with_decimals(
        &mut self,
        token_id: TokenId,
        account_id: AccountId,
        amount: i64,
        expected_decimals: u32,
    ) -> &mut Self {
        self._token_transfer(token_id, account_id, amount, true, Some(expected_decimals))
    }

    /// Returns all the token transfers associated associated with this transaction.
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

    /// Returns the decimals associated with each token.
    pub fn get_token_decimals(&self) -> HashMap<TokenId, u32> {
        self.data()
            .token_transfers
            .iter()
            .filter_map(|it| it.expected_decimals.map(|decimals| (it.token_id, decimals)))
            .collect()
    }

    fn _nft_transfer(
        &mut self,
        nft_id: NftId,
        sender_account_id: AccountId,
        receiver_account_id: AccountId,
        approved: bool,
    ) -> &mut Self {
        let NftId { token_id, serial } = nft_id;
        let transfer = TokenNftTransfer {
            token_id,
            serial,
            sender: sender_account_id,
            receiver: receiver_account_id,
            is_approved: approved,
        };

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

    /// Add an approved nft transfer to the transaction.
    pub fn approved_nft_transfer(
        &mut self,
        nft_id: impl Into<NftId>,
        sender_account_id: AccountId,
        receiver_account_id: AccountId,
    ) -> &mut Self {
        self._nft_transfer(nft_id.into(), sender_account_id, receiver_account_id, true)
    }

    /// Add a non-approved nft transfer to the transaction.
    pub fn nft_transfer(
        &mut self,
        nft_id: impl Into<NftId>,
        sender_account_id: AccountId,
        receiver_account_id: AccountId,
    ) -> &mut Self {
        self._nft_transfer(nft_id.into(), sender_account_id, receiver_account_id, false)
    }

    /// Returns all the NFT transfers associated with this transaction.
    pub fn get_nft_transfers(&self) -> HashMap<TokenId, Vec<TokenNftTransfer>> {
        self.data()
            .token_transfers
            .iter()
            .map(|it| (it.token_id, it.nft_transfers.clone()))
            .collect()
    }
}

impl TransactionExecute for TransferTransactionData {
    // noinspection DuplicatedCode
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { CryptoServiceClient::new(channel).crypto_transfer(request).await })
    }
}

impl TransactionData for TransferTransactionData {}

impl ValidateChecksums for TransferTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        for transfer in &self.transfers {
            transfer.account_id.validate_checksums(ledger_id)?;
        }
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

impl FromProtobuf<services::AccountAmount> for Transfer {
    fn from_protobuf(pb: services::AccountAmount) -> crate::Result<Self> {
        Ok(Self {
            amount: pb.amount,
            account_id: AccountId::from_protobuf(pb_getf!(pb, account_id)?)?,
            is_approval: pb.is_approval,
        })
    }
}

impl ToProtobuf for Transfer {
    type Protobuf = services::AccountAmount;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::AccountAmount {
            amount: self.amount,
            account_id: Some(self.account_id.to_protobuf()),
            is_approval: self.is_approval,
        }
    }
}

impl FromProtobuf<services::TokenTransferList> for TokenTransfer {
    fn from_protobuf(pb: services::TokenTransferList) -> crate::Result<Self> {
        let token_id = TokenId::from_protobuf(pb_getf!(pb, token)?)?;

        Ok(Self {
            token_id,
            transfers: Vec::from_protobuf(pb.transfers)?,
            nft_transfers: pb
                .nft_transfers
                .into_iter()
                .map(|pb| TokenNftTransfer::from_protobuf(pb, token_id))
                .collect::<Result<Vec<_>, _>>()?,
            expected_decimals: pb.expected_decimals,
        })
    }
}

impl ToProtobuf for TokenTransfer {
    type Protobuf = services::TokenTransferList;

    fn to_protobuf(&self) -> Self::Protobuf {
        let transfers = self.transfers.to_protobuf();
        let nft_transfers = self.nft_transfers.to_protobuf();

        services::TokenTransferList {
            token: Some(self.token_id.to_protobuf()),
            transfers,
            nft_transfers,
            expected_decimals: self.expected_decimals,
        }
    }
}

impl ToProtobuf for TokenNftTransfer {
    type Protobuf = services::NftTransfer;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::NftTransfer {
            sender_account_id: Some(self.sender.to_protobuf()),
            receiver_account_id: Some(self.receiver.to_protobuf()),
            serial_number: self.serial as i64,
            is_approval: self.is_approved,
        }
    }
}

impl ToTransactionDataProtobuf for TransferTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::CryptoTransfer(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for TransferTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::CryptoTransfer(self.to_protobuf())
    }
}

impl From<TransferTransactionData> for AnyTransactionData {
    fn from(transaction: TransferTransactionData) -> Self {
        Self::Transfer(transaction)
    }
}

impl FromProtobuf<services::CryptoTransferTransactionBody> for TransferTransactionData {
    fn from_protobuf(pb: services::CryptoTransferTransactionBody) -> crate::Result<Self> {
        let transfers = pb.transfers.map(|it| it.account_amounts);
        let transfers = Option::from_protobuf(transfers)?.unwrap_or_default();

        Ok(Self { transfers, token_transfers: Vec::from_protobuf(pb.token_transfers)? })
    }
}

impl ToProtobuf for TransferTransactionData {
    type Protobuf = services::CryptoTransferTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        let transfers = self
            .transfers
            .is_empty()
            .not()
            .then(|| services::TransferList { account_amounts: self.transfers.to_protobuf() });

        let token_transfers = self.token_transfers.to_protobuf();

        services::CryptoTransferTransactionBody { transfers, token_transfers }
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
        AccountId,
        AnyTransaction,
        Hbar,
        TokenId,
        TransferTransaction,
    };

    fn make_transaction() -> TransferTransaction {
        let mut tx = TransferTransaction::new_for_tests();

        tx.hbar_transfer(AccountId::new(0, 0, 5008), Hbar::from_tinybars(400))
            .hbar_transfer(AccountId::new(0, 0, 5006), Hbar::from_tinybars(800).negated())
            .approved_hbar_transfer(AccountId::new(0, 0, 5007), Hbar::from_tinybars(400))
            .token_transfer(TokenId::new(0, 0, 5), AccountId::new(0, 0, 5008), 400)
            .token_transfer_with_decimals(
                TokenId::new(0, 0, 5),
                AccountId::new(0, 0, 5006),
                -800,
                3,
            )
            .token_transfer_with_decimals(TokenId::new(0, 0, 5), AccountId::new(0, 0, 5007), 400, 3)
            .token_transfer(TokenId::new(0, 0, 4), AccountId::new(0, 0, 5008), 1)
            .approved_token_transfer(TokenId::new(0, 0, 4), AccountId::new(0, 0, 5006), -1)
            .nft_transfer(
                TokenId::new(0, 0, 3).nft(2),
                AccountId::new(0, 0, 5008),
                AccountId::new(0, 0, 5007),
            )
            .approved_nft_transfer(
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
            CryptoTransfer(
                CryptoTransferTransactionBody {
                    transfers: Some(
                        TransferList {
                            account_amounts: [
                                AccountAmount {
                                    account_id: Some(
                                        AccountId {
                                            shard_num: 0,
                                            realm_num: 0,
                                            account: Some(
                                                AccountNum(
                                                    5008,
                                                ),
                                            ),
                                        },
                                    ),
                                    amount: 400,
                                    is_approval: false,
                                },
                                AccountAmount {
                                    account_id: Some(
                                        AccountId {
                                            shard_num: 0,
                                            realm_num: 0,
                                            account: Some(
                                                AccountNum(
                                                    5006,
                                                ),
                                            ),
                                        },
                                    ),
                                    amount: -800,
                                    is_approval: false,
                                },
                                AccountAmount {
                                    account_id: Some(
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
                                    amount: 400,
                                    is_approval: true,
                                },
                            ],
                        },
                    ),
                    token_transfers: [
                        TokenTransferList {
                            token: Some(
                                TokenId {
                                    shard_num: 0,
                                    realm_num: 0,
                                    token_num: 5,
                                },
                            ),
                            transfers: [
                                AccountAmount {
                                    account_id: Some(
                                        AccountId {
                                            shard_num: 0,
                                            realm_num: 0,
                                            account: Some(
                                                AccountNum(
                                                    5008,
                                                ),
                                            ),
                                        },
                                    ),
                                    amount: 400,
                                    is_approval: false,
                                },
                                AccountAmount {
                                    account_id: Some(
                                        AccountId {
                                            shard_num: 0,
                                            realm_num: 0,
                                            account: Some(
                                                AccountNum(
                                                    5006,
                                                ),
                                            ),
                                        },
                                    ),
                                    amount: -800,
                                    is_approval: false,
                                },
                                AccountAmount {
                                    account_id: Some(
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
                                    amount: 400,
                                    is_approval: false,
                                },
                            ],
                            nft_transfers: [],
                            expected_decimals: Some(
                                3,
                            ),
                        },
                        TokenTransferList {
                            token: Some(
                                TokenId {
                                    shard_num: 0,
                                    realm_num: 0,
                                    token_num: 4,
                                },
                            ),
                            transfers: [
                                AccountAmount {
                                    account_id: Some(
                                        AccountId {
                                            shard_num: 0,
                                            realm_num: 0,
                                            account: Some(
                                                AccountNum(
                                                    5008,
                                                ),
                                            ),
                                        },
                                    ),
                                    amount: 1,
                                    is_approval: false,
                                },
                                AccountAmount {
                                    account_id: Some(
                                        AccountId {
                                            shard_num: 0,
                                            realm_num: 0,
                                            account: Some(
                                                AccountNum(
                                                    5006,
                                                ),
                                            ),
                                        },
                                    ),
                                    amount: -1,
                                    is_approval: true,
                                },
                            ],
                            nft_transfers: [],
                            expected_decimals: None,
                        },
                        TokenTransferList {
                            token: Some(
                                TokenId {
                                    shard_num: 0,
                                    realm_num: 0,
                                    token_num: 3,
                                },
                            ),
                            transfers: [],
                            nft_transfers: [
                                NftTransfer {
                                    sender_account_id: Some(
                                        AccountId {
                                            shard_num: 0,
                                            realm_num: 0,
                                            account: Some(
                                                AccountNum(
                                                    5008,
                                                ),
                                            ),
                                        },
                                    ),
                                    receiver_account_id: Some(
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
                                    serial_number: 2,
                                    is_approval: false,
                                },
                                NftTransfer {
                                    sender_account_id: Some(
                                        AccountId {
                                            shard_num: 0,
                                            realm_num: 0,
                                            account: Some(
                                                AccountNum(
                                                    5008,
                                                ),
                                            ),
                                        },
                                    ),
                                    receiver_account_id: Some(
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
                                    serial_number: 1,
                                    is_approval: true,
                                },
                                NftTransfer {
                                    sender_account_id: Some(
                                        AccountId {
                                            shard_num: 0,
                                            realm_num: 0,
                                            account: Some(
                                                AccountNum(
                                                    5008,
                                                ),
                                            ),
                                        },
                                    ),
                                    receiver_account_id: Some(
                                        AccountId {
                                            shard_num: 0,
                                            realm_num: 0,
                                            account: Some(
                                                AccountNum(
                                                    5006,
                                                ),
                                            ),
                                        },
                                    ),
                                    serial_number: 3,
                                    is_approval: false,
                                },
                                NftTransfer {
                                    sender_account_id: Some(
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
                                    receiver_account_id: Some(
                                        AccountId {
                                            shard_num: 0,
                                            realm_num: 0,
                                            account: Some(
                                                AccountNum(
                                                    5006,
                                                ),
                                            ),
                                        },
                                    ),
                                    serial_number: 4,
                                    is_approval: false,
                                },
                            ],
                            expected_decimals: None,
                        },
                        TokenTransferList {
                            token: Some(
                                TokenId {
                                    shard_num: 0,
                                    realm_num: 0,
                                    token_num: 2,
                                },
                            ),
                            transfers: [],
                            nft_transfers: [
                                NftTransfer {
                                    sender_account_id: Some(
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
                                    receiver_account_id: Some(
                                        AccountId {
                                            shard_num: 0,
                                            realm_num: 0,
                                            account: Some(
                                                AccountNum(
                                                    5006,
                                                ),
                                            ),
                                        },
                                    ),
                                    serial_number: 4,
                                    is_approval: false,
                                },
                            ],
                            expected_decimals: None,
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
    fn get_decimals() {
        let mut tx = TransferTransaction::new();
        const TOKEN: TokenId = TokenId::new(0, 0, 5);

        assert_eq!(tx.get_token_decimals().get(&TOKEN), None);

        tx.token_transfer(TOKEN, AccountId::new(0, 0, 8), 100);
        assert_eq!(tx.get_token_decimals().get(&TOKEN), None);

        tx.token_transfer_with_decimals(TOKEN, AccountId::new(0, 0, 7), -100, 5);
        assert_eq!(tx.get_token_decimals().get(&TOKEN), Some(&5));
    }
}
