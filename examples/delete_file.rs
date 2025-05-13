// SPDX-License-Identifier: Apache-2.0

use std::iter;

use clap::Parser;
use hedera::{AccountId, Client, FileCreateTransaction, FileDeleteTransaction, PrivateKey};

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

    let receipt = FileCreateTransaction::new()
        .contents(&b"Hiero is great!"[..])
        .keys(iter::once(args.operator_key.public_key()))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let new_file_id = receipt.file_id.unwrap();

    println!("file: {new_file_id}");

    FileDeleteTransaction::new()
        .file_id(new_file_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    println!("File deleted successfully");

    Ok(())
}
