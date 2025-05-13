// SPDX-License-Identifier: Apache-2.0

use std::cmp;
use std::num::NonZeroUsize;

use hedera_proto::services;
use hedera_proto::services::file_service_client::FileServiceClient;
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};
use crate::transaction::{
    AnyTransactionData,
    ChunkData,
    ChunkInfo,
    ChunkedTransactionData,
    ToSchedulableTransactionDataProtobuf,
    ToTransactionDataProtobuf,
    TransactionData,
    TransactionExecute,
    TransactionExecuteChunked,
};
use crate::{
    BoxGrpcFuture,
    Error,
    FileId,
    Transaction,
    ValidateChecksums,
};

/// Append the given contents to the end of the specified file.
///
pub type FileAppendTransaction = Transaction<FileAppendTransactionData>;

#[derive(Debug, Clone)]
pub struct FileAppendTransactionData {
    /// The file to which the bytes will be appended.
    file_id: Option<FileId>,

    chunk_data: ChunkData,
}

impl Default for FileAppendTransactionData {
    fn default() -> Self {
        Self {
            file_id: None,
            chunk_data: ChunkData {
                chunk_size: NonZeroUsize::new(4096).unwrap(),
                ..Default::default()
            },
        }
    }
}
impl FileAppendTransaction {
    /// Returns the file to which the bytes will be appended.
    #[must_use]
    pub fn get_file_id(&self) -> Option<FileId> {
        self.data().file_id
    }

    /// Sets the file to which the bytes will be appended.
    pub fn file_id(&mut self, id: impl Into<FileId>) -> &mut Self {
        self.data_mut().file_id = Some(id.into());
        self
    }

    /// Retuns the bytes that will be appended to the end of the specified file.
    pub fn get_contents(&self) -> Option<&[u8]> {
        Some(self.data().chunk_data.data.as_slice())
    }

    /// Sets the bytes that will be appended to the end of the specified file.
    pub fn contents(&mut self, contents: impl Into<Vec<u8>>) -> &mut Self {
        self.data_mut().chunk_data.data = contents.into();
        self
    }
}

impl TransactionData for FileAppendTransactionData {
    fn default_max_transaction_fee(&self) -> crate::Hbar {
        crate::Hbar::new(5)
    }

    fn maybe_chunk_data(&self) -> Option<&ChunkData> {
        Some(self.chunk_data())
    }

    fn wait_for_receipt(&self) -> bool {
        true
    }
}

impl ChunkedTransactionData for FileAppendTransactionData {
    fn chunk_data(&self) -> &ChunkData {
        &self.chunk_data
    }

    fn chunk_data_mut(&mut self) -> &mut ChunkData {
        &mut self.chunk_data
    }
}

impl TransactionExecute for FileAppendTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { FileServiceClient::new(channel).append_content(request).await })
    }
}

impl TransactionExecuteChunked for FileAppendTransactionData {}

impl ValidateChecksums for FileAppendTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.file_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for FileAppendTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        services::transaction_body::Data::FileAppend(services::FileAppendTransactionBody {
            file_id: self.file_id.to_protobuf(),
            contents: self.chunk_data.message_chunk(chunk_info).to_vec(),
        })
    }
}

impl ToSchedulableTransactionDataProtobuf for FileAppendTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        assert!(self.chunk_data.used_chunks() == 1);

        services::schedulable_transaction_body::Data::FileAppend(
            services::FileAppendTransactionBody {
                file_id: self.file_id.to_protobuf(),
                contents: self.chunk_data.data.clone(),
            },
        )
    }
}

impl From<FileAppendTransactionData> for AnyTransactionData {
    fn from(transaction: FileAppendTransactionData) -> Self {
        Self::FileAppend(transaction)
    }
}

impl FromProtobuf<services::FileAppendTransactionBody> for FileAppendTransactionData {
    fn from_protobuf(pb: services::FileAppendTransactionBody) -> crate::Result<Self> {
        Self::from_protobuf(Vec::from([pb]))
    }
}

impl FromProtobuf<Vec<services::FileAppendTransactionBody>> for FileAppendTransactionData {
    fn from_protobuf(pb: Vec<services::FileAppendTransactionBody>) -> crate::Result<Self> {
        let total_chunks = pb.len();

        let mut iter = pb.into_iter();
        let pb_first = iter.next().expect("Empty transaction (should've been handled earlier)");

        let file_id = Option::from_protobuf(pb_first.file_id)?;

        let mut largest_chunk_size = pb_first.contents.len();
        let mut contents = pb_first.contents;

        // note: no other SDK checks for correctness here... so let's not do it here either?

        for item in iter {
            largest_chunk_size = cmp::max(largest_chunk_size, item.contents.len());
            contents.extend_from_slice(&item.contents);
        }

        Ok(Self {
            file_id,
            chunk_data: ChunkData {
                max_chunks: total_chunks,
                chunk_size: NonZeroUsize::new(largest_chunk_size)
                    .unwrap_or_else(|| NonZeroUsize::new(1).unwrap()),
                data: contents,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::transaction::test_helpers::{
        check_body,
        transaction_bodies,
    };
    use crate::{
        AnyTransaction,
        FileAppendTransaction,
        FileId,
    };

    const FILE_ID: FileId = FileId::new(0, 0, 10);

    const CONTENTS: &[u8] = br#"{"foo": 231}"#;

    fn make_transaction() -> FileAppendTransaction {
        let mut tx = FileAppendTransaction::new_for_tests();
        tx.file_id(FILE_ID).contents(CONTENTS).freeze().unwrap();

        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        // unlike most transactions we *do* need to do this like in case it's chunked.
        // granted, trying to do anything with a chunked transaction without a Client is hard.
        let txes = transaction_bodies(tx);

        // this is kinda a mess... But it works.
        let txes: Vec<_> = txes.into_iter().map(check_body).collect();

        expect![[r#"
            [
                FileAppend(
                    FileAppendTransactionBody {
                        file_id: Some(
                            FileId {
                                shard_num: 0,
                                realm_num: 0,
                                file_num: 10,
                            },
                        ),
                        contents: [
                            123,
                            34,
                            102,
                            111,
                            111,
                            34,
                            58,
                            32,
                            50,
                            51,
                            49,
                            125,
                        ],
                    },
                ),
                FileAppend(
                    FileAppendTransactionBody {
                        file_id: Some(
                            FileId {
                                shard_num: 0,
                                realm_num: 0,
                                file_num: 10,
                            },
                        ),
                        contents: [
                            123,
                            34,
                            102,
                            111,
                            111,
                            34,
                            58,
                            32,
                            50,
                            51,
                            49,
                            125,
                        ],
                    },
                ),
            ]
        "#]]
        .assert_debug_eq(&txes);
    }

    #[test]
    fn to_from_bytes() {
        let tx = make_transaction();

        let tx2 = AnyTransaction::from_bytes(&tx.to_bytes().unwrap()).unwrap();

        let tx = transaction_bodies(tx);
        let tx2 = transaction_bodies(tx2);

        assert_eq!(tx, tx2);
    }

    #[test]
    fn get_set_file_id() {
        let mut tx = FileAppendTransaction::new();
        tx.file_id(FILE_ID);

        assert_eq!(tx.get_file_id(), Some(FILE_ID));
    }

    #[test]
    fn get_set_contents() {
        let mut tx = FileAppendTransaction::new();
        tx.contents(CONTENTS);

        assert_eq!(tx.get_contents(), Some(CONTENTS));
    }

    #[test]
    #[should_panic]
    fn get_set_file_id_frozen_panics() {
        let mut tx = make_transaction();
        tx.file_id(FILE_ID);
    }

    #[test]
    #[should_panic]
    fn get_set_contents_frozen_panics() {
        let mut tx = make_transaction();
        tx.contents(CONTENTS);
    }
}
