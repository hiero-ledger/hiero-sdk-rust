// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::smart_contract_service_client::SmartContractServiceClient;
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::query::{
    AnyQueryData,
    QueryExecute,
    ToQueryProtobuf,
};
use crate::{
    BoxGrpcFuture,
    ContractId,
    Error,
    FromProtobuf,
    Query,
    ToProtobuf,
    ValidateChecksums,
};

/// Get the runtime bytecode for a smart contract instance.
pub type ContractBytecodeQuery = Query<ContractBytecodeQueryData>;

#[derive(Default, Debug, Clone)]
pub struct ContractBytecodeQueryData {
    /// The contract for which information is requested.
    contract_id: Option<ContractId>,
}

impl ContractBytecodeQuery {
    /// Gets the contract for which information is requested.
    #[must_use]
    pub fn get_contract_id(&self) -> Option<ContractId> {
        self.data.contract_id
    }

    /// Sets the contract for which information is requested.
    pub fn contract_id(&mut self, contract_id: ContractId) -> &mut Self {
        self.data.contract_id = Some(contract_id);
        self
    }
}

impl From<ContractBytecodeQueryData> for AnyQueryData {
    #[inline]
    fn from(data: ContractBytecodeQueryData) -> Self {
        Self::ContractBytecode(data)
    }
}

impl ToQueryProtobuf for ContractBytecodeQueryData {
    fn to_query_protobuf(&self, header: services::QueryHeader) -> services::Query {
        let contract_id = self.contract_id.to_protobuf();

        services::Query {
            query: Some(services::query::Query::ContractGetBytecode(
                services::ContractGetBytecodeQuery { contract_id, header: Some(header) },
            )),
        }
    }
}

impl QueryExecute for ContractBytecodeQueryData {
    type Response = Vec<u8>;

    fn execute(
        &self,
        channel: Channel,
        request: services::Query,
    ) -> BoxGrpcFuture<'_, services::Response> {
        Box::pin(async {
            SmartContractServiceClient::new(channel).contract_get_bytecode(request).await
        })
    }
}

impl ValidateChecksums for ContractBytecodeQueryData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.contract_id.validate_checksums(ledger_id)
    }
}

impl FromProtobuf<services::response::Response> for Vec<u8> {
    fn from_protobuf(pb: services::response::Response) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let pb = pb_getv!(pb, ContractGetBytecodeResponse, services::response::Response);

        Ok(pb.bytecode)
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::query::ToQueryProtobuf;
    use crate::{
        ContractBytecodeQuery,
        ContractId,
        Hbar,
    };

    #[test]
    fn serialize() {
        expect![[r#"
            Query {
                query: Some(
                    ContractGetBytecode(
                        ContractGetBytecodeQuery {
                            header: Some(
                                QueryHeader {
                                    payment: None,
                                    response_type: AnswerOnly,
                                },
                            ),
                            contract_id: Some(
                                ContractId {
                                    shard_num: 0,
                                    realm_num: 0,
                                    contract: Some(
                                        ContractNum(
                                            5005,
                                        ),
                                    ),
                                },
                            ),
                        },
                    ),
                ),
            }
        "#]]
        .assert_debug_eq(
            &ContractBytecodeQuery::new()
                .contract_id(crate::ContractId::new(0, 0, 5005))
                .max_payment_amount(Hbar::from_tinybars(100_000))
                .data
                .to_query_protobuf(Default::default()),
        );
    }

    #[test]
    fn get_set_contract_id() {
        let mut query = ContractBytecodeQuery::new();
        query.contract_id(ContractId::new(0, 0, 5005));

        assert_eq!(query.get_contract_id(), Some(ContractId::new(0, 0, 5005)));
    }
}
