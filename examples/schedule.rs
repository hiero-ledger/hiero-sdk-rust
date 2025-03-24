// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hedera::{
    AccountCreateTransaction, AccountId, Client, Hbar, KeyList, PrivateKey, ScheduleInfoQuery, ScheduleSignTransaction, TransferTransaction
};
use time::OffsetDateTime;

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

    // Generate a Ed25519 private, public key pair
    let key1 = PrivateKey::generate_ed25519();
    let key2 = PrivateKey::generate_ed25519();

    println!("private key 1 = {key1}");
    println!("public key 1 = {}", key1.public_key());
    println!("private key 2 = {key2}");
    println!("public key 2 = {}", key2.public_key());

    let new_account_id = AccountCreateTransaction::new()
        .key(KeyList::from([key1.public_key(), key2.public_key()]))
        .initial_balance(Hbar::from_tinybars(1000))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    println!("new account ID: {new_account_id}");

    let mut tx = TransferTransaction::new();

    tx.hbar_transfer(new_account_id, -Hbar::new(1))
        .hbar_transfer(args.operator_account_id, Hbar::new(1));

    let response = tx
        .schedule()
        .expiration_time(OffsetDateTime::now_utc() + time::Duration::days(1))
        .wait_for_expiry(true)
        .execute(&client)
        .await?;

    println!("scheduled transaction ID = {}", response.transaction_id);

    let schedule_id = response.get_receipt(&client).await?.schedule_id.unwrap();
    println!("schedule ID = {schedule_id}");

    let record = response.get_record(&client).await?;
    println!("record = {record:?}");

    ScheduleSignTransaction::new()
        .schedule_id(schedule_id)
        .freeze_with(&client)?
        .sign(key1)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info = ScheduleInfoQuery::new()
        .schedule_id(schedule_id)
        .execute(&client)
        .await?;

    println!("schedule info = {info:?}");

    ScheduleSignTransaction::new()
        .schedule_id(schedule_id)
        .freeze_with(&client)?
        .sign(key2)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let transaction_id = response.transaction_id;

    println!("The following link should query the mirror node for the scheduled transaction:");

    println!(
        "https://{}.mirrornode.hedera.com/api/v1/transactions/{}",
        args.hedera_network,
        format_args!(
            "{}-{}-{}",
            transaction_id.account_id,
            transaction_id.valid_start.unix_timestamp(),
            transaction_id.valid_start.nanosecond()
        )
    );

    Ok(())
}
