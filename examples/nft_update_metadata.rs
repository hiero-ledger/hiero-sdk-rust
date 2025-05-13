// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hedera::{
    AccountCreateTransaction, AccountId, Client, Hbar, NftId, PrivateKey, TokenCreateTransaction, TokenInfoQuery, TokenMintTransaction, TokenNftInfoQuery, TokenType, TokenUpdateNftsTransaction, TransferTransaction
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

    // Generate a supply key
    let supply_key = PrivateKey::generate_ed25519();
    // Generate a metadata key
    let metadata_key = PrivateKey::generate_ed25519();
    // Initial metadata
    let metadata: Vec<u8> = vec![1];
    // New metadata
    let new_metadata: Vec<u8> = vec![1, 2];

    let token_create_receipt = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .token_type(TokenType::NonFungibleUnique)
        .treasury_account_id(client.get_operator_account_id().unwrap())
        .supply_key(client.get_operator_public_key().unwrap())
        .metadata_key(metadata_key.public_key())
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .sign(args.operator_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let token_id = token_create_receipt.token_id.unwrap();

    println!("Token id: {token_id:?}");

    let token_info = TokenInfoQuery::new()
        .token_id(token_id)
        .execute(&client)
        .await?;

    println!("Token metadata key: {:?}", token_info.metadata_key);

    // Mint the token
    let token_mint_receipt = TokenMintTransaction::new()
        .token_id(token_id)
        .metadata([metadata])
        .freeze_with(&client)?
        .sign(supply_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    println!(
        "Status of token mint transaction: {:?}",
        token_create_receipt.status
    );

    let nft_serial = *token_mint_receipt.serials.first().unwrap() as u64;

    let nft_id = NftId {
        token_id,
        serial: nft_serial,
    };

    let token_nfts_info = TokenNftInfoQuery::new()
        .nft_id(nft_id)
        .execute(&client)
        .await?;

    println!("Set token NFT metadata: {:?}", token_nfts_info.metadata);

    let account_id = AccountCreateTransaction::new()
        .key(client.get_operator_public_key().unwrap())
        .max_automatic_token_associations(10)
        .initial_balance(Hbar::new(100))
        .freeze_with(&client)?
        .sign(args.operator_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    println!("New Account id: {account_id:?}");

    let transfer_nft_tx = TransferTransaction::new()
        .nft_transfer(
            nft_id,
            client.get_operator_account_id().unwrap(),
            account_id,
        )
        .freeze_with(&client)?
        .sign(args.operator_key.clone())
        .execute(&client)
        .await?;

    let transfer_nft_response = transfer_nft_tx.get_receipt(&client).await?;

    println!(
        "Status of transfer NFT transaction: {:?}",
        transfer_nft_response.status
    );

    let token_update_nfts_receipt = TokenUpdateNftsTransaction::new()
        .token_id(token_id)
        .serials(vec![nft_serial as i64])
        .metadata(new_metadata)
        .freeze_with(&client)?
        .sign(metadata_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    println!(
        "Status of token update NFT transaction: {:?}",
        token_update_nfts_receipt.status
    );

    let token_nft_info = TokenNftInfoQuery::new()
        .nft_id(nft_id)
        .execute(&client)
        .await?;

    println!("Updated token NFT metadata: {:?}", token_nft_info.metadata);

    Ok(())
}
