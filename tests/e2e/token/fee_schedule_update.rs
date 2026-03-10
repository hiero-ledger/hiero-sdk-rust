use assert_matches::assert_matches;
use hiero_sdk::{
    FixedFee,
    FixedFeeData,
    FractionalFee,
    FractionalFeeData,
    Hbar,
    Status,
    TokenFeeScheduleUpdateTransaction,
    TokenInfoQuery,
};

use crate::account::Account;
use crate::common::{
    setup_nonfree,
    TestEnvironment,
};
use crate::token::{
    CreateFungibleToken,
    Key,
    TokenKeys,
};

const TOKEN_PARAMS: CreateFungibleToken = CreateFungibleToken {
    initial_supply: 0,
    keys: TokenKeys { fee_schedule: Some(Key::Owner), ..TokenKeys::DEFAULT },
};

#[tokio::test]
async fn basic() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;
    let token = super::FungibleToken::create(&client, &account, TOKEN_PARAMS).await?;

    let info = TokenInfoQuery::new().token_id(token.id).execute(&client).await?;

    assert!(info.custom_fees.is_empty());

    let custom_fees = Vec::from([
        FixedFee {
            fee: FixedFeeData::from_hbar(Hbar::from_tinybars(10)),
            fee_collector_account_id: Some(account.id),
            all_collectors_are_exempt: false,
        }
        .into(),
        FractionalFee {
            fee: FractionalFeeData {
                denominator: 20,
                numerator: 1,
                minimum_amount: 1,
                maximum_amount: 10,
                assessment_method: hiero_sdk::FeeAssessmentMethod::Exclusive,
            },
            fee_collector_account_id: Some(account.id),
            all_collectors_are_exempt: false,
        }
        .into(),
    ]);

    TokenFeeScheduleUpdateTransaction::new()
        .token_id(token.id)
        .custom_fees(custom_fees.clone())
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info = TokenInfoQuery::new().token_id(token.id).execute(&client).await?;

    assert_eq!(custom_fees, info.custom_fees);

    token.delete(&client).await?;
    account.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn invalid_signature_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;
    let token = super::FungibleToken::create(&client, &account, TOKEN_PARAMS).await?;

    let info = TokenInfoQuery::new().token_id(token.id).execute(&client).await?;

    assert!(info.custom_fees.is_empty());

    let custom_fees = Vec::from([
        FixedFee {
            fee: FixedFeeData::from_hbar(Hbar::from_tinybars(10)),
            fee_collector_account_id: Some(account.id),
            all_collectors_are_exempt: false,
        }
        .into(),
        FractionalFee {
            fee: FractionalFeeData {
                denominator: 20,
                numerator: 1,
                minimum_amount: 1,
                maximum_amount: 10,
                assessment_method: hiero_sdk::FeeAssessmentMethod::Exclusive,
            },
            fee_collector_account_id: Some(account.id),
            all_collectors_are_exempt: false,
        }
        .into(),
    ]);

    let res = TokenFeeScheduleUpdateTransaction::new()
        .token_id(token.id)
        .custom_fees(custom_fees.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    token.delete(&client).await?;
    account.delete(&client).await?;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, .. })
    );

    Ok(())
}
