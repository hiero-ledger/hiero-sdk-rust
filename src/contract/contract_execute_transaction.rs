// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::smart_contract_service_client::SmartContractServiceClient;
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::protobuf::FromProtobuf;
use crate::transaction::{
    AnyTransactionData,
    ChunkInfo,
    ToSchedulableTransactionDataProtobuf,
    ToTransactionDataProtobuf,
    TransactionData,
    TransactionExecute,
};
use crate::{
    BoxGrpcFuture,
    ContractFunctionParameters,
    ContractId,
    Error,
    Hbar,
    ToProtobuf,
    Transaction,
    ValidateChecksums,
};

/// Call a function of the given smart contract instance, giving it
/// parameters as its inputs.
///
/// It can use the given amount of gas, and any unspent gas will
/// be refunded to the paying account.
///
/// If this function stores information, it is charged gas to store it.
/// There is a fee in hbars to maintain that storage until the expiration time,
/// and that fee is added as part of the transaction fee.
///
pub type ContractExecuteTransaction = Transaction<ContractExecuteTransactionData>;

#[derive(Default, Debug, Clone)]
pub struct ContractExecuteTransactionData {
    /// The contract instance to call.
    contract_id: Option<ContractId>,

    /// The maximum amount of gas to use for the call.
    gas: u64,

    /// The number of hbars sent with this function call.
    payable_amount: Hbar,

    /// The function parameters as their raw bytes.
    function_parameters: Vec<u8>,
}

impl ContractExecuteTransaction {
    /// Returns the contract instance to call.
    #[must_use]
    pub fn get_contract_id(&self) -> Option<ContractId> {
        self.data().contract_id
    }

    /// Sets the contract instance to call.
    pub fn contract_id(&mut self, contract_id: ContractId) -> &mut Self {
        self.data_mut().contract_id = Some(contract_id);
        self
    }

    /// Returns the maximum amount of gas to use for the call.
    #[must_use]
    pub fn get_gas(&self) -> u64 {
        self.data().gas
    }

    /// Sets the maximum amount of gas to use for the call.
    pub fn gas(&mut self, gas: u64) -> &mut Self {
        self.data_mut().gas = gas;
        self
    }

    /// Returns the number of hbars to be sent with this function call.
    #[must_use]
    pub fn get_payable_amount(&self) -> Hbar {
        self.data().payable_amount
    }

    /// Sets the number of hbars to be sent with this function call.
    pub fn payable_amount(&mut self, amount: Hbar) -> &mut Self {
        self.data_mut().payable_amount = amount;
        self
    }

    /// Returns the function parameters as their raw bytes.
    #[must_use]
    pub fn get_function_parameters(&self) -> &[u8] {
        &self.data().function_parameters
    }

    /// Sets the function parameters as their raw bytes.
    pub fn function_parameters(&mut self, data: Vec<u8>) -> &mut Self {
        self.data_mut().function_parameters = data;
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
}

impl TransactionData for ContractExecuteTransactionData {}

impl TransactionExecute for ContractExecuteTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async {
            SmartContractServiceClient::new(channel).contract_call_method(request).await
        })
    }
}

impl ValidateChecksums for ContractExecuteTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.contract_id.validate_checksums(ledger_id)?;
        Ok(())
    }
}

impl ToTransactionDataProtobuf for ContractExecuteTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::ContractCall(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for ContractExecuteTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::ContractCall(self.to_protobuf())
    }
}

impl From<ContractExecuteTransactionData> for AnyTransactionData {
    fn from(transaction: ContractExecuteTransactionData) -> Self {
        Self::ContractExecute(transaction)
    }
}

impl FromProtobuf<services::ContractCallTransactionBody> for ContractExecuteTransactionData {
    fn from_protobuf(pb: services::ContractCallTransactionBody) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            contract_id: Option::from_protobuf(pb.contract_id)?,
            gas: pb.gas as u64,
            payable_amount: Hbar::from_tinybars(pb.amount),
            function_parameters: pb.function_parameters,
        })
    }
}

impl ToProtobuf for ContractExecuteTransactionData {
    type Protobuf = services::ContractCallTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        #[allow(deprecated)]
        services::ContractCallTransactionBody {
            gas: self.gas as i64,
            amount: self.payable_amount.to_tinybars(),
            contract_id: self.contract_id.to_protobuf(),
            function_parameters: self.function_parameters.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hedera_proto::services;

    use crate::contract::ContractExecuteTransactionData;
    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
    };
    use crate::{
        AnyTransaction,
        ContractExecuteTransaction,
        ContractId,
        Hbar,
    };

    const CONTRACT_ID: ContractId = ContractId::new(0, 0, 5007);
    const GAS: u64 = 10;
    const PAYABLE_AMOUNT: Hbar = Hbar::from_tinybars(1000);

    fn function_parameters() -> Vec<u8> {
        Vec::from([24, 43, 11])
    }

    fn make_transaction() -> ContractExecuteTransaction {
        let mut tx = ContractExecuteTransaction::new_for_tests();

        tx.contract_id(CONTRACT_ID)
            .gas(GAS)
            .payable_amount(PAYABLE_AMOUNT)
            .function_parameters(function_parameters())
            .freeze()
            .unwrap();

        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            ContractCall(
                ContractCallTransactionBody {
                    contract_id: Some(
                        ContractId {
                            shard_num: 0,
                            realm_num: 0,
                            contract: Some(
                                ContractNum(
                                    5007,
                                ),
                            ),
                        },
                    ),
                    gas: 10,
                    amount: 1000,
                    function_parameters: [
                        24,
                        43,
                        11,
                    ],
                },
            )
        "#]]
        .assert_debug_eq(&tx)
    }

    #[test]
    fn to_from_bytes() {
        let tx = make_transaction();

        let tx2 = AnyTransaction::from_bytes(&tx.to_bytes().unwrap()).unwrap();

        let tx = transaction_body(tx);

        let tx2 = transaction_body(tx2);

        assert_eq!(tx, tx2);
    }

    #[test]
    fn from_proto_body() {
        let tx = services::ContractCallTransactionBody {
            contract_id: Some(CONTRACT_ID.to_protobuf()),
            gas: GAS as _,
            amount: PAYABLE_AMOUNT.to_tinybars(),
            function_parameters: function_parameters(),
        };

        let tx = ContractExecuteTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(tx.contract_id, Some(CONTRACT_ID));
        assert_eq!(tx.gas, GAS);
        assert_eq!(tx.payable_amount, PAYABLE_AMOUNT);
        assert_eq!(tx.function_parameters, function_parameters());
    }

    #[test]
    fn get_set_contract_id() {
        let mut tx = ContractExecuteTransaction::new();
        tx.contract_id(CONTRACT_ID);

        assert_eq!(tx.get_contract_id(), Some(CONTRACT_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_contract_id_frozen_panics() {
        make_transaction().contract_id(CONTRACT_ID);
    }

    #[test]
    fn get_set_gas() {
        let mut tx = ContractExecuteTransaction::new();
        tx.gas(GAS);

        assert_eq!(tx.get_gas(), GAS);
    }

    #[test]
    #[should_panic]
    fn get_set_gas_frozen_panics() {
        make_transaction().gas(GAS);
    }

    #[test]
    fn get_set_payable_amount() {
        let mut tx = ContractExecuteTransaction::new();
        tx.payable_amount(PAYABLE_AMOUNT);

        assert_eq!(tx.get_payable_amount(), PAYABLE_AMOUNT);
    }

    #[test]
    #[should_panic]
    fn get_set_payable_amount_frozen_panics() {
        make_transaction().payable_amount(PAYABLE_AMOUNT);
    }

    #[test]
    fn get_set_function_parameters() {
        let mut tx = ContractExecuteTransaction::new();
        tx.function_parameters(function_parameters());

        assert_eq!(tx.get_function_parameters(), function_parameters());
    }

    #[test]
    #[should_panic]
    fn get_set_function_parameters_frozen_panics() {
        make_transaction().function_parameters(function_parameters());
    }
}
