// SPDX-License-Identifier: Apache-2.0

use assert_matches::assert_matches;
use clap::Parser;
use hedera::{AccountCreateTransaction, AccountId, Client, Hbar, PrivateKey};

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
     * Step 0: Create and Configure the Client
     */
    let client =
        Client::for_mirror_network(vec!["testnet.mirrornode.hedera.com:443".to_owned()]).await?;

    // Set the operator account ID and key that will pay and sign all generated transactions.
    client.set_operator(args.operator_account_id, args.operator_key);

    /*
     * Step 1: Genereate ED25519 key pair
     */
    println!("Generating ED25519 key pair...");
    let private_key = PrivateKey::generate_ed25519();

    /*
     * Step 2: Create an account
     */
    let alice_id = AccountCreateTransaction::new()
        .key(private_key.public_key())
        .initial_balance(Hbar::new(5))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id;

    let alice_id = assert_matches!(alice_id, Some(id) => id);

    println!("Alice's ID = {alice_id}");

    Ok(())
}
