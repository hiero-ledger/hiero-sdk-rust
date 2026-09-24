// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hiero_sdk::{AccountId, AccountInfoQuery, Client, NodeAddressBookQuery, PrivateKey};

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

    // let client = Client::for_mainnet();
    let client = Client::for_testnet();

    // `AccountBalanceQuery` is no longer supported; `AccountInfoQuery` returns the balance
    // and is paid for by the operator.
    client.set_operator(args.operator_account_id, args.operator_key);

    dbg!(NodeAddressBookQuery::new()
        .execute(&client)
        .await?
        .node_addresses
        .into_iter()
        .map(|it| (it.node_account_id, it.service_endpoints))
        .collect::<Vec<_>>());

    let id = AccountId::from(7);

    let info = AccountInfoQuery::new()
        .account_id(id)
        // .node_account_ids([AccountId::from(7)])
        .execute(&client)
        .await?;

    println!("balance = {}", info.balance);

    Ok(())
}
