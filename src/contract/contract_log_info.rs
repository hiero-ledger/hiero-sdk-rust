use crate::proto::services;
use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};
use crate::ContractId;

/// The log information for an event returned by a smart contract function call.
/// One function call may return several such events.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ContractLogInfo {
    /// Address of the contract that emitted the event.
    pub contract_id: ContractId,

    /// Bloom filter for this log.
    pub bloom: Vec<u8>,

    /// A list of topics this log is relevent to.
    pub topics: Vec<Vec<u8>>,

    /// The log's data payload.
    pub data: Vec<u8>,
}

impl ContractLogInfo {
    /// Create a new `ContractLogInfo` from protobuf-encoded `bytes`.
    ///
    /// # Errors
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the bytes fails to produce a valid protobuf.
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the protobuf fails.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        FromProtobuf::from_bytes(bytes)
    }

    /// Convert `self` to a protobuf-encoded [`Vec<u8>`].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        ToProtobuf::to_bytes(self)
    }
}

impl FromProtobuf<services::ContractLoginfo> for ContractLogInfo {
    fn from_protobuf(pb: services::ContractLoginfo) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            contract_id: ContractId::from_protobuf(pb_getf!(pb, contract_id)?)?,
            bloom: pb.bloom,
            topics: pb.topic,
            data: pb.data,
        })
    }
}

impl ToProtobuf for ContractLogInfo {
    type Protobuf = services::ContractLoginfo;

    fn to_protobuf(&self) -> Self::Protobuf {
        Self::Protobuf {
            contract_id: Some(self.contract_id.to_protobuf()),
            bloom: self.bloom.clone(),
            topic: self.topics.clone(),
            data: self.data.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use prost::Message;

    use crate::proto::services;
    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::ContractLogInfo;

    fn make_info() -> services::ContractLoginfo {
        services::ContractLoginfo {
            contract_id: Some(services::ContractId {
                shard_num: 0,
                realm_num: 0,
                contract: Some(services::contract_id::Contract::ContractNum(10)),
            }),
            bloom: b"bloom".to_vec(),
            topic: Vec::from([b"bloom".to_vec()]),
            data: b"data".to_vec(),
        }
    }

    #[test]
    fn from_protobuf() {
        expect![[r#"
            ContractLogInfo {
                contract_id: "0.0.10",
                bloom: [
                    98,
                    108,
                    111,
                    111,
                    109,
                ],
                topics: [
                    [
                        98,
                        108,
                        111,
                        111,
                        109,
                    ],
                ],
                data: [
                    100,
                    97,
                    116,
                    97,
                ],
            }
        "#]]
        .assert_debug_eq(&ContractLogInfo::from_protobuf(make_info()).unwrap());
    }

    #[test]
    fn to_protobuf() {
        expect![[r#"
            ContractLoginfo {
                contract_id: Some(
                    ContractId {
                        shard_num: 0,
                        realm_num: 0,
                        contract: Some(
                            ContractNum(
                                10,
                            ),
                        ),
                    },
                ),
                bloom: [
                    98,
                    108,
                    111,
                    111,
                    109,
                ],
                topic: [
                    [
                        98,
                        108,
                        111,
                        111,
                        109,
                    ],
                ],
                data: [
                    100,
                    97,
                    116,
                    97,
                ],
            }
        "#]]
        .assert_debug_eq(&ContractLogInfo::from_protobuf(make_info()).unwrap().to_protobuf())
    }

    #[test]
    fn from_bytes() {
        expect![[r#"
            ContractLogInfo {
                contract_id: "0.0.10",
                bloom: [
                    98,
                    108,
                    111,
                    111,
                    109,
                ],
                topics: [
                    [
                        98,
                        108,
                        111,
                        111,
                        109,
                    ],
                ],
                data: [
                    100,
                    97,
                    116,
                    97,
                ],
            }
        "#]]
        .assert_debug_eq(&ContractLogInfo::from_bytes(&make_info().encode_to_vec()).unwrap());
    }
}
