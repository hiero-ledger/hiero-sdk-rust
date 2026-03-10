use std::iter::repeat;

use assert_matches::assert_matches;
use futures_util::stream::{
    self,
    TryStreamExt,
};
use futures_util::StreamExt;
use hiero_sdk::{
    Client,
    NftId,
    PrivateKey,
    Status,
    TokenCreateTransaction,
    TokenId,
    TokenInfoQuery,
    TokenMintTransaction,
    TokenNftInfoQuery,
    TokenType,
    TokenUpdateNftsTransaction,
};
use time::{
    Duration,
    OffsetDateTime,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

#[tokio::test]
async fn update_nft_metadata() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let metadata_key = PrivateKey::generate_ed25519();
    let nft_count = 4;
    let initial_metadata_list: Vec<Vec<u8>> = repeat(vec![9, 1, 6]).take(nft_count).collect();
    let updated_metadata: Vec<u8> = vec![3, 4];
    let updated_metadata_list: Vec<Vec<u8>> =
        repeat(updated_metadata.clone()).take(nft_count).collect();

    let token_id = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .token_type(TokenType::NonFungibleUnique)
        .treasury_account_id(client.get_operator_account_id().unwrap())
        .admin_key(client.get_operator_public_key().unwrap())
        .supply_key(client.get_operator_public_key().unwrap())
        .metadata_key(metadata_key.public_key())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    // Mint the token
    let receipt = TokenMintTransaction::new()
        .metadata(initial_metadata_list.clone())
        .token_id(token_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let nft_serials = receipt.serials;
    let metadata_list = get_metadata_list(&client, &token_id, &nft_serials).await?;

    assert_eq!(metadata_list, initial_metadata_list);

    _ = TokenUpdateNftsTransaction::new()
        .token_id(token_id)
        .serials(nft_serials.clone())
        .metadata(updated_metadata)
        .freeze_with(&client)?
        .sign(metadata_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let new_metadata_list = get_metadata_list(&client, &token_id, &nft_serials).await?;
    assert_eq!(new_metadata_list, updated_metadata_list);

    Ok(())
}

#[tokio::test]
async fn cannot_update_without_signed_metadata_key_error() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let supply_key = PrivateKey::generate_ed25519();
    let metadata_key = PrivateKey::generate_ed25519();
    let nft_count = 4;
    let initial_metadata_list: Vec<Vec<u8>> = repeat(vec![9, 1, 6]).take(nft_count).collect();
    let updated_metadata: Vec<u8> = vec![3, 4];

    // Create token with metadata key
    let token_id = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .token_type(TokenType::NonFungibleUnique)
        .treasury_account_id(client.get_operator_account_id().unwrap())
        .admin_key(client.get_operator_public_key().unwrap())
        .supply_key(supply_key.public_key())
        .metadata_key(metadata_key.public_key())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    let updated_info = TokenInfoQuery::new().token_id(token_id).execute(&client).await?;

    assert_eq!(updated_info.metadata_key.unwrap(), metadata_key.public_key().into());

    // Mint token
    let mint_receipt = TokenMintTransaction::new()
        .metadata(initial_metadata_list.clone())
        .token_id(token_id)
        .freeze_with(&client)?
        .sign(supply_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let nft_serials = mint_receipt.serials;

    // Update Nfts without signing with metadata key
    let res = TokenUpdateNftsTransaction::new()
        .token_id(token_id)
        .serials(nft_serials)
        .metadata(updated_metadata)
        .freeze_with(&client)?
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, .. })
    );

    Ok(())
}

#[tokio::test]
async fn cannot_update_without_set_metadata_key_error() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let supply_key = PrivateKey::generate_ed25519();
    let metadata_key = PrivateKey::generate_ed25519();
    let nft_count = 4;
    let initial_metadata_list: Vec<Vec<u8>> = repeat(vec![9, 1, 6]).take(nft_count).collect();
    let updated_metadata: Vec<u8> = vec![3, 4];

    // Create token without metadata key
    let token_id = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .token_type(TokenType::NonFungibleUnique)
        .treasury_account_id(client.get_operator_account_id().unwrap())
        .admin_key(client.get_operator_public_key().unwrap())
        .supply_key(supply_key.public_key())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    let token_info = TokenInfoQuery::new().token_id(token_id).execute(&client).await?;

    assert_eq!(token_info.metadata_key, None);

    // Mint Token
    let mint_receipt = TokenMintTransaction::new()
        .metadata(initial_metadata_list.clone())
        .token_id(token_id)
        .freeze_with(&client)?
        .sign(supply_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let nft_serials = mint_receipt.serials;

    // Update Nfts without a set metadata key
    let res = TokenUpdateNftsTransaction::new()
        .token_id(token_id)
        .serials(nft_serials)
        .metadata(updated_metadata)
        .freeze_with(&client)?
        .sign(metadata_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, .. })
    );

    Ok(())
}

async fn get_metadata_list(
    client: &Client,
    token_id: &TokenId,
    serials: &Vec<i64>,
) -> anyhow::Result<Vec<Vec<u8>>> {
    let list = stream::iter(
        serials.into_iter().map(|it| NftId { token_id: token_id.to_owned(), serial: *it as u64 }),
    )
    .then(|nft_id| {
        let client_clone = client;
        async move {
            match TokenNftInfoQuery::new().nft_id(nft_id).execute(&client_clone).await {
                Ok(info) => Ok(info.metadata),
                Err(err) => anyhow::bail!("error calling TokenNftInfoQuery: {err}"), // CHANGE ERROR MESSAGE
            }
        }
    })
    .try_collect::<Vec<_>>()
    .await?;

    Ok(list)
}
