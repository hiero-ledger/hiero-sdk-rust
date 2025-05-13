use hedera::{
    AccountBalanceQuery,
    AccountCreateTransaction,
    Client,
    CustomFixedFee,
    Hbar,
    Key,
    PrivateKey,
    TokenCreateTransaction,
    TokenId,
    TopicCreateTransaction,
    TopicInfoQuery,
    TopicMessageSubmitTransaction,
    TopicUpdateTransaction,
    TransactionId,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};
use crate::topic::Topic;

#[tokio::test]
async fn basic() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let topic_id = TopicCreateTransaction::new()
        .admin_key(op.private_key.public_key())
        .topic_memo("[e2e::TopicCreateTransaction]")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .topic_id
        .unwrap();

    let topic = Topic { id: topic_id };

    topic.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn fieldless() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let _topic_id = TopicCreateTransaction::new()
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .topic_id
        .unwrap();
    Ok(())
}

#[tokio::test]
async fn autoset_auto_renew_account() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let topic_id = TopicCreateTransaction::new()
        .admin_key(client.get_operator_public_key().unwrap())
        .topic_memo("[e2e::TopicCreateTransaction]")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .topic_id
        .unwrap();

    let info = TopicInfoQuery::new().topic_id(topic_id).execute(&client).await?;
    assert_eq!(info.auto_renew_account_id.unwrap(), client.get_operator_account_id().unwrap());
    Ok(())
}

async fn create_token(client: &Client) -> anyhow::Result<TokenId> {
    let operator_account_id = client.get_operator_account_id().unwrap();
    let operator_key = client.get_operator_public_key().unwrap();

    let receipt = TokenCreateTransaction::new()
        .name("Test Token")
        .symbol("FT")
        .treasury_account_id(operator_account_id)
        .initial_supply(1_000_000)
        .decimals(2)
        .admin_key(operator_key.clone())
        .supply_key(operator_key)
        .execute(client)
        .await?
        .get_receipt(client)
        .await?;

    Ok(receipt.token_id.unwrap())
}

#[tokio::test]
async fn creates_and_updates_revenue_generating_topic() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let fee_exempt_keys = vec![PrivateKey::generate_ecdsa(), PrivateKey::generate_ecdsa()];

    let token1 = create_token(&client).await?;
    let token2 = create_token(&client).await?;

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let custom_fixed_fees = vec![
        CustomFixedFee::new(1, Some(token1), Some(op.account_id)),
        CustomFixedFee::new(2, Some(token2), Some(op.account_id)),
    ];

    // Create revenue-generating topic
    let receipt = TopicCreateTransaction::new()
        .fee_schedule_key(op.private_key.public_key())
        .submit_key(op.private_key.public_key())
        .admin_key(op.private_key.public_key())
        .fee_exempt_keys(fee_exempt_keys.iter().map(|key| key.public_key().into()).collect())
        .custom_fees(custom_fixed_fees)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let topic_id = receipt.topic_id.unwrap();

    let info = TopicInfoQuery::new().topic_id(topic_id).execute(&client).await?;

    assert_eq!(
        info.fee_schedule_key.unwrap().to_bytes(),
        Key::Single(op.private_key.public_key()).to_bytes()
    );

    // Update the revenue-generating topic
    let new_fee_exempt_keys = vec![PrivateKey::generate_ecdsa(), PrivateKey::generate_ecdsa()];
    let new_fee_schedule_key = PrivateKey::generate_ecdsa();

    let new_token1 = create_token(&client).await?;
    let new_token2 = create_token(&client).await?;

    let new_custom_fixed_fees = vec![
        CustomFixedFee::new(3, Some(new_token1), Some(op.account_id)),
        CustomFixedFee::new(4, Some(new_token2), Some(op.account_id)),
    ];

    TopicUpdateTransaction::new()
        .topic_id(topic_id)
        .fee_exempt_keys(new_fee_exempt_keys.iter().map(|key| key.public_key().into()).collect())
        .fee_schedule_key(new_fee_schedule_key.public_key())
        .custom_fees(new_custom_fixed_fees.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let updated_info = TopicInfoQuery::new().topic_id(topic_id).execute(&client).await?;

    assert_eq!(
        updated_info.fee_schedule_key.unwrap().to_bytes(),
        Key::Single(new_fee_schedule_key.public_key()).to_bytes()
    );

    // Validate updated fee exempt keys
    for (idx, key) in new_fee_exempt_keys.iter().enumerate() {
        assert_eq!(
            updated_info.fee_exempt_keys[idx].to_bytes(),
            Key::Single(key.public_key()).to_bytes()
        );
    }

    // Validate updated custom fees
    for (idx, fee) in new_custom_fixed_fees.iter().enumerate() {
        assert_eq!(updated_info.custom_fees[idx].amount, fee.amount);
        assert_eq!(updated_info.custom_fees[idx].denominating_token_id, fee.denominating_token_id);
    }

    Ok(())
}

#[tokio::test]
async fn create_revenue_generating_topic_with_invalid_fee_exempt_key_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let fee_exempt_key = PrivateKey::generate_ecdsa();
    let fee_exempt_key_list_with_duplicates =
        vec![Key::Single(fee_exempt_key.public_key()), Key::Single(fee_exempt_key.public_key())];

    let result = TopicCreateTransaction::new()
        .admin_key(op.private_key.public_key())
        .fee_exempt_keys(fee_exempt_key_list_with_duplicates)
        .execute(&client)
        .await;

    assert!(matches!(
        result,
        Err(hedera::Error::TransactionPreCheckStatus {
            status: hedera::Status::FeeExemptKeyListContainsDuplicatedKeys,
            ..
        })
    ));

    // Test exceeding key limit
    let fee_exempt_key_list_exceeding_limit =
        (0..11).map(|_| Key::Single(PrivateKey::generate_ecdsa().public_key())).collect::<Vec<_>>();

    let result = TopicCreateTransaction::new()
        .admin_key(op.private_key.public_key())
        .fee_exempt_keys(fee_exempt_key_list_exceeding_limit)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert!(matches!(
        result.unwrap_err(),
        hedera::Error::ReceiptStatus {
            status: hedera::Status::MaxEntriesForFeeExemptKeyListExceeded,
            ..
        }
    ));

    Ok(())
}

// Continuing with more test conversions...
#[tokio::test]
async fn update_fee_schedule_key_without_permission_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let receipt = TopicCreateTransaction::new()
        .admin_key(op.private_key.public_key())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let topic_id = receipt.topic_id.unwrap();
    let fee_schedule_key = PrivateKey::generate_ed25519();

    let result = TopicUpdateTransaction::new()
        .topic_id(topic_id)
        .fee_schedule_key(fee_schedule_key.public_key())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert!(matches!(
        result.unwrap_err(),
        hedera::Error::ReceiptStatus { status: hedera::Status::FeeScheduleKeyCannotBeUpdated, .. }
    ));

    Ok(())
}

#[tokio::test]
async fn update_custom_fees_without_fee_schedule_key_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    // Create a topic without fee schedule key
    let receipt = TopicCreateTransaction::new()
        .admin_key(op.private_key.public_key())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let topic_id = receipt.topic_id.unwrap();

    let token1 = create_token(&client).await?;
    let token2 = create_token(&client).await?;

    let custom_fixed_fees = vec![
        CustomFixedFee::new(1, Some(token1), Some(op.account_id)),
        CustomFixedFee::new(2, Some(token2), Some(op.account_id)),
    ];

    let result = TopicUpdateTransaction::new()
        .topic_id(topic_id)
        .custom_fees(custom_fixed_fees)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert!(matches!(
        result.unwrap_err(),
        hedera::Error::ReceiptStatus { status: hedera::Status::FeeScheduleKeyNotSet, .. }
    ));

    Ok(())
}

#[tokio::test]
async fn charges_hbar_fee_with_limits_applied() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let hbar_amount: u64 = 100_000_000;
    let private_key = PrivateKey::generate_ecdsa();

    let custom_fixed_fee = CustomFixedFee::new(hbar_amount / 2, None, Some(op.account_id));

    let receipt = TopicCreateTransaction::new()
        .admin_key(op.private_key.public_key())
        .fee_schedule_key(op.private_key.public_key())
        .add_custom_fee(custom_fixed_fee)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let topic_id = receipt.topic_id.unwrap();

    let account_receipt = AccountCreateTransaction::new()
        .initial_balance(Hbar::new(1))
        .key(private_key.public_key())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let account_id = account_receipt.account_id.unwrap();

    client.set_operator(account_id, private_key);

    TopicMessageSubmitTransaction::new()
        .topic_id(topic_id)
        .message("Hello, Hiero™ hashgraph!".as_bytes().to_vec())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    client.set_operator(op.account_id, PrivateKey::generate_ecdsa());

    let account_info = AccountBalanceQuery::new().account_id(account_id).execute(&client).await?;

    assert!(account_info.hbars.to_tinybars() < (hbar_amount / 2) as i64);

    Ok(())
}

#[tokio::test]
async fn exempts_fee_exempt_keys_from_hbar_fees() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let hbar_amount: u64 = 100_000_000;
    let fee_exempt_key1 = PrivateKey::generate_ecdsa();
    let fee_exempt_key2 = PrivateKey::generate_ecdsa();

    let custom_fixed_fee = CustomFixedFee::new(hbar_amount / 2, None, Some(op.account_id));

    let receipt = TopicCreateTransaction::new()
        .admin_key(op.private_key.public_key())
        .fee_schedule_key(op.private_key.public_key())
        .fee_exempt_keys(vec![
            Key::Single(fee_exempt_key1.public_key()),
            Key::Single(fee_exempt_key2.public_key()),
        ])
        .add_custom_fee(custom_fixed_fee)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let topic_id = receipt.topic_id.unwrap();

    let payer_account_receipt = AccountCreateTransaction::new()
        .initial_balance(Hbar::new(1))
        .key(fee_exempt_key1.public_key())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let payer_account_id = payer_account_receipt.account_id.unwrap();

    client.set_operator(payer_account_id, fee_exempt_key1);

    TopicMessageSubmitTransaction::new()
        .topic_id(topic_id)
        .message("Hello, Hiero™ hashgraph!".as_bytes().to_vec())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    client.set_operator(payer_account_id, PrivateKey::generate_ecdsa());

    let account_info =
        AccountBalanceQuery::new().account_id(payer_account_id).execute(&client).await?;

    assert!(account_info.hbars.to_tinybars() > (hbar_amount / 2) as i64);

    Ok(())
}

// Test temporarily taken out until can figure out a solution for a separate freeze
#[tokio::test]
async fn automatically_assign_auto_renew_account_id_on_topic_create() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let topic_receipt =
        TopicCreateTransaction::new().execute(&client).await?.get_receipt(&client).await?;

    let topic_id = topic_receipt.topic_id.unwrap();

    let info = TopicInfoQuery::new().topic_id(topic_id).execute(&client).await?;

    assert!(info.auto_renew_account_id.is_some());

    Ok(())
}

#[tokio::test]
async fn create_with_transaction_id_assigns_auto_renew_account_id_to_transaction_id_account_id(
) -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let private_key = PrivateKey::generate_ecdsa();
    let public_key = private_key.public_key();

    let account_receipt = AccountCreateTransaction::new()
        .key(public_key)
        .initial_balance(Hbar::new(10))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let account_id = account_receipt.account_id.unwrap();

    let topic_receipt = TopicCreateTransaction::new()
        .transaction_id(TransactionId::generate(account_id))
        .freeze_with(&client)?
        .sign(private_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let topic_id = topic_receipt.topic_id.unwrap();

    let topic_info = TopicInfoQuery::new().topic_id(topic_id).execute(&client).await?;

    assert_eq!(topic_info.auto_renew_account_id, Some(account_id));

    Ok(())
}
