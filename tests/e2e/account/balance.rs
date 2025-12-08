// todo(network tls):
// - canConnectToPreviewnetWithTLS
// - canConnectToTestnetWithTLS
// - canConnectToMainnetWithTLS
// - cannotConnectToPreviewnetWhenNetworkNameIsNullAndCertificateVerificationIsEnabled

use assert_matches::assert_matches;
use hiero_sdk::{
    AccountBalanceQuery,
    AccountId,
    Hbar,
    Status,
    TokenBurnTransaction,
    TokenCreateTransaction,
    TokenDeleteTransaction,
};
use time::{
    Duration,
    OffsetDateTime,
};

use crate::account::Account;
use crate::common::{
    setup_global,
    setup_nonfree,
    TestEnvironment,
};

#[tokio::test]
async fn query() -> anyhow::Result<()> {
    let TestEnvironment { config, client } = setup_global();

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to lack of operator");

        return Ok(());
    };

    let balance = AccountBalanceQuery::new().account_id(op.account_id).execute(&client).await?;

    log::trace!("successfully queried balance: {balance:?}");

    anyhow::ensure!(balance.account_id == op.account_id);
    anyhow::ensure!(balance.hbars.to_tinybars() > 0);

    Ok(())
}

#[tokio::test]
async fn query_cost() -> anyhow::Result<()> {
    let TestEnvironment { config, client } = setup_global();

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to lack of operator");
        return Ok(());
    };

    let mut query = AccountBalanceQuery::new();

    query.account_id(op.account_id).max_payment_amount(Hbar::new(1));

    let cost = query.get_cost(&client).await?;

    assert_eq!(cost, Hbar::ZERO);

    let balance = query.payment_amount(cost).execute(&client).await?;

    anyhow::ensure!(balance.account_id == op.account_id);
    anyhow::ensure!(balance.hbars.to_tinybars() > 0);

    Ok(())
}

#[tokio::test]
async fn query_cost_big_max() -> anyhow::Result<()> {
    let TestEnvironment { config, client } = setup_global();

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to lack of operator");
        return Ok(());
    };

    let mut query = AccountBalanceQuery::new();

    query.account_id(op.account_id).max_payment_amount(Hbar::new(1_000_000));

    let cost = query.get_cost(&client).await?;

    assert_eq!(cost, Hbar::ZERO);

    let balance = query.payment_amount(cost).execute(&client).await?;

    anyhow::ensure!(balance.account_id == op.account_id);
    anyhow::ensure!(balance.hbars.to_tinybars() > 0);

    Ok(())
}

#[tokio::test]
async fn query_cost_small_max() -> anyhow::Result<()> {
    let TestEnvironment { config, client } = setup_global();

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to lack of operator");
        return Ok(());
    };

    let mut query = AccountBalanceQuery::new();

    query.account_id(op.account_id).max_payment_amount(Hbar::from_tinybars(1));

    let cost = query.get_cost(&client).await?;

    assert_eq!(cost, Hbar::ZERO);

    let balance = query.payment_amount(cost).execute(&client).await?;

    anyhow::ensure!(balance.account_id == op.account_id);
    anyhow::ensure!(balance.hbars.to_tinybars() > 0);

    Ok(())
}

#[tokio::test]
async fn invalid_account_id_fails() -> anyhow::Result<()> {
    let TestEnvironment { config: _, client } = setup_global();

    let res = AccountBalanceQuery::new()
        .account_id(AccountId {
            shard: 1,
            realm: 0,
            num: 3,
            alias: None,
            evm_address: None,
            checksum: None,
        })
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::QueryNoPaymentPreCheckStatus { status: Status::InvalidAccountId })
    );

    Ok(())
}

#[tokio::test]
async fn query_token_balances() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(10), &client).await?;

    let token_id = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("f")
        .initial_supply(10000)
        .decimals(50)
        .treasury_account_id(account.id)
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .admin_key(account.key.public_key())
        .supply_key(account.key.public_key())
        .freeze_default(false)
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    let balance = AccountBalanceQuery::new().account_id(account.id).execute(&client).await?;

    #[allow(deprecated)]
    {
        assert_eq!(balance.tokens.get(&token_id).copied(), Some(10000));
        assert_eq!(balance.token_decimals.get(&token_id).copied(), Some(50));
    }

    TokenBurnTransaction::new()
        .token_id(token_id)
        .amount(10000_u64)
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let _ = TokenDeleteTransaction::new()
        .token_id(token_id)
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    account.delete(&client).await?;

    Ok(())
}
