use assert_matches::assert_matches;
use hedera::{
    account_info_flow,
    AccountInfoQuery,
    Hbar,
    Key,
    PrivateKey,
    Status,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

#[tokio::test]
async fn query() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to lack of operator");

        return Ok(());
    };

    let info = AccountInfoQuery::new().account_id(op.account_id).execute(&client).await?;

    assert_eq!(info.account_id, op.account_id);
    assert!(!info.is_deleted);
    assert_eq!(info.key, Key::Single(op.private_key.public_key()));
    assert!(info.balance.to_tinybars() > 0);
    assert_eq!(info.proxy_received, Hbar::ZERO);

    Ok(())
}

#[tokio::test]
async fn query_cost_for_operator() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to lack of operator");

        return Ok(());
    };

    let mut query = AccountInfoQuery::new();

    query.account_id(op.account_id).max_payment_amount(Hbar::new(1));

    let cost = query.get_cost(&client).await?;

    let info = query.payment_amount(cost).execute(&client).await?;

    assert_eq!(info.account_id, op.account_id);

    Ok(())
}

#[tokio::test]
async fn query_cost_big_max() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to lack of operator");

        return Ok(());
    };

    let mut query = AccountInfoQuery::new();

    query.account_id(op.account_id).max_payment_amount(Hbar::MAX);

    let cost = query.get_cost(&client).await?;

    let info = query.payment_amount(cost).execute(&client).await?;

    assert_eq!(info.account_id, op.account_id);

    Ok(())
}

#[tokio::test]
async fn query_cost_small_max_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to lack of operator");

        return Ok(());
    };

    let mut query = AccountInfoQuery::new();

    query.account_id(op.account_id).max_payment_amount(Hbar::from_tinybars(1));

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

    Ok(())
}

#[tokio::test]
#[ignore]
async fn get_cost_insufficient_tx_fee_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to lack of operator");

        return Ok(());
    };

    let res = AccountInfoQuery::new() //
        .account_id(op.account_id)
        .max_payment_amount(Hbar::from_tinybars(10000))
        .payment_amount(Hbar::from_tinybars(1))
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::QueryPaymentPreCheckStatus { status: Status::InsufficientTxFee, .. })
    );

    Ok(())
}

#[tokio::test]
async fn flow_verify_transaction() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to lack of operator");

        return Ok(());
    };

    let new_key = PrivateKey::generate_ed25519();

    let new_public_key = new_key.public_key();

    let mut signed_tx = hedera::AccountCreateTransaction::new();
    signed_tx
        .set_key_without_alias(new_public_key)
        .initial_balance(Hbar::from_tinybars(1000))
        .freeze_with(&client)?
        .sign_with_operator(&client)?;

    let mut unsigned_tx = hedera::AccountCreateTransaction::new();
    unsigned_tx
        .set_key_without_alias(new_public_key)
        .initial_balance(Hbar::from_tinybars(1000))
        .freeze_with(&client)?;

    assert_matches!(
        account_info_flow::verify_transaction_signature(&client, op.account_id, &mut signed_tx)
            .await,
        Ok(())
    );

    assert_matches!(
        account_info_flow::verify_transaction_signature(&client, op.account_id, &mut unsigned_tx)
            .await,
        Err(hedera::Error::SignatureVerify(_))
    );

    Ok(())
}
