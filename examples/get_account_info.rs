// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hedera::{AccountId, AccountInfoQuery, Client, PrivateKey};

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, env)]
    operator_account_id: AccountId,

    #[clap(long, env)]
    operator_key: PrivateKey,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let args = Args::parse();

    let client = Client::for_testnet();

    client.set_operator(args.operator_account_id, args.operator_key);

    let id = AccountId::from(34_938_045);

    let info = AccountInfoQuery::new()
        .account_id(id)
        .execute(&client)
        .await?;

    println!("info = {info:#?}");

    Ok(())
}
