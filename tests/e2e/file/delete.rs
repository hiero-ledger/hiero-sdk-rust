use assert_matches::assert_matches;
use hiero_sdk::{
    FileCreateTransaction,
    FileDeleteTransaction,
    FileInfoQuery,
    Status,
};

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

    let file_id = FileCreateTransaction::new()
        .keys([op.private_key.public_key()])
        .contents("[rust::e2e::file_delete::1]")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .file_id
        .unwrap();

    FileDeleteTransaction::new()
        .file_id(file_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info = FileInfoQuery::new().file_id(file_id).execute(&client).await?;

    assert!(info.is_deleted);

    Ok(())
}

#[tokio::test]
async fn immutable_file_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let file_id = FileCreateTransaction::new()
        .contents("[rust::e2e::file_delete::2]")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .file_id
        .unwrap();

    let res = FileDeleteTransaction::new()
        .file_id(file_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(res, Err(hiero_sdk::Error::ReceiptStatus { status: Status::Unauthorized, .. }));

    Ok(())
}
