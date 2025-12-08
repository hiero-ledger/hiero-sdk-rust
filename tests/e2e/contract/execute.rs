use assert_matches::assert_matches;
use hiero_sdk::{
    ContractDeleteTransaction,
    ContractExecuteTransaction,
    ContractFunctionParameters,
    Status,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};
use crate::contract::ContractAdminKey;

#[tokio::test]
async fn basic() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let contract_id =
        super::create_contract(&client, op.private_key.public_key(), ContractAdminKey::Operator)
            .await?;

    let _ = ContractExecuteTransaction::new()
        .contract_id(contract_id)
        .gas(200_000)
        .function_with_parameters(
            "setMessage",
            ContractFunctionParameters::new().add_string("new message"),
        )
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

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
async fn missing_contract_id_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let res = ContractExecuteTransaction::new()
        .gas(200_000)
        .function_with_parameters(
            "setMessage",
            ContractFunctionParameters::new().add_string("new message"),
        )
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::TransactionPreCheckStatus { status: Status::InvalidContractId, .. })
    );

    Ok(())
}

#[tokio::test]
async fn missing_function_parameters_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let contract_id =
        super::create_contract(&client, op.private_key.public_key(), ContractAdminKey::Operator)
            .await?;

    let res = ContractExecuteTransaction::new()
        .contract_id(contract_id)
        .gas(200_000)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::ContractRevertExecuted, .. })
    );

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
async fn missing_gas_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let contract_id =
        super::create_contract(&client, op.private_key.public_key(), ContractAdminKey::Operator)
            .await?;

    let res = ContractExecuteTransaction::new()
        .contract_id(contract_id)
        .function_with_parameters(
            "setMessage",
            ContractFunctionParameters::new().add_string("new message"),
        )
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::TransactionPreCheckStatus { status: Status::InsufficientGas, .. })
    );

    ContractDeleteTransaction::new()
        .transfer_account_id(op.account_id)
        .contract_id(contract_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}
