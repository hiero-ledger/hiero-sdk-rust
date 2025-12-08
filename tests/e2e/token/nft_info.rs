use assert_matches::assert_matches;
use hiero_sdk::{
    Hbar,
    NftId,
    Status,
    TokenId,
    TokenNftInfoQuery,
};

use crate::account::Account;
use crate::common::{
    setup_nonfree,
    TestEnvironment,
};
use crate::token::Nft;

#[tokio::test]
async fn basic() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;
    let token = Nft::create(&client, &account).await?;

    let serials = token.mint(&client, [[50_u8]]).await?;

    let nft_id = token.id.nft(serials[0] as u64);

    let nft_info = TokenNftInfoQuery::new().nft_id(nft_id).execute(&client).await?;

    assert_eq!(nft_info.nft_id, nft_id);
    assert_eq!(nft_info.account_id, account.id);
    assert_eq!(nft_info.metadata, vec![50]);

    token.burn(&client, serials).await?;
    token.delete(&client).await?;
    account.delete(&client).await?;

    Ok(())
}

#[tokio::test]

async fn invalid_nft_id_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let res = TokenNftInfoQuery::new()
        .nft_id(NftId { token_id: TokenId::new(0, 0, 0), serial: 2023 })
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::QueryNoPaymentPreCheckStatus { status: Status::InvalidNftId })
    );

    Ok(())
}

#[tokio::test]
async fn invalid_serial_number_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let res = TokenNftInfoQuery::new()
        .nft_id(NftId { token_id: TokenId::new(0, 0, 0), serial: u64::MAX })
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::QueryNoPaymentPreCheckStatus {
            status: Status::InvalidTokenNftSerialNumber
        })
    );

    Ok(())
}
