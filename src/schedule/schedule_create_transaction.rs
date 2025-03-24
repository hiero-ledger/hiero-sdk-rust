// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::schedule_service_client::ScheduleServiceClient;
use time::OffsetDateTime;
use tonic::transport::Channel;

use super::schedulable_transaction_body::SchedulableTransactionBody;
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
    Key,
    Transaction,
    ValidateChecksums,
};

/// Create a new schedule entity (or simply, schedule) in the network's action queue.
///
/// Upon `SUCCESS`, the receipt contains the `ScheduleId` of the created schedule. A schedule
/// entity includes a `scheduled_transaction_body` to be executed.
///
/// When the schedule has collected enough signing keys to satisfy the schedule's signing
/// requirements, the schedule can be executed.
///
pub type ScheduleCreateTransaction = Transaction<ScheduleCreateTransactionData>;

#[derive(Default, Debug, Clone)]
pub struct ScheduleCreateTransactionData {
    scheduled_transaction: Option<SchedulableTransactionBody>,

    schedule_memo: Option<String>,

    admin_key: Option<Key>,

    payer_account_id: Option<AccountId>,

    expiration_time: Option<OffsetDateTime>,

    wait_for_expiry: bool,
}

impl ScheduleCreateTransaction {
    // note(sr): not sure what the right way to go about this is?
    // pub fn get_scheduled_transaction(&self) -> Option<&SchedulableTransactionBody> {
    //     self.data().scheduled_transaction.as_ref()
    // }

    /// Sets the scheduled transaction.
    ///
    /// # Panics
    /// panics if the transaction is not schedulable, a transaction can be non-schedulable due to:
    /// - being a transaction kind that's non-schedulable, IE, `EthereumTransaction`, or
    /// - being a chunked transaction with multiple chunks.
    pub fn scheduled_transaction<D>(&mut self, transaction: Transaction<D>) -> &mut Self
    where
        D: TransactionExecute,
    {
        let body = transaction.into_body();

        // this gets infered right but `foo.into().try_into()` looks really really weird.
        let data: AnyTransactionData = body.data.into();

        self.data_mut().scheduled_transaction = Some(SchedulableTransactionBody {
            max_transaction_fee: body.max_transaction_fee,
            transaction_memo: body.transaction_memo,
            data: Box::new(data.try_into().unwrap()),
        });

        self
    }

    /// Returns the timestamp for when the transaction should be evaluated for execution and then expire.
    #[must_use]
    pub fn get_expiration_time(&self) -> Option<OffsetDateTime> {
        self.data().expiration_time
    }

    /// Sets the timestamp for when the transaction should be evaluated for execution and then expire.
    pub fn expiration_time(&mut self, time: OffsetDateTime) -> &mut Self {
        self.data_mut().expiration_time = Some(time);
        self
    }

    /// Returns `true` if the transaction will be evaluated at `expiration_time` instead
    /// of when all the required signatures are received, `false` otherwise.
    #[must_use]
    pub fn get_wait_for_expiry(&self) -> bool {
        self.data().wait_for_expiry
    }

    /// Sets if the transaction will be evaluated for execution at `expiration_time` instead
    /// of when all required signatures are received.
    pub fn wait_for_expiry(&mut self, wait: bool) -> &mut Self {
        self.data_mut().wait_for_expiry = wait;
        self
    }

    /// Returns the id of the account to be charged the service fee for the scheduled transaction at
    /// the consensus time it executes (if ever).
    #[must_use]
    pub fn get_payer_account_id(&self) -> Option<AccountId> {
        self.data().payer_account_id
    }

    /// Sets the id of the account to be charged the service fee for the scheduled transaction at
    /// the consensus time that it executes (if ever).
    pub fn payer_account_id(&mut self, id: AccountId) -> &mut Self {
        self.data_mut().payer_account_id = Some(id);
        self
    }

    /// Returns the memo for the schedule entity.
    #[must_use]
    pub fn get_schedule_memo(&self) -> Option<&str> {
        self.data().schedule_memo.as_deref()
    }

    /// Sets the memo for the schedule entity.
    pub fn schedule_memo(&mut self, memo: impl Into<String>) -> &mut Self {
        self.data_mut().schedule_memo = Some(memo.into());
        self
    }

    /// Returns the Hedera key which can be used to sign a `ScheduleDelete` and remove the schedule.
    #[must_use]
    pub fn get_admin_key(&self) -> Option<&Key> {
        self.data().admin_key.as_ref()
    }

    /// Sets the Hedera key which can be used to sign a `ScheduleDelete` and remove the schedule.
    pub fn admin_key(&mut self, key: impl Into<Key>) -> &mut Self {
        self.data_mut().admin_key = Some(key.into());
        self
    }
}

impl TransactionData for ScheduleCreateTransactionData {}

impl TransactionExecute for ScheduleCreateTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { ScheduleServiceClient::new(channel).create_schedule(request).await })
    }
}

impl ValidateChecksums for ScheduleCreateTransactionData {
    fn validate_checksums(&self, ledger_id: &crate::ledger_id::RefLedgerId) -> Result<(), Error> {
        self.payer_account_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for ScheduleCreateTransactionData {
    // not really anything I can do about this
    #[allow(clippy::too_many_lines)]
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        let body = self.scheduled_transaction.as_ref().map(|scheduled| {
            let data = scheduled.data.to_schedulable_transaction_data_protobuf();

            services::SchedulableTransactionBody {
                data: Some(data),
                memo: scheduled.transaction_memo.clone(),
                // FIXME: does not use the client to default the max transaction fee
                transaction_fee: scheduled
                    .max_transaction_fee
                    .unwrap_or_else(|| scheduled.data.default_max_transaction_fee())
                    .to_tinybars() as u64,
            }
        });

        let payer_account_id = self.payer_account_id.to_protobuf();
        let admin_key = self.admin_key.to_protobuf();
        let expiration_time = self.expiration_time.map(Into::into);

        services::transaction_body::Data::ScheduleCreate(services::ScheduleCreateTransactionBody {
            scheduled_transaction_body: body,
            memo: self.schedule_memo.clone().unwrap_or_default(),
            admin_key,
            payer_account_id,
            expiration_time,
            wait_for_expiry: self.wait_for_expiry,
        })
    }
}

impl From<ScheduleCreateTransactionData> for AnyTransactionData {
    fn from(transaction: ScheduleCreateTransactionData) -> Self {
        Self::ScheduleCreate(transaction)
    }
}

impl FromProtobuf<services::ScheduleCreateTransactionBody> for ScheduleCreateTransactionData {
    fn from_protobuf(pb: services::ScheduleCreateTransactionBody) -> crate::Result<Self> {
        Ok(Self {
            scheduled_transaction: Option::from_protobuf(pb.scheduled_transaction_body)?,
            schedule_memo: Some(pb.memo),
            admin_key: Option::from_protobuf(pb.admin_key)?,
            payer_account_id: Option::from_protobuf(pb.payer_account_id)?,
            expiration_time: pb.expiration_time.map(Into::into),
            wait_for_expiry: pb.wait_for_expiry,
        })
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hedera_proto::services;
    use time::OffsetDateTime;

    use super::ScheduleCreateTransactionData;
    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
        unused_private_key,
        VALID_START,
    };
    use crate::transaction::ToSchedulableTransactionDataProtobuf;
    use crate::{
        AccountId,
        AnyTransaction,
        Hbar,
        PublicKey,
        ScheduleCreateTransaction,
        TransferTransaction,
    };

    fn scheduled_transaction() -> TransferTransaction {
        let mut tx = TransferTransaction::new();
        tx.hbar_transfer("0.0.555".parse().unwrap(), -Hbar::new(10))
            .hbar_transfer("0.0.666".parse().unwrap(), Hbar::new(10));
        tx
    }

    fn admin_key() -> PublicKey {
        unused_private_key().public_key()
    }

    const PAYER_ACCOUNT_ID: AccountId = AccountId::new(0, 0, 222);
    const SCHEDULE_MEMO: &str = "hi";
    const EXPIRATION_TIME: OffsetDateTime = VALID_START;

    fn make_transaction() -> ScheduleCreateTransaction {
        let mut tx = ScheduleCreateTransaction::new_for_tests();

        tx.scheduled_transaction(scheduled_transaction())
            .admin_key(admin_key())
            .payer_account_id(PAYER_ACCOUNT_ID)
            .schedule_memo(SCHEDULE_MEMO)
            .expiration_time(EXPIRATION_TIME)
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
            ScheduleCreate(
                ScheduleCreateTransactionBody {
                    scheduled_transaction_body: Some(
                        SchedulableTransactionBody {
                            transaction_fee: 200000000,
                            memo: "",
                            data: Some(
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
                                                                        555,
                                                                    ),
                                                                ),
                                                            },
                                                        ),
                                                        amount: -1000000000,
                                                        is_approval: false,
                                                    },
                                                    AccountAmount {
                                                        account_id: Some(
                                                            AccountId {
                                                                shard_num: 0,
                                                                realm_num: 0,
                                                                account: Some(
                                                                    AccountNum(
                                                                        666,
                                                                    ),
                                                                ),
                                                            },
                                                        ),
                                                        amount: 1000000000,
                                                        is_approval: false,
                                                    },
                                                ],
                                            },
                                        ),
                                        token_transfers: [],
                                    },
                                ),
                            ),
                        },
                    ),
                    memo: "hi",
                    admin_key: Some(
                        Key {
                            key: Some(
                                Ed25519(
                                    [
                                        224,
                                        200,
                                        236,
                                        39,
                                        88,
                                        165,
                                        135,
                                        159,
                                        250,
                                        194,
                                        38,
                                        161,
                                        60,
                                        12,
                                        81,
                                        107,
                                        121,
                                        158,
                                        114,
                                        227,
                                        81,
                                        65,
                                        160,
                                        221,
                                        130,
                                        143,
                                        148,
                                        211,
                                        121,
                                        136,
                                        164,
                                        183,
                                    ],
                                ),
                            ),
                        },
                    ),
                    payer_account_id: Some(
                        AccountId {
                            shard_num: 0,
                            realm_num: 0,
                            account: Some(
                                AccountNum(
                                    222,
                                ),
                            ),
                        },
                    ),
                    expiration_time: Some(
                        Timestamp {
                            seconds: 1554158542,
                            nanos: 0,
                        },
                    ),
                    wait_for_expiry: false,
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
        let tx = services::ScheduleCreateTransactionBody {
            scheduled_transaction_body: Some(services::SchedulableTransactionBody {
                transaction_fee: Hbar::new(2).to_tinybars() as _,
                memo: String::new(),
                data: Some(
                    scheduled_transaction().data().to_schedulable_transaction_data_protobuf(),
                ),
            }),
            memo: SCHEDULE_MEMO.to_owned(),
            admin_key: Some(admin_key().to_protobuf()),
            payer_account_id: Some(PAYER_ACCOUNT_ID.to_protobuf()),
            expiration_time: Some(EXPIRATION_TIME.to_protobuf()),
            wait_for_expiry: false,
        };

        let tx = ScheduleCreateTransactionData::from_protobuf(tx).unwrap();

        expect![[r#"
            SchedulableTransactionBody {
                data: Transfer(
                    TransferTransactionData {
                        transfers: [
                            Transfer {
                                account_id: "0.0.555",
                                amount: -1000000000,
                                is_approval: false,
                            },
                            Transfer {
                                account_id: "0.0.666",
                                amount: 1000000000,
                                is_approval: false,
                            },
                        ],
                        token_transfers: [],
                    },
                ),
                max_transaction_fee: Some(
                    "2 ℏ",
                ),
                transaction_memo: "",
            }
        "#]]
        .assert_debug_eq(&tx.scheduled_transaction.unwrap());

        assert_eq!(tx.schedule_memo.as_deref(), Some(SCHEDULE_MEMO));
        assert_eq!(tx.admin_key, Some(admin_key().into()));
        assert_eq!(tx.payer_account_id, Some(PAYER_ACCOUNT_ID));
        assert_eq!(tx.expiration_time, Some(EXPIRATION_TIME));
        assert_eq!(tx.wait_for_expiry, false);
    }

    mod get_set {
        use super::*;
        #[test]
        fn admin_key() {
            let mut tx = ScheduleCreateTransaction::new();
            tx.admin_key(super::admin_key());

            assert_eq!(tx.get_admin_key(), Some(&super::admin_key().into()));
        }

        #[test]
        #[should_panic]
        fn admin_key_frozen_panics() {
            make_transaction().admin_key(super::admin_key());
        }

        #[test]
        fn payer_account_id() {
            let mut tx = ScheduleCreateTransaction::new();
            tx.payer_account_id(PAYER_ACCOUNT_ID);

            assert_eq!(tx.get_payer_account_id(), Some(PAYER_ACCOUNT_ID));
        }

        #[test]
        #[should_panic]
        fn payer_account_id_frozen_panics() {
            make_transaction().payer_account_id(PAYER_ACCOUNT_ID);
        }

        #[test]
        fn expiration_time() {
            let mut tx = ScheduleCreateTransaction::new();
            tx.expiration_time(EXPIRATION_TIME);

            assert_eq!(tx.get_expiration_time(), Some(EXPIRATION_TIME));
        }

        #[test]
        #[should_panic]
        fn expiration_time_frozen_panics() {
            make_transaction().expiration_time(EXPIRATION_TIME);
        }

        #[test]
        fn wait_for_expiry() {
            let mut tx = ScheduleCreateTransaction::new();
            tx.wait_for_expiry(true);

            assert_eq!(tx.get_wait_for_expiry(), true);
        }

        #[test]
        #[should_panic]
        fn wait_for_expiry_frozen_panics() {
            make_transaction().wait_for_expiry(true);
        }
    }
}
