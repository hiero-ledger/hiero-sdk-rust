use assert_matches::assert_matches;
use hiero_sdk::{
    PrivateKey,
    Status,
    TopicCreateTransaction,
    TopicDeleteTransaction,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};
use crate::topic::Topic;

#[tokio::test]
async fn basic() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let topic = Topic::create(&client).await?;

    TopicDeleteTransaction::new()
        .topic_id(topic.id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}

#[tokio::test]
async fn immutable_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let topic_id = TopicCreateTransaction::new()
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .topic_id
        .unwrap();

    let res = TopicDeleteTransaction::new()
        .topic_id(topic_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;
    assert_matches!(res, Err(hiero_sdk::Error::ReceiptStatus { status: Status::Unauthorized, .. }));

    Ok(())
}

#[tokio::test]
async fn wrong_admin_key_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let admin_key = PrivateKey::generate_ed25519();

    let topic_id = TopicCreateTransaction::new()
        .admin_key(admin_key.public_key())
        .sign(admin_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .topic_id
        .unwrap();

    let res = TopicDeleteTransaction::new()
        .topic_id(topic_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, .. })
    );

    Ok(())
}
