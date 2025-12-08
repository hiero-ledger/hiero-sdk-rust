use assert_matches::assert_matches;
use hiero_sdk::{
    Hbar,
    Status,
    TokenCreateTransaction,
    TokenMintTransaction,
    TokenSupplyType,
    TokenType,
};
use time::{
    Duration,
    OffsetDateTime,
};

use crate::account::Account;
use crate::common::{
    setup_nonfree,
    TestEnvironment,
};
use crate::token::{
    CreateFungibleToken,
    FungibleToken,
    Key,
    Nft,
    TokenKeys,
};

const TOKEN_KEYS: TokenKeys = TokenKeys { supply: Some(Key::Owner), ..TokenKeys::DEFAULT };

#[tokio::test]
async fn basic() -> anyhow::Result<()> {
    const INITIAL_SUPPLY: u64 = 1_000_000;
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;
    let token = super::FungibleToken::create(
        &client,
        &account,
        CreateFungibleToken { initial_supply: INITIAL_SUPPLY, keys: TOKEN_KEYS },
    )
    .await?;

    let receipt = TokenMintTransaction::new()
        .amount(10)
        .token_id(token.id)
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    assert_eq!(receipt.total_supply, INITIAL_SUPPLY + 10);

    token.burn(&client, INITIAL_SUPPLY + 10).await?;
    token.delete(&client).await?;
    account.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn over_supply_limit_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;

    let token_id = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .token_supply_type(hiero_sdk::TokenSupplyType::Finite)
        .max_supply(5)
        .treasury_account_id(account.id)
        .admin_key(account.key.public_key())
        .supply_key(account.key.public_key())
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    let token = FungibleToken { id: token_id, owner: account.clone() };

    let res = TokenMintTransaction::new()
        .token_id(token.id)
        .amount(6)
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::TokenMaxSupplyReached, .. })
    );

    token.delete(&client).await?;
    account.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn missing_token_id_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let res = TokenMintTransaction::new().amount(6).execute(&client).await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::TransactionPreCheckStatus { status: Status::InvalidTokenId, .. })
    );

    Ok(())
}

#[tokio::test]
async fn zero() -> anyhow::Result<()> {
    const INITIAL_SUPPLY: u64 = 1_000_000;
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;
    let token = super::FungibleToken::create(
        &client,
        &account,
        CreateFungibleToken { initial_supply: INITIAL_SUPPLY, keys: TOKEN_KEYS },
    )
    .await?;

    let receipt = TokenMintTransaction::new()
        .token_id(token.id)
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    assert_eq!(receipt.total_supply, INITIAL_SUPPLY);

    token.burn(&client, INITIAL_SUPPLY).await?;
    token.delete(&client).await?;
    account.delete(&client).await?;

    Ok(())
}

#[tokio::test]

async fn missing_supply_key_sig_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;
    let token = super::FungibleToken::create(
        &client,
        &account,
        CreateFungibleToken { initial_supply: 0, keys: TOKEN_KEYS },
    )
    .await?;

    let res = TokenMintTransaction::new()
        .token_id(token.id)
        .amount(10)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, .. })
    );

    token.delete(&client).await?;
    account.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn nfts() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;

    let token_id = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .treasury_account_id(account.id)
        .admin_key(account.key.public_key())
        .supply_key(account.key.public_key())
        .token_type(TokenType::NonFungibleUnique)
        .token_supply_type(TokenSupplyType::Finite)
        .max_supply(5000)
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    let token = Nft { id: token_id, owner: account.clone() };

    let mint_receipt = TokenMintTransaction::new()
        .token_id(token_id)
        .metadata((0..10).map(|it| [it]))
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    assert_eq!(mint_receipt.serials.len(), 10);

    token.burn(&client, mint_receipt.serials).await?;
    token.delete(&client).await?;

    account.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn nft_metadata_too_long_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;

    let token_id = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .treasury_account_id(account.id)
        .admin_key(account.key.public_key())
        .supply_key(account.key.public_key())
        .token_type(TokenType::NonFungibleUnique)
        .token_supply_type(TokenSupplyType::Finite)
        .max_supply(5000)
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    let token = Nft { id: token_id, owner: account.clone() };

    let res = TokenMintTransaction::new()
        .token_id(token_id)
        .metadata([[1; 101]])
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::MetadataTooLong, .. })
    );

    token.delete(&client).await?;
    account.delete(&client).await?;

    Ok(())
}
