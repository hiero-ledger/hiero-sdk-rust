use hiero_sdk::{
    AccountCreateTransaction,
    FeeEstimateMode,
    FeeEstimateQuery,
    Hbar,
    PrivateKey,
    TransferTransaction,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

#[tokio::test]
#[ignore = "HIP-1261: requires mirror node with fee estimate endpoint"]
async fn transfer_transaction_intrinsic_mode() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_id = config.operator.as_ref().unwrap().account_id;

    let mut tx = TransferTransaction::new();
    tx.hbar_transfer(operator_id, Hbar::new(-1))
        .hbar_transfer("0.0.3".parse()?, Hbar::new(1));

    let response = FeeEstimateQuery::new()
        .set_mode(FeeEstimateMode::Intrinsic)
        .set_transaction(&mut tx, &client)?
        .execute(&client)
        .await?;

    assert!(response.total > 0, "expected non-zero total fee estimate");
    assert!(response.node.is_some(), "expected node fee component");
    assert!(response.service.is_some(), "expected service fee component");
    assert!(response.network.is_some(), "expected network fee component");

    Ok(())
}

#[tokio::test]
#[ignore = "HIP-1261: requires mirror node with fee estimate endpoint"]
async fn transfer_transaction_state_mode() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_id = config.operator.as_ref().unwrap().account_id;

    let mut tx = TransferTransaction::new();
    tx.hbar_transfer(operator_id, Hbar::new(-1))
        .hbar_transfer("0.0.3".parse()?, Hbar::new(1));

    let response = FeeEstimateQuery::new()
        .set_mode(FeeEstimateMode::State)
        .set_transaction(&mut tx, &client)?
        .execute(&client)
        .await?;

    assert!(response.total > 0, "expected non-zero total fee estimate");
    assert!(response.node.is_some(), "expected node fee component");
    assert!(response.service.is_some(), "expected service fee component");
    assert!(response.network.is_some(), "expected network fee component");

    Ok(())
}

#[tokio::test]
#[ignore = "HIP-1261: requires mirror node with fee estimate endpoint"]
async fn account_create_transaction() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let key = PrivateKey::generate_ed25519();

    let mut tx = AccountCreateTransaction::new();
    tx.set_key_without_alias(key.public_key()).initial_balance(Hbar::new(1));

    let response = FeeEstimateQuery::new()
        .set_transaction(&mut tx, &client)?
        .execute(&client)
        .await?;

    assert!(response.total > 0, "expected non-zero total fee estimate");

    let node = response.node.expect("expected node fee component");
    assert!(node.base > 0, "expected non-zero node base fee");

    let service = response.service.expect("expected service fee component");
    assert!(service.base > 0, "expected non-zero service base fee");

    Ok(())
}

#[tokio::test]
#[ignore = "HIP-1261: requires mirror node with fee estimate endpoint"]
async fn high_volume_throttle() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_id = config.operator.as_ref().unwrap().account_id;

    let mut tx = TransferTransaction::new();
    tx.hbar_transfer(operator_id, Hbar::new(-1))
        .hbar_transfer("0.0.3".parse()?, Hbar::new(1));

    let response = FeeEstimateQuery::new()
        .set_transaction(&mut tx, &client)?
        .set_high_volume_throttle(5000)
        .execute(&client)
        .await?;

    assert!(response.total > 0, "expected non-zero total fee estimate");

    Ok(())
}

#[tokio::test]
#[ignore = "HIP-1261: requires mirror node with fee estimate endpoint"]
async fn intrinsic_vs_state_mode_both_return_fees() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_id = config.operator.as_ref().unwrap().account_id;

    let mut tx = TransferTransaction::new();
    tx.hbar_transfer(operator_id, Hbar::new(-1))
        .hbar_transfer("0.0.3".parse()?, Hbar::new(1));

    let intrinsic = FeeEstimateQuery::new()
        .set_mode(FeeEstimateMode::Intrinsic)
        .set_transaction(&mut tx, &client)?
        .execute(&client)
        .await?;

    let state = FeeEstimateQuery::new()
        .set_mode(FeeEstimateMode::State)
        .set_transaction(&mut tx, &client)?
        .execute(&client)
        .await?;

    assert!(intrinsic.total > 0, "expected non-zero intrinsic fee");
    assert!(state.total > 0, "expected non-zero state fee");

    Ok(())
}

#[tokio::test]
#[ignore = "HIP-1261: requires mirror node with fee estimate endpoint"]
async fn missing_transaction_bytes_error() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let result = FeeEstimateQuery::new().execute(&client).await;

    assert!(result.is_err(), "expected error when no transaction bytes set");

    Ok(())
}

#[tokio::test]
#[ignore = "HIP-1261: requires mirror node with fee estimate endpoint"]
async fn fee_extras_populated() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_id = config.operator.as_ref().unwrap().account_id;

    let mut tx = TransferTransaction::new();
    tx.hbar_transfer(operator_id, Hbar::new(-1))
        .hbar_transfer("0.0.3".parse()?, Hbar::new(1));

    let response = FeeEstimateQuery::new()
        .set_transaction(&mut tx, &client)?
        .execute(&client)
        .await?;

    let node = response.node.expect("expected node fee component");
    // Signature verification should appear as an extra
    assert!(!node.extras.is_empty(), "expected at least one node fee extra");

    let sig_extra = node.extras.iter().find(|e| {
        e.name.as_deref().map_or(false, |n| n.contains("SIGNATURE"))
    });
    assert!(sig_extra.is_some(), "expected signature verification extra in node fees");

    Ok(())
}
