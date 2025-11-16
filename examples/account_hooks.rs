// SPDX-License-Identifier: Apache-2.0

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
