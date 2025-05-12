// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::file_service_client::FileServiceClient;
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::query::{
    AnyQueryData,
    Query,
    QueryExecute,
    ToQueryProtobuf,
};
use crate::{
    BoxGrpcFuture,
    Error,
    FileContentsResponse,
    FileId,
    ToProtobuf,
    ValidateChecksums,
};

/// Get the contents of a file.
pub type FileContentsQuery = Query<FileContentsQueryData>;

#[derive(Clone, Default, Debug)]
pub struct FileContentsQueryData {
    /// The file ID for which contents are requested.
    file_id: Option<FileId>,
}

impl From<FileContentsQueryData> for AnyQueryData {
    #[inline]
    fn from(data: FileContentsQueryData) -> Self {
        Self::FileContents(data)
    }
}

impl FileContentsQuery {
    /// Returns the ID of the file for which contents are requested.
    #[must_use]
    pub fn get_file_id(&self) -> Option<FileId> {
        self.data.file_id
    }

    /// Sets the file ID for which contents are requested.
    pub fn file_id(&mut self, id: impl Into<FileId>) -> &mut Self {
        self.data.file_id = Some(id.into());
        self
    }
}

impl ToQueryProtobuf for FileContentsQueryData {
    fn to_query_protobuf(&self, header: services::QueryHeader) -> services::Query {
        services::Query {
            query: Some(services::query::Query::FileGetContents(services::FileGetContentsQuery {
                header: Some(header),
                file_id: self.file_id.to_protobuf(),
            })),
        }
    }
}

impl QueryExecute for FileContentsQueryData {
    type Response = FileContentsResponse;

    fn execute(
        &self,
        channel: Channel,
        request: services::Query,
    ) -> BoxGrpcFuture<'_, services::Response> {
        Box::pin(async { FileServiceClient::new(channel).get_file_content(request).await })
    }
}

impl ValidateChecksums for FileContentsQueryData {
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
        Hbar,
    };

    #[test]
    fn serialize() {
        expect![[r#"
            Query {
                query: Some(
                    FileGetContents(
                        FileGetContentsQuery {
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
            &FileContentsQuery::new()
                .file_id(FileId::new(0, 0, 5005))
                .max_payment_amount(Hbar::from_tinybars(100_000))
                .data
                .to_query_protobuf(Default::default()),
        );
    }

    #[test]
    fn get_set_file_id() {
        let mut query = FileContentsQuery::new();
        query.file_id(FileId::new(0, 0, 5005));

        assert_eq!(query.get_file_id(), Some(FileId::new(0, 0, 5005)));
    }
}
