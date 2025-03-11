use hedera::{
    TopicCreateTransaction,
    TopicInfoQuery,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};
use crate::topic::Topic;

#[tokio::test]
async fn basic() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let topic_id = TopicCreateTransaction::new()
        .admin_key(op.private_key.public_key())
        .topic_memo("[e2e::TopicCreateTransaction]")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .topic_id
        .unwrap();

    let topic = Topic { id: topic_id };

    topic.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn fieldless() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let _topic_id = TopicCreateTransaction::new()
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .topic_id
        .unwrap();
    Ok(())
}

#[tokio::test]
async fn autoset_auto_renew_account() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let topic_id = TopicCreateTransaction::new()
        .admin_key(client.get_operator_public_key().unwrap())
        .topic_memo("[e2e::TopicCreateTransaction]")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .topic_id
        .unwrap();

    let info = TopicInfoQuery::new().topic_id(topic_id).execute(&client).await?;
    assert_eq!(info.auto_renew_account_id.unwrap(), client.get_operator_account_id().unwrap());
    Ok(())
}
