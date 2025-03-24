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
    ContractInfo,
    Error,
    Query,
    ToProtobuf,
    ValidateChecksums,
};

/// Get information about a smart contract instance.
pub type ContractInfoQuery = Query<ContractInfoQueryData>;

#[derive(Default, Debug, Clone)]
pub struct ContractInfoQueryData {
    /// The contract for which information is requested.
    contract_id: Option<ContractId>,
}

impl ContractInfoQuery {
    /// Returns the contract for which information is requested.
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

impl From<ContractInfoQueryData> for AnyQueryData {
    #[inline]
    fn from(data: ContractInfoQueryData) -> Self {
        Self::ContractInfo(data)
    }
}

impl ToQueryProtobuf for ContractInfoQueryData {
    fn to_query_protobuf(&self, header: services::QueryHeader) -> services::Query {
        let contract_id = self.contract_id.to_protobuf();

        services::Query {
            query: Some(services::query::Query::ContractGetInfo(services::ContractGetInfoQuery {
                contract_id,
                header: Some(header),
            })),
        }
    }
}

impl QueryExecute for ContractInfoQueryData {
    type Response = ContractInfo;

    fn execute(
        &self,
        channel: Channel,
        request: services::Query,
    ) -> BoxGrpcFuture<'_, services::Response> {
        Box::pin(async {
            SmartContractServiceClient::new(channel).get_contract_info(request).await
        })
    }
}

impl ValidateChecksums for ContractInfoQueryData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.contract_id.validate_checksums(ledger_id)
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::query::ToQueryProtobuf;
    use crate::{
        ContractId,
        ContractInfoQuery,
        Hbar,
    };

    #[test]
    fn serialize() {
        expect![[r#"
            Query {
                query: Some(
                    ContractGetInfo(
                        ContractGetInfoQuery {
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
            &ContractInfoQuery::new()
                .contract_id(crate::ContractId {
                    shard: 0,
                    realm: 0,
                    num: 5005,
                    evm_address: None,
                    checksum: None,
                })
                .max_payment_amount(Hbar::from_tinybars(100_000))
                .data
                .to_query_protobuf(Default::default()),
        );
    }

    #[test]
    fn get_set_contract_id() {
        let mut query = ContractInfoQuery::new();
        query.contract_id(ContractId::new(0, 0, 5005));

        assert_eq!(query.get_contract_id(), Some(ContractId::new(0, 0, 5005)));
    }
}
