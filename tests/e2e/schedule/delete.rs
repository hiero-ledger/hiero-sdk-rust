use assert_matches::assert_matches;
use hiero_sdk::{
    Hbar,
    ScheduleDeleteTransaction,
    Status,
    TransferTransaction,
};

use crate::account::Account;
use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

#[tokio::test]
async fn basic() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let account = Account::create(Hbar::new(1), &client).await?;

    let schedule_id = {
        let mut tx = TransferTransaction::new();
        tx.hbar_transfer(account.id, Hbar::new(-1)).hbar_transfer(op.account_id, Hbar::new(1));

        tx.schedule()
            .admin_key(op.private_key.public_key())
            .execute(&client)
            .await?
            .get_receipt(&client)
            .await?
            .schedule_id
            .unwrap()
    };

    ScheduleDeleteTransaction::new()
        .schedule_id(schedule_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}

#[tokio::test]
async fn missing_admin_key_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let account = Account::create(Hbar::new(1), &client).await?;

    let schedule_id = {
        let mut tx = TransferTransaction::new();
        tx.hbar_transfer(account.id, Hbar::new(-1)).hbar_transfer(op.account_id, Hbar::new(1));

        tx.schedule().execute(&client).await?.get_receipt(&client).await?.schedule_id.unwrap()
    };

    let res = ScheduleDeleteTransaction::new()
        .schedule_id(schedule_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::ScheduleIsImmutable, .. })
    );

    Ok(())
}

#[tokio::test]
async fn double_delete_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let account = Account::create(Hbar::new(1), &client).await?;

    let schedule_id = {
        let mut tx = TransferTransaction::new();
        tx.hbar_transfer(account.id, Hbar::new(-1)).hbar_transfer(op.account_id, Hbar::new(1));

        tx.schedule()
            .admin_key(op.private_key.public_key())
            .execute(&client)
            .await?
            .get_receipt(&client)
            .await?
            .schedule_id
            .unwrap()
    };

    ScheduleDeleteTransaction::new()
        .schedule_id(schedule_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let res = ScheduleDeleteTransaction::new()
        .schedule_id(schedule_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::ScheduleAlreadyDeleted, .. })
    );

    Ok(())
}

#[tokio::test]
async fn missing_schedule_id_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let res = ScheduleDeleteTransaction::new().execute(&client).await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::TransactionPreCheckStatus { status: Status::InvalidScheduleId, .. })
    );

    Ok(())
}
