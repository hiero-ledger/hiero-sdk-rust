// SPDX-License-Identifier: Apache-2.0

use assert_matches::assert_matches;
use clap::Parser;
use hedera::{AccountId, Client, FileCreateTransaction, PrivateKey};

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

    let receipt = FileCreateTransaction::new()
        .contents(&b"Hedera Hashgraph is great!"[..])
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let new_file_id = assert_matches!(receipt.file_id, Some(id) => id);

    println!("file address = {new_file_id}");

    Ok(())
}
