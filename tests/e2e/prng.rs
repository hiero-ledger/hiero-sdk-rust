use hiero_sdk::PrngTransaction;

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

#[tokio::test]
async fn basic() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let record =
        PrngTransaction::new().range(100).execute(&client).await?.get_record(&client).await?;

    assert!(record.prng_number.is_some_and(|it| it < 100));

    Ok(())
}
