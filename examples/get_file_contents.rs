// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hedera::{AccountId, Client, FileContentsQuery, FileId, PrivateKey};

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, env)]
    operator_account_id: AccountId,

    #[clap(long, env)]
    operator_key: PrivateKey,

    #[clap(long, env, default_value = "0.0.34945328")]
    file: FileId,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let args = Args::parse();

    let client = Client::for_testnet();

    client.set_operator(args.operator_account_id, args.operator_key);

    let cr = FileContentsQuery::new()
        .file_id(args.file)
        .execute(&client)
        .await?;

    let contents = String::from_utf8(cr.contents)?;

    println!("contents: {contents}");

    Ok(())
}
