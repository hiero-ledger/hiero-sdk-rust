use assert_matches::assert_matches;
use hiero_sdk::{
    ContractDeleteTransaction,
    ContractInfoQuery,
    ContractUpdateTransaction,
    Key,
    Status,
};

use super::ContractAdminKey;
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

    let contract_id =
        super::create_contract(&client, op.private_key.public_key(), ContractAdminKey::Operator)
            .await?;

    ContractUpdateTransaction::new()
        .contract_id(contract_id)
        .contract_memo("[e2e::ContractUpdateTransaction]")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info = ContractInfoQuery::new().contract_id(contract_id).execute(&client).await?;

    assert_eq!(info.contract_id, contract_id);
    assert_eq!(info.account_id.to_string(), info.contract_id.to_string());
    assert_eq!(info.admin_key, Some(Key::Single(op.private_key.public_key())));
    assert_eq!(info.storage, 128);
    assert_eq!(info.contract_memo, "[e2e::ContractUpdateTransaction]");

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

    let res = ContractUpdateTransaction::new()
        .contract_memo("[e2e::ContractUpdateTransaction]")
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::TransactionPreCheckStatus { status: Status::InvalidContractId, .. })
    );

    Ok(())
}

#[tokio::test]
async fn immutable_contract_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let contract_id = super::create_contract(&client, op.private_key.public_key(), None).await?;

    let res = ContractUpdateTransaction::new()
        .contract_id(contract_id)
        .contract_memo("[e2e::ContractUpdateTransaction]")
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
