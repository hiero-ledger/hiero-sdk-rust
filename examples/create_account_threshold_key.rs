// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hedera::{
    AccountBalanceQuery, AccountCreateTransaction, AccountId, Client, Hbar, Key, KeyList, PrivateKey, TransferTransaction
};

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

    // Generate three Ed25519::new private, public key pairs.
    // You do not need the private keys to create the Threshold Key List,
    // you only need the public keys, and if you're doing things correctly,
    // you probably shouldn't have these private keys.
    let private_keys = [
        PrivateKey::generate_ed25519(),
        PrivateKey::generate_ed25519(),
        PrivateKey::generate_ed25519(),
    ];

    println!("public keys:");
    for public_key in private_keys.iter().map(PrivateKey::public_key) {
        println!("{public_key}");
    }

    // require 2 of the 3 keys we generated to sign on anything modifying this account
    let transaction_key = KeyList {
        keys: private_keys
            .iter()
            .map(PrivateKey::public_key)
            .map(Key::from)
            .collect(),
        threshold: Some(2),
    };

    let transaction_response = AccountCreateTransaction::new()
        .key(transaction_key)
        .initial_balance(Hbar::new(10))
        .execute(&client)
        .await?;

    // This will wait for the receipt to become available
    let receipt = transaction_response.get_receipt(&client).await?;

    let new_account_id = receipt.account_id.unwrap();

    println!("account = {new_account_id}");

    let transfer_transaction_response = TransferTransaction::new()
        .hbar_transfer(new_account_id, Hbar::new(10).negated())
        .hbar_transfer(AccountId::from(3), Hbar::new(10))
        // we sign with 2 of the 3 keys
        .sign(private_keys[0].clone())
        .sign(private_keys[1].clone())
        .execute(&client)
        .await?;

    // (important!) wait for the transfer to go to consensus
    transfer_transaction_response.get_receipt(&client).await?;

    let balance_after = AccountBalanceQuery::new()
        .account_id(new_account_id)
        .execute(&client)
        .await?
        .hbars;

    println!("account balance after transfer: {balance_after}");

    Ok(())
}
