// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hedera::{AccountId, Client, PrivateKey, PrngTransaction};

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

    client.set_operator(args.operator_account_id, args.operator_key);

    let record = PrngTransaction::new()
        .range(100)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    println!("generated random number = {:?}", record.prng_number);

    Ok(())
}
