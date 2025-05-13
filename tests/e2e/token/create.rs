use assert_matches::assert_matches;
use hedera::{
    AccountId,
    AnyCustomFee,
    FixedFee,
    FixedFeeData,
    FractionalFee,
    FractionalFeeData,
    Hbar,
    PrivateKey,
    RoyaltyFee,
    Status,
    TokenCreateTransaction,
    TokenId,
    TokenInfoQuery,
    TokenType,
};
use time::{
    Duration,
    OffsetDateTime,
};

use crate::account::Account;
use crate::common::{
    setup_nonfree,
    TestEnvironment,
};
use crate::token::{
    FungibleToken,
    Nft,
};

fn fixed_fee(fee_collector: AccountId) -> AnyCustomFee {
    FixedFee {
        fee: FixedFeeData { amount: 10, denominating_token_id: Some(TokenId::new(0, 0, 0)) },
        fee_collector_account_id: Some(fee_collector),
        all_collectors_are_exempt: false,
    }
    .into()
}

fn fractional_fee(fee_collector: AccountId) -> AnyCustomFee {
    FractionalFee {
        fee: FractionalFeeData {
            denominator: 20,
            numerator: 1,
            minimum_amount: 1,
            maximum_amount: 20,
            assessment_method: hedera::FeeAssessmentMethod::Exclusive,
        },
        fee_collector_account_id: Some(fee_collector),
        all_collectors_are_exempt: false,
    }
    .into()
}

#[tokio::test]
async fn basic() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;

    let token_id = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .decimals(3)
        .initial_supply(0)
        .treasury_account_id(account.id)
        .admin_key(account.key.public_key())
        .freeze_key(account.key.public_key())
        .wipe_key(account.key.public_key())
        .kyc_key(account.key.public_key())
        .supply_key(account.key.public_key())
        .fee_schedule_key(account.key.public_key())
        .freeze_default(false)
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    let token = FungibleToken { id: token_id, owner: account.clone() };

    token.delete(&client).await?;
    account.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn minimal() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;

    TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .treasury_account_id(account.id)
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}

#[tokio::test]
async fn missing_name_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;

    let res = TokenCreateTransaction::new()
        .symbol("F")
        .treasury_account_id(account.id)
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::MissingTokenName, .. })
    );

    account.delete(&client).await?;
    Ok(())
}

#[tokio::test]
async fn missing_symbol_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;

    let res = TokenCreateTransaction::new()
        .name("ffff")
        .treasury_account_id(account.id)
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::MissingTokenSymbol, .. })
    );

    account.delete(&client).await?;
    Ok(())
}

#[tokio::test]
async fn missing_treasury_account_id_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let res = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::TransactionPreCheckStatus {
            status: Status::InvalidTreasuryAccountForToken,
            ..
        })
    );

    Ok(())
}

#[tokio::test]
async fn missing_treasury_account_id_sig_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;

    let res = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .treasury_account_id(account.id)
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::InvalidSignature, .. })
    );

    account.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn missing_admin_key_sig_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;

    let admin_key = PrivateKey::generate_ed25519();

    let res = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .treasury_account_id(account.id)
        .admin_key(admin_key.public_key())
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::InvalidSignature, .. })
    );

    account.delete(&client).await?;
    Ok(())
}

#[tokio::test]
async fn custom_fees() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;

    let custom_fees = [fixed_fee(account.id), fractional_fee(account.id)];

    let token_id = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .treasury_account_id(account.id)
        .admin_key(account.key.public_key())
        .custom_fees(custom_fees)
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    let token = FungibleToken { id: token_id, owner: account.clone() };

    token.delete(&client).await?;
    account.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn too_many_custom_fees_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;

    let res = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .admin_key(account.key.public_key())
        .treasury_account_id(account.id)
        .custom_fees(vec![fixed_fee(account.id); 11])
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::CustomFeesListTooLong, .. })
    );

    account.delete(&client).await?;
    Ok(())
}

#[tokio::test]
async fn ten_fixed_fees() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;

    let token_id = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .admin_key(account.key.public_key())
        .treasury_account_id(account.id)
        .custom_fees(vec![fixed_fee(account.id); 10])
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    let token = FungibleToken { id: token_id, owner: account.clone() };

    token.delete(&client).await?;
    account.delete(&client).await?;
    Ok(())
}

#[tokio::test]
async fn ten_fractional_fees() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;

    let token_id = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .admin_key(account.key.public_key())
        .treasury_account_id(account.id)
        .custom_fees(vec![fractional_fee(account.id); 10])
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    let token = FungibleToken { id: token_id, owner: account.clone() };
    token.delete(&client).await?;

    account.delete(&client).await?;
    Ok(())
}

#[tokio::test]
async fn fractional_fee_min_bigger_than_max_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;

    let fee = FractionalFee {
        fee: FractionalFeeData {
            denominator: 3,
            numerator: 1,
            minimum_amount: 3,
            maximum_amount: 2,
            assessment_method: hedera::FeeAssessmentMethod::Exclusive,
        },
        fee_collector_account_id: Some(account.id),
        all_collectors_are_exempt: false,
    }
    .into();

    let res = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .admin_key(account.key.public_key())
        .treasury_account_id(account.id)
        .custom_fees([fee])
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus {
            status: Status::FractionalFeeMaxAmountLessThanMinAmount,
            ..
        })
    );

    account.delete(&client).await?;
    Ok(())
}

#[tokio::test]
async fn invalid_fee_collector_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;

    let fee = FixedFee {
        fee: FixedFeeData::from_hbar(Hbar::from_tinybars(1)),
        fee_collector_account_id: None,
        all_collectors_are_exempt: false,
    }
    .into();

    let res = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .admin_key(account.key.public_key())
        .treasury_account_id(account.id)
        .custom_fees([fee])
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::InvalidCustomFeeCollector, .. })
    );

    account.delete(&client).await?;
    Ok(())
}

#[tokio::test]
async fn negative_fee_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;

    let fee = FixedFee {
        fee: FixedFeeData::from_hbar(Hbar::from_tinybars(-1)),
        fee_collector_account_id: Some(account.id),
        all_collectors_are_exempt: false,
    }
    .into();

    let res = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .admin_key(account.key.public_key())
        .treasury_account_id(account.id)
        .custom_fees([fee])
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::CustomFeeMustBePositive, .. })
    );

    account.delete(&client).await?;
    Ok(())
}

#[tokio::test]
async fn zero_denominator_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;

    let fee = FractionalFee {
        fee: FractionalFeeData {
            denominator: 0,
            numerator: 1,
            minimum_amount: 1,
            maximum_amount: 10,
            assessment_method: hedera::FeeAssessmentMethod::Exclusive,
        },
        fee_collector_account_id: Some(account.id),
        all_collectors_are_exempt: false,
    }
    .into();

    let res = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .admin_key(account.key.public_key())
        .treasury_account_id(account.id)
        .custom_fees([fee])
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::FractionDividesByZero, .. })
    );

    account.delete(&client).await?;
    Ok(())
}

#[tokio::test]
async fn nfts() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;

    let token_id = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .token_type(TokenType::NonFungibleUnique)
        .treasury_account_id(account.id)
        .admin_key(account.key.public_key())
        .freeze_key(account.key.public_key())
        .wipe_key(account.key.public_key())
        .kyc_key(account.key.public_key())
        .supply_key(account.key.public_key())
        .freeze_default(false)
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    let token = Nft { id: token_id, owner: account.clone() };

    token.delete(&client).await?;
    account.delete(&client).await?;
    Ok(())
}

#[tokio::test]
async fn royalty_fee() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;

    let fee = RoyaltyFee {
        fee: hedera::RoyaltyFeeData {
            denominator: 10,
            numerator: 1,
            fallback_fee: Some(FixedFeeData::from_hbar(Hbar::new(1))),
        },
        fee_collector_account_id: Some(account.id),
        all_collectors_are_exempt: false,
    };

    let token_id = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .treasury_account_id(account.id)
        .supply_key(account.key.public_key())
        .admin_key(account.key.public_key())
        .token_type(TokenType::NonFungibleUnique)
        .custom_fees([fee.into()])
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    let token = Nft { id: token_id, owner: account.clone() };

    token.delete(&client).await?;
    account.delete(&client).await?;
    Ok(())
}

#[tokio::test]
async fn auto_renew_account() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;

    let token_id = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .treasury_account_id(account.id)
        .auto_renew_account_id(account.id)
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    let info = TokenInfoQuery::new().token_id(token_id).execute(&client).await?;

    // auto renew account should be set to operator account
    assert_eq!(info.auto_renew_account.unwrap(), account.id);
    Ok(())
}

#[tokio::test]
async fn autoset_auto_renew_account() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;

    let token_id = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .treasury_account_id(account.id)
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    let info = TokenInfoQuery::new().token_id(token_id).execute(&client).await?;
    // auto renew account should be set to operator account
    assert_eq!(info.auto_renew_account.unwrap(), client.get_operator_account_id().unwrap());
    Ok(())
}
