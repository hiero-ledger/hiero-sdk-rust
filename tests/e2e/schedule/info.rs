use assert_matches::assert_matches;
use hedera::{
    AccountCreateTransaction,
    Hbar,
    KeyList,
    PrivateKey,
    ScheduleCreateTransaction,
    ScheduleInfoQuery,
    Status,
    TransferTransaction,
};

use crate::account::Account;
use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

#[tokio::test]
#[ignore = "not implemented in Hiero yet"]
async fn basic() -> anyhow::Result<()> {
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
    info.scheduled_transaction().unwrap();

    Ok(())
}

#[tokio::test]
async fn query() -> anyhow::Result<()> {
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

    let info = ScheduleInfoQuery::new().schedule_id(schedule_id).execute(&client).await?;

    assert_eq!(info.admin_key, None);
    assert_eq!(info.creator_account_id, op.account_id);
    assert_eq!(info.deleted_at, None);
    assert_eq!(info.executed_at, None);
    assert!(info.expiration_time.is_some());
    // assert_eq!(info.ledger_id, client.ledger_id());
    assert_eq!(info.memo, "");
    assert_eq!(info.payer_account_id, Some(op.account_id));
    let _ = info.scheduled_transaction()?;

    assert_eq!(
        info.signatories,
        KeyList { keys: vec![op.private_key.public_key().into()].into(), threshold: None }
    );
    assert!(!info.wait_for_expiry);

    account.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn missing_schedule_id_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let res = ScheduleInfoQuery::new().execute(&client).await;

    assert_matches!(
        res,
        Err(hedera::Error::QueryNoPaymentPreCheckStatus { status: Status::InvalidScheduleId })
    );

    Ok(())
}

#[tokio::test]
async fn query_cost() -> anyhow::Result<()> {
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

    let mut query = ScheduleInfoQuery::new();

    query.schedule_id(schedule_id);

    let cost = query.get_cost(&client).await?;

    _ = query.payment_amount(cost).execute(&client).await?;

    account.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn query_cost_big_max() -> anyhow::Result<()> {
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

    let mut query = ScheduleInfoQuery::new();

    query.schedule_id(schedule_id).max_payment_amount(Hbar::new(1000));

    let cost = query.get_cost(&client).await?;

    let _ = query.payment_amount(cost).execute(&client).await?;

    account.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn query_cost_small_max_fails() -> anyhow::Result<()> {
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

    let mut query = ScheduleInfoQuery::new();

    query.schedule_id(schedule_id).max_payment_amount(Hbar::from_tinybars(1));

    let cost = query.get_cost(&client).await?;

    let res = query.execute(&client).await;

    let (max_query_payment, query_cost) = assert_matches!(
        res,
        Err(hedera::Error::MaxQueryPaymentExceeded {
            max_query_payment,
            query_cost
        }) => (max_query_payment, query_cost)
    );

    assert_eq!(max_query_payment, Hbar::from_tinybars(1));
    // note: there's a very small chance this fails if the cost of a FileContentsQuery changes right when we execute it.
    assert_eq!(query_cost, cost);

    account.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn query_cost_insufficient_tx_fee_fails() -> anyhow::Result<()> {
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

    let mut query = ScheduleInfoQuery::new();

    let res = query
        .schedule_id(schedule_id)
        .max_payment_amount(Hbar::from_tinybars(10000))
        .payment_amount(Hbar::from_tinybars(1))
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::QueryPaymentPreCheckStatus { status: Status::InsufficientTxFee, .. })
    );

    account.delete(&client).await?;

    Ok(())
}
