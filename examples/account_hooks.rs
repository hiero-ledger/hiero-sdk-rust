// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::Path;

use clap::Parser;
use hedera::{
    AccountCreateTransaction, AccountId, AccountUpdateTransaction, Client, ContractCreateTransaction, ContractId, EvmHookSpec, Hbar, HookCreationDetails, HookExtensionPoint, LambdaEvmHook, PrivateKey
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

    println!("Account Hooks Example Start!");

    /*
     * Step 1: Create the hook contract.
     */
    println!("Creating hook contract...");

    let contract_id = create_hook_contract(&client, &args.operator_key).await?;
    println!("Hook contract created with ID: {contract_id}");

    /*
     * Step 2: Demonstrate creating an account with hooks.
     */
    println!("\n=== Creating Account with Hooks ===");
    println!("Creating account with lambda EVM hook...");

    let account_key = PrivateKey::generate_ed25519();
    let account_public_key = account_key.public_key();

    // Create a lambda EVM hook
    let spec = EvmHookSpec::new(Some(contract_id));
    let lambda_hook = LambdaEvmHook::new(spec, vec![]);

    // Create hook creation details
    let admin_key = client.get_operator_public_key().unwrap();
    let mut hook_with_id_1002 = HookCreationDetails::new(
        HookExtensionPoint::AccountAllowanceHook,
        1002,
        Some(lambda_hook),
    );
    hook_with_id_1002.admin_key = Some(admin_key.into());

    let account_receipt = AccountCreateTransaction::new()
        .set_key_without_alias(account_public_key)
        .initial_balance(Hbar::new(1))
        .add_hook(hook_with_id_1002)
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
     * Step 3: Demonstrate adding hooks to an existing account.
     */
    println!("\n=== Adding Hooks to Existing Account ===");
    println!("Adding hooks to existing account...");

    // Create basic lambda hooks with no storage updates
    let spec1 = EvmHookSpec::new(Some(contract_id));
    let basic_hook = LambdaEvmHook::new(spec1, vec![]);
    let mut hook_with_id_1 = HookCreationDetails::new(
        HookExtensionPoint::AccountAllowanceHook,
        1,
        Some(basic_hook),
    );
    hook_with_id_1.admin_key = Some(admin_key.into());

    let spec2 = EvmHookSpec::new(Some(contract_id));
    let basic_hook2 = LambdaEvmHook::new(spec2, vec![]);
    let mut hook_with_id_2 = HookCreationDetails::new(
        HookExtensionPoint::AccountAllowanceHook,
        2,
        Some(basic_hook2),
    );
    hook_with_id_2.admin_key = Some(admin_key.into());

    match AccountUpdateTransaction::new()
        .account_id(account_id)
        .add_hook(hook_with_id_1)
        .add_hook(hook_with_id_2)
        .freeze_with(&client)?
        .sign(account_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await
    {
        Ok(_) => println!("Successfully added hooks to account!"),
        Err(error) => println!("Failed to execute hook transaction: {error}"),
    }

    /*
     * Step 4: Demonstrate hook deletion.
     */
    println!("\n=== Deleting Hooks from Account ===");
    println!("Deleting hooks from account...");

    match AccountUpdateTransaction::new()
        .account_id(account_id)
        .delete_hook(1)
        .delete_hook(2)
        .freeze_with(&client)?
        .sign(account_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await
    {
        Ok(_) => println!("Successfully deleted hooks (IDs: 1, 2)"),
        Err(error) => println!("Failed to execute hook deletion: {error}"),
    }

    println!("Account Hooks Example Complete!");

    Ok(())
}
