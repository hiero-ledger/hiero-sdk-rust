// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use hedera::{
    AccountBalanceQuery, AccountCreateTransaction, AccountId, BatchTransaction, Client, Hbar, PrivateKey, TransferTransaction
};

#[tokio::main]
async fn main() -> hedera::Result<()> {
    // Create client for testnet (you can also use mainnet or previewnet)
    let client = Client::for_testnet();

    // Set operator (the account that pays for transactions)
    let operator_key = PrivateKey::from_str_ed25519(
        "302e020100300506032b657004220420a869f4c6191b9c8c99933e7f6b6611711737e4b1a1a5a4cb5370e719a1f6df98"
    )?;
    let operator_account = AccountId::from_str("0.0.1001")?;
    client.set_operator(operator_account, operator_key);

    println!("BatchTransaction Example");
    println!("========================");

    // Step 1: Create a batch key
    // This key will be used to sign the batch transaction itself
    let batch_key = PrivateKey::generate_ed25519();
    println!("Generated batch key: {}", batch_key.public_key());

    // Step 2: Create some accounts that will be involved in transfers
    let alice_key = PrivateKey::generate_ed25519();
    let alice = create_account(&client, alice_key.public_key(), Hbar::new(5)).await?;
    println!("Created Alice account: {}", alice);

    let bob_key = PrivateKey::generate_ed25519();
    let bob = create_account(&client, bob_key.public_key(), Hbar::new(3)).await?;
    println!("Created Bob account: {}", bob);

    // Step 3: Create individual transactions and prepare them for batching
    println!("\nPreparing batch transactions...");

    // Create a transfer from Alice to the operator
    let mut alice_transfer = TransferTransaction::new();
    alice_transfer
        .hbar_transfer(alice, Hbar::new(-1)) // Alice sends 1 HBAR
        .hbar_transfer(operator_account, Hbar::new(1)); // Operator receives 1 HBAR

    // Freeze the transaction and set batch key
    alice_transfer.freeze_with(&client)?;
    alice_transfer.set_batch_key(batch_key.public_key().into());
    alice_transfer.sign(alice_key.clone());

    // Create a transfer from Bob to the operator
    let mut bob_transfer = TransferTransaction::new();
    bob_transfer
        .hbar_transfer(bob, Hbar::new(-2)) // Bob sends 2 HBAR
        .hbar_transfer(operator_account, Hbar::new(2)); // Operator receives 2 HBAR

    // Freeze the transaction and set batch key
    bob_transfer.freeze_with(&client)?;
    bob_transfer.set_batch_key(batch_key.public_key().into());
    bob_transfer.sign(bob_key.clone());

    // Step 4: Get balances before batch execution
    println!("\nBalances before batch execution:");
    print_balance(&client, "Alice", alice).await?;
    print_balance(&client, "Bob", bob).await?;
    print_balance(&client, "Operator", operator_account).await?;

    // Step 5: Create and execute the batch transaction
    println!("\nExecuting batch transaction...");

    let mut batch = BatchTransaction::new();
    batch.add_inner_transaction(alice_transfer.into())?;
    batch.add_inner_transaction(bob_transfer.into())?;
    batch.freeze_with(&client)?;
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
    print_balance(&client, "Operator", operator_account).await?;

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
    public_key: hedera::PublicKey,
    initial_balance: Hbar,
) -> hedera::Result<AccountId> {
    let response = AccountCreateTransaction::new()
        .set_key_without_alias(public_key)
        .initial_balance(initial_balance)
        .execute(client)
        .await?;

    let receipt = response.get_receipt(client).await?;
    receipt.account_id.ok_or_else(|| {
        hedera::Error::TimedOut(Box::new(hedera::Error::GrpcStatus(
            tonic::Status::not_found("account_id not found in receipt"),
        )))
    })
}

async fn print_balance(client: &Client, name: &str, account_id: AccountId) -> hedera::Result<()> {
    let balance = AccountBalanceQuery::new()
        .account_id(account_id)
        .execute(client)
        .await?;
    println!("{}: {} HBAR", name, balance.hbars);
    Ok(())
}
