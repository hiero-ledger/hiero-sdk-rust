// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hedera::{AccountId, Client, ExchangeRates, FileContentsQuery, FileId, PrivateKey};

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, env)]
    operator_account_id: AccountId,

    #[clap(long, env)]
    operator_key: PrivateKey,

    #[clap(long, env, default_value = "testnet")]
    hedera_network: String,

    #[clap(long, env, default_value_t = FileId::EXCHANGE_RATES)]
    hedera_exchange_rates_file: FileId,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let args = Args::parse();

    let client = Client::for_name(&args.hedera_network)?;

    client.set_operator(args.operator_account_id, args.operator_key.clone());

    let response = FileContentsQuery::new()
        .file_id(args.hedera_exchange_rates_file)
        .execute(&client)
        .await?;

    let ExchangeRates {
        current_rate,
        next_rate,
    } = ExchangeRates::from_bytes(&response.contents)?;

    println!("Current numerator: {}", current_rate.cents);
    println!("Current denominator: {}", current_rate.hbars);
    println!("Current expiration time: {}", current_rate.expiration_time);
    println!(
        "Current Exchange Rate: {}",
        current_rate.exchange_rate_in_cents()
    );

    println!("Next numerator: {}", next_rate.cents);
    println!("Next denominator: {}", next_rate.hbars);
    println!("Next expiration time: {}", next_rate.expiration_time);
    println!("Next Exchange Rate: {}", next_rate.exchange_rate_in_cents());

    Ok(())
}
