use fraction::Fraction;
use hiero_sdk_proto::services;

use crate::token::custom_fees::{
    AnyCustomFee,
    CustomFee,
    Fee,
    FixedFeeData,
    FractionalFeeData,
    RoyaltyFeeData,
};
use crate::{
    AccountId,
    FromProtobuf,
    ToProtobuf,
    TokenId,
};

#[test]
fn custom_fee_can_convert_to_protobuf() -> anyhow::Result<()> {
    let custom_fee = AnyCustomFee {
        fee_collector_account_id: Some(AccountId::from(1)),
        fee: FixedFeeData { denominating_token_id: Some(TokenId::from(2)), amount: 1000 }.into(),
        all_collectors_are_exempt: true,
    };

    let custom_fee_proto = custom_fee.to_protobuf();

    assert_eq!(Some(custom_fee.fee.to_protobuf()), custom_fee_proto.fee);
    assert_eq!(
        custom_fee.fee_collector_account_id.to_protobuf(),
        custom_fee_proto.fee_collector_account_id
    );
    assert_eq!(custom_fee.all_collectors_are_exempt, custom_fee_proto.all_collectors_are_exempt);

    Ok(())
}

#[test]
fn custom_fixed_fee_can_be_created_from_protobuf() -> anyhow::Result<()> {
    let custom_fee_proto = services::CustomFee {
        fee: Some(services::custom_fee::Fee::FixedFee(services::FixedFee {
            denominating_token_id: Some(TokenId::from(2).to_protobuf()),
            amount: 1000,
        })),
        fee_collector_account_id: Some(AccountId::from(1).to_protobuf()),
        all_collectors_are_exempt: true,
    };

    let custom_fee = CustomFee::from_protobuf(custom_fee_proto.clone()).unwrap();

    assert_eq!(Some(custom_fee.fee.to_protobuf()), custom_fee_proto.fee);
    assert_eq!(
        custom_fee.fee_collector_account_id.to_protobuf(),
        custom_fee_proto.fee_collector_account_id
    );

    Ok(())
}

#[test]
fn fee_can_convert_to_protobuf() -> anyhow::Result<()> {
    let amount = 1000;
    let fee = Fee::Fixed(FixedFeeData { amount, denominating_token_id: Some(TokenId::from(1)) });

    let fee_proto = fee.to_protobuf();

    let fixed_fee_proto = match fee_proto {
        services::custom_fee::Fee::FixedFee(fixed_fee) => Some(fixed_fee),
        _ => None,
    };

    assert_eq!(fixed_fee_proto.unwrap().amount, amount);

    Ok(())
}

#[test]
fn fee_can_be_created_from_protobuf() -> anyhow::Result<()> {
    let amount = 1000;
    let fee_proto = services::custom_fee::Fee::FixedFee(services::FixedFee {
        denominating_token_id: Some(TokenId::from(2).to_protobuf()),
        amount,
    });

    let custom_fee = Fee::from_protobuf(fee_proto).unwrap();

    let fixed_fee = match custom_fee {
        Fee::Fixed(fixed_fee) => Some(fixed_fee),
        _ => None,
    };

    assert_eq!(fixed_fee.unwrap().amount, amount);

    Ok(())
}

#[test]
fn fixed_fee_can_convert_to_protobuf() -> anyhow::Result<()> {
    let amount = 1000;
    let fixed_fee = FixedFeeData { amount, denominating_token_id: Some(TokenId::from(2)) };

    let fixed_fee_proto = fixed_fee.to_protobuf();

    assert_eq!(fixed_fee_proto.amount, amount);

    Ok(())
}

#[test]
fn fixed_fee_can_be_created_from_protobuf() -> anyhow::Result<()> {
    let amount = 1000;
    let fixed_fee_proto =
        services::FixedFee { amount, denominating_token_id: Some(TokenId::from(2).to_protobuf()) };

    let fixed_fee = FixedFeeData::from_protobuf(fixed_fee_proto).unwrap();

    assert_eq!(fixed_fee.amount, amount);

    Ok(())
}

#[test]
fn fractional_fee_can_convert_to_protobuf() -> anyhow::Result<()> {
    let minimum_amount = 500;
    let maximum_amount = 1000;
    let net_of_transfers = true;
    let assessment_method = super::FeeAssessmentMethod::Exclusive;

    let fractional_fee = FractionalFeeData {
        denominator: 1,
        numerator: 2,
        minimum_amount,
        maximum_amount,
        assessment_method,
    };

    let fractional_fee_proto = fractional_fee.to_protobuf();

    assert_eq!(fractional_fee_proto.minimum_amount, minimum_amount);
    assert_eq!(fractional_fee_proto.maximum_amount, maximum_amount);
    assert_eq!(fractional_fee_proto.net_of_transfers, net_of_transfers);

    Ok(())
}

#[test]
fn fractional_fee_can_be_created_from_protobuf() -> anyhow::Result<()> {
    let minimum_amount = 500;
    let maximum_amount = 1000;
    let net_of_transfers = true;
    let assessment_method = super::FeeAssessmentMethod::Exclusive;

    let fractional_fee_protobuf = services::FractionalFee {
        fractional_amount: Some(services::Fraction { numerator: 1, denominator: 2 }),
        minimum_amount,
        maximum_amount,
        net_of_transfers,
    };

    let fractional_fee = FractionalFeeData::from_protobuf(fractional_fee_protobuf).unwrap();

    assert_eq!(fractional_fee.minimum_amount, minimum_amount);
    assert_eq!(fractional_fee.maximum_amount, maximum_amount);
    assert_eq!(fractional_fee.assessment_method, assessment_method);

    Ok(())
}

#[test]
fn royalty_fee_can_convert_to_protobuf() -> anyhow::Result<()> {
    let fallback_fee = FixedFeeData { denominating_token_id: Some(TokenId::from(1)), amount: 1000 };
    let exchange_value_fraction: Fraction = (1, 2).into();

    let royalty_fee =
        RoyaltyFeeData { fallback_fee: Some(fallback_fee.clone()), numerator: 1, denominator: 2 };

    let royalty_fee_proto = royalty_fee.to_protobuf();

    assert_eq!(royalty_fee_proto.fallback_fee, Some(fallback_fee.to_protobuf()));
    assert_eq!(royalty_fee_proto.exchange_value_fraction, Some(exchange_value_fraction.into()));

    Ok(())
}

#[test]
fn royalty_fee_can_be_created_from_protobuf() -> anyhow::Result<()> {
    let amount = 1000;
    let numerator = 1;
    let denominator = 2;

    let fallback_fee =
        services::FixedFee { denominating_token_id: Some(TokenId::from(1).to_protobuf()), amount };
    let exchange_value_fraction = services::Fraction { numerator, denominator };

    let royalty_fee_proto = services::RoyaltyFee {
        fallback_fee: Some(fallback_fee),
        exchange_value_fraction: Some(exchange_value_fraction),
    };

    let royalty_fee = RoyaltyFeeData::from_protobuf(royalty_fee_proto).unwrap();

    assert_eq!(royalty_fee.fallback_fee.map(|it| it.amount), Some(amount));
    assert_eq!(royalty_fee.denominator, denominator as u64);
    assert_eq!(royalty_fee.numerator, numerator as u64);

    Ok(())
}
