// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use time::{
    Duration,
    OffsetDateTime,
};

use crate::protobuf::ToProtobuf;
use crate::{
    AccountId,
    FileId,
    FromProtobuf,
    KeyList,
    LedgerId,
};

/// Response from [`FileInfoQuery`][crate::FileInfoQuery].
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// The file ID of the file for which information is requested.
    pub file_id: FileId,

    /// Number of bytes in contents.
    pub size: u64,

    /// Current time which this account is set to expire.
    pub expiration_time: Option<OffsetDateTime>,

    /// The auto renew period for this file.
    ///
    /// # Network Support
    /// Please note that this not supported on any hedera network at this time.
    pub auto_renew_period: Option<Duration>,

    /// The account to be used at this file's expiration time to extend the
    /// life of the file.
    ///
    /// # Network Support
    /// Please note that this not supported on any hedera network at this time.
    pub auto_renew_account_id: Option<AccountId>,

    /// True if deleted but not yet expired.
    pub is_deleted: bool,

    /// One of these keys must sign in order to modify or delete the file.
    pub keys: KeyList,

    /// Memo associated with the file.
    pub file_memo: String,

    /// Ledger ID for the network the response was returned from.
    pub ledger_id: LedgerId,
}

impl FileInfo {
    /// Create a new `FileInfo` from protobuf-encoded `bytes`.
    ///
    /// # Errors
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the bytes fails to produce a valid protobuf.
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the protobuf fails.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        FromProtobuf::<services::file_get_info_response::FileInfo>::from_bytes(bytes)
    }

    /// Convert `self` to a protobuf-encoded [`Vec<u8>`].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        ToProtobuf::to_bytes(self)
    }
}

impl FromProtobuf<services::response::Response> for FileInfo {
    #[allow(deprecated)]
    fn from_protobuf(pb: services::response::Response) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let response = pb_getv!(pb, FileGetInfo, services::response::Response);
        let info = pb_getf!(response, file_info)?;
        Self::from_protobuf(info)
    }
}

impl FromProtobuf<services::file_get_info_response::FileInfo> for FileInfo {
    #[allow(deprecated)]
    fn from_protobuf(pb: services::file_get_info_response::FileInfo) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let file_id = pb_getf!(pb, file_id)?;
        let ledger_id = LedgerId::from_bytes(pb.ledger_id);

        Ok(Self {
            file_id: FileId::from_protobuf(file_id)?,
            size: pb.size as u64,
            expiration_time: pb.expiration_time.map(Into::into),
            auto_renew_account_id: None,
            auto_renew_period: None,
            is_deleted: pb.deleted,
            file_memo: pb.memo,
            ledger_id,
            keys: KeyList::from_protobuf(pb.keys.unwrap_or_default())?,
        })
    }
}

impl ToProtobuf for FileInfo {
    type Protobuf = services::file_get_info_response::FileInfo;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::file_get_info_response::FileInfo {
            file_id: Some(self.file_id.to_protobuf()),
            size: self.size as i64,
            expiration_time: self.expiration_time.to_protobuf(),
            deleted: self.is_deleted,
            memo: self.file_memo.clone(),
            ledger_id: self.ledger_id.to_bytes(),
            keys: Some(self.keys.to_protobuf()),
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hedera_proto::services;
    use prost::Message;

    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::transaction::test_helpers::unused_private_key;
    use crate::{
        FileInfo,
        Key,
        LedgerId,
    };

    fn make_info() -> services::file_get_info_response::FileInfo {
        services::file_get_info_response::FileInfo {
            file_id: Some(services::FileId { shard_num: 0, realm_num: 0, file_num: 1 }),
            size: 2,
            expiration_time: Some(services::Timestamp { seconds: 0, nanos: 3_000 }),
            deleted: true,
            keys: Some(services::KeyList {
                keys: Vec::from([Key::from(unused_private_key().public_key()).to_protobuf()]),
            }),
            ledger_id: LedgerId::mainnet().to_bytes(),

            ..Default::default()
        }
    }

    #[test]
    fn from_protobuf() {
        expect![[r#"
            FileInfo {
                file_id: "0.0.1",
                size: 2,
                expiration_time: Some(
                    1970-01-01 0:00:00.000003 +00:00:00,
                ),
                auto_renew_period: None,
                auto_renew_account_id: None,
                is_deleted: true,
                keys: KeyList {
                    keys: [
                        Single(
                            "302a300506032b6570032100e0c8ec2758a5879ffac226a13c0c516b799e72e35141a0dd828f94d37988a4b7",
                        ),
                    ],
                    threshold: None,
                },
                file_memo: "",
                ledger_id: "mainnet",
            }
        "#]].assert_debug_eq(&FileInfo::from_protobuf(make_info()).unwrap());
    }

    #[test]
    fn to_protobuf() {
        expect![[r#"
            FileInfo {
                file_id: Some(
                    FileId {
                        shard_num: 0,
                        realm_num: 0,
                        file_num: 1,
                    },
                ),
                size: 2,
                expiration_time: Some(
                    Timestamp {
                        seconds: 0,
                        nanos: 3000,
                    },
                ),
                deleted: true,
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
                memo: "",
                ledger_id: [
                    0,
                ],
            }
        "#]]
        .assert_debug_eq(&FileInfo::from_protobuf(make_info()).unwrap().to_protobuf());
    }

    #[test]
    fn from_bytes() {
        expect![[r#"
            FileInfo {
                file_id: "0.0.1",
                size: 2,
                expiration_time: Some(
                    1970-01-01 0:00:00.000003 +00:00:00,
                ),
                auto_renew_period: None,
                auto_renew_account_id: None,
                is_deleted: true,
                keys: KeyList {
                    keys: [
                        Single(
                            "302a300506032b6570032100e0c8ec2758a5879ffac226a13c0c516b799e72e35141a0dd828f94d37988a4b7",
                        ),
                    ],
                    threshold: None,
                },
                file_memo: "",
                ledger_id: "mainnet",
            }
        "#]].assert_debug_eq(&FileInfo::from_bytes(&make_info().encode_to_vec()).unwrap());
    }
}
