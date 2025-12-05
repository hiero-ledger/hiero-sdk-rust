use std::collections::HashMap;

use hiero_sdk::Client;

#[tokio::test]
async fn initialize_with_mirror_network() -> anyhow::Result<()> {
    let mirror_network_str = "testnet.mirrornode.hedera.com:443";
    let client = Client::for_mirror_network(vec![mirror_network_str.to_owned()]).await?;
    let mirror_network = client.mirror_network();

    assert_eq!(mirror_network.len(), 1);
    assert_eq!(mirror_network[0], mirror_network_str);
    assert_ne!(client.network(), HashMap::new());

    Ok(())
}
