// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hedera::{
    AccountId, Client, KeyList, PrivateKey, PublicKey, TokenCreateTransaction, TokenInfoQuery, TokenKeyValidation, TokenUpdateTransaction
};
use time::{Duration, OffsetDateTime};

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, env)]
    operator_id: AccountId,

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

    client.set_operator(args.operator_id, args.operator_key.clone());

    // Generate a higher-privileged key.
    let admin_key = PrivateKey::generate_ed25519();

    // Generate the lower-privileged keys that will be modified.
    // Note: Lower-privileged keys are KYC, Freeze, Wipe, and Supply, Fee Schedule, Metadata key.
    let supply_key = PrivateKey::generate_ed25519();
    let wipe_key = PrivateKey::generate_ed25519();
    let new_supply_key = PrivateKey::generate_ed25519();

    // Generate an invalid key to update the supply key.
    let unusable = PublicKey::from_str_ed25519(
        "0x0000000000000000000000000000000000000000000000000000000000000000",
    )
    .unwrap();

    // Create the NFT token with admin, wipe, and supply keys.
    let token_id = TokenCreateTransaction::new()
        .name("Example NFT")
        .symbol("ENFT")
        .token_type(hedera::TokenType::NonFungibleUnique)
        .treasury_account_id(client.get_operator_account_id().unwrap())
        .admin_key(admin_key.public_key())
        .wipe_key(wipe_key.public_key())
        .supply_key(supply_key.public_key())
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .freeze_with(&client)?
        .sign(admin_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    let token_info = TokenInfoQuery::new()
        .token_id(token_id)
        .execute(&client)
        .await?;

    println!("Admin Key: {:?}", token_info.admin_key);
    println!("Supply Key: {:?}", token_info.supply_key);
    println!("Wipe Key: {:?}", token_info.wipe_key);

    println!("------");
    println!("Removing Wipe Key...");

    // Remove the wipe key with an empty key list, signing with the admin key.
    _ = TokenUpdateTransaction::new()
        .token_id(token_id)
        .wipe_key(KeyList::new())
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(admin_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let token_info = TokenInfoQuery::new()
        .token_id(token_id)
        .execute(&client)
        .await?;

    println!("Wipe Key (after removal): {:?}", token_info.wipe_key);

    println!("------");
    println!("Removing Admin Key...");

    // Remove the admin key with an empty key list, signing with the admin key.
    _ = TokenUpdateTransaction::new()
        .token_id(token_id)
        .admin_key(KeyList::new())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .freeze_with(&client)?
        .sign(admin_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let token_info = TokenInfoQuery::new()
        .token_id(token_id)
        .execute(&client)
        .await?;

    println!("Admin Key (after removal): {:?}", token_info.admin_key);

    println!("------");
    println!("Updating Supply Key...");

    // Update the supply key with a new key, signing with the old supply key and the new supply key.
    _ = TokenUpdateTransaction::new()
        .token_id(token_id)
        .supply_key(new_supply_key.public_key())
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(supply_key)
        .sign(new_supply_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let token_info = TokenInfoQuery::new()
        .token_id(token_id)
        .execute(&client)
        .await?;

    println!("Supply Key (after update): {:?}", token_info.supply_key);

    println!("------");
    println!("Removing Supply Key...");

    // Remove the supply key with an invalid key, signing with the new supply key.
    _ = TokenUpdateTransaction::new()
        .token_id(token_id)
        .supply_key(unusable)
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .freeze_with(&client)?
        .sign(new_supply_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let token_info = TokenInfoQuery::new()
        .token_id(token_id)
        .execute(&client)
        .await?;

    let supply_key = token_info.supply_key.unwrap();

    println!("Supply Key (after removal): {:?}", supply_key);

    Ok(())
}
