use std::time::Duration;

use clap::Parser;
use futures_util::StreamExt;
use hiero_sdk::{
    AccountId, Client, PrivateKey, TopicCreateTransaction, TopicMessageQuery, TopicMessageSubmitTransaction
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

    // generate a submit key to use with the topic.
    let submit_key = PrivateKey::generate_ecdsa();

    let topic_id = TopicCreateTransaction::new()
        .topic_memo("sdk::rust::consensus_pub_sub_with_submit_key")
        .submit_key(submit_key.public_key())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .topic_id
        .unwrap();

    println!("Created Topic `{topic_id}` with submit key `{submit_key}`");

    println!("Waiting 10s for the mirror node to catch up");

    tokio::time::sleep(Duration::from_secs(10)).await;

    let _handle: tokio::task::JoinHandle<hiero_sdk::Result<()>> = tokio::spawn({
        let client = client.clone();
        async move {
            println!("sending 5 messages");

            for i in 0..5 {
                let v: i64 = rand::random();
                let message = format!("random message: {v}");

                println!("publishing message {i}: `{message}`");

                TopicMessageSubmitTransaction::new()
                    .topic_id(topic_id)
                    .message(message)
                    .sign(submit_key.clone())
                    .execute(&client)
                    .await?
                    .get_receipt(&client)
                    .await?;

                tokio::time::sleep(Duration::from_secs(2)).await;
            }

            println!(
                "Finished sending the messages, press ctrl+c to exit once they're all recieved"
            );

            Ok(())
        }
    });

    let client = client.clone();
    let mut stream = TopicMessageQuery::new()
        .topic_id(topic_id)
        .subscribe(&client);

    while let Some(elem) = stream.next().await {
        let elem = match elem {
            Ok(it) => it,
            Err(e) => {
                eprintln!("Error while handling message stream: {e:?}");
                break;
            }
        };

        println!(
            "(seq: `{}`, contents: `{}`) reached consensus at {}",
            elem.sequence_number,
            String::from_utf8_lossy(&elem.contents),
            elem.consensus_timestamp
        );
    }

    Ok(())
}
