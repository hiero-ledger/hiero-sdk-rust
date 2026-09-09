use std::collections::HashMap;
use std::time::Duration;

use anyhow::Context;
use hiero_sdk::{
    AccountId,
    Client,
};

use crate::common::{
    setup_global,
    TestEnvironment,
};

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

// Every node of the network answers the COST_ANSWER probe, both one at a time and via `ping_all`.
#[tokio::test]
async fn ping_all_network_nodes() -> anyhow::Result<()> {
    let TestEnvironment { client, .. } = setup_global();

    for (address, node_account_id) in client.network() {
        client
            .ping(node_account_id)
            .await
            .with_context(|| format!("node {node_account_id} at {address} should be reachable"))?;
    }

    client.ping_all().await?;

    Ok(())
}

// The probe is free and unsigned, so a client without an operator can ping.
#[tokio::test]
async fn ping_without_operator() -> anyhow::Result<()> {
    let TestEnvironment { client, .. } = setup_global();

    let client_without_operator = Client::for_network(client.network())?;

    for (address, node_account_id) in client_without_operator.network() {
        client_without_operator
            .ping(node_account_id)
            .await
            .with_context(|| format!("node {node_account_id} at {address} should be reachable"))?;
    }

    Ok(())
}

// A gRPC-level failure fails the ping.
#[tokio::test]
async fn ping_unreachable_node_fails() -> anyhow::Result<()> {
    // Nothing listens on port 1, so the connection is refused.
    let node_account_id = AccountId::new(0, 0, 99);
    let client = Client::for_network(HashMap::from([("127.0.0.1:1".to_owned(), node_account_id)]))?;
    client.set_max_attempts(1);

    let result = client.ping_with_timeout(node_account_id, Duration::from_secs(10)).await;

    assert!(result.is_err(), "pinging an unreachable node must fail");

    Ok(())
}
