use assert_matches::assert_matches;
use hedera::{
    Hbar,
    Status,
    TopicInfoQuery,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};
use crate::topic::Topic;

#[tokio::test]
async fn query() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let topic = Topic::create(&client).await?;

    let info = TopicInfoQuery::new().topic_id(topic.id).execute(&client).await?;

    assert_eq!(info.topic_memo, "[e2e::TopicCreateTransaction]");

    topic.delete(&client).await?;

    Ok(())
}
#[tokio::test]
async fn query_cost() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let topic = Topic::create(&client).await?;
    let mut query = TopicInfoQuery::new();

    query.topic_id(topic.id);

    let cost = query.get_cost(&client).await?;

    let info = query.payment_amount(cost).execute(&client).await?;

    assert_eq!(info.topic_memo, "[e2e::TopicCreateTransaction]");

    topic.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn query_cost_big_max() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let topic = Topic::create(&client).await?;

    let mut query = TopicInfoQuery::new();

    query.topic_id(topic.id).max_payment_amount(Hbar::new(1000));

    let cost = query.get_cost(&client).await?;

    let info = query.payment_amount(cost).execute(&client).await?;

    assert_eq!(info.topic_memo, "[e2e::TopicCreateTransaction]");

    topic.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn query_cost_small_max() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let topic = Topic::create(&client).await?;

    let mut query = TopicInfoQuery::new();

    query.topic_id(topic.id).max_payment_amount(Hbar::from_tinybars(1));

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
    // note: there's a very small chance this fails if the cost of a TopicContentsQuery changes right when we execute it.
    assert_eq!(query_cost, cost);

    topic.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn query_cost_insufficient_tx_fee() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let topic = Topic::create(&client).await?;

    let res = TopicInfoQuery::new()
        .topic_id(topic.id)
        .max_payment_amount(Hbar::from_tinybars(10000))
        .payment_amount(Hbar::from_tinybars(1))
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::QueryPaymentPreCheckStatus { status: Status::InsufficientTxFee, .. })
    );

    topic.delete(&client).await?;

    Ok(())
}
