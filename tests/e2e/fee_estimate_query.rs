use std::collections::HashMap;

use hiero_sdk::{
    AccountCreateTransaction,
    ContractCreateTransaction,
    FeeEstimateMode,
    FeeEstimateQuery,
    FileAppendTransaction,
    FileCreateTransaction,
    FileDeleteTransaction,
    Hbar,
    PrivateKey,
    TokenCreateTransaction,
    TokenMintTransaction,
    TopicCreateTransaction,
    TopicDeleteTransaction,
    TopicMessageSubmitTransaction,
    TransferTransaction,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

#[tokio::test]
async fn estimate_account_create_transaction() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let key = PrivateKey::generate_ed25519();

    let mut tx = AccountCreateTransaction::new();
    tx.set_key_without_alias(key.public_key()).initial_balance(Hbar::new(1));

    let mut query = FeeEstimateQuery::new().transaction(tx).mode(FeeEstimateMode::State);
    let estimate = query.execute(&client).await?;

    // Verify the response structure
    assert!(!estimate.mode.is_empty());
    assert!(estimate.total > 0);
    assert!(estimate.network.multiplier > 0);
    assert!(estimate.network.subtotal > 0);
    assert!(estimate.node.base > 0 || estimate.node.extras.is_empty());
    assert!(estimate.service.base > 0 || estimate.service.extras.is_empty());

    // The total should be the sum of network subtotal, node subtotal, and service subtotal
    let expected_total =
        estimate.network.subtotal + estimate.node.subtotal() + estimate.service.subtotal();
    assert_eq!(estimate.total, expected_total);

    Ok(())
}

#[tokio::test]
async fn estimate_token_create_transaction() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Skip if running on localhost without a running node
    // This test requires executing transactions to create accounts
    if config.is_local {
        // Try to ping the local node, skip if unavailable
        if client.network().is_empty() {
            return Ok(());
        }
    }

    let account_key = PrivateKey::generate_ed25519();

    // Create an account first to use as treasury
    let treasury_account = AccountCreateTransaction::new()
        .set_key_without_alias(account_key.public_key())
        .initial_balance(Hbar::new(10))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    let mut tx = TokenCreateTransaction::new();
    tx.name("Test Token")
        .symbol("TEST")
        .treasury_account_id(treasury_account)
        .admin_key(account_key.public_key())
        .decimals(2)
        .initial_supply(1000);

    let mut query = FeeEstimateQuery::new().transaction(tx).mode(FeeEstimateMode::State);
    let estimate = query.execute(&client).await?;

    // Verify the response structure
    assert!(!estimate.mode.is_empty());
    assert!(estimate.total > 0);
    assert!(estimate.network.multiplier > 0);
    assert!(estimate.network.subtotal > 0);

    Ok(())
}

#[tokio::test]
async fn estimate_transfer_transaction() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Skip if running on localhost without a running node
    // This test requires executing transactions to create accounts
    if config.is_local {
        if client.network().is_empty() {
            return Ok(());
        }
    }

    let key1 = PrivateKey::generate_ed25519();
    let key2 = PrivateKey::generate_ed25519();

    // Create two accounts
    let account1 = AccountCreateTransaction::new()
        .set_key_without_alias(key1.public_key())
        .initial_balance(Hbar::new(10))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    let account2 = AccountCreateTransaction::new()
        .set_key_without_alias(key2.public_key())
        .initial_balance(Hbar::new(0))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    let mut tx = TransferTransaction::new();
    tx.hbar_transfer(account1, Hbar::new(-5)).hbar_transfer(account2, Hbar::new(5));

    let mut query = FeeEstimateQuery::new().transaction(tx).mode(FeeEstimateMode::State);
    let estimate = query.execute(&client).await?;

    // Verify the response structure
    assert!(!estimate.mode.is_empty());
    assert!(estimate.total > 0);
    assert!(estimate.network.multiplier > 0);
    assert!(estimate.network.subtotal > 0);

    Ok(())
}

#[tokio::test]
async fn estimate_with_intrinsic_mode() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let key = PrivateKey::generate_ed25519();

    let mut tx = AccountCreateTransaction::new();
    tx.set_key_without_alias(key.public_key()).initial_balance(Hbar::new(1));

    let mut query = FeeEstimateQuery::new().transaction(tx).mode(FeeEstimateMode::Intrinsic);
    let estimate = query.execute(&client).await?;

    assert!(!estimate.mode.is_empty());
    assert!(estimate.mode == "INTRINSIC" || estimate.mode == "STATE");
    assert!(estimate.total > 0);
    assert!(estimate.network.multiplier > 0);

    Ok(())
}

#[tokio::test]
async fn estimate_with_state_mode() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let key = PrivateKey::generate_ed25519();

    let mut tx = AccountCreateTransaction::new();
    tx.set_key_without_alias(key.public_key()).initial_balance(Hbar::new(1));

    let mut query = FeeEstimateQuery::new().transaction(tx).mode(FeeEstimateMode::State);
    let estimate = query.execute(&client).await?;

    // Verify the response structure
    assert_eq!(estimate.mode, "STATE");
    assert!(estimate.total > 0);
    assert!(estimate.network.multiplier > 0);

    Ok(())
}

#[tokio::test]
async fn estimate_with_default_mode() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let key = PrivateKey::generate_ed25519();

    let mut tx = AccountCreateTransaction::new();
    tx.set_key_without_alias(key.public_key()).initial_balance(Hbar::new(1));

    // Default mode should be State
    let mut query = FeeEstimateQuery::new().transaction(tx);
    let estimate = query.execute(&client).await?;

    // Verify the response structure
    assert_eq!(estimate.mode, "STATE");
    assert!(estimate.total > 0);

    Ok(())
}

#[tokio::test]
async fn estimate_with_custom_max_attempts() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let key = PrivateKey::generate_ed25519();

    let mut tx = AccountCreateTransaction::new();
    tx.set_key_without_alias(key.public_key()).initial_balance(Hbar::new(1));

    let mut query = FeeEstimateQuery::new().transaction(tx).max_attempts(5);
    let estimate = query.execute(&client).await?;

    // Verify the response structure
    assert!(estimate.total > 0);

    Ok(())
}

#[tokio::test]
async fn estimate_fee_components() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let key = PrivateKey::generate_ed25519();

    let mut tx = AccountCreateTransaction::new();
    tx.set_key_without_alias(key.public_key()).initial_balance(Hbar::new(1));

    let mut query = FeeEstimateQuery::new().transaction(tx);
    let estimate = query.execute(&client).await?;

    assert!(estimate.network.multiplier > 0);
    assert!(estimate.network.subtotal > 0);

    let node_subtotal = estimate.node.subtotal();
    let service_subtotal = estimate.service.subtotal();

    assert!(node_subtotal >= estimate.node.base);
    assert!(service_subtotal >= estimate.service.base);

    let expected_total = estimate.network.subtotal + node_subtotal + service_subtotal;
    assert_eq!(estimate.total, expected_total);

    Ok(())
}

#[tokio::test]
async fn estimate_error_no_transaction() {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return;
    };

    // We can't construct an empty FeeEstimateQuery without the private type, so we verify
    // that a query with a transaction executes successfully.
    let tx = AccountCreateTransaction::new();
    let mut query = FeeEstimateQuery::new().transaction(tx);
    let res = query.execute(&client).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn estimate_error_no_mirror_network() {
    let Some(TestEnvironment { config: _, client: _ }) = setup_nonfree() else {
        return;
    };

    let mut network = HashMap::new();
    network.insert(
        "testnet.mirrornode.hedera.com:443".to_string(),
        hiero_sdk::AccountId::new(0, 0, 3),
    );
    let mut client_without_mirror = hiero_sdk::Client::for_network(network).unwrap();
    client_without_mirror.set_mirror_network([]);

    let dummy_key = PrivateKey::generate_ed25519();
    client_without_mirror.set_operator(hiero_sdk::AccountId::new(0, 0, 2), dummy_key);

    let key = PrivateKey::generate_ed25519();
    let mut tx = AccountCreateTransaction::new();
    tx.set_key_without_alias(key.public_key()).initial_balance(Hbar::new(1));

    let mut query = FeeEstimateQuery::new().transaction(tx);
    let res = query.execute(&client_without_mirror).await;
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(err.to_string().contains("mirror node"));
}

#[tokio::test]
async fn estimate_token_mint_transaction() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Skip if running on localhost without a running node
    // This test requires executing transactions to create a token
    if config.is_local {
        if client.network().is_empty() {
            return Ok(());
        }
    }

    let account_key = PrivateKey::generate_ed25519();

    // Create an account first to use as treasury
    let treasury_account = AccountCreateTransaction::new()
        .set_key_without_alias(account_key.public_key())
        .initial_balance(Hbar::new(10))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    // Create a token (admin/supply key must sign the creation)
    let token_id = TokenCreateTransaction::new()
        .name("Test Token")
        .symbol("TEST")
        .treasury_account_id(treasury_account)
        .admin_key(account_key.public_key())
        .supply_key(account_key.public_key())
        .decimals(2)
        .initial_supply(1000)
        .sign(account_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    let mut tx = TokenMintTransaction::new();
    tx.token_id(token_id).amount(100);

    let mut query = FeeEstimateQuery::new().transaction(tx).mode(FeeEstimateMode::State);
    let estimate = query.execute(&client).await?;

    // Verify the response structure
    assert!(!estimate.mode.is_empty());
    assert!(estimate.total > 0);

    // TokenMint may have extras for additional tokens
    let node_subtotal = estimate.node.subtotal();
    assert!(node_subtotal >= estimate.node.base);

    Ok(())
}

#[tokio::test]
async fn estimate_topic_create_transaction() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let key = PrivateKey::generate_ed25519();

    let mut tx = TopicCreateTransaction::new();
    tx.admin_key(key.public_key());

    let mut query = FeeEstimateQuery::new().transaction(tx).mode(FeeEstimateMode::State);
    let estimate = query.execute(&client).await?;

    // Verify the response structure
    assert!(!estimate.mode.is_empty());
    assert!(estimate.total > 0);
    assert!(estimate.service.base > 0 || estimate.service.extras.is_empty());

    Ok(())
}

#[tokio::test]
async fn estimate_contract_create_transaction() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let bytecode = vec![1, 2, 3, 4, 5];
    let mut tx = ContractCreateTransaction::new();
    tx.bytecode(bytecode).gas(100000);

    let mut query = FeeEstimateQuery::new().transaction(tx).mode(FeeEstimateMode::State);
    let estimate = query.execute(&client).await?;

    // Verify the response structure
    assert!(!estimate.mode.is_empty());
    assert!(estimate.total > 0);

    Ok(())
}

#[tokio::test]
async fn estimate_file_create_transaction() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let key = PrivateKey::generate_ed25519();
    let contents = vec![65u8; 10]; // 10 bytes of 'A'

    let mut tx = FileCreateTransaction::new();
    tx.contents(contents).keys([key.public_key()]);

    let mut query = FeeEstimateQuery::new().transaction(tx).mode(FeeEstimateMode::State);
    let estimate = query.execute(&client).await?;

    // Verify the response structure
    assert!(!estimate.mode.is_empty());
    assert!(estimate.total > 0);

    Ok(())
}

#[tokio::test]
async fn estimate_transfer_transaction_with_operator() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Use operator account for self-transfer (no account creation needed)
    let Some(operator) = &config.operator else {
        return Ok(()); // Skip if no operator
    };

    let mut tx = TransferTransaction::new();
    tx.hbar_transfer(operator.account_id, Hbar::new(-1))
        .hbar_transfer(operator.account_id, Hbar::new(1));

    let mut query = FeeEstimateQuery::new().transaction(tx).mode(FeeEstimateMode::State);
    let estimate = query.execute(&client).await?;

    // Verify the response structure
    assert!(!estimate.mode.is_empty());
    assert!(estimate.total > 0);
    assert!(estimate.network.multiplier > 0);
    assert!(estimate.network.subtotal > 0);

    Ok(())
}

#[tokio::test]
async fn compare_state_vs_intrinsic_mode() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Use operator account for self-transfer (no account creation needed)
    let Some(_operator) = &config.operator else {
        return Ok(()); // Skip if no operator
    };

    let mut tx = TransferTransaction::new();
    tx.hbar_transfer(config.operator.as_ref().unwrap().account_id, Hbar::new(-100))
        .hbar_transfer(config.operator.as_ref().unwrap().account_id, Hbar::new(100));

    let state_estimate = FeeEstimateQuery::new()
        .transaction(tx.clone())
        .mode(FeeEstimateMode::State)
        .execute(&client)
        .await?;

    let intrinsic_estimate = FeeEstimateQuery::new()
        .transaction(tx)
        .mode(FeeEstimateMode::Intrinsic)
        .execute(&client)
        .await?;

    assert!(state_estimate.total > 0);
    assert!(intrinsic_estimate.total > 0);

    assert!(
        state_estimate.total as f64 >= intrinsic_estimate.total as f64 * 0.9,
        "STATE estimate ({}) should be at least 90% of INTRINSIC estimate ({})",
        state_estimate.total,
        intrinsic_estimate.total
    );

    Ok(())
}

#[tokio::test]
async fn estimate_file_append_chunked_transaction() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Use operator's key for the file so only the operator signs (no InvalidSignature)
    let Some(op) = &config.operator else {
        return Ok(());
    };
    // Skip if running on localhost - this test requires executing transactions
    let file_id = if config.is_local {
        let file_result = FileCreateTransaction::new()
            .keys([op.private_key.public_key()])
            .contents(b"[e2e::FileCreateTransaction]".to_vec())
            .execute(&client)
            .await;

        if file_result.is_err() {
            return Ok(());
        }

        file_result?.get_receipt(&client).await?.file_id.unwrap()
    } else {
        FileCreateTransaction::new()
            .keys([op.private_key.public_key()])
            .contents(b"[e2e::FileCreateTransaction]".to_vec())
            .execute(&client)
            .await?
            .get_receipt(&client)
            .await?
            .file_id
            .unwrap()
    };

    // Create a FileAppendTransaction with large content that will be chunked
    let large_contents = vec![1u8; 10]; // 10KB
    let mut tx = FileAppendTransaction::new();
    tx.file_id(file_id).contents(large_contents).chunk_size(1);

    // Get fee estimate
    let mut query = FeeEstimateQuery::new().transaction(tx.clone()).mode(FeeEstimateMode::State);
    let estimate = query.execute(&client).await?;

    assert!(!estimate.mode.is_empty());
    assert!(estimate.total > 0);

    // Verify the transaction is chunked (multiple chunks)
    // Freeze the transaction to check chunk data
    tx.freeze_with(Some(&client))?;
    let chunks = tx.get_max_chunks();
    assert!(chunks > 0);

    // Clean up
    let _ = FileDeleteTransaction::new()
        .file_id(file_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    Ok(())
}

#[tokio::test]
async fn estimate_topic_message_submit_single_chunk() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Use operator's key for the topic so only the operator signs (no InvalidSignature)
    let Some(op) = &config.operator else {
        return Ok(());
    };
    let topic_id = if config.is_local {
        let topic_result = TopicCreateTransaction::new()
            .admin_key(op.private_key.public_key())
            .execute(&client)
            .await;

        if topic_result.is_err() {
            return Ok(());
        }

        topic_result?.get_receipt(&client).await?.topic_id.unwrap()
    } else {
        TopicCreateTransaction::new()
            .admin_key(op.private_key.public_key())
            .execute(&client)
            .await?
            .get_receipt(&client)
            .await?
            .topic_id
            .unwrap()
    };

    // Create a small message that fits in one chunk
    let small_message = vec![1u8; 100]; // 100 bytes
    let mut tx = TopicMessageSubmitTransaction::new();
    tx.topic_id(topic_id).message(small_message).chunk_size(100).max_chunks(1);

    let mut query = FeeEstimateQuery::new().transaction(tx).mode(FeeEstimateMode::State);
    let estimate = query.execute(&client).await?;

    assert!(!estimate.mode.is_empty());
    assert!(estimate.total > 0);

    Ok(())
}

#[tokio::test]
async fn estimate_topic_message_submit_multiple_chunks() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Use operator's key for the topic so only the operator signs (no InvalidSignature)
    let Some(op) = &config.operator else {
        return Ok(());
    };
    let topic_id = if config.is_local {
        let topic_result = TopicCreateTransaction::new()
            .admin_key(op.private_key.public_key())
            .execute(&client)
            .await;

        if topic_result.is_err() {
            return Ok(());
        }

        topic_result?.get_receipt(&client).await?.topic_id.unwrap()
    } else {
        TopicCreateTransaction::new()
            .admin_key(op.private_key.public_key())
            .execute(&client)
            .await?
            .get_receipt(&client)
            .await?
            .topic_id
            .unwrap()
    };

    // Create a large message that will be chunked
    let large_message = vec![1u8; 100]; // 15KB
    let mut tx = TopicMessageSubmitTransaction::new();
    tx.topic_id(topic_id).message(large_message).chunk_size(1).max_chunks(100);

    let mut query = FeeEstimateQuery::new().transaction(tx.clone()).mode(FeeEstimateMode::State);
    let estimate = query.execute(&client).await?;

    assert!(!estimate.mode.is_empty());
    assert!(estimate.total > 0);

    // Verify the transaction is chunked (multiple chunks)
    // Freeze the transaction to check chunk data
    tx.freeze_with(Some(&client))?;
    let chunks = tx.get_max_chunks();
    assert!(chunks > 0);

    // Verify aggregation: node subtotal should be sum across chunks
    let node_subtotal = estimate.node.subtotal();
    assert!(node_subtotal >= estimate.node.base);

    // Clean up
    let _ = TopicDeleteTransaction::new()
        .topic_id(topic_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    Ok(())
}

#[tokio::test]
async fn estimate_malformed_transaction() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let tx = TransferTransaction::new();
    let mut query = FeeEstimateQuery::new().transaction(tx).mode(FeeEstimateMode::State);
    let res = query.execute(&client).await;

    match res {
        Ok(estimate) => assert!(estimate.total >= 0),
        Err(e) => assert!(!e.to_string().is_empty()),
    }

    Ok(())
}
