use hiero_sdk::{
    FileAppendTransaction,
    FileContentsQuery,
    FileCreateTransaction,
    FileDeleteTransaction,
    FileInfoQuery,
};
use time::Duration;

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};
use crate::resources;

#[tokio::test]
async fn basic() -> anyhow::Result<()> {
    // There are potential bugs in FileAppendTransaction which require more than one node to trigger.
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    // assume_not_local_node

    let file_id = FileCreateTransaction::new()
        .keys([op.private_key.public_key()])
        .contents("[rust::e2e::file_append::1]")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .file_id
        .unwrap();

    FileAppendTransaction::new()
        .file_id(file_id)
        .contents("update")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info = FileInfoQuery::new().file_id(file_id).execute(&client).await?;

    assert_eq!(info.file_id, file_id);
    assert_eq!(info.size, 33);

    FileDeleteTransaction::new()
        .file_id(file_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}

#[tokio::test]
async fn large_contents() -> anyhow::Result<()> {
    // There are potential bugs in FileAppendTransaction which require more than one node to trigger.
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    // assume_not_local_node

    let file_id = FileCreateTransaction::new()
        .keys([op.private_key.public_key()])
        .contents("[rust::e2e::file_append::2]")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .file_id
        .unwrap();

    FileAppendTransaction::new()
        .file_id(file_id)
        .contents(resources::BIG_CONTENTS)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let contents = FileContentsQuery::new().file_id(file_id).execute(&client).await?;

    assert_eq!(
        String::from_utf8(contents.contents).unwrap(),
        format!("[rust::e2e::file_append::2]{}", resources::BIG_CONTENTS)
    );

    let info = FileInfoQuery::new().file_id(file_id).execute(&client).await?;

    assert_eq!(info.file_id, file_id);
    assert_eq!(info.size, 13521);

    FileDeleteTransaction::new()
        .file_id(file_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}

#[tokio::test]
async fn large_contents_small_valid_duration() -> anyhow::Result<()> {
    // There are potential bugs in FileAppendTransaction which require more than one node to trigger.
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    // assume_not_local_node

    let file_id = FileCreateTransaction::new()
        .keys([op.private_key.public_key()])
        .contents("[rust::e2e::file_append::3]")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .file_id
        .unwrap();

    // note the transaction_valid_duration, this is the *only* difference between the last test and this one, I'm not sure it actually even properly tests that.
    FileAppendTransaction::new()
        .file_id(file_id)
        .contents(resources::BIG_CONTENTS)
        .transaction_valid_duration(Duration::seconds(25))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let contents = FileContentsQuery::new().file_id(file_id).execute(&client).await?;

    assert_eq!(
        String::from_utf8(contents.contents).unwrap(),
        format!("[rust::e2e::file_append::3]{}", resources::BIG_CONTENTS)
    );

    let info = FileInfoQuery::new().file_id(file_id).execute(&client).await?;

    assert_eq!(info.file_id, file_id);
    assert_eq!(info.size, 13521);

    FileDeleteTransaction::new()
        .file_id(file_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}
