// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hedera::{
    AccountCreateTransaction, AccountId, Client, Hbar, KeyList, PrivateKey, ScheduleInfoQuery, ScheduleSignTransaction, TransactionId, TransferTransaction
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

    // Generate 3 random keys
    let key1 = PrivateKey::generate_ed25519();
    let key2 = PrivateKey::generate_ed25519();
    let key3 = PrivateKey::generate_ed25519();

    // Create a keylist from those keys. This key will be used as the new account's key
    // The reason we want to use a `KeyList` is to simulate a multi-party system where
    // multiple keys are required to sign.
    let key_list = KeyList::from([key1.public_key(), key2.public_key(), key3.public_key()]);

    println!("key1 private = {key1}");
    println!("key1 public = {}", key1.public_key());
    println!("key1 private = {key2}");
    println!("key2 public = {}", key2.public_key());
    println!("key1 private = {key3}");
    println!("key3 public = {}", key3.public_key());
    println!("key_list = {key_list:?}");

    // Creat the account with the `KeyList`
    // The only _required_ property here is `key`
    let response = AccountCreateTransaction::new()
        .node_account_ids([AccountId::from(3)])
        .key(key_list)
        .initial_balance(Hbar::new(10))
        .execute(&client)
        .await?;

    // This will wait for the receipt to become available
    let receipt = response.get_receipt(&client).await?;

    let account_id = receipt.account_id.unwrap();

    println!("accountId = {account_id}");

    // Generate a `TransactionId`. This id is used to query the inner scheduled transaction
    // after we expect it to have been executed
    let transaction_id = TransactionId::generate(args.operator_account_id);

    println!("transaction_id for scheduled transaction = {transaction_id}");

    // Create a transfer transaction with 2/3 signatures.
    let mut transfer = TransferTransaction::new();

    transfer
        .hbar_transfer(account_id, -Hbar::new(1))
        .hbar_transfer(args.operator_account_id, Hbar::new(1));

    // Schedule the transaction
    let receipt = transfer
        .schedule()
        .payer_account_id(args.operator_account_id)
        .admin_key(args.operator_key.public_key())
        .freeze_with(&client)?
        .sign(key2.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Get the schedule ID from the receipt
    let schedule_id = receipt.schedule_id.unwrap();

    println!("schedule_id = {schedule_id}");

    // Get the schedule info to see if `signatories` is populated with 2/3 signatures
    let info = ScheduleInfoQuery::new()
        .node_account_ids([response.node_account_id])
        .schedule_id(schedule_id)
        .execute(&client)
        .await?;

    println!("Schedule Info = {:?}", info);

    let transfer = info
        .scheduled_transaction()?
        .downcast::<TransferTransaction>()
        .unwrap();

    let transfers = transfer.get_hbar_transfers();

    // Make sure the transfer transaction is what we expect
    anyhow::ensure!(transfers.len() == 2, "more transfers than expected");

    anyhow::ensure!(transfers[&account_id] == -Hbar::new(1));
    anyhow::ensure!(transfers[&args.operator_account_id] == Hbar::new(1));

    println!("sending schedule sign transaction");

    // Finally send this last signature to Hedera. This last signature _should_ mean the transaction executes
    // since all 3 signatures have been provided.
    ScheduleSignTransaction::new()
        .node_account_ids([response.node_account_id])
        .schedule_id(schedule_id)
        .freeze_with(&client)?
        .sign(key3)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Query the schedule info again
    ScheduleInfoQuery::new()
        .node_account_ids([response.node_account_id])
        .schedule_id(schedule_id)
        .execute(&client)
        .await?;

    Ok(())
}
