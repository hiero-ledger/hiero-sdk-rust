use hiero_sdk::{
    FeeSchedules,
    FileContentsQuery,
    FileId,
};

use crate::common::TestEnvironment;

#[tokio::test]
#[ignore]
async fn fetch_fee_schedules() -> anyhow::Result<()> {
    let TestEnvironment { client, config: _ } = crate::common::setup_global();

    let contents =
        FileContentsQuery::new().file_id(FileId::new(0, 0, 111)).execute(&client).await?.contents;

    let fee_schedules = FeeSchedules::from_bytes(&contents)?;

    assert!(fee_schedules.current.is_some());

    Ok(())
}
