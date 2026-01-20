use assert_matches::assert_matches;
use hiero_sdk::{
    Hbar,
    Status,
    TokenAssociateTransaction,
    TokenBurnTransaction,
    TransferTransaction,
};

use crate::account::Account;
use crate::common::{
    setup_nonfree,
    TestEnvironment,
};
use crate::token::{
    CreateFungibleToken,
    Key,
    Nft,
    TokenKeys,
};

const KEYS: TokenKeys = TokenKeys { supply: Some(Key::Owner), ..TokenKeys::DEFAULT };

#[tokio::test]
async fn basic() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;

    let token = super::FungibleToken::create(
        &client,
        &account,
        CreateFungibleToken { initial_supply: 10, keys: KEYS },
    )
    .await?;

    let receipt = TokenBurnTransaction::new()
        .amount(10_u64)
        .token_id(token.id)
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    assert_eq!(receipt.total_supply, 0);

    token.delete(&client).await?;
    account.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn missing_token_id_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let res = TokenBurnTransaction::new().amount(10_u64).execute(&client).await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::TransactionPreCheckStatus { status: Status::InvalidTokenId, .. })
    );

    Ok(())
}

#[tokio::test]
async fn burn_zero() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;
    let token = super::FungibleToken::create(
        &client,
        &account,
        CreateFungibleToken { initial_supply: 0, keys: KEYS },
    )
    .await?;

    let receipt = TokenBurnTransaction::new()
        .token_id(token.id)
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    assert_eq!(receipt.total_supply, 0);

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
        CreateFungibleToken { initial_supply: 0, keys: KEYS },
    )
    .await?;

    let res = TokenBurnTransaction::new()
        .token_id(token.id)
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
async fn burn_nfts() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;
    let token = Nft::create(&client, &account).await?;

    let serials = token.mint_incremental(&client, 10).await?;

    // this is specifically what we're testing here.
    TokenBurnTransaction::new()
        .serials(serials)
        .token_id(token.id)
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    token.delete(&client).await?;
    account.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn unowned_nft_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let (alice, bob) = tokio::try_join!(
        Account::create(Hbar::new(0), &client),
        Account::create(Hbar::new(0), &client)
    )?;

    let token = Nft::create(&client, &alice).await?;
    let serials = token.mint_incremental(&client, 1).await?;

    let nft = token.id.nft(serials[0] as u64);

    TokenAssociateTransaction::new()
        .account_id(bob.id)
        .token_ids([token.id])
        .sign(bob.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    TransferTransaction::new()
        .nft_transfer(nft, alice.id, bob.id)
        .sign(alice.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let res = TokenBurnTransaction::new()
        .serials([nft.serial as i64])
        .token_id(token.id)
        .sign(alice.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::TreasuryMustOwnBurnedNft, .. })
    );

    TransferTransaction::new()
        .nft_transfer(nft, bob.id, alice.id)
        .sign(bob.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    token.burn(&client, [nft.serial as i64]).await?;
    token.delete(&client).await?;

    tokio::try_join!(alice.delete(&client), bob.delete(&client))?;

    Ok(())
}
