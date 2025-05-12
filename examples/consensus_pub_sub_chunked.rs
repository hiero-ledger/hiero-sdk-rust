// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

mod resources;

use clap::Parser;
use futures_util::StreamExt;
use hedera::{
    AccountId, Client, PrivateKey, TopicCreateTransaction, TopicMessageQuery, TopicMessageSubmitTransaction, Transaction
};
use tokio::task::JoinHandle;

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

    client.set_operator(args.operator_account_id, args.operator_key.clone());

    // generate a submit key to use with the topic.
    let submit_key = PrivateKey::generate_ed25519();

    let topic_id = TopicCreateTransaction::new()
        .topic_memo("sdk::rust::consensus_pub_sub_chunked")
        .submit_key(submit_key.public_key())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .topic_id
        .unwrap();

    println!("Created Topic `{topic_id}`");

    println!("Waiting 10s for the mirror node to catch up");

    tokio::time::sleep(Duration::from_secs(10)).await;

    let _handle: JoinHandle<hedera::Result<()>> = tokio::spawn({
        let client = client.clone();
        async move {
            println!(
                "about to prepare a transaction to send a message of {} bytes",
                resources::BIG_CONTENTS.len()
            );

            let mut tx = TopicMessageSubmitTransaction::new();

            // note: this used to set `max_chunks(15)` with a comment saying that the default is 10, but it's 20.
            tx.topic_id(topic_id)
                .message(resources::BIG_CONTENTS)
                .sign_with_operator(&client)?;

            // serialize to bytes so we can be signed "somewhere else" by the submit key
            let transaction_bytes = tx.to_bytes()?;

            // now pretend we sent those bytes across the network
            // parse them into a transaction so we can sign as the submit key
            let tx = Transaction::from_bytes(&transaction_bytes)?;

            // view out the message size from the parsed transaction
            // this can be useful to display what we are about to sign

            let mut tx: TopicMessageSubmitTransaction = tx.downcast().unwrap();

            println!(
                "about to send a transaction with a message of {} bytes",
                tx.get_message().map_or(0, |it| it.len())
            );

            // sign with that submit key
            tx.sign(submit_key);

            // now actually submit the transaction
            // get the receipt to ensure there were no errors
            tx.execute(&client).await?.get_receipt(&client).await?;

            println!("Finished sending the message, press ctrl+c to exit once it's recieved");

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
            "(seq: `{}`, contents: `{}` bytes) reached consensus at {}",
            elem.sequence_number,
            elem.contents.len(),
            elem.consensus_timestamp
        );
    }

    Ok(())
}
