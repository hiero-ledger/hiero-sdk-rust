// SPDX-License-Identifier: Apache-2.0

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

// Hook contract bytecode
// For a real example, you'd read this from a file
const HOOK_BYTECODE: &str = "6080604052348015600e575f5ffd5b506103da8061001c5f395ff3fe60806040526004361061001d575f3560e01c80630b6c5c0414610021575b5f5ffd5b61003b6004803603810190610036919061021c565b610051565b60405161004891906102ed565b60405180910390f35b5f61016d73ffffffffffffffffffffffffffffffffffffffff163073ffffffffffffffffffffffffffffffffffffffff16146100c2576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016100b990610386565b60405180910390fd5b60019050979650505050505050565b5f5ffd5b5f5ffd5b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f610102826100d9565b9050919050565b610112816100f8565b811461011c575f5ffd5b50565b5f8135905061012d81610109565b92915050565b5f819050919050565b61014581610133565b811461014f575f5ffd5b50565b5f813590506101608161013c565b92915050565b5f5ffd5b5f5ffd5b5f5ffd5b5f5f83601f84011261018757610186610166565b5b8235905067ffffffffffffffff8111156101a4576101a361016a565b5b6020830191508360018202830111156101c0576101bf61016e565b5b9250929050565b5f5f83601f8401126101dc576101db610166565b5b8235905067ffffffffffffffff8111156101f9576101f861016a565b5b6020830191508360018202830111156102155761021461016e565b5b9250929050565b5f5f5f5f5f5f5f60a0888a031215610237576102366100d1565b5b5f6102448a828b0161011f565b97505060206102558a828b01610152565b96505060406102668a828b01610152565b955050606088013567ffffffffffffffff811115610287576102866100d5565b5b6102938a828b01610172565b9450945050608088013567ffffffffffffffff8111156102b6576102b56100d5565b5b6102c28a828b016101c7565b925092505092959891949750929550565b5f8115159050919050565b6102e7816102d3565b82525050565b5f6020820190506103005f8301846102de565b92915050565b5f82825260208201905092915050565b7f436f6e74726163742063616e206f6e6c792062652063616c6c656420617320615f8201527f20686f6f6b000000000000000000000000000000000000000000000000000000602082015250565b5f610370602583610306565b915061037b82610316565b604082019050919050565b5f6020820190508181035f83015261039d81610364565b905091905056fea2646970667358221220a8c76458204f8bb9a86f59ec2f0ccb7cbe8ae4dcb65700c4b6ee91a39404083a64736f6c634300081e0033";

async fn create_hook_contract(
    client: &Client,
    operator_key: &PrivateKey,
) -> anyhow::Result<ContractId> {
    let bytecode = hex::decode(HOOK_BYTECODE)?;

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
        .hook_id(hook_id)
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
