use hiero_sdk::{
    FileCreateTransaction,
    FileDeleteTransaction,
    FileInfoQuery,
    Key,
    KeyList,
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
        .contents("[rust::e2e::file_create]")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .file_id
        .unwrap();

    let info = FileInfoQuery::new().file_id(file_id).execute(&client).await?;

    assert_eq!(info.file_id, file_id);
    assert_eq!(info.size, 24);
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
async fn empty_file() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let file_id = FileCreateTransaction::new()
        .keys([op.private_key.public_key()])
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .file_id
        .unwrap();

    let info = FileInfoQuery::new().file_id(file_id).execute(&client).await?;

    assert_eq!(info.file_id, file_id);
    assert_eq!(info.size, 0);
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
async fn no_keys() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let file_id = FileCreateTransaction::new()
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .file_id
        .unwrap();

    let info = FileInfoQuery::new().file_id(file_id).execute(&client).await?;

    assert_eq!(info.file_id, file_id);
    assert_eq!(info.size, 0);
    assert!(!info.is_deleted);
    assert_eq!(info.keys, KeyList { keys: Vec::new(), threshold: None });

    Ok(())
}
