// SPDX-License-Identifier: Apache-2.0

use assert_matches::assert_matches;
use clap::Parser;
use hedera::{AccountCreateTransaction, AccountId, Client, PrivateKey};

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

    let new_key = PrivateKey::generate_ed25519();

    println!("private key = {new_key}");
    println!("public key = {}", new_key.public_key());

    let response = AccountCreateTransaction::new()
        .key(new_key.public_key())
        .execute(&client)
        .await?;

    let receipt = response.get_receipt(&client).await?;

    let new_account_id = assert_matches!(receipt.account_id, Some(id) => id);

    println!("account address = {new_account_id}");

    Ok(())
}
