// SPDX-License-Identifier: Apache-2.0

use assert_matches::assert_matches;
use hedera::{
    Hbar,
    Status,
    TokenPauseTransaction,
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
    TokenKeys,
};

const TOKEN_PARAMS: CreateFungibleToken = CreateFungibleToken {
    initial_supply: 0,
    keys: TokenKeys { pause: Some(Key::Owner), ..TokenKeys::DEFAULT },
};

#[tokio::test]
async fn basic() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let owner = Account::create(Hbar::new(0), &client).await?;

    let token = FungibleToken::create(&client, &owner, TOKEN_PARAMS).await?;

    TokenPauseTransaction::new()
        .token_id(token.id)
        .sign(token.owner.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}

#[tokio::test]
async fn missing_token_id_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let res = TokenPauseTransaction::new().execute(&client).await;

    assert_matches!(
        res,
        Err(hedera::Error::TransactionPreCheckStatus { status: Status::InvalidTokenId, .. })
    );

    Ok(())
}

#[tokio::test]
async fn missing_pause_key_sig_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let owner = Account::create(Hbar::new(0), &client).await?;

    let token = FungibleToken::create(&client, &owner, TOKEN_PARAMS).await?;

    let res = TokenPauseTransaction::new()
        .token_id(token.id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::InvalidSignature, .. })
    );

    Ok(())
}

#[tokio::test]
async fn missing_pause_key_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let owner = Account::create(Hbar::new(0), &client).await?;

    let token = FungibleToken::create(&client, &owner, CreateFungibleToken::default()).await?;

    let res = TokenPauseTransaction::new()
        .token_id(token.id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::TokenHasNoPauseKey, .. })
    );

    Ok(())
}
