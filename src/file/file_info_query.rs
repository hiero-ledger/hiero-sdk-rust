// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::file_service_client::FileServiceClient;
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::query::{
    AnyQueryData,
    QueryExecute,
    ToQueryProtobuf,
};
use crate::{
    BoxGrpcFuture,
    Error,
    FileId,
    FileInfo,
    Query,
    ToProtobuf,
    ValidateChecksums,
};

/// Get all the information about a file.
pub type FileInfoQuery = Query<FileInfoQueryData>;

#[derive(Default, Clone, Debug)]
pub struct FileInfoQueryData {
    file_id: Option<FileId>,
}

impl From<FileInfoQueryData> for AnyQueryData {
    #[inline]
    fn from(data: FileInfoQueryData) -> Self {
        Self::FileInfo(data)
    }
}

impl FileInfoQuery {
    /// Returns the ID of the file for which information is requested.
    #[must_use]
    pub fn get_file_id(&self) -> Option<FileId> {
        self.data.file_id
    }

    /// Sets the ID of the file for which information is requested.
    pub fn file_id(&mut self, id: impl Into<FileId>) -> &mut Self {
        self.data.file_id = Some(id.into());
        self
    }
}

impl ToQueryProtobuf for FileInfoQueryData {
    fn to_query_protobuf(&self, header: services::QueryHeader) -> services::Query {
        let file_id = self.file_id.to_protobuf();

        services::Query {
            query: Some(services::query::Query::FileGetInfo(services::FileGetInfoQuery {
                file_id,
                header: Some(header),
            })),
        }
    }
}

impl QueryExecute for FileInfoQueryData {
    type Response = FileInfo;

    fn execute(
        &self,
        channel: Channel,
        request: services::Query,
    ) -> BoxGrpcFuture<'_, services::Response> {
        Box::pin(async { FileServiceClient::new(channel).get_file_info(request).await })
    }
}

impl ValidateChecksums for FileInfoQueryData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.file_id.validate_checksums(ledger_id)
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::query::ToQueryProtobuf;
    use crate::{
        FileContentsQuery,
        FileId,
        FileInfoQuery,
        Hbar,
    };

    #[test]
    fn serialize() {
        expect![[r#"
            Query {
                query: Some(
                    FileGetInfo(
                        FileGetInfoQuery {
                            header: Some(
                                QueryHeader {
                                    payment: None,
                                    response_type: AnswerOnly,
                                },
                            ),
                            file_id: Some(
                                FileId {
                                    shard_num: 0,
                                    realm_num: 0,
                                    file_num: 5005,
                                },
                            ),
                        },
                    ),
                ),
            }
        "#]]
        .assert_debug_eq(
            &FileInfoQuery::new()
                .file_id(FileId::new(0, 0, 5005))
                .max_payment_amount(Hbar::from_tinybars(100_000))
                .data
                .to_query_protobuf(Default::default()),
        )
    }

    #[test]
    fn get_set_file_id() {
        let mut query = FileContentsQuery::new();
        query.file_id(FileId::new(0, 0, 5005));

        assert_eq!(query.get_file_id(), Some(FileId::new(0, 0, 5005)));
    }
}
