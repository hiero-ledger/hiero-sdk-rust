use hiero_sdk::{
    Key,
    TopicInfoQuery,
    TopicMessageQuery,
    TopicMessageSubmitTransaction,
};
use time::OffsetDateTime;

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};
use crate::resources;
use crate::topic::Topic;

#[tokio::test]
async fn basic() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Skip if using local node.
    // Note: Remove when multinode is supported
    if config.is_local {
        return Ok(());
    }

    let topic = Topic::create(&client).await?;

    let info = TopicInfoQuery::new().topic_id(topic.id).execute(&client).await?;

    assert_eq!(info.topic_id, topic.id);
    assert_eq!(info.topic_memo, "[e2e::TopicCreateTransaction]");
    assert_eq!(info.sequence_number, 0);
    assert_eq!(info.admin_key, Some(Key::from(client.get_operator_public_key().unwrap())));

    tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

    tokio::spawn({
        let id = topic.id;
        let client = client.clone();
        async move {
            TopicMessageSubmitTransaction::new()
                .topic_id(id)
                .message("Hello, from HCS!")
                .execute(&client)
                .await?
                .get_receipt(&client)
                .await?;

            anyhow::Ok(())
        }
    });

    let fut = async {
        for _ in 0..20 {
            let res = TopicMessageQuery::new()
                .topic_id(topic.id)
                .start_time(OffsetDateTime::UNIX_EPOCH)
                .limit(1)
                .execute(&client)
                .await;

            tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

            // topic not found -> try again
            if let Err(hiero_sdk::Error::GrpcStatus(status)) = &res {
                if status.code() == tonic::Code::NotFound {
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    continue;
                }
            }

            return res.map_err(anyhow::Error::from);
        }

        anyhow::bail!("Couldn't get topic after 20 attempts")
    };

    let messages = tokio::time::timeout(std::time::Duration::from_secs(60), fut).await??;

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].contents, "Hello, from HCS!".as_bytes());
    topic.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn large() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Skip if using local node.
    // Note: Remove when multinode is supported
    if config.is_local {
        return Ok(());
    }

    let topic = Topic::create(&client).await?;

    tokio::spawn({
        let id = topic.id;
        let client = client.clone();

        async move {
            TopicMessageSubmitTransaction::new()
                .topic_id(id)
                .message(resources::BIG_CONTENTS)
                .execute(&client)
                .await?
                .get_receipt(&client)
                .await?;

            anyhow::Ok(())
        }
    });

    let fut = async {
        for _ in 0..20 {
            let res = TopicMessageQuery::new()
                .topic_id(topic.id)
                .start_time(OffsetDateTime::UNIX_EPOCH)
                .limit(14)
                .execute(&client)
                .await;

            // topic not found -> try again
            if let Err(hiero_sdk::Error::GrpcStatus(status)) = &res {
                if status.code() == tonic::Code::NotFound {
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    continue;
                }
            }

            return res.map_err(anyhow::Error::from);
        }

        anyhow::bail!("Couldn't get topic after 20 attempts")
    };

    let messages = tokio::time::timeout(std::time::Duration::from_secs(60), fut).await??;

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].contents, resources::BIG_CONTENTS.as_bytes());
    topic.delete(&client).await?;

    Ok(())
}
