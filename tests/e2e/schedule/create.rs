use std::hash::{
    DefaultHasher,
    Hash,
    Hasher,
};
use std::thread::sleep;

use assert_matches::assert_matches;
use hedera::{
    AccountBalanceQuery,
    AccountCreateTransaction,
    AccountDeleteTransaction,
    AccountUpdateTransaction,
    Hbar,
    Key,
    KeyList,
    PrivateKey,
    ScheduleCreateTransaction,
    ScheduleId,
    ScheduleInfoQuery,
    ScheduleSignTransaction,
    Status,
    TopicCreateTransaction,
    TopicMessageSubmitTransaction,
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

// Seconds in a day
const TEST_SECONDS: i64 = 86400;

#[tokio::test]
#[ignore = "not implemented in Hiero yet"]
async fn create_account() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let key = PrivateKey::generate_ed25519();

    let mut transaction = AccountCreateTransaction::new();
    transaction.key(key.public_key());

    let schedule_id = ScheduleCreateTransaction::new()
        .scheduled_transaction(transaction)
        .admin_key(op.private_key.public_key())
        .payer_account_id(op.account_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .schedule_id
        .unwrap();

    let info = ScheduleInfoQuery::new().schedule_id(schedule_id).execute(&client).await?;

    assert!(info.executed_at.is_some());

    Ok(())
}

#[tokio::test]
#[ignore = "not implemented in Hiero yet"]
async fn create_account_schedule() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let key = PrivateKey::generate_ed25519();

    let mut transaction = AccountCreateTransaction::new();
    transaction.key(key.public_key());

    let schedule_id = transaction
        .schedule()
        .admin_key(op.private_key.public_key())
        .payer_account_id(op.account_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .schedule_id
        .unwrap();

    let info = ScheduleInfoQuery::new().schedule_id(schedule_id).execute(&client).await?;

    assert!(info.executed_at.is_some());
    info.scheduled_transaction().unwrap();

    Ok(())
}

#[tokio::test]
async fn transfer() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let key1 = PrivateKey::generate_ed25519();
    let key2 = PrivateKey::generate_ed25519();
    let key3 = PrivateKey::generate_ed25519();

    let key_list = KeyList::from([key1.public_key(), key2.public_key().into(), key3.public_key()]);

    // Create the account with the `KeyList`
    let mut transaction = AccountCreateTransaction::new();
    let receipt = transaction
        .key(key_list)
        .initial_balance(Hbar::new(1))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let account_id = receipt.account_id.unwrap();

    // Create a transfer transaction with 2/3 signatures.
    let mut transfer = TransferTransaction::new();

    transfer.hbar_transfer(account_id, Hbar::new(-1)).hbar_transfer(op.account_id, Hbar::new(1));

    // Schedule the transactoin
    let mut scheduled = transfer.schedule();

    let receipt = scheduled.execute(&client).await?.get_receipt(&client).await?;

    // Get the schedule ID from the receipt
    let schedule_id = receipt.schedule_id.unwrap();

    // Get the schedule info to see if `signatories` is populated with 2/3 signatures
    let info = ScheduleInfoQuery::new().schedule_id(schedule_id).execute(&client).await?;

    assert_eq!(info.executed_at, None);

    // Finally send this last signature to Hiero. This last signature _should_ mean the transaction executes
    // since all 3 signatures have been provided.
    ScheduleSignTransaction::new()
        .schedule_id(schedule_id)
        .sign(key1.clone())
        .sign(key2.clone())
        .sign(key3.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info = ScheduleInfoQuery::new().schedule_id(schedule_id).execute(&client).await?;

    assert!(info.executed_at.is_some());

    AccountDeleteTransaction::new()
        .account_id(account_id)
        .transfer_account_id(op.account_id)
        .sign(key1)
        .sign(key2)
        .sign(key3)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}

#[tokio::test]
async fn double_schedule_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let account = Account::create(Hbar::new(1), &client).await?;

    let mut transfer = TransferTransaction::new();

    transfer.hbar_transfer(op.account_id, Hbar::new(-1)).hbar_transfer(account.id, Hbar::new(1));

    let schedule_id_1 = transfer
        .schedule()
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .schedule_id
        .unwrap();

    let info1 = ScheduleInfoQuery::new().schedule_id(schedule_id_1).execute(&client).await?;

    assert!(info1.executed_at.is_some());

    let mut transfer = TransferTransaction::new();

    transfer.hbar_transfer(op.account_id, Hbar::new(-1)).hbar_transfer(account.id, Hbar::new(1));

    let res = transfer.schedule().execute(&client).await?.get_receipt(&client).await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::IdenticalScheduleAlreadyCreated, .. })
    );

    account.delete(&client).await?;
    Ok(())
}

#[tokio::test]
async fn topic_message() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    // This is the submit key
    let key = PrivateKey::generate_ed25519();

    let topic_id = TopicCreateTransaction::new()
        .admin_key(op.private_key.public_key())
        .auto_renew_account_id(op.account_id)
        .topic_memo("HCS Topic_")
        .submit_key(key.public_key())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .topic_id
        .unwrap();

    let mut transaction = TopicMessageSubmitTransaction::new();

    transaction.topic_id(topic_id).message("scheduled hcs message");

    // create schedule
    let schedule_id = transaction
        .schedule()
        .admin_key(op.private_key.public_key())
        .payer_account_id(op.account_id)
        .schedule_memo(format!(
            "mirror scheduled E2E signature on create and sign_{}",
            OffsetDateTime::now_utc()
        ))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .schedule_id
        .unwrap();

    let info = ScheduleInfoQuery::new().schedule_id(schedule_id).execute(&client).await?;

    assert_eq!(info.schedule_id, schedule_id);

    ScheduleSignTransaction::new()
        .schedule_id(schedule_id)
        .sign(key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info = ScheduleInfoQuery::new().schedule_id(schedule_id).execute(&client).await?;

    assert!(info.executed_at.is_some());

    Ok(())
}

#[tokio::test]
async fn can_sign_schedule() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let account = Account::create(Hbar::new(10), &client).await?;

    // Create transaction to schedule
    let mut transfer = TransferTransaction::new();

    transfer.hbar_transfer(op.account_id, Hbar::new(1)).hbar_transfer(account.id, Hbar::new(-1));

    // Schedule transaction
    let schedule_id = transfer
        .schedule()
        .schedule_memo("HIP-423 E2E test")
        .expiration_time(OffsetDateTime::now_utc() + Duration::seconds(TEST_SECONDS))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .schedule_id
        .unwrap();

    let info = ScheduleInfoQuery::new().schedule_id(schedule_id).execute(&client).await?;

    // Verify the transaction hasn't executed yet
    assert_eq!(info.executed_at, None);

    // Schedule sign
    _ = ScheduleSignTransaction::new()
        .schedule_id(schedule_id)
        .freeze_with(&client)?
        .sign(account.key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info = ScheduleInfoQuery::new().schedule_id(schedule_id).execute(&client).await?;

    // Verify the transaction has executed
    assert!(info.executed_at.is_some());

    assert_eq!(schedule_id.checksum, None);
    assert_eq!(schedule_id, ScheduleId::from_bytes(&schedule_id.to_bytes()[..])?);

    let mut hasher = DefaultHasher::new();

    schedule_id.hash(&mut hasher);
    assert_ne!(hasher.finish(), 0);

    Ok(())
}

#[tokio::test]
async fn schedule_ahead_one_year_fail() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let account = Account::create(Hbar::new(10), &client).await?;

    // Create transaction to schedule
    let mut transfer = TransferTransaction::new();
    transfer.hbar_transfer(op.account_id, Hbar::new(1)).hbar_transfer(account.id, Hbar::new(-1));

    let res = transfer
        .schedule()
        .schedule_memo("HIP-423 E2E test")
        .expiration_time(OffsetDateTime::now_utc() + Duration::days(365))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus {
            status: Status::ScheduleExpirationTimeTooFarInFuture,
            ..
        })
    );

    Ok(())
}

#[tokio::test]
async fn schedule_in_the_past_fail() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let account = Account::create(Hbar::new(10), &client).await?;

    // Create transaction to schedule
    let mut transfer = TransferTransaction::new();
    transfer.hbar_transfer(op.account_id, Hbar::new(1)).hbar_transfer(account.id, Hbar::new(-1));

    let res = transfer
        .schedule()
        .schedule_memo("HIP-423 E2E test")
        .expiration_time(OffsetDateTime::now_utc() - Duration::seconds(10))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus {
            status: Status::ScheduleExpirationTimeMustBeHigherThanConsensusTime,
            ..
        })
    );

    Ok(())
}

#[tokio::test]
async fn sign_schedule_and_wait_for_expiry() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let account = Account::create(Hbar::new(10), &client).await?;

    // Create transaction to schedule
    let mut transfer = TransferTransaction::new();

    transfer.hbar_transfer(op.account_id, Hbar::new(1)).hbar_transfer(account.id, Hbar::new(-1));

    // Schedule transaction
    let schedule_id = transfer
        .schedule()
        .schedule_memo("HIP-423 E2E test")
        .wait_for_expiry(true)
        .expiration_time(OffsetDateTime::now_utc() + Duration::seconds(TEST_SECONDS))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .schedule_id
        .unwrap();

    let info = ScheduleInfoQuery::new().schedule_id(schedule_id).execute(&client).await?;

    // Verify the transaction hasn't executed yet
    assert_eq!(info.executed_at, None);

    // Schedule sign
    _ = ScheduleSignTransaction::new()
        .schedule_id(schedule_id)
        .freeze_with(&client)?
        .sign(account.key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info = ScheduleInfoQuery::new().schedule_id(schedule_id).execute(&client).await?;

    // Verify the transaction hasn't executed yet
    assert!(info.executed_at.is_none());

    assert_eq!(schedule_id.checksum, None);
    assert_eq!(schedule_id, ScheduleId::from_bytes(&schedule_id.to_bytes()[..])?);

    let mut hasher = DefaultHasher::new();
    schedule_id.hash(&mut hasher);
    assert_ne!(hasher.finish(), 0);

    Ok(())
}

#[tokio::test]
async fn sign_with_multi_sig_and_update_signing_requirements() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let key1 = PrivateKey::generate_ed25519();
    let key2 = PrivateKey::generate_ed25519();
    let key3 = PrivateKey::generate_ed25519();
    let key4 = PrivateKey::generate_ed25519();

    let key_list = KeyList {
        keys: vec![key1.public_key().into(), key2.public_key().into(), key3.public_key().into()],
        threshold: Some(2),
    };

    let account_id = AccountCreateTransaction::new()
        .key(key_list)
        .initial_balance(Hbar::new(10))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    // Create transaction to schedule
    let mut transfer = TransferTransaction::new();

    transfer.hbar_transfer(op.account_id, Hbar::new(1)).hbar_transfer(account_id, Hbar::new(-1));

    // Schedule transaction
    let schedule_id = transfer
        .schedule()
        .schedule_memo("HIP-423 E2E test")
        .expiration_time(OffsetDateTime::now_utc() + Duration::seconds(86400))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .schedule_id
        .unwrap();

    let info = ScheduleInfoQuery::new().schedule_id(schedule_id).execute(&client).await?;

    // Verify the transaction hasn't executed yet
    assert_eq!(info.executed_at, None);

    // Schedule sign
    _ = ScheduleSignTransaction::new()
        .schedule_id(schedule_id)
        .freeze_with(&client)?
        .sign(key1.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info = ScheduleInfoQuery::new().schedule_id(schedule_id).execute(&client).await?;

    // Verify the transaction hasn't executed yet
    assert_eq!(info.executed_at, None);

    // Update the signing requirements
    _ = AccountUpdateTransaction::new()
        .account_id(account_id)
        .key(Key::Single(key4.public_key()))
        .freeze_with(&client)?
        .sign(key1)
        .sign(key2)
        .sign(key4.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info = ScheduleInfoQuery::new().schedule_id(schedule_id).execute(&client).await?;

    // Verify the transaction hasn't executed yet
    assert_eq!(info.executed_at, None);

    // Schedule sign
    _ = ScheduleSignTransaction::new()
        .schedule_id(schedule_id)
        .freeze_with(&client)?
        .sign(key4)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info = ScheduleInfoQuery::new().schedule_id(schedule_id).execute(&client).await?;

    // Verify the schedule is executed
    assert!(info.executed_at.is_some());

    Ok(())
}

#[tokio::test]
async fn sign_with_multi_sig() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let key1 = PrivateKey::generate_ed25519();
    let key2 = PrivateKey::generate_ed25519();
    let key3 = PrivateKey::generate_ed25519();

    let key_list = KeyList {
        keys: vec![key1.public_key().into(), key2.public_key().into(), key3.public_key().into()],
        threshold: Some(2),
    };

    let account_id = AccountCreateTransaction::new()
        .key(key_list)
        .initial_balance(Hbar::new(10))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    // Create transaction to schedule
    let mut transfer = TransferTransaction::new();

    transfer.hbar_transfer(op.account_id, Hbar::new(1)).hbar_transfer(account_id, Hbar::new(-1));

    // Schedule transaction
    let schedule_id = transfer
        .schedule()
        .schedule_memo("HIP-423 E2E test")
        .expiration_time(OffsetDateTime::now_utc() + Duration::seconds(86400))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .schedule_id
        .unwrap();

    let info = ScheduleInfoQuery::new().schedule_id(schedule_id).execute(&client).await?;

    // Verify the transaction hasn't executed yet
    assert_eq!(info.executed_at, None);

    // Schedule sign
    _ = ScheduleSignTransaction::new()
        .schedule_id(schedule_id)
        .freeze_with(&client)?
        .sign(key1.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info = ScheduleInfoQuery::new().schedule_id(schedule_id).execute(&client).await?;

    // Verify the transaction has still not executed
    assert_eq!(info.executed_at, None);

    // Update the signing requirements
    _ = AccountUpdateTransaction::new()
        .account_id(account_id)
        .key(key1.public_key())
        .freeze_with(&client)?
        .sign(key1)
        .sign(key2.clone())
        .execute(&client)
        .await?
        .get_receipt(&client);

    let info = ScheduleInfoQuery::new().schedule_id(schedule_id).execute(&client).await?;

    // Verify the transaction has still not executed
    assert_eq!(info.executed_at, None);

    // Schedule sign with one key
    _ = ScheduleSignTransaction::new()
        .schedule_id(schedule_id)
        .freeze_with(&client)?
        .sign(key2)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info = ScheduleInfoQuery::new().schedule_id(schedule_id).execute(&client).await?;

    // Verify the schedule is executed
    assert!(info.executed_at.is_some());

    Ok(())
}

#[tokio::test]
async fn execute_with_short_exp_time() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let account = Account::create(Hbar::new(10), &client).await?;

    // Create transaction to schedule
    let mut transfer = TransferTransaction::new();

    transfer.hbar_transfer(op.account_id, Hbar::new(1)).hbar_transfer(account.id, Hbar::new(-1));

    // Schedule transaction
    let schedule_id = transfer
        .schedule()
        .schedule_memo("HIP-423 E2E test")
        .wait_for_expiry(true)
        .expiration_time(OffsetDateTime::now_utc() + Duration::seconds(10))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .schedule_id
        .unwrap();

    let info = ScheduleInfoQuery::new().schedule_id(schedule_id).execute(&client).await?;

    // Verify the transaction hasn't executed yet
    assert_eq!(info.executed_at, None);

    // Sign
    _ = ScheduleSignTransaction::new()
        .schedule_id(schedule_id)
        .freeze_with(&client)?
        .sign(account.key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info = ScheduleInfoQuery::new().schedule_id(schedule_id).execute(&client).await?;

    // Verify the transaction has still not executed
    assert_eq!(info.executed_at, None);

    let initial_balance =
        AccountBalanceQuery::new().account_id(account.id).execute(&client).await?;

    sleep(std::time::Duration::from_millis(10_000));

    let new_balance = AccountBalanceQuery::new().account_id(account.id).execute(&client).await?;

    // Verify the schedule is executed after 10 seconds
    assert_eq!(initial_balance.hbars, new_balance.hbars + Hbar::new(1));

    Ok(())
}
