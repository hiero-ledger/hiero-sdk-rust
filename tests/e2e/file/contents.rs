use assert_matches::assert_matches;
use hedera::{
    FileContentsQuery,
    FileCreateTransaction,
    FileDeleteTransaction,
    Hbar,
    Status,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

#[tokio::test]
async fn query() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let file_id = FileCreateTransaction::new()
        .keys([op.private_key.public_key()])
        .contents("[rust::e2e::file_contents::1]")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .file_id
        .unwrap();

    let contents = FileContentsQuery::new().file_id(file_id).execute(&client).await?;

    assert_eq!(contents.contents, b"[rust::e2e::file_contents::1]");

    FileDeleteTransaction::new()
        .file_id(file_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}

#[tokio::test]
async fn query_empty() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let file_id = FileCreateTransaction::new()
        .keys([op.private_key.public_key()])
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .file_id
        .unwrap();

    let contents = FileContentsQuery::new().file_id(file_id).execute(&client).await?;

    assert_eq!(contents.contents, b"");

    FileDeleteTransaction::new()
        .file_id(file_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}

#[tokio::test]
async fn missing_file_id_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let res = FileContentsQuery::new().execute(&client).await;

    assert_matches!(
        res,
        Err(hedera::Error::QueryNoPaymentPreCheckStatus { status: Status::InvalidFileId })
    );

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

    let file_id = FileCreateTransaction::new()
        .keys([op.private_key.public_key()])
        .contents("[rust::e2e::file_contents::2]")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .file_id
        .unwrap();

    let mut query = FileContentsQuery::new();

    query.file_id(file_id).max_payment_amount(Hbar::new(1000));

    let cost = query.get_cost(&client).await?;

    let contents = query.payment_amount(cost).execute(&client).await?;

    assert_eq!(contents.contents, b"[rust::e2e::file_contents::2]");

    FileDeleteTransaction::new()
        .file_id(file_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

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

    let file_id = FileCreateTransaction::new()
        .keys([op.private_key.public_key()])
        .contents("[rust::e2e::file_contents::3]")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .file_id
        .unwrap();

    let mut query = FileContentsQuery::new();

    query.file_id(file_id).max_payment_amount(Hbar::from_tinybars(1));

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

    FileDeleteTransaction::new()
        .file_id(file_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}

#[tokio::test]
async fn query_insufficient_tx_fee_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let file_id = FileCreateTransaction::new()
        .keys([op.private_key.public_key()])
        .contents("[rust::e2e::file_contents::4]")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .file_id
        .unwrap();

    let res = FileContentsQuery::new()
        .file_id(file_id)
        .max_payment_amount(Hbar::from_tinybars(10000))
        .payment_amount(Hbar::from_tinybars(1))
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::QueryPaymentPreCheckStatus { status: Status::InsufficientTxFee, .. })
    );

    FileDeleteTransaction::new()
        .file_id(file_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}
