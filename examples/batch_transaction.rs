// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hiero_sdk::{
    AccountBalanceQuery, AccountCreateTransaction, AccountId, BatchTransaction, Client, Hbar, PrivateKey, TransferTransaction
};

#[derive(Parser, Debug)]
struct Args {
    /// Operator account ID (can also be set via OPERATOR_ACCOUNT_ID environment variable)
    #[clap(long, env)]
    operator_account_id: AccountId,

    /// Operator private key (can also be set via OPERATOR_KEY environment variable)
    #[clap(long, env)]
    operator_key: PrivateKey,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file if present
    let _ = dotenvy::dotenv();
    let args = Args::parse();

    // Create client for testnet (you can also use mainnet or previewnet)
    let client = Client::for_testnet();

    // Set operator (the account that pays for transactions)
    client.set_operator(args.operator_account_id, args.operator_key.clone());

    println!("BatchTransaction Example");
    println!("========================");

    // Step 1: Create a batch key
    // This key will be used to sign the batch transaction itself.
    // IMPORTANT: The BatchTransaction MUST be signed with this key before execution!
    let batch_key = PrivateKey::generate_ecdsa();
    println!("Generated batch key: {}", batch_key.public_key());

    // Step 2: Create some accounts that will be involved in transfers
    let alice_key = PrivateKey::generate_ecdsa();
    let alice = create_account(&client, alice_key.public_key(), Hbar::new(5)).await?;
    println!("Created Alice account: {}", alice);

    let bob_key = PrivateKey::generate_ecdsa();
    let bob = create_account(&client, bob_key.public_key(), Hbar::new(3)).await?;
    println!("Created Bob account: {}", bob);

    // Step 3: Create individual transactions and prepare them for batching
    println!("\nPreparing batch transactions...");

    // Create a transfer from Alice to the operator
    let mut alice_transfer = TransferTransaction::new();
    alice_transfer
        .hbar_transfer(alice, Hbar::new(-1)) // Alice sends 1 HBAR
        .hbar_transfer(args.operator_account_id, Hbar::new(1)); // Operator receives 1 HBAR

    // Set Batch Key and Batchify the transaction
    client.set_operator(alice, alice_key.clone());
    alice_transfer.set_batch_key(batch_key.public_key().into());
    alice_transfer.batchify(&client, batch_key.public_key().into())?;

    // Create a transfer from Bob to the operator
    let mut bob_transfer = TransferTransaction::new();
    bob_transfer
        .hbar_transfer(bob, Hbar::new(-2)) // Bob sends 2 HBAR
        .hbar_transfer(args.operator_account_id, Hbar::new(2)); // Operator receives 2 HBAR

    // Set Batch Key and Batchify the transaction
    client.set_operator(bob, bob_key.clone());
    bob_transfer.set_batch_key(batch_key.public_key().into());
    bob_transfer.batchify(&client, batch_key.public_key().into())?;

    // Step 4: Get balances before batch execution
    println!("\nBalances before batch execution:");
    print_balance(&client, "Alice", alice).await?;
    print_balance(&client, "Bob", bob).await?;
    print_balance(&client, "Operator", args.operator_account_id).await?;

    // Step 5: Create and execute the batch transaction
    println!("\nExecuting batch transaction...");

    client.set_operator(args.operator_account_id, args.operator_key.clone());
    let mut batch = BatchTransaction::new();
    batch.add_inner_transaction(alice_transfer.into())?;
    batch.add_inner_transaction(bob_transfer.into())?;

    // Sign the batch transaction with the batch key (CRITICAL!)
    batch.sign(batch_key);

    // Execute the batch transaction
    let response = batch.execute(&client).await?;
    let receipt = response.get_receipt(&client).await?;

    println!("Batch transaction executed successfully!");
    println!("Transaction ID: {}", response.transaction_id);
    println!("Status: {:?}", receipt.status);

    // Step 6: Get balances after batch execution
    println!("\nBalances after batch execution:");
    print_balance(&client, "Alice", alice).await?;
    print_balance(&client, "Bob", bob).await?;
    print_balance(&client, "Operator", args.operator_account_id).await?;

    // Step 7: Get inner transaction IDs
    println!("\nInner transaction IDs:");
    for (i, tx_id) in batch.get_inner_transaction_ids().iter().enumerate() {
        if let Some(id) = tx_id {
            println!("Transaction {}: {}", i + 1, id);
        }
    }

    println!("\nBatchTransaction example completed successfully!");

    Ok(())
}

async fn create_account(
    client: &Client,
    public_key: hiero_sdk::PublicKey,
    initial_balance: Hbar,
) -> hiero_sdk::Result<AccountId> {
    let response = AccountCreateTransaction::new()
        .set_key_without_alias(public_key)
        .initial_balance(initial_balance)
        .execute(client)
        .await?;

    let receipt = response.get_receipt(client).await?;
    receipt.account_id.ok_or_else(|| {
        hiero_sdk::Error::TimedOut(Box::new(hiero_sdk::Error::GrpcStatus(
            tonic::Status::not_found("account_id not found in receipt"),
        )))
    })
}

async fn print_balance(
    client: &Client,
    name: &str,
    account_id: AccountId,
) -> hiero_sdk::Result<()> {
    let balance = AccountBalanceQuery::new()
        .account_id(account_id)
        .execute(client)
        .await?;
    println!("{}: {} HBAR", name, balance.hbars);
    Ok(())
}
