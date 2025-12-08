// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hiero_sdk::{AccountId, Client, FileId, NodeAddressBookQuery, PrivateKey};

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, env)]
    operator_account_id: AccountId,

    #[clap(long, env)]
    operator_key: PrivateKey,

    #[clap(long, env, default_value = "testnet")]
    hedera_network: String,

    #[clap(long, env, default_value_t = FileId::get_exchange_rates_file_id_for(0, 0))]
    hedera_exchange_rates_file: FileId,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let args = Args::parse();

    println!("Getting address book for shard 0, realm 0");
    let client = Client::for_mirror_network_with_shard_realm(
        vec!["testnet.mirrornode.hedera.com:443".to_string()],
        0,
        0,
    )
    .await?;

    client.set_operator(args.operator_account_id, args.operator_key.clone());

    println!("Getting address book for shard 0, realm 0");
    let response = NodeAddressBookQuery::new()
        .shard(0)
        .realm(0)
        .execute(&client)
        .await?;

    for node in response.node_addresses {
        println!("{}", node.node_id);
    }

    Ok(())
}
