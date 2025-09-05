use hedera::{
    AccountCreateTransaction,
    AccountInfoQuery,
    AccountUpdateTransaction,
    Hbar,
    Key,
    PrivateKey,
    TokenCreateTransaction,
    TransferTransaction,
};
use time::Duration;

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

#[tokio::test]
async fn set_key() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let key1 = PrivateKey::generate_ed25519();
    let key2 = PrivateKey::generate_ed25519();

    let account_id = AccountCreateTransaction::new()
        .set_key_without_alias(key1.public_key())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    let info = AccountInfoQuery::new().account_id(account_id).execute(&client).await?;

    // none of these are relevant to this test (they should already be tested by the create E2E tests)
    // assertThat(info.accountId).isEqualTo(accountId);
    // assertThat(info.isDeleted).isFalse();
    // assertThat(info.balance).isEqualTo(new Hbar(0));
    // assertThat(info.autoRenewPeriod).isEqualTo(Duration.ofDays(90));
    // assertThat(info.proxyAccountId).isNull();
    // assertThat(info.proxyReceived).isEqualTo(Hbar.ZERO);

    assert_eq!(info.key, Key::Single(key1.public_key()));

    AccountUpdateTransaction::new()
        .account_id(account_id)
        .key(key2.public_key())
        .freeze_with(&client)?
        .sign(key1)
        .sign(key2.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info = AccountInfoQuery::new().account_id(account_id).execute(&client).await?;

    assert_eq!(info.account_id, account_id);
    assert!(!info.is_deleted);
    assert_eq!(info.key, Key::Single(key2.public_key()));
    assert_eq!(info.balance, Hbar::ZERO);
    assert_eq!(info.auto_renew_period, Some(Duration::days(90)));
    assert_eq!(info.proxy_received, Hbar::ZERO);

    Ok(())
}

#[tokio::test]
async fn missing_account_id_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let res = AccountUpdateTransaction::new().execute(&client).await;

    assert_matches::assert_matches!(
        res,
        Err(hedera::Error::TransactionPreCheckStatus {
            status: hedera::Status::AccountIdDoesNotExist,
            ..
        })
    );

    Ok(())
}

#[tokio::test]
async fn cannot_update_max_token_association_to_lower_value_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account_key = PrivateKey::generate_ed25519();

    // Create account with max token associations of 1
    let account_id = AccountCreateTransaction::new()
        .set_key_without_alias(account_key.public_key())
        .max_automatic_token_associations(1)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    // Create token
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

    // Associate token with account
    _ = TransferTransaction::new()
        .token_transfer(token_id, client.get_operator_account_id().unwrap(), -10)
        .token_transfer(token_id, account_id, 10)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Update account max token associations to 0
    let res = AccountUpdateTransaction::new()
        .account_id(account_id)
        .max_automatic_token_associations(0)
        .freeze_with(&client)?
        .sign(account_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches::assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus {
            status: hedera::Status::ExistingAutomaticAssociationsExceedGivenLimit,
            ..
        })
    );

    Ok(())
}
