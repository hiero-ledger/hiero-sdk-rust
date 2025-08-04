use assert_matches::assert_matches;
use hedera::{
    Hbar,
    NetworkVersionInfoQuery,
    Status,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

#[tokio::test]
async fn query() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let _info = NetworkVersionInfoQuery::new().execute(&client).await?;

    Ok(())
}

#[tokio::test]
async fn query_cost() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let mut query = NetworkVersionInfoQuery::new();

    query.max_payment_amount(Hbar::new(1));

    let cost = query.get_cost(&client).await?;

    let _info = query.payment_amount(cost).execute(&client).await?;

    Ok(())
}

#[tokio::test]
async fn query_cost_big_max() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let mut query = NetworkVersionInfoQuery::new();

    query.max_payment_amount(Hbar::MAX);

    let cost = query.get_cost(&client).await?;

    let _info = query.payment_amount(cost).execute(&client).await?;

    Ok(())
}

#[tokio::test]
async fn query_cost_small_max_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let mut query = NetworkVersionInfoQuery::new();

    query.max_payment_amount(Hbar::from_tinybars(1));

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
    // note: there's a very small chance this fails if the cost of a NetworkVersionInfoQuery changes right when we execute it.
    assert_eq!(query_cost, cost);

    Ok(())
}

#[tokio::test]
async fn get_cost_insufficient_tx_fee_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let res = NetworkVersionInfoQuery::new() //
        .max_payment_amount(Hbar::from_tinybars(10000))
        .payment_amount(Hbar::from_tinybars(1))
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::QueryPaymentPreCheckStatus { status: Status::InsufficientTxFee, .. })
    );

    Ok(())
}
