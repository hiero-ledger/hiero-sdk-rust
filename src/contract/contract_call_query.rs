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
    AccountId,
    BoxGrpcFuture,
    ContractFunctionParameters,
    ContractFunctionResult,
    ContractId,
    Error,
    Query,
    ToProtobuf,
    ValidateChecksums,
};

/// Call a function of the given smart contract instance.
/// It will consume the entire given amount of gas.
///
/// This is performed locally on the particular node that the client is communicating with.
/// It cannot change the state of the contract instance (and so, cannot spend
/// anything from the instance's cryptocurrency account).
///
pub type ContractCallQuery = Query<ContractCallQueryData>;

#[derive(Default, Debug, Clone)]
pub struct ContractCallQueryData {
    /// The contract instance to call.
    contract_id: Option<ContractId>,

    /// The amount of gas to use for the call.
    gas: u64,

    /// The function parameters as their raw bytes.
    function_parameters: Vec<u8>,

    /// The sender for this transaction.
    sender_account_id: Option<AccountId>,
}

impl ContractCallQuery {
    /// Gets the contract instance to call.
    #[must_use]
    pub fn get_contract_id(&self) -> Option<ContractId> {
        self.data.contract_id
    }

    /// Sets the contract to make a static call against.
    pub fn contract_id(&mut self, contract_id: ContractId) -> &mut Self {
        self.data.contract_id = Some(contract_id);
        self
    }

    /// Gets the amount of gas to use for the call.
    #[must_use]
    pub fn get_gas(&self) -> u64 {
        self.data.gas
    }

    /// Sets the amount of gas to use for the call.
    pub fn gas(&mut self, gas: u64) -> &mut Self {
        self.data.gas = gas;
        self
    }

    /// Gets the function parameters as their raw bytes.
    #[must_use]
    pub fn get_contract_parameters(&self) -> &[u8] {
        self.data.function_parameters.as_ref()
    }

    /// Sets the function parameters as their raw bytes.
    pub fn function_parameters(&mut self, data: Vec<u8>) -> &mut Self {
        self.data.function_parameters = data;
        self
    }

    /// Sets the function with no parameters.
    pub fn function(&mut self, name: &str) -> &mut Self {
        self.function_with_parameters(name, &ContractFunctionParameters::new())
    }

    /// Sets the function with parameters.
    pub fn function_with_parameters(
        &mut self,
        name: &str,
        parameters: &ContractFunctionParameters,
    ) -> &mut Self {
        self.function_parameters(parameters.to_bytes(Some(name)))
    }

    /// Gets the sender for this transaction.
    #[must_use]
    pub fn get_sender_account_id(&self) -> Option<AccountId> {
        self.data.sender_account_id
    }

    /// Sets the sender for this transaction.
    pub fn sender_account_id(&mut self, sender_account_id: AccountId) -> &mut Self {
        self.data.sender_account_id = Some(sender_account_id);
        self
    }
}

impl From<ContractCallQueryData> for AnyQueryData {
    #[inline]
    fn from(data: ContractCallQueryData) -> Self {
        Self::ContractCall(data)
    }
}

impl ToQueryProtobuf for ContractCallQueryData {
    fn to_query_protobuf(&self, header: services::QueryHeader) -> services::Query {
        let contract_id = self.contract_id.to_protobuf();
        let sender_id = self.sender_account_id.to_protobuf();

        services::Query {
            query: Some(services::query::Query::ContractCallLocal(
                #[allow(deprecated)]
                services::ContractCallLocalQuery {
                    contract_id,
                    gas: self.gas as i64,
                    function_parameters: self.function_parameters.clone(),
                    max_result_size: 0,
                    header: Some(header),
                    sender_id,
                },
            )),
        }
    }
}

impl QueryExecute for ContractCallQueryData {
    type Response = ContractFunctionResult;

    fn execute(
        &self,
        channel: Channel,
        request: services::Query,
    ) -> BoxGrpcFuture<'_, services::Response> {
        Box::pin(async {
            SmartContractServiceClient::new(channel).contract_call_local_method(request).await
        })
    }
}

impl ValidateChecksums for ContractCallQueryData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.contract_id.validate_checksums(ledger_id)?;
        self.sender_account_id.validate_checksums(ledger_id)
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hedera_proto::services;

    use crate::query::ToQueryProtobuf;
    use crate::{
        AccountId,
        ContractCallQuery,
        ContractFunctionParameters,
        ContractId,
        Hbar,
    };

    fn make_query() -> ContractCallQuery {
        let mut query = ContractCallQuery::new();

        query
            .contract_id(crate::ContractId::new(0, 0, 5005))
            .gas(1541)
            .sender_account_id("1.2.3".parse().unwrap())
            .max_payment_amount(Hbar::from_tinybars(100_000));

        query
    }

    #[test]
    fn serialize() {
        expect![[r#"
            Query {
                query: Some(
                    ContractCallLocal(
                        ContractCallLocalQuery {
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
                            gas: 1541,
                            function_parameters: [
                                18,
                                74,
                                131,
                                250,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                64,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                128,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                5,
                                72,
                                101,
                                108,
                                108,
                                111,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                6,
                                119,
                                111,
                                114,
                                108,
                                100,
                                33,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                            ],
                            max_result_size: 0,
                            sender_id: Some(
                                AccountId {
                                    shard_num: 1,
                                    realm_num: 2,
                                    account: Some(
                                        AccountNum(
                                            3,
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
            &make_query()
                .function_with_parameters(
                    "foo",
                    ContractFunctionParameters::new().add_string("Hello").add_string("world!"),
                )
                .data
                .to_query_protobuf(services::QueryHeader::default()),
        );
    }

    #[test]
    fn function_parameters() {
        expect![[r#"
            Query {
                query: Some(
                    ContractCallLocal(
                        ContractCallLocalQuery {
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
                            gas: 1541,
                            function_parameters: [
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                64,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                128,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                5,
                                72,
                                101,
                                108,
                                108,
                                111,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                6,
                                119,
                                111,
                                114,
                                108,
                                100,
                                33,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                            ],
                            max_result_size: 0,
                            sender_id: Some(
                                AccountId {
                                    shard_num: 1,
                                    realm_num: 2,
                                    account: Some(
                                        AccountNum(
                                            3,
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
            &make_query()
                .function_parameters(
                    ContractFunctionParameters::new()
                        .add_string("Hello")
                        .add_string("world!")
                        .to_bytes(None),
                )
                .data
                .to_query_protobuf(services::QueryHeader::default()),
        );
    }

    #[test]
    fn get_set_contract_id() {
        let mut query = ContractCallQuery::new();
        query.contract_id(ContractId::new(0, 0, 5005));

        assert_eq!(query.get_contract_id(), Some(ContractId::new(0, 0, 5005)));
    }

    #[test]
    fn get_set_gas() {
        let mut query = ContractCallQuery::new();
        query.gas(1541);

        assert_eq!(query.get_gas(), 1541);
    }

    #[test]
    fn get_set_contract_parameters() {
        const BYTES: [u8; 6] = [0x0a, 0x0b, 0x0c, 0x01, 0x02, 0x03];
        let mut query = ContractCallQuery::new();
        query.function_parameters(Vec::from(BYTES));

        assert_eq!(query.get_contract_parameters(), &BYTES);
    }

    #[test]
    fn get_set_sender_account_id() {
        let mut query = ContractCallQuery::new();
        query.sender_account_id(AccountId::new(1, 2, 3));

        assert_eq!(query.get_sender_account_id(), Some(AccountId::new(1, 2, 3)));
    }
}
