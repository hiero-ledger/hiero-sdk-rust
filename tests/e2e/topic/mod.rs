use hiero_sdk::{
    Client,
    TopicCreateTransaction,
    TopicDeleteTransaction,
    TopicId,
};

mod create;
mod delete;
mod info;
mod message;
mod message_submit;
mod revenue_schedule;
mod update;
// mod message;
// mod message_submit;
// mod update;

struct Topic {
    id: TopicId,
}

impl Topic {
    async fn create(client: &Client) -> anyhow::Result<Self> {
        let id = TopicCreateTransaction::new()
            .admin_key(client.get_operator_public_key().unwrap())
            .topic_memo("[e2e::TopicCreateTransaction]")
            .execute(client)
            .await?
            .get_receipt(client)
            .await?
            .topic_id
            .unwrap();

        Ok(Self { id })
    }

    async fn delete(self, client: &Client) -> anyhow::Result<()> {
        TopicDeleteTransaction::new()
            .topic_id(self.id)
            .execute(client)
            .await?
            .get_receipt(client)
            .await?;

        Ok(())
    }
}
