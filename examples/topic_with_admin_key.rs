// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hedera::{
    AccountId, Client, Key, KeyList, PrivateKey, TopicCreateTransaction, TopicId, TopicInfoQuery, TopicUpdateTransaction
};

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, env)]
    operator_account_id: AccountId,

    #[clap(long, env)]
    operator_key: PrivateKey,

    #[clap(long, env, default_value = "testnet")]
    hedera_network: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let args = Args::parse();

    let client = Client::for_name(&args.hedera_network)?;

    client.set_operator(args.operator_account_id, args.operator_key);

    let (initial_admin_keys, topic_id) = create_topic_with_admin_key(&client).await?;
    update_topic_admin_key_and_memo(&client, initial_admin_keys, topic_id).await?;

    Ok(())
}

async fn create_topic_with_admin_key(
    client: &Client,
) -> anyhow::Result<(Vec<PrivateKey>, TopicId)> {
    // Generate the initial keys that are part of the adminKey's thresholdKey.
    // 3 ED25519 keys part of a 2-of-3 threshold key.
    let initial_admin_keys: Vec<_> = std::iter::repeat_with(PrivateKey::generate_ed25519)
        .take(3)
        .collect();

    let threshold_key = KeyList {
        keys: initial_admin_keys
            .iter()
            .map(PrivateKey::public_key)
            .map(Key::from)
            .collect(),
        threshold: Some(2),
    };

    let mut transaction = TopicCreateTransaction::new();

    transaction
        .topic_memo("demo topic")
        .admin_key(threshold_key)
        .freeze_with(client)?;

    for key in initial_admin_keys.iter().skip(1).cloned() {
        println!("Signing ConsensusTopicCreateTransaction with key {key}");
        transaction.sign(key);
    }

    let topic_id = transaction
        .execute(client)
        .await?
        .get_receipt(client)
        .await?
        .topic_id
        .unwrap();

    println!("Created new topic {topic_id} with 2-of-3 threshold key as admin_key");

    Ok((initial_admin_keys, topic_id))
}

async fn update_topic_admin_key_and_memo(
    client: &Client,
    initial_admin_keys: Vec<PrivateKey>,
    topic_id: TopicId,
) -> anyhow::Result<()> {
    // Generate the new keys that are part of the adminKey's thresholdKey.
    // 4 ED25519 keys part of a 3-of-4 threshold key.
    let new_admin_keys: Vec<_> = std::iter::repeat_with(PrivateKey::generate_ed25519)
        .take(4)
        .collect();

    let threshold_key = KeyList {
        keys: new_admin_keys
            .iter()
            .map(PrivateKey::public_key)
            .map(Key::from)
            .collect(),
        threshold: Some(3),
    };

    let mut transaction = TopicUpdateTransaction::new();

    transaction
        .topic_id(topic_id)
        .topic_memo("updated example topic")
        .admin_key(threshold_key)
        .freeze_with(client)?;

    // Sign with the initial adminKey. 2 of the 3 keys already part of the topic's adminKey.
    // Note that this time we're using a different subset of keys ([1, 0], rather than [1, 2])
    for key in initial_admin_keys.iter().rev().skip(1).cloned() {
        println!("Signing ConsensusTopicUpdateTransaction with initial admin key {key}",);
        transaction.sign(key);
    }

    for key in new_admin_keys.iter().skip(1).cloned() {
        println!("Signing ConsensusTopicUpdateTransaction with new admin key {key}",);
        transaction.sign(key);
    }

    transaction
        .execute(client)
        .await?
        .get_receipt(client)
        .await?;

    println!("Updated topic {topic_id} with 3-of-4 threshold key as adminKey");

    let topic_info = TopicInfoQuery::new()
        .topic_id(topic_id)
        .execute(client)
        .await?;

    println!("{topic_info:?}");

    Ok(())
}
