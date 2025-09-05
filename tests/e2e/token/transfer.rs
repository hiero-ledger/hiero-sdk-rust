use assert_matches::assert_matches;
use hedera::{
    AccountCreateTransaction,
    FixedFee,
    FixedFeeData,
    Hbar,
    PrivateKey,
    Status,
    TokenAssociateTransaction,
    TokenCreateTransaction,
    TokenId,
    TokenWipeTransaction,
    TransferTransaction,
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
    TokenKeys,
};

const TOKEN_PARAMS: CreateFungibleToken = CreateFungibleToken {
    initial_supply: 10,
    keys: TokenKeys { supply: Some(Key::Owner), ..TokenKeys::DEFAULT },
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

    TransferTransaction::new()
        .token_transfer(token.id, bob.id, -10)
        .token_transfer(token.id, alice.id, 10)
        .sign(bob.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    token.burn(&client, 10).await?;

    token.delete(&client).await?;

    tokio::try_join!(alice.delete(&client), bob.delete(&client))?;

    Ok(())
}

#[tokio::test]
async fn insufficient_balance_for_fee_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let (alice, bob, cherry) = tokio::try_join!(
        Account::create(Hbar::new(0), &client),
        Account::create(Hbar::new(0), &client),
        Account::create(Hbar::new(0), &client),
    )?;

    let fee = FixedFee {
        all_collectors_are_exempt: true,
        fee_collector_account_id: Some(alice.id),
        fee: FixedFeeData {
            denominating_token_id: Some(TokenId::new(0, 0, 0)),
            amount: 5_000_000_000,
        },
    };

    let token_id = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .initial_supply(1)
        .custom_fees([fee.into()])
        .treasury_account_id(alice.id)
        .freeze_default(false)
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .admin_key(alice.key.public_key())
        .wipe_key(alice.key.public_key())
        .fee_schedule_key(alice.key.public_key())
        .sign(alice.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    let token = FungibleToken { id: token_id, owner: alice.clone() };

    TokenAssociateTransaction::new()
        .account_id(bob.id)
        .token_ids([token.id])
        .freeze_with(&client)?
        .sign(bob.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    TokenAssociateTransaction::new()
        .account_id(cherry.id)
        .token_ids([token.id])
        .freeze_with(&client)?
        .sign(cherry.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    TransferTransaction::new()
        .token_transfer(token.id, alice.id, -1)
        .token_transfer(token.id, bob.id, 1)
        .sign(alice.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let res = TransferTransaction::new()
        .token_transfer(token.id, bob.id, -1)
        .token_transfer(token.id, cherry.id, 1)
        .sign(bob.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus {
            status: Status::InsufficientSenderAccountBalanceForCustomFee,
            ..
        })
    );

    TokenWipeTransaction::new()
        .account_id(bob.id)
        .token_id(token.id)
        .amount(1_u64)
        .sign(alice.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    token.delete(&client).await?;

    tokio::try_join!(alice.delete(&client), bob.delete(&client), cherry.delete(&client))?;

    Ok(())
}

#[tokio::test]
async fn unowned_token_fails() -> anyhow::Result<()> {
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
        .freeze_with(&client)?
        .sign(bob.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // notice the swapped direction
    let res = TransferTransaction::new()
        .token_transfer(token.id, bob.id, -10)
        .token_transfer(token.id, alice.id, 10)
        .sign(bob.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::InsufficientTokenBalance, .. })
    );

    token.burn(&client, 10).await?;

    token.delete(&client).await?;

    tokio::try_join!(alice.delete(&client), bob.delete(&client))?;

    Ok(())
}

#[tokio::test]
async fn decimals() -> anyhow::Result<()> {
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
        .freeze_with(&client)?
        .sign(bob.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    TransferTransaction::new()
        .token_transfer_with_decimals(token.id, alice.id, -10, 3)
        .token_transfer_with_decimals(token.id, bob.id, 10, 3)
        .sign(alice.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    TransferTransaction::new()
        .token_transfer_with_decimals(token.id, bob.id, -10, 3)
        .token_transfer_with_decimals(token.id, alice.id, 10, 3)
        .sign(bob.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    token.burn(&client, 10).await?;

    token.delete(&client).await?;

    tokio::try_join!(alice.delete(&client), bob.delete(&client))?;

    Ok(())
}

#[tokio::test]
async fn incorrect_decimals_fails() -> anyhow::Result<()> {
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
        .freeze_with(&client)?
        .sign(bob.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let res = TransferTransaction::new()
        .token_transfer_with_decimals(token.id, alice.id, -10, 2)
        .token_transfer_with_decimals(token.id, bob.id, 10, 2)
        .sign(alice.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::UnexpectedTokenDecimals, .. })
    );

    token.burn(&client, 10).await?;

    token.delete(&client).await?;

    tokio::try_join!(alice.delete(&client), bob.delete(&client))?;

    Ok(())
}

#[tokio::test]
async fn transfer_to_account_with_unlimited_associations() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let sender_key = PrivateKey::generate_ed25519();
    let receiver_key = PrivateKey::generate_ed25519();

    let token_id = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .initial_supply(100_000)
        .treasury_account_id(client.get_operator_account_id().unwrap())
        .admin_key(client.get_operator_public_key().unwrap())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    let sender_id = AccountCreateTransaction::new()
        .set_key_without_alias(sender_key.public_key())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    let receiver_id = AccountCreateTransaction::new()
        .set_key_without_alias(receiver_key.public_key())
        .max_automatic_token_associations(-1)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    _ = TokenAssociateTransaction::new()
        .account_id(sender_id)
        .token_ids([token_id])
        .freeze_with(&client)?
        .sign(sender_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    _ = TransferTransaction::new()
        .token_transfer(token_id, client.get_operator_account_id().unwrap(), -10)
        .token_transfer(token_id, sender_id, 10)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    _ = TransferTransaction::new()
        .token_transfer(token_id, sender_id, -10)
        .token_transfer(token_id, receiver_id, 10)
        .freeze_with(&client)?
        .sign(sender_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}
