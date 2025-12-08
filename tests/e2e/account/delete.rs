use assert_matches::assert_matches;
use hiero_sdk::{
    AccountCreateTransaction,
    AccountDeleteTransaction,
    AccountInfoQuery,
    Hbar,
    PrivateKey,
    Status,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

#[tokio::test]
async fn create_then_delete() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let key = PrivateKey::generate_ed25519();

    let receipt = AccountCreateTransaction::new()
        .set_key_without_alias(key.public_key())
        .initial_balance(Hbar::new(1))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let account_id = receipt.account_id.unwrap();

    AccountDeleteTransaction::new()
        .transfer_account_id(client.get_operator_account_id().unwrap())
        .account_id(account_id)
        .sign(key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let res = AccountInfoQuery::new().account_id(account_id).execute(&client).await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::QueryNoPaymentPreCheckStatus { status: Status::AccountDeleted })
    );

    Ok(())
}

#[tokio::test]
async fn missing_account_id_fails() {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return;
    };

    let res = AccountDeleteTransaction::new()
        .transfer_account_id(client.get_operator_account_id().unwrap())
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::TransactionPreCheckStatus {
            status: Status::AccountIdDoesNotExist,
            ..
        })
    );
}

#[tokio::test]
async fn missing_deletee_signature_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let key = PrivateKey::generate_ed25519();

    let receipt = AccountCreateTransaction::new()
        .set_key_without_alias(key.public_key())
        .initial_balance(Hbar::new(1))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let account_id = receipt.account_id.unwrap();

    let res = AccountDeleteTransaction::new()
        .transfer_account_id(op.account_id)
        .account_id(account_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, .. })
    );

    Ok(())
}
