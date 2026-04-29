use hiero_sdk::{
    AccountCreateTransaction,
    AccountInfoQuery,
    Hbar,
    Key,
    PrivateKey,
    Status,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

#[tokio::test]
async fn high_volume_account_create() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let key = PrivateKey::generate_ed25519();

    let receipt = AccountCreateTransaction::new()
        .set_key_without_alias(key.public_key())
        .initial_balance(Hbar::new(1))
        .set_high_volume(true)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let account_id = receipt.account_id.unwrap();

    let info = AccountInfoQuery::new().account_id(account_id).execute(&client).await?;

    assert_eq!(info.account_id, account_id);
    assert!(!info.is_deleted);
    assert_eq!(info.key, Key::Single(key.public_key()));

    Ok(())
}

#[tokio::test]
async fn high_volume_account_create_with_max_fee() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let key = PrivateKey::generate_ed25519();
    let max_fee = Hbar::new(10);

    let receipt = AccountCreateTransaction::new()
        .set_key_without_alias(key.public_key())
        .initial_balance(Hbar::new(1))
        .set_high_volume(true)
        .max_transaction_fee(max_fee)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let account_id = receipt.account_id.unwrap();

    let info = AccountInfoQuery::new().account_id(account_id).execute(&client).await?;

    assert_eq!(info.account_id, account_id);
    assert!(!info.is_deleted);
    assert_eq!(info.key, Key::Single(key.public_key()));

    Ok(())
}

#[tokio::test]
async fn high_volume_account_create_insufficient_fee_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let key = PrivateKey::generate_ed25519();
    let insufficient_fee = Hbar::from_tinybars(1);

    let res = AccountCreateTransaction::new()
        .set_key_without_alias(key.public_key())
        .set_high_volume(true)
        .max_transaction_fee(insufficient_fee)
        .execute(&client)
        .await;

    assert_matches::assert_matches!(
        res,
        Err(hiero_sdk::Error::TransactionPreCheckStatus {
            status: Status::InsufficientTxFee,
            ..
        })
    );

    Ok(())
}

#[tokio::test]
async fn high_volume_fee_differs_from_normal_fee() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let key_normal = PrivateKey::generate_ed25519();
    let record_normal = AccountCreateTransaction::new()
        .set_key_without_alias(key_normal.public_key())
        .initial_balance(Hbar::new(1))
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    let key_hv = PrivateKey::generate_ed25519();
    let record_high_volume = AccountCreateTransaction::new()
        .set_key_without_alias(key_hv.public_key())
        .initial_balance(Hbar::new(1))
        .set_high_volume(true)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    let fee_normal = record_normal.transaction_fee.to_tinybars();
    let fee_high_volume = record_high_volume.transaction_fee.to_tinybars();

    assert_ne!(fee_high_volume, fee_normal, "high-volume fee should differ from normal fee");

    Ok(())
}

#[tokio::test]
async fn high_volume_fee_is_higher_than_normal_fee() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let key_normal = PrivateKey::generate_ed25519();
    let record_normal = AccountCreateTransaction::new()
        .set_key_without_alias(key_normal.public_key())
        .initial_balance(Hbar::new(1))
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    let key_hv = PrivateKey::generate_ed25519();
    let record_high_volume = AccountCreateTransaction::new()
        .set_key_without_alias(key_hv.public_key())
        .initial_balance(Hbar::new(1))
        .set_high_volume(true)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    let fee_normal = record_normal.transaction_fee.to_tinybars();
    let fee_high_volume = record_high_volume.transaction_fee.to_tinybars();

    assert!(
        fee_high_volume > fee_normal,
        "high-volume fee ({fee_high_volume}) should be higher than normal fee ({fee_normal})"
    );

    Ok(())
}

#[tokio::test]
async fn high_volume_pricing_multiplier_is_set_in_record() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let key = PrivateKey::generate_ed25519();

    let record = AccountCreateTransaction::new()
        .set_key_without_alias(key.public_key())
        .initial_balance(Hbar::new(1))
        .set_high_volume(true)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    assert!(
        record.high_volume_pricing_multiplier > 0,
        "high_volume_pricing_multiplier should be set when high_volume is true, got {}",
        record.high_volume_pricing_multiplier
    );

    Ok(())
}

#[tokio::test]
async fn normal_transaction_has_zero_pricing_multiplier() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let key = PrivateKey::generate_ed25519();

    let record = AccountCreateTransaction::new()
        .set_key_without_alias(key.public_key())
        .initial_balance(Hbar::new(1))
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    assert_eq!(
        record.high_volume_pricing_multiplier, 0,
        "high_volume_pricing_multiplier should be 0 for normal transactions"
    );

    Ok(())
}
