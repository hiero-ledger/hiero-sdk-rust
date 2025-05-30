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

use hedera::{AccountCreateTransaction, AccountId, AnyTransaction, Hbar};

// HIP-745: Tests for serializing and deserializing incomplete non-frozen transactions
#[tokio::test]
async fn basic() -> anyhow::Result<()> {
    // Create an incomplete transaction (not setting all required fields)
    let mut tx = AccountCreateTransaction::new();

    tx.initial_balance(Hbar::from_tinybars(100)).transaction_memo("HIP-745 test").node_account_ids(
        [AccountId::new(0, 0, 5005), AccountId::new(0, 0, 5006), AccountId::new(0, 0, 5007)],
    );

    let bytes = tx.to_bytes().expect("Failed to serialize transaction");

    // Deserialize the transaction
    let tx2 = AnyTransaction::from_bytes(&bytes)
        .expect("Failed to deserialize transaction")
        .downcast::<AccountCreateTransaction>()
        .unwrap();

    assert_eq!(tx.get_transaction_memo(), tx2.get_transaction_memo());
    assert_eq!(tx.get_initial_balance(), tx2.get_initial_balance());
    assert_eq!(tx.get_transaction_id(), tx2.get_transaction_id());
    assert_eq!(tx.get_transaction_valid_duration(), tx2.get_transaction_valid_duration());
    assert_eq!(tx.get_node_account_ids(), tx2.get_node_account_ids());

    Ok(())
}
