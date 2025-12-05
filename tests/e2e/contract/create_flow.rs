use assert_matches::assert_matches;
use hiero_sdk::{
    ContractCreateFlow,
    ContractDeleteTransaction,
    ContractFunctionParameters,
    ContractInfoQuery,
    Key,
    PrivateKey,
    Status,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};
use crate::contract::SMART_CONTRACT_BYTECODE;

#[tokio::test]
async fn basic() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let contract_id = ContractCreateFlow::new()
        .bytecode_hex(SMART_CONTRACT_BYTECODE)?
        .admin_key(op.private_key.public_key())
        .gas(2000000)
        .constructor_parameters(
            ContractFunctionParameters::new().add_string("Hello from Hedera.").to_bytes(None),
        )
        .contract_memo("[e2e::ContractCreateFlow]".to_owned())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .contract_id
        .unwrap();

    let info = ContractInfoQuery::new().contract_id(contract_id).execute(&client).await?;

    assert_eq!(info.contract_id, contract_id);
    assert_eq!(info.account_id.to_string(), info.contract_id.to_string());
    assert_eq!(info.admin_key, Some(Key::Single(op.private_key.public_key())));
    assert_eq!(info.storage, 128);
    assert_eq!(info.contract_memo, "[e2e::ContractCreateFlow]");

    ContractDeleteTransaction::new()
        .transfer_account_id(op.account_id)
        .contract_id(contract_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}

#[tokio::test]
async fn admin_key_missing_signature_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let admin_key = PrivateKey::generate_ed25519();

    let res = ContractCreateFlow::new()
        .bytecode_hex(SMART_CONTRACT_BYTECODE)?
        .admin_key(admin_key.public_key())
        .gas(200_000)
        .constructor_parameters(
            ContractFunctionParameters::new().add_string("Hello from Hedera.").to_bytes(None),
        )
        .contract_memo("[e2e::ContractCreateFlow]".to_owned())
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, .. })
    );

    Ok(())
}

#[tokio::test]
async fn admin_key() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let admin_key = PrivateKey::generate_ed25519();
    let contract_id = ContractCreateFlow::new()
        .bytecode_hex(SMART_CONTRACT_BYTECODE)?
        .admin_key(admin_key.public_key())
        .gas(2000000)
        .constructor_parameters(
            ContractFunctionParameters::new().add_string("Hello from Hedera.").to_bytes(None),
        )
        .contract_memo("[e2e::ContractCreateFlow]".to_owned())
        .sign(admin_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .contract_id
        .unwrap();

    let info = ContractInfoQuery::new().contract_id(contract_id).execute(&client).await?;

    assert_eq!(info.contract_id, contract_id);
    assert_eq!(info.account_id.to_string(), info.contract_id.to_string());
    assert_eq!(info.admin_key, Some(Key::Single(admin_key.public_key())));
    assert_eq!(info.storage, 128);
    assert_eq!(info.contract_memo, "[e2e::ContractCreateFlow]");

    ContractDeleteTransaction::new()
        .transfer_account_id(op.account_id)
        .contract_id(contract_id)
        .sign(admin_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}

#[tokio::test]
async fn admin_key_sign_with() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let admin_key = PrivateKey::generate_ed25519();
    let contract_id = ContractCreateFlow::new()
        .bytecode_hex(SMART_CONTRACT_BYTECODE)?
        .admin_key(admin_key.public_key())
        .gas(2000000)
        .constructor_parameters(
            ContractFunctionParameters::new().add_string("Hello from Hedera.").to_bytes(None),
        )
        .contract_memo("[e2e::ContractCreateFlow]".to_owned())
        .sign_with(admin_key.public_key(), {
            let admin_key = admin_key.clone();
            move |message| admin_key.sign(message)
        })
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .contract_id
        .unwrap();

    let info = ContractInfoQuery::new().contract_id(contract_id).execute(&client).await?;

    assert_eq!(info.contract_id, contract_id);
    assert_eq!(info.account_id.to_string(), info.contract_id.to_string());
    assert_eq!(info.admin_key, Some(Key::Single(admin_key.public_key())));
    assert_eq!(info.storage, 128);
    assert_eq!(info.contract_memo, "[e2e::ContractCreateFlow]");

    ContractDeleteTransaction::new()
        .transfer_account_id(op.account_id)
        .contract_id(contract_id)
        .sign(admin_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}
