use hiero_sdk::FileId;

#[tokio::test]
async fn should_get_address_book_file_id_for_shard_realm() -> anyhow::Result<()> {
    let file_id = FileId::get_address_book_file_id_for(1, 1);
    assert_eq!(file_id.shard, 1);
    assert_eq!(file_id.realm, 1);
    Ok(())
}

#[tokio::test]
async fn should_get_exchange_rates_file_id_for_shard_realm() -> anyhow::Result<()> {
    let file_id = FileId::get_exchange_rates_file_id_for(1, 1);
    assert_eq!(file_id.shard, 1);
    assert_eq!(file_id.realm, 1);
    Ok(())
}

#[tokio::test]
async fn should_get_fee_schedule_file_id_for_shard_realm() -> anyhow::Result<()> {
    let file_id = FileId::get_fee_schedule_file_id_for(1, 1);
    assert_eq!(file_id.shard, 1);
    assert_eq!(file_id.realm, 1);
    Ok(())
}
