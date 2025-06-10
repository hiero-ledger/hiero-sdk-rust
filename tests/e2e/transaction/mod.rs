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

use hedera::{
    AccountCreateTransaction,
    AccountId,
    AnyTransaction,
    Hbar,
    PrivateKey,
    Status,
    TransactionId,
};
use time::Duration;

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};
// HIP-745: Tests for serializing and deserializing incomplete non-frozen transactions
#[tokio::test]
async fn basic() -> anyhow::Result<()> {
    // Create an incomplete transaction (not setting all required fields)
    let mut tx = AccountCreateTransaction::new();

    let account_id = AccountId::new(0, 0, 1);
    let tx_id = TransactionId::generate(account_id.clone());
    tx.initial_balance(Hbar::from_tinybars(100))
        .transaction_id(tx_id)
        .node_account_ids([AccountId::new(0, 0, 1), AccountId::new(0, 0, 2)])
        .transaction_memo("HIP-745 test")
        .transaction_valid_duration(Duration::new(1000, 0));

    let bytes = tx.to_bytes().expect("Failed to serialize transaction");

    // Deserialize the transaction
    let tx2 = AnyTransaction::from_bytes(&bytes)
        .expect("Failed to deserialize transaction")
        .downcast::<AccountCreateTransaction>()
        .unwrap();

    println!("tx2: {:?}", tx2.get_transaction_memo());
    println!("tx: {:?}", tx.get_transaction_memo());

    assert_eq!(tx.get_transaction_id(), tx2.get_transaction_id());
    assert_eq!(tx.get_node_account_ids(), tx2.get_node_account_ids());
    assert_eq!(tx.get_transaction_memo(), tx2.get_transaction_memo());
    assert_eq!(tx.get_initial_balance(), tx2.get_initial_balance());
    assert_eq!(tx.get_transaction_valid_duration(), tx2.get_transaction_valid_duration());

    Ok(())
}

#[tokio::test]
async fn frozen_serialized_transaction_can_be_deserialized() -> anyhow::Result<()> {
    let TestEnvironment { client, config: _ } = crate::common::setup_global();
    let mut tx = AccountCreateTransaction::new();

    let _ = tx
        .initial_balance(Hbar::from_tinybars(100))
        .transaction_memo("HIP-745 test")
        .freeze_with(&client);

    let bytes = tx.to_bytes().expect("Failed to serialize transaction");
    // Deserialize the transaction
    let mut tx2 = AnyTransaction::from_bytes(&bytes)
        .expect("Failed to deserialize transaction")
        .downcast::<AccountCreateTransaction>()
        .unwrap();

    tx2.freeze_with(&client).expect("Failed to freeze transaction");

    assert_eq!(tx.get_transaction_id(), tx2.get_transaction_id());
    assert_eq!(tx.get_node_account_ids(), tx2.get_node_account_ids());
    assert_eq!(tx.get_transaction_memo(), tx2.get_transaction_memo());
    assert_eq!(tx.get_initial_balance(), tx2.get_initial_balance());

    Ok(())
}

#[tokio::test]
async fn serialized_deserialized_transaction_can_be_executed() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let mut tx = AccountCreateTransaction::new();
    let key = PrivateKey::generate_ed25519();
    let _ = tx
        .initial_balance(Hbar::from_tinybars(100))
        .key(key.public_key())
        .transaction_memo("HIP-745 test")
        .freeze_with(&client);

    let bytes = tx.to_bytes().expect("Failed to serialize transaction");

    let mut tx2 = AnyTransaction::from_bytes(&bytes)
        .expect("Failed to deserialize transaction")
        .downcast::<AccountCreateTransaction>()
        .unwrap();

    let receipt = tx2.execute(&client).await?.get_receipt(&client).await?;

    assert_eq!(receipt.status, Status::Success);

    Ok(())
}

#[tokio::test]
async fn serialized_deserialized_transaction_can_be_executed_non_frozen() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let mut tx = AccountCreateTransaction::new();
    let key = PrivateKey::generate_ed25519();
    let _ = tx
        .initial_balance(Hbar::from_tinybars(100))
        .key(key.public_key())
        .transaction_memo("HIP-745 test");

    let bytes = tx.to_bytes().expect("Failed to serialize transaction");

    let mut tx2 = AnyTransaction::from_bytes(&bytes)
        .expect("Failed to deserialize transaction")
        .downcast::<AccountCreateTransaction>()
        .unwrap();

    let receipt = tx2.execute(&client).await?.get_receipt(&client).await?;

    assert_eq!(receipt.status, Status::Success);

    Ok(())
}
