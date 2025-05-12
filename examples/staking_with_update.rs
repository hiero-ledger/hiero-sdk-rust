// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hedera::{
    AccountCreateTransaction, AccountId, AccountInfoQuery, AccountUpdateTransaction, Client, Hbar, PrivateKey
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

    client.set_operator(args.operator_account_id, args.operator_key.clone());

    // Create Alice account
    let new_key = PrivateKey::generate_ed25519();

    println!("private key: {new_key}");
    println!("public key: {}", new_key.public_key());

    // Create an account and stake to an acount ID
    // In this case we're staking to account ID 3 which happens to be
    // the account ID of node 0, we're only doing this as an example.
    // If you really want to stake to node 0, you should use
    // `.setStakedNodeId()` instead
    let new_account_id = AccountCreateTransaction::new()
        .key(new_key.public_key())
        .initial_balance(Hbar::new(10))
        .staked_account_id("0.0.3".parse()?)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    println!("new account ID: {new_account_id}");

    // Show the required key used to sign the account update transaction to
    // stake the accounts hbar i.e. the fee payer key and key to authorize
    // changes to the account should be different
    println!(
        "key required to update staking information: {}",
        new_key.public_key()
    );

    println!(
        "fee payer aka operator key: {}",
        args.operator_key.public_key()
    );

    // Query the account info, it should show the staked account ID
    // to be 0.0.3 just like what we set it to
    let info = AccountInfoQuery::new()
        .account_id(new_account_id)
        .execute(&client)
        .await?;

    println!("staking info: {:?}", info.staking);

    // Use the `AccountUpdateTransaction` to unstake the account's hbars
    //
    // If this succeeds then we should no longer have a staked account ID
    AccountUpdateTransaction::new()
        .account_id(new_account_id)
        .clear_staked_account_id()
        .sign(new_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Query the account info, it should show the staked account ID
    // to be `None` just like what we set it to
    let info = AccountInfoQuery::new()
        .account_id(new_account_id)
        .execute(&client)
        .await?;

    println!("staking info: {:?}", info.staking);

    Ok(())
}
