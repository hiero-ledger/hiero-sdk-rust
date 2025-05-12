// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use clap::Parser;
// `use futures::TryStreamExt`, this is better practice though.
use futures_util::TryStreamExt;
use hedera::{
    AccountId, Client, PrivateKey, TopicId, TopicMessageQuery, TopicMessageSubmitTransaction
};
use parking_lot::RwLock;
use tokio::time::sleep;

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, env)]
    operator_account_id: AccountId,

    #[clap(long, env)]
    operator_key: PrivateKey,

    #[clap(long, env, default_value = "0.0.34945875")]
    topic: TopicId,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let args = Args::parse();

    let client = Client::for_testnet();

    client.set_operator(args.operator_account_id, args.operator_key);

    let message_send_times = Arc::new(RwLock::new(HashMap::new()));

    tokio::spawn({
        let client = client.clone();
        let message_send_times = message_send_times.clone();

        async move {
            for index in 0.. {
                let message = format!("hello, {index}");
                let time = Instant::now();

                message_send_times.write().insert(message.clone(), time);

                // send a message, crash the example program if it fails
                TopicMessageSubmitTransaction::new()
                    .topic_id(args.topic)
                    .message(message.as_bytes())
                    .execute(&client)
                    .await
                    .unwrap();

                sleep(Duration::from_millis(500)).await;
            }
        }
    });

    let mut stream = TopicMessageQuery::new()
        .topic_id(args.topic)
        .subscribe(&client);

    let mut latencies = Vec::new();

    while let Some(tm) = stream.try_next().await? {
        let message = String::from_utf8(tm.contents)?;

        let times = message_send_times.read();
        let start = times.get(&message).unwrap();
        let latency = start.elapsed();

        println!(
            "recv: {}, message: {:?}, latency: {:.3?}",
            tm.sequence_number, message, latency,
        );

        latencies.push(latency.as_secs_f64());

        if latencies.len() == 100 {
            let avg: f64 = latencies.iter().copied().sum::<f64>() / (latencies.len() as f64);

            println!("---- average latency: {avg:.5}s ----");

            latencies.clear();
        }
    }

    Ok(())
}
