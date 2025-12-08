use assert_matches::assert_matches;
use hiero_sdk::{
    Hbar,
    Status,
    TokenAssociateTransaction,
    TokenGrantKycTransaction,
};

use crate::account::Account;
use crate::common::{
    setup_nonfree,
    TestEnvironment,
};
use crate::token::{
    CreateFungibleToken,
    Key,
    TokenKeys,
};

const TOKEN_PARAMS: CreateFungibleToken = CreateFungibleToken {
    initial_supply: 0,
    keys: TokenKeys { kyc: Some(Key::Owner), ..TokenKeys::DEFAULT },
};

#[tokio::test]
async fn basic() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let (alice, bob) = tokio::try_join!(
        Account::create(Hbar::new(0), &client),
        Account::create(Hbar::new(0), &client)
    )?;

    let token = super::FungibleToken::create(&client, &alice, TOKEN_PARAMS).await?;

    TokenAssociateTransaction::new()
        .account_id(bob.id)
        .token_ids([token.id])
        .sign(bob.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    TokenGrantKycTransaction::new()
        .account_id(bob.id)
        .token_id(token.id)
        .sign(alice.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    token.delete(&client).await?;

    tokio::try_join!(alice.delete(&client), bob.delete(&client))?;

    Ok(())
}

#[tokio::test]
async fn missing_token_id_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;

    let res = TokenGrantKycTransaction::new()
        .account_id(account.id)
        .sign(account.key.clone())
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::TransactionPreCheckStatus { status: Status::InvalidTokenId, .. })
    );

    account.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn missing_account_id_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;
    let token = super::FungibleToken::create(
        &client,
        &account,
        CreateFungibleToken { initial_supply: 0, ..TOKEN_PARAMS },
    )
    .await?;

    let res = TokenGrantKycTransaction::new()
        .token_id(token.id)
        .sign(account.key.clone())
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::TransactionPreCheckStatus { status: Status::InvalidAccountId, .. })
    );

    token.delete(&client).await?;
    account.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn non_associated_token_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let (alice, bob) = tokio::try_join!(
        Account::create(Hbar::new(0), &client),
        Account::create(Hbar::new(0), &client)
    )?;

    let token = super::FungibleToken::create(&client, &alice, TOKEN_PARAMS).await?;

    let res = TokenGrantKycTransaction::new()
        .account_id(bob.id)
        .token_id(token.id)
        .sign(alice.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::TokenNotAssociatedToAccount, .. })
    );

    token.delete(&client).await?;

    tokio::try_join!(alice.delete(&client), bob.delete(&client))?;

    Ok(())
}
