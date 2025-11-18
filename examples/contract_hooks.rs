// SPDX-License-Identifier: Apache-2.0

mod resources;

use std::fs;
use std::path::Path;

use clap::Parser;
use hedera::{
    AccountId, Client, ContractCreateTransaction, ContractId, ContractUpdateTransaction, EvmHookSpec, HookCreationDetails, HookExtensionPoint, LambdaEvmHook, PrivateKey
};

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, env, default_value = "0.0.2")]
    operator_account_id: AccountId,

    #[clap(
        long,
        env,
        default_value = "302e020100300506032b65700422042091132178e72057a1d7528025956fe39b0b847f200ab59b2fdd367017f3087137"
    )]
    operator_key: PrivateKey,

    #[clap(long, env, default_value = "testnet")]
    hedera_network: String,
}

const HOOK_BYTECODE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/lambda_sstore_hook.hex"
);

fn load_hook_bytecode() -> anyhow::Result<Vec<u8>> {
    let bytecode_hex = fs::read_to_string(Path::new(HOOK_BYTECODE_PATH))?;
    let cleaned_hex: String = bytecode_hex.split_whitespace().collect();
    let bytecode = hex::decode(cleaned_hex)?;
    Ok(bytecode)
}

async fn create_hook_contract(
    client: &Client,
    operator_key: &PrivateKey,
) -> anyhow::Result<ContractId> {
    let bytecode = load_hook_bytecode()?;

    let receipt = ContractCreateTransaction::new()
        .admin_key(operator_key.public_key())
        .gas(500_000)
        .bytecode(bytecode)
        .freeze_with(client)?
        .sign(operator_key.clone())
        .execute(client)
        .await?
        .get_receipt(client)
        .await?;

    Ok(receipt.contract_id.unwrap())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    let args = Args::parse();

    let client = Client::for_name(&args.hedera_network)?;
    client.set_operator(args.operator_account_id, args.operator_key.clone());

    println!("Contract Hooks Example Start!");

    /*
     * Step 1: Create the hook contract.
     */
    println!("Creating hook contract...");

    let hook_contract_id = create_hook_contract(&client, &args.operator_key).await?;
    println!("Hook contract created with ID: {hook_contract_id}");

    /*
     * Step 2: Demonstrate creating a contract with hooks.
     */
    println!("\n=== Creating Contract with Hooks ===");
    println!("Creating contract with lambda EVM hook...");

    let simple_contract_bytecode = hex::decode(resources::simple_bytecode())?;

    // Build a basic lambda EVM hook (no admin key, no storage updates)
    let spec = EvmHookSpec::new(Some(hook_contract_id));
    let lambda_hook = LambdaEvmHook::new(spec, vec![]);
    let hook_with_id_1 = HookCreationDetails::new(
        HookExtensionPoint::AccountAllowanceHook,
        1,
        Some(lambda_hook),
    );

    let contract_receipt = ContractCreateTransaction::new()
        .admin_key(args.operator_key.public_key())
        .gas(400_000)
        .bytecode(simple_contract_bytecode)
        .add_hook(hook_with_id_1)
        .freeze_with(&client)?
        .sign(args.operator_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let contract_with_hooks_id = contract_receipt.contract_id.unwrap();
    println!("Created contract with ID: {contract_with_hooks_id}");
    println!("Successfully created contract with basic lambda hook!");

    /*
     * Step 3: Demonstrate adding hooks to an existing contract.
     */
    println!("\n=== Adding Hooks to Existing Contract ===");
    println!("Adding hooks to existing contract...");

    let admin_key = client.get_operator_public_key().unwrap();
    let spec3 = EvmHookSpec::new(Some(hook_contract_id));
    let basic_hook = LambdaEvmHook::new(spec3, vec![]);
    let mut hook_with_id_3 = HookCreationDetails::new(
        HookExtensionPoint::AccountAllowanceHook,
        3,
        Some(basic_hook),
    );
    hook_with_id_3.admin_key = Some(admin_key.into());

    match ContractUpdateTransaction::new()
        .contract_id(contract_with_hooks_id)
        .add_hook(hook_with_id_3)
        .freeze_with(&client)?
        .sign(args.operator_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await
    {
        Ok(_) => println!("Successfully added hooks to contract!"),
        Err(error) => println!("Failed to execute hook transaction: {error}"),
    }

    /*
     * Step 4: Demonstrate hook deletion.
     */
    println!("\n=== Deleting Hooks from Contract ===");
    println!("Deleting hooks from contract...");

    match ContractUpdateTransaction::new()
        .contract_id(contract_with_hooks_id)
        .delete_hook(1) // Delete hook created during contract creation
        .delete_hook(3) // Delete hook added via contract update
        .freeze_with(&client)?
        .sign(args.operator_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await
    {
        Ok(_) => println!("Successfully deleted hooks with IDs: 1 and 3"),
        Err(error) => println!("Failed to execute hook deletion: {error}"),
    }

    println!("Contract Hooks Example Complete!");

    Ok(())
}
