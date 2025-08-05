use assert_matches::assert_matches;
use hedera::{
    ContractDeleteTransaction,
    ContractInfoQuery,
    Hbar,
    Key,
    Status,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};
use crate::contract::ContractAdminKey;

#[tokio::test]
async fn query() -> anyhow::Result<()> {
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

    // assertThat(contractId.hashCode()).isGreaterThan(0);
    // assertThat(contractId.compareTo(ContractId.fromBytes(contractId.toBytes()))).isZero();

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
async fn query_no_admin_key() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let contract_id = super::create_contract(&client, op.private_key.public_key(), None).await?;

    let info = ContractInfoQuery::new().contract_id(contract_id).execute(&client).await?;

    assert_eq!(info.contract_id, contract_id);
    assert_eq!(info.account_id.to_string(), info.contract_id.to_string());
    // TODO: Fix this when we know it's correct
    // assertEquals(info.adminKey, contractId);
    assert!(info.admin_key.is_some());
    assert_eq!(info.storage, 128);
    assert_eq!(info.contract_memo, "[e2e::ContractCreateTransaction]");

    Ok(())
}

#[tokio::test]
async fn missing_contract_id_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let res = ContractInfoQuery::new().execute(&client).await;

    assert_matches!(
        res,
        Err(hedera::Error::QueryNoPaymentPreCheckStatus { status: Status::InvalidContractId })
    );

    Ok(())
}

#[tokio::test]
async fn query_cost_big_max() -> anyhow::Result<()> {
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

    let mut query = ContractInfoQuery::new();

    query.contract_id(contract_id).max_payment_amount(Hbar::new(10000));

    let cost = query.get_cost(&client).await?;

    let _info = query.payment_amount(cost).execute(&client).await?;

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
async fn query_cost_small_max_fails() -> anyhow::Result<()> {
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

    let mut query = ContractInfoQuery::new();

    query.contract_id(contract_id).max_payment_amount(Hbar::from_tinybars(1));

    let cost = query.get_cost(&client).await?;

    let res = query.execute(&client).await;

    let (max_query_payment, query_cost) = assert_matches!(
        res,
        Err(hedera::Error::MaxQueryPaymentExceeded {
            max_query_payment,
            query_cost
        }) => (max_query_payment, query_cost)
    );

    assert_eq!(max_query_payment, Hbar::from_tinybars(1));
    // note: there's a very small chance this fails if the cost of a AccountInfoQuery changes right when we execute it.
    assert_eq!(query_cost, cost);

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
async fn query_cost_insufficient_tx_fee_fails() -> anyhow::Result<()> {
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

    let res = ContractInfoQuery::new()
        .contract_id(contract_id)
        .max_payment_amount(Hbar::from_tinybars(10000))
        .payment_amount(Hbar::from_tinybars(1))
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::QueryPaymentPreCheckStatus { status: Status::InsufficientTxFee, .. })
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
