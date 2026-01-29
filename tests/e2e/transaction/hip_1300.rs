use std::str::FromStr;

use hiero_sdk::{
    AccountCreateTransaction,
    AccountId,
    Error,
    FileCreateTransaction,
    Hbar,
    PrivateKey,
    Status,
};

use crate::common::setup_nonfree;

const MAXIMUM_TRANSACTION_SIZE: usize = 130000;

#[tokio::test]
async fn create_transaction_with_more_than_6kbs_of_data_with_signatures() -> anyhow::Result<()> {
    let Some(env) = setup_nonfree() else { return Ok(()) };
    let client = env.client;

    let system_account_id = AccountId::from_str("0.0.2")?;
    let system_account_key = PrivateKey::from_str(
        "302e020100300506032b65700422042091132178e72057a1d7528025956fe39b0b847f200ab59b2fdd367017f3087137",
    )?;

    client.set_operator(system_account_id, system_account_key);

    let mut transaction = AccountCreateTransaction::new();
    transaction
        .set_key_without_alias(PrivateKey::generate_ed25519().public_key())
        .freeze_with(&client)?;

    while transaction.to_bytes()?.len() < MAXIMUM_TRANSACTION_SIZE {
        transaction.sign(PrivateKey::generate_ed25519());
    }

    transaction.execute(&client).await?.get_receipt(&client).await?;

    Ok(())
}

#[tokio::test]
async fn create_transaction_with_more_than_6kbs_of_data_in_a_file() -> anyhow::Result<()> {
    let Some(env) = setup_nonfree() else { return Ok(()) };
    let client = env.client;

    // We need to set the system operator here as well, as per the JS test implication
    // that these tests run in a suite where system operator might be set.
    // However, in Rust tests are independent.
    // The test name implies "should create ...", so it likely needs privileges.
    let system_account_id = AccountId::from_str("0.0.2")?;
    let system_account_key = PrivateKey::from_str(
        "302e020100300506032b65700422042091132178e72057a1d7528025956fe39b0b847f200ab59b2fdd367017f3087137",
    )?;
    client.set_operator(system_account_id, system_account_key);

    let mut file = FileCreateTransaction::new();
    file.contents(vec![1; 1024 * 10]).freeze_with(&client)?;

    file.execute(&client).await?.get_receipt(&client).await?;

    Ok(())
}

#[tokio::test]
async fn should_not_create_transaction_with_more_than_6kbs_of_data_in_a_file_if_normal_account_is_used(
) -> anyhow::Result<()> {
    let Some(env) = setup_nonfree() else { return Ok(()) };
    let client = env.client;

    let regular_user_key = PrivateKey::generate_ed25519();
    let initial_regular_balance = Hbar::new(10);

    let regular_account_id = AccountCreateTransaction::new()
        .set_key_without_alias(regular_user_key.public_key())
        .initial_balance(initial_regular_balance)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    client.set_operator(regular_account_id, regular_user_key);

    let mut transaction = FileCreateTransaction::new();
    transaction.contents(vec![1; 1024 * 10]).freeze_with(&client)?;

    let result = transaction.execute(&client).await;

    match result {
        Ok(response) => match response.get_receipt(&client).await {
            Ok(_) => anyhow::bail!("Transaction should have failed"),
            Err(Error::ReceiptStatus { status: Status::TransactionOversize, .. }) => {}
            Err(e) => anyhow::bail!("Expected TRANSACTION_OVERSIZE error, got: {:?}", e),
        },
        Err(Error::TransactionPreCheckStatus { status: Status::TransactionOversize, .. }) => {}
        Err(e) => anyhow::bail!("Expected TRANSACTION_OVERSIZE error, got: {:?}", e),
    }

    Ok(())
}

#[tokio::test]
async fn should_not_create_transaction_with_more_than_6kbs_of_data_with_signatures_without_system_account(
) -> anyhow::Result<()> {
    let Some(env) = setup_nonfree() else { return Ok(()) };
    let client = env.client;

    let regular_user_key = PrivateKey::generate_ed25519();
    let initial_regular_balance = Hbar::new(10);

    let regular_account_id = AccountCreateTransaction::new()
        .set_key_without_alias(regular_user_key.public_key())
        .initial_balance(initial_regular_balance)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    client.set_operator(regular_account_id, regular_user_key.clone());

    let mut transaction = AccountCreateTransaction::new();
    transaction.set_key_without_alias(regular_user_key.public_key()).freeze_with(&client)?;

    while transaction.to_bytes()?.len() < MAXIMUM_TRANSACTION_SIZE {
        transaction.sign(PrivateKey::generate_ed25519());
    }

    let result = transaction.execute(&client).await;

    match result {
        Ok(response) => match response.get_receipt(&client).await {
            Ok(_) => anyhow::bail!("Transaction should have failed"),
            Err(Error::ReceiptStatus { status: Status::TransactionOversize, .. }) => {}
            Err(e) => anyhow::bail!("Expected TRANSACTION_OVERSIZE error, got: {:?}", e),
        },
        Err(Error::TransactionPreCheckStatus { status: Status::TransactionOversize, .. }) => {}
        Err(e) => anyhow::bail!("Expected TRANSACTION_OVERSIZE error, got: {:?}", e),
    }

    Ok(())
}
