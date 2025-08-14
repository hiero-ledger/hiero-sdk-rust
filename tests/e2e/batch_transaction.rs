use std::str::FromStr;

use hedera::{
    AccountCreateTransaction,
    AccountId,
    AccountInfoQuery,
    BatchTransaction,
    FileId,
    FreezeTransaction,
    FreezeType,
    Hbar,
    PrivateKey,
    TopicCreateTransaction,
    TopicMessageSubmitTransaction,
};
use time::{
    Duration,
    OffsetDateTime,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};
use crate::resources::BIG_CONTENTS;

#[tokio::test]
async fn can_execute_batch_transaction() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Given
    let operator_id = AccountId::new(0, 0, 2);
    let operator_key = PrivateKey::from_str(
        "302e020100300506032b65700422042091132178e72057a1d7528025956fe39b0b847f200ab59b2fdd367017f3087137"
    )?;

    let account_key = PrivateKey::generate_ed25519();

    let mut inner_transaction = AccountCreateTransaction::new();
    inner_transaction.set_key_without_alias(account_key.public_key()).initial_balance(Hbar::new(1));

    // Use batchify to prepare the transaction
    inner_transaction.batchify(&client, operator_key.public_key().into())?;

    // When / Then
    let mut batch_transaction = BatchTransaction::new();
    batch_transaction.add_inner_transaction(inner_transaction.into())?;

    client.set_operator(operator_id, operator_key);

    let tx_response = batch_transaction.execute(&client).await?;
    let _tx_receipt = tx_response.get_receipt(&client).await?;

    Ok(())
}

#[tokio::test]
async fn can_execute_large_batch_transaction() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Given
    let operator_id = AccountId::new(0, 0, 2);
    let operator_key = PrivateKey::from_str(
        "302e020100300506032b65700422042091132178e72057a1d7528025956fe39b0b847f200ab59b2fdd367017f3087137"
    )?;

    client.set_operator(operator_id, operator_key.clone());

    // When
    let mut batch_transaction = BatchTransaction::new();

    // Create 15 account creation transactions (smaller batch to test limits)
    for i in 0..15 {
        let account_key = PrivateKey::generate_ed25519();

        let mut inner_transaction = AccountCreateTransaction::new();
        inner_transaction
            .set_key_without_alias(account_key.public_key())
            .initial_balance(Hbar::new(1));

        // Use batchify to prepare the transaction
        inner_transaction.batchify(&client, operator_key.public_key().into())?;

        batch_transaction.add_inner_transaction(inner_transaction.into())?;
    }

    // Then
    let tx_response = batch_transaction.execute(&client).await?;
    let _tx_receipt = tx_response.get_receipt(&client).await?;

    // Verify we can get all inner transaction IDs
    let inner_tx_ids = batch_transaction.get_inner_transaction_ids();
    assert_eq!(inner_tx_ids.len(), 15);

    Ok(())
}

#[tokio::test]
async fn cannot_execute_batch_transaction_without_inner_transactions() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Given
    let operator_id = AccountId::new(0, 0, 2);
    let operator_key = PrivateKey::from_str(
        "302e020100300506032b65700422042091132178e72057a1d7528025956fe39b0b847f200ab59b2fdd367017f3087137"
    )?;

    client.set_operator(operator_id, operator_key);

    // When / Then
    let mut empty_batch = BatchTransaction::new();

    // Attempting to execute an empty batch transaction should fail
    let result = empty_batch.execute(&client).await;

    assert!(result.is_err(), "Expected batch transaction without inner transactions to fail");

    Ok(())
}

#[tokio::test]
async fn cannot_execute_batch_transaction_with_blacklisted_transaction() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Given
    let operator_id = AccountId::new(0, 0, 2);
    let operator_key = PrivateKey::from_str(
        "302e020100300506032b65700422042091132178e72057a1d7528025956fe39b0b847f200ab59b2fdd367017f3087137"
    )?;

    client.set_operator(operator_id, operator_key.clone());

    // Create a blacklisted transaction (FreezeTransaction)
    let mut freeze_transaction = FreezeTransaction::new();
    freeze_transaction
        .file_id(FileId::new(0, 0, 150)) // Use a default file ID
        .start_time(OffsetDateTime::now_utc() + Duration::seconds(30))
        .freeze_type(FreezeType::FreezeOnly);

    // Try to batchify the freeze transaction (this should work)
    freeze_transaction.batchify(&client, operator_key.public_key().into())?;

    // When / Then
    let mut batch_transaction = BatchTransaction::new();

    // Attempting to add a blacklisted transaction should fail
    let result = batch_transaction.add_inner_transaction(freeze_transaction.into());

    assert!(result.is_err(), "Expected adding blacklisted transaction to batch to fail");

    Ok(())
}

#[tokio::test]
#[ignore]
async fn cannot_execute_batch_transaction_with_invalid_inner_batch_key() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Given
    let operator_id = AccountId::new(0, 0, 2);
    let operator_key = PrivateKey::from_str(
        "302e020100300506032b65700422042091132178e72057a1d7528025956fe39b0b847f200ab59b2fdd367017f3087137"
    )?;

    let account_key = PrivateKey::generate_ed25519(); // Different key from operator

    // Create an inner transaction with the WRONG batch key (accountKey instead of operatorKey)
    let mut inner_transaction = AccountCreateTransaction::new();
    inner_transaction.set_key_without_alias(account_key.public_key()).initial_balance(Hbar::new(1));

    // Batchify with the wrong key - this should cause issues later
    inner_transaction.batchify(&client, account_key.public_key().into())?; // Wrong key!

    client.set_operator(operator_id, operator_key.clone());

    // When / Then
    let mut batch_transaction = BatchTransaction::new();
    batch_transaction.add_inner_transaction(inner_transaction.into())?; // This should succeed

    // Attempting to execute should fail due to batch key mismatch
    let result = batch_transaction.execute(&client).await;

    assert!(result.is_err(), "Expected batch transaction with invalid inner batch key to fail");

    Ok(())
}

#[tokio::test]
async fn cannot_execute_batch_transaction_without_batchifying_inner() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Given
    let operator_id = AccountId::new(0, 0, 2);
    let operator_key = PrivateKey::from_str(
        "302e020100300506032b65700422042091132178e72057a1d7528025956fe39b0b847f200ab59b2fdd367017f3087137"
    )?;

    let account_key = PrivateKey::generate_ed25519();

    client.set_operator(operator_id, operator_key);

    // Create an inner transaction WITHOUT batchifying it (no freeze, no batch key)
    let mut inner_transaction = AccountCreateTransaction::new();
    inner_transaction.set_key_without_alias(account_key.public_key()).initial_balance(Hbar::new(1));

    // NOTE: We deliberately do NOT call batchify() here!

    // When / Then
    let mut batch_transaction = BatchTransaction::new();

    // Attempting to add an un-batchified transaction should fail
    let result = batch_transaction.add_inner_transaction(inner_transaction.into());

    assert!(result.is_err(), "Expected adding non-batchified transaction to batch to fail");

    Ok(())
}

#[tokio::test]
async fn can_execute_batch_transaction_with_chunked_inner() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Given
    let operator_id = AccountId::new(0, 0, 2);
    let operator_key = PrivateKey::from_str(
        "302e020100300506032b65700422042091132178e72057a1d7528025956fe39b0b847f200ab59b2fdd367017f3087137"
    )?;

    client.set_operator(operator_id, operator_key.clone());

    // When - First create a topic
    let mut topic_create = TopicCreateTransaction::new();
    topic_create.admin_key(operator_key.public_key()).topic_memo("testMemo");

    let topic_response = topic_create.execute(&client).await?;
    let topic_receipt = topic_response.get_receipt(&client).await?;
    let topic_id =
        topic_receipt.topic_id.ok_or_else(|| anyhow::anyhow!("Topic ID not found in receipt"))?;

    // Create a large topic message that will be chunked
    let mut inner_transaction = TopicMessageSubmitTransaction::new();
    inner_transaction.topic_id(topic_id).message(BIG_CONTENTS.as_bytes().to_vec());

    // Batchify the large message transaction
    inner_transaction.batchify(&client, operator_key.public_key().into())?;

    // Then - Add to batch and execute
    let mut batch_transaction = BatchTransaction::new();
    batch_transaction.add_inner_transaction(inner_transaction.into())?;

    let tx_response = batch_transaction.execute(&client).await?;
    let _tx_receipt = tx_response.get_receipt(&client).await?;

    Ok(())
}

#[tokio::test]
#[ignore] // Ignored for now as we run the tests with 0.0.2 which does not incur fees
async fn batch_transaction_incurs_fees_even_if_one_inner_failed() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Given
    let operator_id = AccountId::new(0, 0, 2);
    let operator_key = PrivateKey::from_str(
        "302e020100300506032b65700422042091132178e72057a1d7528025956fe39b0b847f200ab59b2fdd367017f3087137"
    )?;

    client.set_operator(operator_id, operator_key.clone());

    // Get initial account balance
    let initial_balance = {
        let account_info = AccountInfoQuery::new().account_id(operator_id).execute(&client).await?;
        account_info.balance
    };

    // Create first inner transaction (should succeed)
    let account_key1 = PrivateKey::generate_ed25519();
    let mut inner_transaction1 = AccountCreateTransaction::new();
    inner_transaction1
        .set_key_without_alias(account_key1.public_key())
        .initial_balance(Hbar::new(1));
    inner_transaction1.batchify(&client, operator_key.public_key().into())?;

    // Create second inner transaction (should fail due to receiver signature required)
    let account_key2 = PrivateKey::generate_ed25519();
    let mut inner_transaction2 = AccountCreateTransaction::new();
    inner_transaction2
        .set_key_without_alias(account_key2.public_key())
        .initial_balance(Hbar::new(1))
        .receiver_signature_required(true); // This will cause failure
    inner_transaction2.batchify(&client, operator_key.public_key().into())?;

    // When
    let mut batch_transaction = BatchTransaction::new();
    batch_transaction
        .set_inner_transactions(vec![inner_transaction1.into(), inner_transaction2.into()])?;

    let tx_response = batch_transaction.execute(&client).await?;

    // Expect the receipt to fail due to the second transaction requiring receiver signature
    let receipt_result = tx_response.get_receipt(&client).await;
    assert!(
        receipt_result.is_err(),
        "Expected batch transaction receipt to fail due to receiver signature requirement"
    );

    // Then - Check that fees were still charged despite the failure
    let final_balance = {
        let account_info = AccountInfoQuery::new().account_id(operator_id).execute(&client).await?;
        account_info.balance
    };

    assert!(
        final_balance < initial_balance,
        "Expected final balance ({}) to be less than initial balance ({}) due to fees",
        final_balance.to_tinybars(),
        initial_balance.to_tinybars()
    );

    Ok(())
}
