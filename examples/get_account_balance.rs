// SPDX-License-Identifier: Apache-2.0

use hedera::{AccountBalanceQuery, AccountId, Client, NodeAddressBookQuery};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let client = Client::for_mainnet();
    let client = Client::for_testnet();
    dbg!(NodeAddressBookQuery::new()
        .execute(&client)
        .await?
        .node_addresses
        .into_iter()
        .map(|it| (it.node_account_id, it.service_endpoints))
        .collect::<Vec<_>>());

    let id = AccountId::from(7);

    let ab = AccountBalanceQuery::new()
        .account_id(id)
        // .node_account_ids([AccountId::from(7)])
        .execute(&client)
        .await?;

    println!("balance = {}", ab.hbars);

    Ok(())
}
