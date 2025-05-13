// SPDX-License-Identifier: Apache-2.0

use hedera::{
    AccountId, Client, ContractCreateFlow, ContractExecuteTransaction, ContractFunctionParameters,
    ContractFunctionResult, ContractId, Hbar, PrivateKey, TransactionId,
};

#[derive(Default)]
pub struct ContractStep {
    pub result_inspector: Option<fn(&ContractFunctionResult)>,
    pub parameters: Option<Box<dyn Fn() -> ContractFunctionParameters>>,
    pub payable_amount: Option<Hbar>,
    pub signers: Vec<PrivateKey>,
    pub fee_payer: Option<AccountId>,
}

pub struct ContractHelper {
    contract_id: ContractId,
    steps: Vec<ContractStep>,
}

impl ContractHelper {
    pub fn new(contract_id: ContractId, steps: Vec<ContractStep>) -> Self {
        Self { contract_id, steps }
    }

    pub async fn execute(&self, client: &Client) -> hedera::Result<()> {
        for (index, step) in self.steps.iter().enumerate() {
            println!("Attempting to execute step {index}");

            let mut tx = ContractExecuteTransaction::new();

            tx.contract_id(self.contract_id).gas(10_000_000);

            if let Some(payable_amount) = step.payable_amount {
                tx.payable_amount(payable_amount);
            }

            let function_name = format!("step{index}");
            let params = step.parameters.as_deref().map(|it| it());

            match params {
                Some(params) => tx.function_with_parameters(&function_name, &params),
                None => tx.function(&function_name),
            };

            if let Some(fee_payer) = step.fee_payer {
                tx.transaction_id(TransactionId::generate(fee_payer));
            }

            tx.freeze_with(client)?;

            for signer in &step.signers {
                tx.sign(signer.clone());
            }

            let record = tx
                .execute(client)
                .await?
                .validate_status(false)
                .get_record(client)
                .await?;

            if let Err(e) = record.receipt.validate_status(true) {
                eprintln!("Error occurred during step{index}: {e}");
                eprintln!("Transaction record: {record:?}");
                break;
            }

            let function_result = record.contract_function_result.unwrap();
            println!("gas used: {}", function_result.gas_used);

            if let Some(inspector) = step.result_inspector {
                inspector(&function_result)
            }

            println!(
                "step {index} completed, and returned valid result. (TransactionId `{}`",
                record.transaction_id
            );
        }

        Ok(())
    }
}

pub async fn create_contract(
    client: &Client,
    bytecode: &str,
    constructor_parameters: ContractFunctionParameters,
) -> hedera::Result<ContractId> {
    let contract_id = ContractCreateFlow::new()
        .bytecode_hex(bytecode)?
        .max_chunks(30)
        .gas(8_000_000)
        .constructor_parameters(constructor_parameters.to_bytes(None))
        .execute(client)
        .await?
        .get_receipt(client)
        .await?
        .contract_id
        .unwrap();

    Ok(contract_id)
}
