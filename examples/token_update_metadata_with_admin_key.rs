// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hedera::{
    AccountId, Client, PrivateKey, TokenCreateTransaction, TokenInfoQuery, TokenType, TokenUpdateTransaction
};
use time::{Duration, OffsetDateTime};

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

    let admin_key = PrivateKey::generate_ed25519();

    // Initial metadata
    let metadata: Vec<u8> = vec![1];
    // New metadata
    let new_metadata: Vec<u8> = vec![1, 2];

    let token_create_receipt = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .token_type(TokenType::FungibleCommon)
        .decimals(3)
        .initial_supply(1000000)
        .treasury_account_id(client.get_operator_account_id().unwrap())
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .admin_key(admin_key.public_key())
        .metadata(metadata)
        .freeze_with(&client)?
        .sign(admin_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Get token id
    let token_id = token_create_receipt.token_id.unwrap();
    println!("Created a mutable token: {token_id:?}");

    let token_info = TokenInfoQuery::new()
        .token_id(token_id)
        .execute(&client)
        .await?;

    println!(
        "Immutable token's metadata after creation: {:?}",
        token_info.metadata
    );

    let token_update_receipt = TokenUpdateTransaction::new()
        .token_id(token_id)
        .metadata(new_metadata)
        .freeze_with(&client)?
        .sign(admin_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    println!(
        "Status of token update transaction: {:?}",
        token_update_receipt.status
    );

    let token_nft_info = TokenInfoQuery::new()
        .token_id(token_id)
        .execute(&client)
        .await?;

    println!(
        "Immutable token's metadata after update: {:?}",
        token_nft_info.metadata
    );

    Ok(())
}
