// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;

use crate::{
    FromProtobuf,
    SemanticVersion,
    ToProtobuf,
};

/// Versions of Hiero Services, and the protobuf schema.
#[derive(Debug, Clone)]
pub struct NetworkVersionInfo {
    /// Version of the protobuf schema in use by the network.
    pub protobuf_version: SemanticVersion,

    /// Version of the Hiero services in use by the network.
    pub services_version: SemanticVersion,
}

impl NetworkVersionInfo {
    /// Create a new `NetworkVersionInfo` from protobuf-encoded `bytes`.
    ///
    /// # Errors
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the bytes fails to produce a valid protobuf.
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the protobuf fails.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        FromProtobuf::<services::NetworkGetVersionInfoResponse>::from_bytes(bytes)
    }

    /// Convert `self` to a protobuf-encoded [`Vec<u8>`].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        ToProtobuf::to_bytes(self)
    }
}

impl FromProtobuf<services::response::Response> for NetworkVersionInfo {
    fn from_protobuf(pb: services::response::Response) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let pb = pb_getv!(pb, NetworkGetVersionInfo, services::response::Response);
        Self::from_protobuf(pb)
    }
}

impl FromProtobuf<services::NetworkGetVersionInfoResponse> for NetworkVersionInfo {
    fn from_protobuf(pb: services::NetworkGetVersionInfoResponse) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let protobuf_version = pb_getf!(pb, hapi_proto_version)?;
        let services_version = pb_getf!(pb, hedera_services_version)?;

        Ok(Self {
            protobuf_version: SemanticVersion::from_protobuf(protobuf_version)?,
            services_version: SemanticVersion::from_protobuf(services_version)?,
        })
    }
}

impl ToProtobuf for NetworkVersionInfo {
    type Protobuf = services::NetworkGetVersionInfoResponse;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::NetworkGetVersionInfoResponse {
            header: None,
            hapi_proto_version: Some(self.protobuf_version.to_protobuf()),
            hedera_services_version: Some(self.services_version.to_protobuf()),
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::protobuf::ToProtobuf;
    use crate::NetworkVersionInfo;

    const INFO: NetworkVersionInfo = NetworkVersionInfo {
        protobuf_version: crate::SemanticVersion {
            major: 1,
            minor: 2,
            patch: 3,
            prerelease: String::new(),
            build: String::new(),
        },
        services_version: crate::SemanticVersion {
            major: 4,
            minor: 5,
            patch: 6,
            prerelease: String::new(),
            build: String::new(),
        },
    };

    #[test]
    fn serialize() {
        expect![[r#"
            NetworkGetVersionInfoResponse {
                header: None,
                hapi_proto_version: Some(
                    SemanticVersion {
                        major: 1,
                        minor: 2,
                        patch: 3,
                        pre: "",
                        build: "",
                    },
                ),
                hedera_services_version: Some(
                    SemanticVersion {
                        major: 4,
                        minor: 5,
                        patch: 6,
                        pre: "",
                        build: "",
                    },
                ),
            }
        "#]]
        .assert_debug_eq(&INFO.to_protobuf());
    }

    #[test]
    fn to_from_bytes() {
        let a = INFO;
        let b = NetworkVersionInfo::from_bytes(&a.to_bytes()).unwrap();

        assert_eq!(a.protobuf_version.to_string(), b.protobuf_version.to_string());
        assert_eq!(a.services_version.to_string(), b.services_version.to_string());
    }
}
