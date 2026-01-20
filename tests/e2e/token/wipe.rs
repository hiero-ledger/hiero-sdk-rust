use assert_matches::assert_matches;
use hiero_sdk::{
    Hbar,
    Status,
    TokenAssociateTransaction,
    TokenWipeTransaction,
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
    TokenKeys,
};

const KEYS: TokenKeys = TokenKeys { wipe: Some(Key::Owner), ..TokenKeys::DEFAULT };

#[tokio::test]
async fn fungible() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let (alice, bob) = tokio::try_join!(
        Account::create(Hbar::new(0), &client),
        Account::create(Hbar::new(0), &client)
    )?;

    let token = super::FungibleToken::create(
        &client,
        &alice,
        CreateFungibleToken { initial_supply: 10, keys: KEYS },
    )
    .await?;

    TokenAssociateTransaction::new()
        .account_id(bob.id)
        .token_ids([token.id])
        .freeze_with(&client)?
        .sign(bob.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    TransferTransaction::new()
        .token_transfer(token.id, alice.id, -10)
        .token_transfer(token.id, bob.id, 10)
        .sign(alice.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    TokenWipeTransaction::new()
        .token_id(token.id)
        .account_id(bob.id)
        .amount(10_u64)
        .sign(alice.key.clone())
        .execute(&client)
        .await?;

    token.delete(&client).await?;

    tokio::try_join!(alice.delete(&client), bob.delete(&client))?;

    Ok(())
}

#[tokio::test]

async fn nft() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let (alice, bob) = tokio::try_join!(
        Account::create(Hbar::new(0), &client),
        Account::create(Hbar::new(0), &client)
    )?;

    let token = super::Nft::create(&client, &alice).await?;

    let associate_fut = async {
        TokenAssociateTransaction::new()
            .account_id(bob.id)
            .token_ids([token.id])
            .sign(bob.key.clone())
            .execute(&client)
            .await?
            .get_receipt(&client)
            .await?;

        Ok(())
    };

    let (serials, _) = tokio::try_join!(token.mint_incremental(&client, 10), associate_fut)?;

    let mut transfer_tx = TransferTransaction::new();

    let (serials_to_transfer, serials) = serials.split_at(4);

    for &serial in serials_to_transfer {
        transfer_tx.nft_transfer(token.id.nft(serial as u64), alice.id, bob.id);
    }

    transfer_tx.sign(alice.key.clone()).execute(&client).await?.get_receipt(&client).await?;

    TokenWipeTransaction::new()
        .token_id(token.id)
        .account_id(bob.id)
        .serials(serials_to_transfer.iter().map(|it| *it as u64))
        .sign(alice.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    token.burn(&client, serials.iter().copied()).await?;

    token.delete(&client).await?;

    tokio::try_join!(alice.delete(&client), bob.delete(&client))?;

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

    let token = super::Nft::create(&client, &alice).await?;

    let associate_fut = async {
        TokenAssociateTransaction::new()
            .account_id(bob.id)
            .token_ids([token.id])
            .sign(bob.key.clone())
            .execute(&client)
            .await?
            .get_receipt(&client)
            .await?;

        Ok(())
    };

    let (serials, _) = tokio::try_join!(token.mint_incremental(&client, 10), associate_fut)?;

    // don't transfer them
    let res = TokenWipeTransaction::new()
        .token_id(token.id)
        .account_id(bob.id)
        .serials(serials[0..4].iter().map(|it| *it as u64))
        .sign(alice.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::AccountDoesNotOwnWipedNft, .. })
    );

    token.burn(&client, serials).await?;
    token.delete(&client).await?;
    tokio::try_join!(alice.delete(&client), bob.delete(&client))?;

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
        CreateFungibleToken { initial_supply: 0, keys: KEYS },
    )
    .await?;

    let res = TokenWipeTransaction::new().token_id(token.id).amount(10_u64).execute(&client).await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::TransactionPreCheckStatus { status: Status::InvalidAccountId, .. })
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

    let account = Account::create(Hbar::new(0), &client).await?;

    let res =
        TokenWipeTransaction::new().account_id(account.id).amount(10_u64).execute(&client).await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::TransactionPreCheckStatus { status: Status::InvalidTokenId, .. })
    );

    account.delete(&client).await?;
    Ok(())
}

#[tokio::test]

async fn missing_amount() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let (alice, bob) = tokio::try_join!(
        Account::create(Hbar::new(0), &client),
        Account::create(Hbar::new(0), &client)
    )?;

    let token = super::FungibleToken::create(
        &client,
        &alice,
        CreateFungibleToken { initial_supply: 10, keys: KEYS },
    )
    .await?;

    TokenAssociateTransaction::new()
        .account_id(bob.id)
        .token_ids([token.id])
        .freeze_with(&client)?
        .sign(bob.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    TransferTransaction::new()
        .token_transfer(token.id, alice.id, -10)
        .token_transfer(token.id, bob.id, 10)
        .sign(alice.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // this is the CUT
    TokenWipeTransaction::new()
        .token_id(token.id)
        .account_id(bob.id)
        .amount(0_u64)
        .sign(alice.key.clone())
        .execute(&client)
        .await?;

    // this is just so that we can actually delete the token.
    TokenWipeTransaction::new()
        .token_id(token.id)
        .account_id(bob.id)
        .amount(10_u64)
        .sign(alice.key.clone())
        .execute(&client)
        .await?;

    token.delete(&client).await?;

    tokio::try_join!(alice.delete(&client), bob.delete(&client))?;

    Ok(())
}
