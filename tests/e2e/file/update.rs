use assert_matches::assert_matches;
use hiero_sdk::{
    FileCreateTransaction,
    FileDeleteTransaction,
    FileInfoQuery,
    FileUpdateTransaction,
    Key,
    KeyList,
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
        .contents("[rust::e2e::file_update::1]")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .file_id
        .unwrap();

    FileUpdateTransaction::new()
        .file_id(file_id)
        .contents(b"updated file".to_vec())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info = FileInfoQuery::new().file_id(file_id).execute(&client).await?;

    assert_eq!(info.file_id, file_id);
    assert_eq!(info.size, 12);
    assert!(!info.is_deleted);
    assert_eq!(
        info.keys,
        KeyList { keys: Vec::from([Key::Single(op.private_key.public_key())]), threshold: None }
    );

    FileDeleteTransaction::new()
        .file_id(file_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}

#[tokio::test]
async fn immutable_file_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let file_id = FileCreateTransaction::new()
        .contents("[rust::e2e::file_update::2]")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .file_id
        .unwrap();

    let res = FileUpdateTransaction::new()
        .file_id(file_id)
        .contents(Vec::from([0]))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(res, Err(hiero_sdk::Error::ReceiptStatus { status: Status::Unauthorized, .. }));

    Ok(())
}

#[tokio::test]
async fn missing_file_id_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let res = FileUpdateTransaction::new().contents(b"contents".to_vec()).execute(&client).await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::TransactionPreCheckStatus { status: Status::InvalidFileId, .. })
    );

    Ok(())
}
