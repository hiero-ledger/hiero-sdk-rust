use assert_matches::assert_matches;
use hiero_sdk::{
    AnyTransaction,
    Status,
    TopicInfoQuery,
    TopicMessageSubmitTransaction,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};
use crate::resources;
use crate::topic::Topic;

#[tokio::test]
async fn basic() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let topic = Topic::create(&client).await?;

    TopicMessageSubmitTransaction::new()
        .topic_id(topic.id)
        .message("Hello, from HCS!")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info = TopicInfoQuery::new().topic_id(topic.id).execute(&client).await?;

    assert_eq!(info.topic_id, topic.id);
    assert_eq!(info.sequence_number, 1);

    topic.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn large_message() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let topic = Topic::create(&client).await?;

    let responses = TopicMessageSubmitTransaction::new()
        .topic_id(topic.id)
        .max_chunks(15)
        .message(resources::BIG_CONTENTS)
        .execute_all(&client)
        .await?;

    for response in responses {
        response.get_receipt(&client).await?;
    }

    let info = TopicInfoQuery::new().topic_id(topic.id).execute(&client).await?;

    assert_eq!(info.topic_id, topic.id);
    assert_eq!(info.sequence_number, 14);

    topic.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn missing_topic_id_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let res = TopicMessageSubmitTransaction::new()
        .max_chunks(15)
        .message(resources::BIG_CONTENTS)
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::TransactionPreCheckStatus { status: Status::InvalidTopicId, .. })
    );

    Ok(())
}

#[tokio::test]
async fn missing_message_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let topic = Topic::create(&client).await?;

    let res = TopicMessageSubmitTransaction::new().topic_id(topic.id).execute(&client).await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::TransactionPreCheckStatus { status: Status::InvalidTopicMessage, .. })
    );

    topic.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn decode_hex_regression_test() -> anyhow::Result<()> {
    let transaction_bytes = hex_literal::hex!(
        "2ac2010a580a130a0b08d38f8f880610a09be91512041899e11c120218041880\
        c2d72f22020878da01330a0418a5a12012103030303030303136323736333737\
        31351a190a130a0b08d38f8f880610a09be91512041899e11c1001180112660a\
        640a20603edaec5d1c974c92cb5bee7b011310c3b84b13dc048424cd6ef146d6\
        a0d4a41a40b6a08f310ee29923e5868aac074468b2bde05da95a806e2f4a4f45\
        2177f129ca0abae7831e595b5beaa1c947e2cb71201642bab33fece5184b0454\
        7afc40850a"
    );

    let transaction = AnyTransaction::from_bytes(&transaction_bytes)?;

    let _id = transaction.get_transaction_id().unwrap();

    Ok(())
}
