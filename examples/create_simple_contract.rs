// SPDX-License-Identifier: Apache-2.0

mod resources;

use clap::Parser;
use hedera::{
    AccountId, Client, ContractCallQuery, ContractCreateTransaction, ContractDeleteTransaction, ContractFunctionParameters, FileCreateTransaction, PrivateKey
};

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, env)]
    operator_account_id: AccountId,

    #[clap(long, env)]
    operator_key: PrivateKey,

    #[clap(long, env, default_value = "testnet")]
    hedera_network: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    let args = Args::parse();

    let client = Client::for_name(&args.hedera_network)?;

    client.set_operator(args.operator_account_id, args.operator_key.clone());

    let bytecode = resources::simple_bytecode();

    // create the contract's bytecode file
    let file_transaction_response = FileCreateTransaction::new()
        // Use the same key as the operator to "own" this file
        .keys([args.operator_key.public_key()])
        .contents(bytecode)
        .execute(&client)
        .await?;

    let file_receipt = file_transaction_response.get_receipt(&client).await?;
    let new_file_id = file_receipt.file_id.unwrap();

    println!("contract bytecode file: {new_file_id}");

    let contract_transaction_response = ContractCreateTransaction::new()
        .bytecode_file_id(new_file_id)
        .gas(500_000)
        .admin_key(args.operator_key.public_key())
        .constructor_parameters(
            ContractFunctionParameters::new()
                .add_string("hello from hedera!")
                .to_bytes(None),
        )
        .execute(&client)
        .await?;

    let contract_receipt = contract_transaction_response.get_receipt(&client).await?;
    let new_contract_id = contract_receipt.contract_id.unwrap();

    println!("new contract ID: {new_contract_id}");

    let contract_call_result = ContractCallQuery::new()
        .contract_id(new_contract_id)
        .gas(500_000)
        .function("greet")
        .execute(&client)
        .await?;

    if let Some(err) = contract_call_result.error_message {
        anyhow::bail!("error calling contract: {err}");
    }

    let message = contract_call_result.get_str(0);
    println!("contract returned message: {message:?}");

    // now delete the contract
    let _contract_delete_result = ContractDeleteTransaction::new()
        .contract_id(new_contract_id)
        .transfer_account_id(args.operator_account_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    println!("Contract successfully deleted");

    Ok(())
}
