use assert_matches::assert_matches;
use hedera::{
    ContractCreateTransaction,
    ContractDeleteTransaction,
    ContractFunctionParameters,
    ContractInfoQuery,
    Key,
    Status,
};

use super::bytecode_file_id;
use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

#[tokio::test]
async fn basic() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let file_id = bytecode_file_id(&client, op.private_key.public_key()).await?;

    let contract_id = ContractCreateTransaction::new()
        .admin_key(op.private_key.public_key())
        .gas(2000000)
        .constructor_parameters(
            ContractFunctionParameters::new().add_string("Hello from Hedera.").to_bytes(None),
        )
        .bytecode_file_id(file_id)
        .contract_memo("[e2e::ContractCreateTransaction]")
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
    assert_eq!(info.contract_memo, "[e2e::ContractCreateTransaction]");

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
async fn no_admin_key() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let file_id = bytecode_file_id(&client, op.private_key.public_key()).await?;

    let contract_id = ContractCreateTransaction::new()
        .gas(2000000)
        .constructor_parameters(
            ContractFunctionParameters::new().add_string("Hello from Hedera.").to_bytes(None),
        )
        .bytecode_file_id(file_id)
        .contract_memo("[e2e::ContractCreateTransaction]")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .contract_id
        .unwrap();

    let info = ContractInfoQuery::new().contract_id(contract_id).execute(&client).await?;

    assert_eq!(info.contract_id, contract_id);
    assert_eq!(info.account_id.to_string(), info.contract_id.to_string());
    assert!(info.admin_key.is_some());
    assert_eq!(info.storage, 128);
    assert_eq!(info.contract_memo, "[e2e::ContractCreateTransaction]");

    Ok(())
}

#[tokio::test]
async fn unset_gas_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let file_id = bytecode_file_id(&client, op.private_key.public_key()).await?;

    let res = ContractCreateTransaction::new()
        .constructor_parameters(
            ContractFunctionParameters::new().add_string("Hello from Hedera.").to_bytes(None),
        )
        .bytecode_file_id(file_id)
        .contract_memo("[e2e::ContractCreateTransaction]")
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::TransactionPreCheckStatus { status: Status::InsufficientGas, .. })
    );

    Ok(())
}

#[tokio::test]
async fn constructor_parameters_unset_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let file_id = bytecode_file_id(&client, op.private_key.public_key()).await?;

    let res = ContractCreateTransaction::new()
        .gas(200_000)
        .bytecode_file_id(file_id)
        .contract_memo("[e2e::ContractCreateTransaction]")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::ContractRevertExecuted, .. })
    );

    Ok(())
}

#[tokio::test]
async fn bytecode_file_id_unset_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let res = ContractCreateTransaction::new()
        .gas(200_000)
        .constructor_parameters(
            ContractFunctionParameters::new().add_string("Hello from Hedera.").to_bytes(None),
        )
        .contract_memo("[e2e::ContractCreateTransaction]")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(res, Err(hedera::Error::ReceiptStatus { status: Status::InvalidFileId, .. }));

    Ok(())
}
