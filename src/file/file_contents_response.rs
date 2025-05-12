// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;

use crate::{
    FileId,
    FromProtobuf,
};

/// Response from [`FileContentsQuery`][crate::FileContentsQuery].
#[derive(Debug, Clone)]
pub struct FileContentsResponse {
    /// The file ID of the file whose contents are being returned.
    pub file_id: FileId,

    // TODO: .contents vs .bytes (?)
    /// The bytes contained in the file.
    pub contents: Vec<u8>,
}

impl FromProtobuf<services::response::Response> for FileContentsResponse {
    fn from_protobuf(pb: services::response::Response) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let pb = pb_getv!(pb, FileGetContents, services::response::Response);
        let file_contents = pb_getf!(pb, file_contents)?;
        let file_id = pb_getf!(file_contents, file_id)?;

        let contents = file_contents.contents;
        let file_id = FileId::from_protobuf(file_id)?;

        Ok(Self { file_id, contents })
    }
}
