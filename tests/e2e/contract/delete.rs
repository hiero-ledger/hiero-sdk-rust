use assert_matches::assert_matches;
use hiero_sdk::{
    ContractDeleteTransaction,
    ContractInfoQuery,
    Status,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};
use crate::contract::ContractAdminKey;

#[tokio::test]
async fn admin_key() -> anyhow::Result<()> {
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

    ContractDeleteTransaction::new()
        .transfer_account_id(op.account_id)
        .contract_id(contract_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let res = ContractInfoQuery::new().contract_id(contract_id).execute(&client).await?;

    assert!(res.is_deleted);

    Ok(())
}

#[tokio::test]
async fn missing_admin_key_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let contract_id = super::create_contract(&client, op.private_key.public_key(), None).await?;

    let res = ContractDeleteTransaction::new()
        .contract_id(contract_id)
        .transfer_account_id(client.get_operator_account_id().unwrap())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::ModifyingImmutableContract, .. })
    );

    Ok(())
}

#[tokio::test]
async fn missing_contract_id_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let res = ContractDeleteTransaction::new().execute(&client).await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::TransactionPreCheckStatus { status: Status::InvalidContractId, .. })
    );

    Ok(())
}
