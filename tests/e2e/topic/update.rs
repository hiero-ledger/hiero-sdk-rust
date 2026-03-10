use hiero_sdk::{
    TopicInfoQuery,
    TopicUpdateTransaction,
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

    TopicUpdateTransaction::new()
        .topic_id(topic.id)
        .clear_auto_renew_account_id()
        .topic_memo("hello")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info = TopicInfoQuery::new().topic_id(topic.id).execute(&client).await?;

    assert_eq!(info.topic_memo, "hello");
    assert_eq!(info.auto_renew_account_id, None);

    topic.delete(&client).await?;
    Ok(())
}
