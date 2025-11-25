// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::Path;

use clap::Parser;
use hedera::{
    AccountCreateTransaction, AccountId, Client, ContractCreateTransaction, ContractId, EvmHookSpec, Hbar, HookCreationDetails, HookEntityId, HookExtensionPoint, HookId, LambdaEvmHook, LambdaSStoreTransaction, LambdaStorageSlot, LambdaStorageUpdate, PrivateKey
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

    println!("Lambda SStore Example Start!");

    /*
     * Step 1: Set up prerequisites - create hook contract and account with lambda hook
     */
    println!("Setting up prerequisites...");

    // Create the hook contract
    println!("Creating hook contract...");
    let contract_id = create_hook_contract(&client, &args.operator_key).await?;
    println!("Hook contract created with ID: {contract_id}");

    // Create account with lambda hook
    println!("Creating account with lambda hook...");
    let account_key = PrivateKey::generate_ed25519();
    let account_public_key = account_key.public_key();

    // Create a lambda EVM hook
    let spec = EvmHookSpec::new(Some(contract_id));
    let lambda_hook = LambdaEvmHook::new(spec, vec![]);

    // Create hook creation details
    let admin_key = client.get_operator_public_key().unwrap();
    let mut hook_details = HookCreationDetails::new(
        HookExtensionPoint::AccountAllowanceHook,
        1,
        Some(lambda_hook),
    );
    hook_details.admin_key = Some(admin_key.into());

    let account_receipt = AccountCreateTransaction::new()
        .set_key_without_alias(account_public_key)
        .initial_balance(Hbar::new(1))
        .add_hook(hook_details)
        .freeze_with(&client)?
        .sign(args.operator_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let account_id = account_receipt.account_id.unwrap();
    println!("account id = {account_id}");
    println!("Successfully created account with lambda hook!");

    /*
     * Step 2: Demonstrate LambdaSStoreTransaction - the core functionality.
     */
    println!("\n=== LambdaSStoreTransaction Example ===");

    // Create storage key (1 byte filled with value 1)
    let mut storage_key = vec![0u8; 1];
    storage_key.fill(1);

    // Create storage value (32 bytes filled with value 200)
    let mut storage_value = vec![0u8; 32];
    storage_value.fill(200);

    let storage_slot = LambdaStorageSlot::new(storage_key.clone(), storage_value.clone());
    let storage_update = LambdaStorageUpdate::StorageSlot(storage_slot);

    // Create HookId for the existing hook (accountId with hook ID 1)
    let hook_entity_id = HookEntityId::new(Some(account_id));
    let hook_id = HookId::new(Some(hook_entity_id.clone()), 1);

    println!("Storage update created:");
    println!("  Storage Key: {:?}", storage_key);
    println!("  Storage Value: {:?}", storage_value);
    println!("  Hook ID: {}", hook_id.hook_id);
    if let Some(entity_id) = &hook_id.entity_id {
        if let Some(acc_id) = &entity_id.account_id {
            println!("  Hook Entity ID: {acc_id}");
        }
    }

    // Execute LambdaSStoreTransaction
    println!("Executing LambdaSStoreTransaction...");
    let lambda_store_receipt = LambdaSStoreTransaction::new()
        .set_hook_id(hook_id)
        .add_storage_update(storage_update)
        .freeze_with(&client)?
        .sign(account_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    println!("Successfully updated lambda hook storage!");
    println!("Transaction completed successfully!");
    println!("Receipt status: {:?}", lambda_store_receipt.status);

    println!("\nLambda SStore Example Complete!");

    Ok(())
}
