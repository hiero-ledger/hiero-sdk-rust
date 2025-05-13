// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::file_service_client::FileServiceClient;
use time::{
    Duration,
    OffsetDateTime,
};
use tonic::transport::Channel;

use crate::entity_id::ValidateChecksums;
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
    Key,
    KeyList,
    Transaction,
};

/// Create a new file, containing the given contents.
pub type FileCreateTransaction = Transaction<FileCreateTransactionData>;

#[derive(Debug, Clone)]
pub struct FileCreateTransactionData {
    /// The memo associated with the file.
    file_memo: String,

    /// All keys at the top level of a key list must sign to create or
    /// modify the file. Any one of the keys at the top level key list
    /// can sign to delete the file.
    keys: Option<KeyList>,

    /// The bytes that are to be the contents of the file.
    contents: Option<Vec<u8>>,

    auto_renew_period: Option<Duration>,

    auto_renew_account_id: Option<AccountId>,

    /// The time at which this file should expire.
    expiration_time: Option<OffsetDateTime>,
}

impl Default for FileCreateTransactionData {
    fn default() -> Self {
        Self {
            file_memo: String::new(),
            keys: None,
            contents: None,
            auto_renew_period: None,
            auto_renew_account_id: None,
            expiration_time: Some(OffsetDateTime::now_utc() + Duration::days(90)),
        }
    }
}

impl FileCreateTransaction {
    /// Returns the memo to be associated with the file.
    #[must_use]
    pub fn get_file_memo(&self) -> &str {
        &self.data().file_memo
    }

    /// Sets the memo associated with the file.
    pub fn file_memo(&mut self, memo: impl Into<String>) -> &mut Self {
        self.data_mut().file_memo = memo.into();
        self
    }

    /// Returns the bytes that are to be the contents of the file.
    #[must_use]
    pub fn get_contents(&self) -> Option<&[u8]> {
        self.data().contents.as_deref()
    }

    /// Sets the bytes that are to be the contents of the file.
    pub fn contents(&mut self, contents: impl Into<Vec<u8>>) -> &mut Self {
        self.data_mut().contents = Some(contents.into());
        self
    }

    /// Returns the keys for this file.
    #[must_use]
    pub fn get_keys(&self) -> Option<&KeyList> {
        self.data().keys.as_ref()
    }

    /// Sets the keys for this file.
    ///
    /// All keys at the top level of a key list must sign to create or
    /// modify the file. Any one of the keys at the top level key list
    /// can sign to delete the file.
    pub fn keys<K: Into<Key>>(&mut self, keys: impl IntoIterator<Item = K>) -> &mut Self {
        self.data_mut().keys = Some(keys.into_iter().map(Into::into).collect());
        self
    }

    /// Returns the auto renew period for this file.
    ///
    /// # Network Support
    /// Please note that this not supported on any hedera network at this time.
    #[must_use]
    pub fn get_auto_renew_period(&self) -> Option<Duration> {
        self.data().auto_renew_period
    }

    /// Sets the auto renew period for this file.
    ///
    /// # Network Support
    /// Please note that this not supported on any hedera network at this time.
    pub fn auto_renew_period(&mut self, duration: Duration) -> &mut Self {
        self.data_mut().auto_renew_period = Some(duration);
        self
    }

    /// Returns the account to be used at the file's expiration time to extend the
    /// life of the file.
    ///
    /// # Network Support
    /// Please note that this not supported on any hedera network at this time.
    #[must_use]
    pub fn get_auto_renew_account_id(&self) -> Option<AccountId> {
        self.data().auto_renew_account_id
    }

    /// Sets the account to be used at the files's expiration time to extend the
    /// life of the file.
    ///
    /// # Network Support
    /// Please note that this not supported on any hedera network at this time.
    pub fn auto_renew_account_id(&mut self, id: AccountId) -> &mut Self {
        self.data_mut().auto_renew_account_id = Some(id);
        self
    }

    /// Returns the time at which this file should expire.
    #[must_use]
    pub fn get_expiration_time(&self) -> Option<OffsetDateTime> {
        self.data().expiration_time
    }

    /// Sets the time at which this file should expire.
    pub fn expiration_time(&mut self, at: OffsetDateTime) -> &mut Self {
        self.require_not_frozen();
        self.data_mut().expiration_time = Some(at);
        self
    }
}

impl TransactionData for FileCreateTransactionData {
    fn default_max_transaction_fee(&self) -> crate::Hbar {
        crate::Hbar::new(5)
    }
}

impl TransactionExecute for FileCreateTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { FileServiceClient::new(channel).create_file(request).await })
    }
}

impl ValidateChecksums for FileCreateTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> crate::Result<()> {
        self.auto_renew_account_id.validate_checksums(ledger_id)?;

        Ok(())
    }
}

impl ToTransactionDataProtobuf for FileCreateTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::FileCreate(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for FileCreateTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::FileCreate(self.to_protobuf())
    }
}

impl From<FileCreateTransactionData> for AnyTransactionData {
    fn from(transaction: FileCreateTransactionData) -> Self {
        Self::FileCreate(transaction)
    }
}

impl FromProtobuf<services::FileCreateTransactionBody> for FileCreateTransactionData {
    fn from_protobuf(pb: services::FileCreateTransactionBody) -> crate::Result<Self> {
        Ok(Self {
            file_memo: pb.memo,
            keys: Option::from_protobuf(pb.keys)?,
            contents: Some(pb.contents),
            auto_renew_period: None,
            auto_renew_account_id: None,
            expiration_time: pb.expiration_time.map(Into::into),
        })
    }
}

impl ToProtobuf for FileCreateTransactionData {
    type Protobuf = services::FileCreateTransactionBody;

    #[allow(deprecated)]
    fn to_protobuf(&self) -> Self::Protobuf {
        services::FileCreateTransactionBody {
            expiration_time: self.expiration_time.to_protobuf(),
            keys: self.keys.to_protobuf(),
            contents: self.contents.clone().unwrap_or_default(),
            shard_id: None,
            realm_id: None,
            new_realm_admin_key: None,
            memo: self.file_memo.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hedera_proto::services;
    use hex_literal::hex;
    use time::OffsetDateTime;

    use crate::file::FileCreateTransactionData;
    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
        unused_private_key,
    };
    use crate::{
        AnyTransaction,
        FileCreateTransaction,
        Key,
        KeyList,
    };

    const CONTENTS: [u8; 4] = hex!("deadbeef");

    const EXPIRATION_TIME: OffsetDateTime = match OffsetDateTime::from_unix_timestamp(1554158728) {
        Ok(it) => it,
        Err(_) => panic!("Panic in `const` unwrap"),
    };

    fn keys() -> impl IntoIterator<Item = Key> {
        [unused_private_key().public_key().into()]
    }

    const FILE_MEMO: &str = "Hello memo";

    fn make_transaction() -> FileCreateTransaction {
        let mut tx = FileCreateTransaction::new_for_tests();

        tx.contents(CONTENTS)
            .expiration_time(EXPIRATION_TIME)
            .keys(keys())
            .file_memo(FILE_MEMO)
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
            FileCreate(
                FileCreateTransactionBody {
                    expiration_time: Some(
                        Timestamp {
                            seconds: 1554158728,
                            nanos: 0,
                        },
                    ),
                    keys: Some(
                        KeyList {
                            keys: [
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
                            ],
                        },
                    ),
                    contents: [
                        222,
                        173,
                        190,
                        239,
                    ],
                    shard_id: None,
                    realm_id: None,
                    new_realm_admin_key: None,
                    memo: "Hello memo",
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
        #[allow(deprecated)]
        let tx = services::FileCreateTransactionBody {
            expiration_time: Some(EXPIRATION_TIME.to_protobuf()),
            keys: Some(KeyList::from_iter(keys()).to_protobuf()),
            contents: CONTENTS.into(),
            memo: FILE_MEMO.to_owned(),
            shard_id: None,
            realm_id: None,
            new_realm_admin_key: None,
        };

        let tx = FileCreateTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(tx.contents.as_deref(), Some(CONTENTS.as_slice()));
        assert_eq!(tx.expiration_time, Some(EXPIRATION_TIME));
        assert_eq!(tx.keys, Some(KeyList::from_iter(keys())));
        assert_eq!(tx.file_memo, FILE_MEMO);
    }

    mod get_set {
        use super::*;

        #[test]
        fn contents() {
            let mut tx = FileCreateTransaction::new();
            tx.contents(CONTENTS);

            assert_eq!(tx.get_contents(), Some(CONTENTS.as_slice()));
        }

        #[test]
        #[should_panic]
        fn contents_frozen_panics() {
            make_transaction().contents(CONTENTS);
        }

        #[test]
        fn expiration_time() {
            let mut tx = FileCreateTransaction::new();
            tx.expiration_time(EXPIRATION_TIME);

            assert_eq!(tx.get_expiration_time(), Some(EXPIRATION_TIME));
        }

        #[test]
        #[should_panic]
        fn expiration_time_frozen_panics() {
            make_transaction().expiration_time(EXPIRATION_TIME);
        }

        #[test]
        fn keys() {
            let mut tx = FileCreateTransaction::new();
            tx.keys(super::keys());

            assert_eq!(tx.get_keys(), Some(&KeyList::from_iter(super::keys())));
        }

        #[test]
        #[should_panic]
        fn keys_frozen_panics() {
            make_transaction().keys(super::keys());
        }

        #[test]
        fn file_memo() {
            let mut tx = FileCreateTransaction::new();
            tx.file_memo(FILE_MEMO);

            assert_eq!(tx.get_file_memo(), FILE_MEMO);
        }

        #[test]
        #[should_panic]
        fn file_memo_frozen_panics() {
            make_transaction().file_memo(FILE_MEMO);
        }
    }
}
