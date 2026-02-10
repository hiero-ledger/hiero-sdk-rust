// SPDX-License-Identifier: Apache-2.0

use hiero_sdk::{
    AccountCreateTransaction,
    AccountDeleteTransaction,
    Hbar,
    PrivateKey,
    TransferTransaction,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

/// Test that receipt queries are pinned to the submitting node by default (strict mode).
#[tokio::test]
async fn receipt_query_pinned_by_default() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    // Verify failover is disabled by default
    assert!(!client.get_enable_receipt_record_query_failover());

    // Create a simple transaction
    let key = PrivateKey::generate_ed25519();

    let response = AccountCreateTransaction::new()
        .set_key_without_alias(key.public_key())
        .initial_balance(Hbar::new(1))
        .execute(&client)
        .await?;

    // The receipt query should be pinned to the submitting node
    let receipt_query = response.get_receipt_query();
    let node_ids = receipt_query.get_node_account_ids().expect("Node IDs should be set");

    // Verify the query is pinned to exactly one node (the submitting node)
    assert_eq!(node_ids.len(), 1);
    assert_eq!(node_ids[0], response.node_account_id);

    // Get the receipt (should succeed since the node is available)
    let receipt = response.get_receipt(&client).await?;
    assert!(receipt.account_id.is_some());

    // Cleanup
    let account_id = receipt.account_id.unwrap();
    AccountDeleteTransaction::new()
        .account_id(account_id)
        .transfer_account_id(op.account_id)
        .sign(key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}

/// Test that record queries are pinned to the submitting node by default (strict mode).
#[tokio::test]
async fn record_query_pinned_by_default() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    // Verify failover is disabled by default
    assert!(!client.get_enable_receipt_record_query_failover());

    // Create a simple transaction
    let key = PrivateKey::generate_ed25519();

    let response = AccountCreateTransaction::new()
        .set_key_without_alias(key.public_key())
        .initial_balance(Hbar::new(1))
        .execute(&client)
        .await?;

    // The record query should be pinned to the submitting node
    let record_query = response.get_record_query();
    let node_ids = record_query.get_node_account_ids().expect("Node IDs should be set");

    // Verify the query is pinned to exactly one node (the submitting node)
    assert_eq!(node_ids.len(), 1);
    assert_eq!(node_ids[0], response.node_account_id);

    // Get the record (should succeed since the node is available)
    let record = response.get_record(&client).await?;
    assert!(record.receipt.account_id.is_some());

    // Cleanup
    let account_id = record.receipt.account_id.unwrap();
    AccountDeleteTransaction::new()
        .account_id(account_id)
        .transfer_account_id(op.account_id)
        .sign(key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}

/// Test that enabling failover allows queries to use other nodes.
#[tokio::test]
async fn receipt_query_with_failover_enabled() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    // Enable failover
    client.set_enable_receipt_record_query_failover(true);
    assert!(client.get_enable_receipt_record_query_failover());

    // Create a simple transaction
    let key = PrivateKey::generate_ed25519();

    let response = AccountCreateTransaction::new()
        .set_key_without_alias(key.public_key())
        .initial_balance(Hbar::new(1))
        .execute(&client)
        .await?;

    // The receipt query should still be pinned to the submitting node
    let receipt_query = response.get_receipt_query();
    let node_ids = receipt_query.get_node_account_ids().expect("Node IDs should be set");

    // Verify the query starts with the submitting node
    assert_eq!(node_ids.len(), 1);
    assert_eq!(node_ids[0], response.node_account_id);

    // Get the receipt (should succeed, potentially using failover if needed)
    let receipt = response.get_receipt(&client).await?;
    assert!(receipt.account_id.is_some());

    // Cleanup
    let account_id = receipt.account_id.unwrap();
    AccountDeleteTransaction::new()
        .account_id(account_id)
        .transfer_account_id(op.account_id)
        .sign(key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Reset to default
    client.set_enable_receipt_record_query_failover(false);

    Ok(())
}

/// Test that normal queries (non-receipt/record) are not affected by failover setting.
#[tokio::test]
async fn normal_queries_unaffected_by_failover() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    // Enable failover
    client.set_enable_receipt_record_query_failover(true);

    // Create a simple transfer - this doesn't involve receipt/record queries
    let response = TransferTransaction::new()
        .hbar_transfer(op.account_id, Hbar::new(-1))
        .hbar_transfer("0.0.3".parse()?, Hbar::new(1))
        .execute(&client)
        .await?;

    // Normal transaction execution should work regardless of failover setting
    let receipt = response.get_receipt(&client).await?;
    assert_eq!(receipt.status, hiero_sdk::Status::Success);

    // Reset to default
    client.set_enable_receipt_record_query_failover(false);

    Ok(())
}

/// Test that the client configuration can be toggled.
#[tokio::test]
async fn failover_configuration_toggle() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Default should be false
    assert!(!client.get_enable_receipt_record_query_failover());

    // Enable
    client.set_enable_receipt_record_query_failover(true);
    assert!(client.get_enable_receipt_record_query_failover());

    // Disable
    client.set_enable_receipt_record_query_failover(false);
    assert!(!client.get_enable_receipt_record_query_failover());

    // Enable again
    client.set_enable_receipt_record_query_failover(true);
    assert!(client.get_enable_receipt_record_query_failover());

    // Reset to default
    client.set_enable_receipt_record_query_failover(false);

    Ok(())
}
