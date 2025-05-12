// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hedera::{
    AccountCreateTransaction, AccountId, AccountInfoQuery, AccountUpdateTransaction, Client, Hbar, Key, KeyList, PrivateKey, ScheduleInfoQuery, ScheduleSignTransaction, TransferTransaction
};
use time::{Duration, OffsetDateTime};
use tokio::time::sleep;

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

    /*
     * Step 0: Create and configure the client
     */
    let client = Client::for_name(&args.hedera_network)?;
    client.set_operator(args.operator_account_id, args.operator_key);

    /*
     * Step 1: Create key pairs
     */
    let key1 = PrivateKey::generate_ed25519();
    let key2 = PrivateKey::generate_ed25519();

    println!("Creating Key List... (w/ threshold, 2 of 2 keys generated above is required to modify the account)");

    let threshold_key = KeyList {
        keys: vec![key1.public_key().into(), key2.public_key().into()],
        threshold: Some(2),
    };

    println!("Created key list: {threshold_key:?}");

    /*
     * Step 2: Create the account
     */
    println!("Creating account with threshold key...");
    let alice_id = AccountCreateTransaction::new()
        .key(Key::KeyList(threshold_key))
        .initial_balance(Hbar::new(2))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    println!("Created account with id: {alice_id}");

    /*
     * Step 3:
     * Schedule a transfer transaction of 1 hbar from the newly created account to the operator account.
     * The transaction will be scheduled with expirationTime = 24 hours from now and waitForExpiry = false.
     */
    println!("Creating new scheduled transaction with 1 day expiry...");
    let mut transfer = TransferTransaction::new();
    transfer
        .hbar_transfer(alice_id, Hbar::new(-1))
        .hbar_transfer(args.operator_account_id, Hbar::new(1));

    let schedule_id = transfer
        .schedule()
        .wait_for_expiry(false)
        .expiration_time(OffsetDateTime::now_utc() + Duration::seconds(86400))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .schedule_id
        .unwrap();

    /*
     * Step 4: Sign the transaction with one key and verify the transaction is not executed
     */
    println!("Signing transaction with key 1...");
    _ = ScheduleSignTransaction::new()
        .schedule_id(schedule_id)
        .freeze_with(&client)?
        .sign(key1.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info = ScheduleInfoQuery::new()
        .schedule_id(schedule_id)
        .execute(&client)
        .await?;

    println!(
        "Scheduled transaction is not executed yet. Executed at: {:?}",
        info.executed_at
    );

    /*
     * Step 5: Sign the transaction with the second key and verify the transaction is executed
     */

    let account_balance = AccountInfoQuery::new()
        .account_id(alice_id)
        .execute(&client)
        .await?
        .balance;

    println!("Alice's account balance before scheduled transaction: {account_balance}");

    println!("Signing transaction with key 2...");
    _ = ScheduleSignTransaction::new()
        .schedule_id(schedule_id)
        .freeze_with(&client)?
        .sign(key2.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let account_balance = AccountInfoQuery::new()
        .account_id(alice_id)
        .execute(&client)
        .await?
        .balance;

    println!("Alice's account balance after scheduled transaction: {account_balance}");

    let info = ScheduleInfoQuery::new()
        .schedule_id(schedule_id)
        .execute(&client)
        .await?;

    println!("Scheduled transaction executed at: {:?}", info.executed_at);

    /*
     * Step 6:
     * Schedule another transfer transaction of 1 Hbar from the account to the operator account
     * with an expirationTime of 10 seconds in the future and waitForExpiry=true.
     */
    println!("Creating new scheduled transaction with 10 second expiry...");
    let mut transfer = TransferTransaction::new();
    transfer
        .hbar_transfer(alice_id, Hbar::new(-1))
        .hbar_transfer(args.operator_account_id, Hbar::new(1));

    let schedule_id = transfer
        .schedule()
        .wait_for_expiry(true)
        .expiration_time(OffsetDateTime::now_utc() + Duration::seconds(10))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .schedule_id
        .unwrap();

    /*
     * Step 7:
     * Sign the transaction with one key and verify the transaction is not executed
     */
    println!("Signing scheduled transaction with key 1...");
    _ = ScheduleSignTransaction::new()
        .schedule_id(schedule_id)
        .freeze_with(&client)?
        .sign(key1.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info = ScheduleInfoQuery::new()
        .schedule_id(schedule_id)
        .execute(&client)
        .await?;

    println!(
        "Scheduled transaction is not executed yet. Executed at: {:?}",
        info.executed_at
    );

    /*
     * Step 8:
     * Update the account's key to be only the one key
     * that has already signed the scheduled transfer.
     */
    println!("Updating account key to only key 1...");
    _ = AccountUpdateTransaction::new()
        .account_id(alice_id)
        .key(key1.public_key())
        .freeze_with(&client)?
        .sign(key1)
        .sign(key2)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    /*
     * Step 9:
     * Verify that the transfer successfully executes roughly at the time of its expiration.
     */
    let account_balance = AccountInfoQuery::new()
        .account_id(alice_id)
        .execute(&client)
        .await?
        .balance;

    println!("Alice's account balance before scheduled transfer: {account_balance}");

    sleep(std::time::Duration::from_millis(10_000)).await;

    let account_balance = AccountInfoQuery::new()
        .account_id(alice_id)
        .execute(&client)
        .await?
        .balance;

    println!("Alice's account balance after scheduled transfer: {account_balance}");

    println!("Successfully executed scheduled transfer");

    Ok(())
}
