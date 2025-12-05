use hiero_sdk::{
    AccountCreateTransaction,
    AccountInfoQuery,
    Hbar,
    Key,
    PrivateKey,
    TransactionId,
    TransferTransaction,
};
use time::{
    Duration,
    OffsetDateTime,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

#[tokio::test]
async fn initial_balance_and_key() -> anyhow::Result<()> {
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

    let info = AccountInfoQuery::new().account_id(account_id).execute(&client).await?;

    assert_eq!(info.account_id, account_id);
    assert!(!info.is_deleted);
    assert_eq!(info.key, Key::Single(key.public_key()));
    assert_eq!(info.balance, Hbar::new(1));
    assert_eq!(info.auto_renew_period, Some(Duration::days(90)));
    assert_eq!(info.proxy_received, Hbar::ZERO);

    Ok(())
}

#[tokio::test]
async fn no_initial_balance() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let key = PrivateKey::generate_ed25519();

    let receipt = AccountCreateTransaction::new()
        .set_key_without_alias(key.public_key())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let account_id = receipt.account_id.unwrap();

    let info = AccountInfoQuery::new().account_id(account_id).execute(&client).await?;

    assert_eq!(info.account_id, account_id);
    assert!(!info.is_deleted);
    assert_eq!(info.key, Key::Single(key.public_key()));
    assert_eq!(info.balance, Hbar::ZERO);
    assert_eq!(info.auto_renew_period, Some(Duration::days(90)));
    assert_eq!(info.proxy_received, Hbar::ZERO);

    Ok(())
}

#[tokio::test]
async fn missing_key_error() {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return;
    };

    let res = AccountCreateTransaction::new().initial_balance(Hbar::new(1)).execute(&client).await;

    assert_matches::assert_matches!(
        res,
        Err(hiero_sdk::Error::TransactionPreCheckStatus { status: hiero_sdk::Status::KeyRequired, .. })
    );
}

#[tokio::test]
async fn alias_key() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let key = PrivateKey::generate_ed25519();

    let alias_id = key.to_account_id(0, 0);

    TransferTransaction::new()
        .hbar_transfer(op.account_id, Hbar::new(-1))
        .hbar_transfer(alias_id, Hbar::new(1))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info = AccountInfoQuery::new().account_id(alias_id).execute(&client).await?;

    assert_eq!(info.alias_key, Some(key.public_key()));

    Ok(())
}

#[tokio::test]
#[ignore = "Explicit disagreement between Java and Rust SDKs"]
async fn manages_expiration() -> anyhow::Result<()> {
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
        .transaction_id(TransactionId {
            account_id: op.account_id,
            valid_start: OffsetDateTime::now_utc() - Duration::seconds(40),
            nonce: None,
            scheduled: false,
        })
        .transaction_valid_duration(Duration::seconds(30))
        .freeze_with(&client)?
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let account_id = receipt.account_id.unwrap();

    let info = AccountInfoQuery::new().account_id(account_id).execute(&client).await?;

    assert_eq!(info.account_id, account_id);
    assert!(!info.is_deleted);
    assert_eq!(info.key, Key::Single(key.public_key()));
    assert_eq!(info.balance, Hbar::ZERO);
    assert_eq!(info.auto_renew_period, Some(Duration::days(90)));

    #[allow(deprecated)]
    {
        assert_eq!(info.proxy_account_id, None);
    }
    assert_eq!(info.proxy_received, Hbar::ZERO);

    Ok(())
}

#[tokio::test]
async fn alias_from_admin_key() -> anyhow::Result<()> {
    // Tests the third row of this table
    // https://github.com/hashgraph/hedera-improvement-proposal/blob/d39f740021d7da592524cffeaf1d749803798e9a/HIP/hip-583.md#signatures
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let admin_key = PrivateKey::generate_ecdsa();
    let evm_address = admin_key.public_key().to_evm_address().unwrap();

    let account_id = AccountCreateTransaction::new()
        .set_key_without_alias(admin_key.public_key())
        .alias(evm_address)
        .freeze_with(&client)?
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    let info = AccountInfoQuery::new().account_id(account_id).execute(&client).await?;

    assert_eq!(info.account_id, account_id);
    assert_eq!(info.contract_account_id, hex::encode(evm_address.to_bytes()));
    assert_eq!(info.key, Key::Single(admin_key.public_key()));

    Ok(())
}

#[tokio::test]
async fn alias_from_admin_key_with_receiver_sig_required() -> anyhow::Result<()> {
    // Tests the fourth row of this table
    // https://github.com/hashgraph/hedera-improvement-proposal/blob/d39f740021d7da592524cffeaf1d749803798e9a/HIP/hip-583.md#signatures
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let admin_key = PrivateKey::generate_ecdsa();
    let evm_address = admin_key.public_key().to_evm_address().unwrap();

    let account_id = AccountCreateTransaction::new()
        .receiver_signature_required(true)
        .set_key_without_alias(admin_key.public_key())
        .alias(evm_address)
        .freeze_with(&client)?
        .sign(admin_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    let info = AccountInfoQuery::new().account_id(account_id).execute(&client).await?;

    assert_eq!(info.account_id, account_id);
    assert_eq!(info.contract_account_id, hex::encode(evm_address.to_bytes()));
    assert_eq!(info.key, Key::Single(admin_key.public_key()));

    Ok(())
}

#[tokio::test]
async fn alias_from_admin_key_with_receiver_sig_required_and_no_signature_errors(
) -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let admin_key = PrivateKey::generate_ecdsa();
    let evm_address = admin_key.public_key().to_evm_address().unwrap();

    let res = AccountCreateTransaction::new()
        .receiver_signature_required(true)
        .set_key_without_alias(admin_key.public_key())
        .alias(evm_address)
        .freeze_with(&client)?
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches::assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: hiero_sdk::Status::InvalidSignature, .. })
    );

    Ok(())
}

#[tokio::test]
async fn alias() -> anyhow::Result<()> {
    // Tests the fifth row of this table
    // https://github.com/hashgraph/hedera-improvement-proposal/blob/d39f740021d7da592524cffeaf1d749803798e9a/HIP/hip-583.md#signatures
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let admin_key = PrivateKey::generate_ed25519();

    let key = PrivateKey::generate_ecdsa();
    let evm_address = key.public_key().to_evm_address().unwrap();

    let account_id = AccountCreateTransaction::new()
        .set_key_without_alias(admin_key.public_key())
        .alias(evm_address)
        .freeze_with(&client)?
        .sign(key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    let info = AccountInfoQuery::new().account_id(account_id).execute(&client).await?;

    assert_eq!(info.account_id, account_id);

    assert_eq!(info.contract_account_id, hex::encode(evm_address.to_bytes()));
    assert_eq!(info.key, Key::Single(admin_key.public_key()));

    Ok(())
}

#[tokio::test]
async fn alias_missing_signature_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let admin_key = PrivateKey::generate_ed25519();

    let key = PrivateKey::generate_ecdsa();
    let evm_address = key.public_key().to_evm_address().unwrap();

    let res = AccountCreateTransaction::new()
        .set_key_without_alias(admin_key.public_key())
        .alias(evm_address)
        .freeze_with(&client)?
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches::assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: hiero_sdk::Status::InvalidSignature, .. })
    );

    Ok(())
}

#[tokio::test]
async fn alias_with_receiver_sig_required() -> anyhow::Result<()> {
    // Tests the sixth row of this table
    // https://github.com/hashgraph/hedera-improvement-proposal/blob/d39f740021d7da592524cffeaf1d749803798e9a/HIP/hip-583.md#signatures
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let admin_key = PrivateKey::generate_ed25519();

    let key = PrivateKey::generate_ecdsa();
    let evm_address = key.public_key().to_evm_address().unwrap();

    let account_id = AccountCreateTransaction::new()
        .receiver_signature_required(true)
        .set_key_without_alias(admin_key.public_key())
        .alias(evm_address)
        .freeze_with(&client)?
        .sign(key)
        .sign(admin_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    let info = AccountInfoQuery::new().account_id(account_id).execute(&client).await?;

    assert_eq!(info.account_id, account_id);

    assert_eq!(info.contract_account_id, hex::encode(evm_address.to_bytes()));
    assert_eq!(info.key, Key::Single(admin_key.public_key()));

    Ok(())
}

#[tokio::test]
async fn alias_with_receiver_sig_required_missing_signature_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let admin_key = PrivateKey::generate_ed25519();

    let key = PrivateKey::generate_ecdsa();
    let evm_address = key.public_key().to_evm_address().unwrap();

    let res = AccountCreateTransaction::new()
        .receiver_signature_required(true)
        .set_key_without_alias(admin_key.public_key())
        .alias(evm_address)
        .freeze_with(&client)?
        .sign(key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches::assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: hiero_sdk::Status::InvalidSignature, .. })
    );

    Ok(())
}

#[tokio::test]
async fn cannot_create_account_with_invalid_negative_max_auto_token_assocation(
) -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let key = PrivateKey::generate_ed25519();

    let res = AccountCreateTransaction::new()
        .set_key_without_alias(key.public_key())
        .max_automatic_token_associations(-2)
        .execute(&client)
        .await;

    assert_matches::assert_matches!(
        res,
        Err(hiero_sdk::Error::TransactionPreCheckStatus {
            status: hiero_sdk::Status::InvalidMaxAutoAssociations,
            ..
        })
    );

    Ok(())
}
