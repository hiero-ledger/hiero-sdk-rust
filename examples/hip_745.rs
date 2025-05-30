/*
 * ‌
 * Hedera Rust SDK
 * ​
 * Copyright (C) 2022 - 2023 Hedera Hashgraph, LLC
 * ​
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * ‍
 */

use std::iter;

use clap::Parser;
use hedera::{
    AccountId, Client, FileAppendTransaction, FileCreateTransaction, PrivateKey, Transaction,
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

    client.set_operator(args.operator_account_id, args.operator_key.clone());

    let receipt = FileCreateTransaction::new()
        .contents(&b"Hedera Hashgraph is great!"[..])
        .keys(iter::once(args.operator_key.public_key()))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let new_file_id = receipt.file_id.unwrap();

    println!("file: {new_file_id}");

    let bytes = FileAppendTransaction::new()
        .file_id(new_file_id)
        .contents(&b"Hedera Hashgraph is great!"[..])
        .to_bytes()
        .unwrap();

    println!("bytes: {:?}", bytes);
    // decode the bytes
    let transaction = Transaction::from_bytes(&bytes)?;
    println!("transaction: {:?}", transaction);
    Ok(())
}
