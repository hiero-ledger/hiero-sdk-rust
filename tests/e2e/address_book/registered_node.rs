use std::net::Ipv4Addr;
use std::str::FromStr;

use assert_matches::assert_matches;
use hiero_sdk::{
    AccountId,
    BlockNodeApi,
    PrivateKey,
    RegisteredEndpointAddress,
    RegisteredEndpointType,
    RegisteredNodeCreateTransaction,
    RegisteredNodeDeleteTransaction,
    RegisteredNodeUpdateTransaction,
    RegisteredServiceEndpoint,
    Status,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

/// Set the operator to account 0.0.2 (address book admin) which has permission
/// to manage registered nodes.
fn set_address_book_operator(client: &hiero_sdk::Client) {
    let operator_key = PrivateKey::from_str(
        "302e020100300506032b65700422042091132178e72057a1d7528025956fe39b0b847f200ab59b2fdd367017f3087137",
    )
    .unwrap();
    client.set_operator(AccountId::new(0, 0, 2), operator_key);
}

fn make_block_node_endpoint() -> RegisteredServiceEndpoint {
    RegisteredServiceEndpoint {
        address: RegisteredEndpointAddress::Ipv4(Ipv4Addr::new(127, 0, 0, 1)),
        port: 8080,
        requires_tls: true,
        endpoint_type: RegisteredEndpointType::BlockNode(vec![
            BlockNodeApi::Status,
            BlockNodeApi::Publish,
        ]),
    }
}

fn make_mirror_node_endpoint() -> RegisteredServiceEndpoint {
    RegisteredServiceEndpoint {
        address: RegisteredEndpointAddress::DomainName("mirror.example.com".to_string()),
        port: 443,
        requires_tls: true,
        endpoint_type: RegisteredEndpointType::MirrorNode,
    }
}

fn make_rpc_relay_endpoint() -> RegisteredServiceEndpoint {
    RegisteredServiceEndpoint {
        address: RegisteredEndpointAddress::Ipv4(Ipv4Addr::new(10, 0, 0, 1)),
        port: 7546,
        requires_tls: false,
        endpoint_type: RegisteredEndpointType::RpcRelay,
    }
}

fn make_general_service_endpoint() -> RegisteredServiceEndpoint {
    RegisteredServiceEndpoint {
        address: RegisteredEndpointAddress::DomainName("api.example.com".to_string()),
        port: 9090,
        requires_tls: true,
        endpoint_type: RegisteredEndpointType::GeneralService("Custom API".to_string()),
    }
}

/// Helper: create a registered node and return its ID.
async fn create_registered_node(
    client: &hiero_sdk::Client,
    admin_key: &PrivateKey,
    description: &str,
    endpoints: Vec<RegisteredServiceEndpoint>,
) -> anyhow::Result<u64> {
    let receipt = RegisteredNodeCreateTransaction::new()
        .admin_key(admin_key.public_key())
        .description(description)
        .service_endpoints(endpoints)
        .freeze_with(client)?
        .sign(admin_key.clone())
        .execute(client)
        .await?
        .get_receipt(client)
        .await?;

    Ok(receipt.registered_node_id)
}

/// Helper: delete a registered node.
async fn delete_registered_node(
    client: &hiero_sdk::Client,
    admin_key: &PrivateKey,
    registered_node_id: u64,
) -> anyhow::Result<()> {
    RegisteredNodeDeleteTransaction::new()
        .registered_node_id(registered_node_id)
        .freeze_with(client)?
        .sign(admin_key.clone())
        .execute(client)
        .await?
        .get_receipt(client)
        .await?;
    Ok(())
}

// ─── Create transaction tests ───

#[tokio::test]
async fn can_create_registered_node_with_block_node() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };
    set_address_book_operator(&client);

    let admin_key = PrivateKey::generate_ed25519();

    let registered_node_id =
        create_registered_node(&client, &admin_key, "block node", vec![make_block_node_endpoint()])
            .await?;

    delete_registered_node(&client, &admin_key, registered_node_id).await?;

    Ok(())
}

#[tokio::test]
async fn can_create_registered_node_with_mirror_node() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };
    set_address_book_operator(&client);

    let admin_key = PrivateKey::generate_ed25519();

    let registered_node_id = create_registered_node(
        &client,
        &admin_key,
        "mirror node",
        vec![make_mirror_node_endpoint()],
    )
    .await?;

    delete_registered_node(&client, &admin_key, registered_node_id).await?;

    Ok(())
}

#[tokio::test]
async fn can_create_registered_node_with_rpc_relay() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };
    set_address_book_operator(&client);

    let admin_key = PrivateKey::generate_ed25519();

    let registered_node_id =
        create_registered_node(&client, &admin_key, "rpc relay", vec![make_rpc_relay_endpoint()])
            .await?;

    delete_registered_node(&client, &admin_key, registered_node_id).await?;

    Ok(())
}

#[tokio::test]
async fn can_create_registered_node_with_general_service() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };
    set_address_book_operator(&client);

    let admin_key = PrivateKey::generate_ed25519();

    let registered_node_id = create_registered_node(
        &client,
        &admin_key,
        "general service",
        vec![make_general_service_endpoint()],
    )
    .await?;

    delete_registered_node(&client, &admin_key, registered_node_id).await?;

    Ok(())
}

#[tokio::test]
async fn can_create_registered_node_with_multiple_endpoint_types() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };
    set_address_book_operator(&client);

    let admin_key = PrivateKey::generate_ed25519();

    let registered_node_id = create_registered_node(
        &client,
        &admin_key,
        "multi-endpoint node",
        vec![
            make_block_node_endpoint(),
            make_mirror_node_endpoint(),
            make_rpc_relay_endpoint(),
            make_general_service_endpoint(),
        ],
    )
    .await?;

    delete_registered_node(&client, &admin_key, registered_node_id).await?;

    Ok(())
}

#[tokio::test]
async fn can_create_registered_node_with_domain_name() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };
    set_address_book_operator(&client);

    let admin_key = PrivateKey::generate_ed25519();

    let endpoint = RegisteredServiceEndpoint {
        address: RegisteredEndpointAddress::DomainName("blocknode.example.com".to_string()),
        port: 443,
        requires_tls: true,
        endpoint_type: RegisteredEndpointType::BlockNode(vec![
            BlockNodeApi::Status,
            BlockNodeApi::SubscribeStream,
            BlockNodeApi::StateProof,
        ]),
    };

    let registered_node_id =
        create_registered_node(&client, &admin_key, "domain block node", vec![endpoint]).await?;

    delete_registered_node(&client, &admin_key, registered_node_id).await?;

    Ok(())
}

#[tokio::test]
async fn create_registered_node_fails_without_admin_key() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };
    set_address_book_operator(&client);

    let res = RegisteredNodeCreateTransaction::new()
        .service_endpoints(vec![make_block_node_endpoint()])
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::TransactionPreCheckStatus { status: Status::KeyRequired, .. })
    );

    Ok(())
}

#[tokio::test]
async fn create_registered_node_fails_with_empty_endpoints() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };
    set_address_book_operator(&client);

    let admin_key = PrivateKey::generate_ed25519();

    let res = RegisteredNodeCreateTransaction::new()
        .admin_key(admin_key.public_key())
        .freeze_with(&client)?
        .sign(admin_key)
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::TransactionPreCheckStatus { status: Status::InvalidRegisteredEndpoint, .. })
    );

    Ok(())
}

// ─── Update transaction tests ───

#[tokio::test]
async fn can_update_registered_node_description() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };
    set_address_book_operator(&client);

    let admin_key = PrivateKey::generate_ed25519();

    let registered_node_id = create_registered_node(
        &client,
        &admin_key,
        "original description",
        vec![make_block_node_endpoint()],
    )
    .await?;

    RegisteredNodeUpdateTransaction::new()
        .registered_node_id(registered_node_id)
        .description("updated description")
        .freeze_with(&client)?
        .sign(admin_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    delete_registered_node(&client, &admin_key, registered_node_id).await?;

    Ok(())
}

#[tokio::test]
async fn can_update_registered_node_service_endpoints() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };
    set_address_book_operator(&client);

    let admin_key = PrivateKey::generate_ed25519();

    let registered_node_id = create_registered_node(
        &client,
        &admin_key,
        "endpoint update test",
        vec![make_block_node_endpoint()],
    )
    .await?;

    // Replace endpoints with different ones
    RegisteredNodeUpdateTransaction::new()
        .registered_node_id(registered_node_id)
        .service_endpoints(vec![make_mirror_node_endpoint(), make_rpc_relay_endpoint()])
        .freeze_with(&client)?
        .sign(admin_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    delete_registered_node(&client, &admin_key, registered_node_id).await?;

    Ok(())
}

#[tokio::test]
async fn can_rotate_admin_key() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };
    set_address_book_operator(&client);

    let old_admin_key = PrivateKey::generate_ed25519();
    let new_admin_key = PrivateKey::generate_ed25519();

    let registered_node_id = create_registered_node(
        &client,
        &old_admin_key,
        "key rotation test",
        vec![make_block_node_endpoint()],
    )
    .await?;

    // Rotate admin key — both old and new must sign
    RegisteredNodeUpdateTransaction::new()
        .registered_node_id(registered_node_id)
        .admin_key(new_admin_key.public_key())
        .freeze_with(&client)?
        .sign(old_admin_key)
        .sign(new_admin_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Delete with the new key to verify the rotation worked
    delete_registered_node(&client, &new_admin_key, registered_node_id).await?;

    Ok(())
}

#[tokio::test]
async fn can_replace_ip_address_with_domain() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };
    set_address_book_operator(&client);

    let admin_key = PrivateKey::generate_ed25519();

    // Create with IP address
    let registered_node_id = create_registered_node(
        &client,
        &admin_key,
        "ip to domain test",
        vec![make_block_node_endpoint()],
    )
    .await?;

    // Update to domain name
    let domain_endpoint = RegisteredServiceEndpoint {
        address: RegisteredEndpointAddress::DomainName("new-blocknode.example.com".to_string()),
        port: 443,
        requires_tls: true,
        endpoint_type: RegisteredEndpointType::BlockNode(vec![
            BlockNodeApi::Status,
            BlockNodeApi::Publish,
        ]),
    };

    RegisteredNodeUpdateTransaction::new()
        .registered_node_id(registered_node_id)
        .service_endpoints(vec![domain_endpoint])
        .freeze_with(&client)?
        .sign(admin_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    delete_registered_node(&client, &admin_key, registered_node_id).await?;

    Ok(())
}

#[tokio::test]
async fn admin_key_rotation_fails_without_new_key_signature() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };
    set_address_book_operator(&client);

    let old_admin_key = PrivateKey::generate_ed25519();
    let new_admin_key = PrivateKey::generate_ed25519();

    let registered_node_id = create_registered_node(
        &client,
        &old_admin_key,
        "missing new key sig test",
        vec![make_block_node_endpoint()],
    )
    .await?;

    // Attempt rotation signed only by old key (missing new key signature)
    let res = RegisteredNodeUpdateTransaction::new()
        .registered_node_id(registered_node_id)
        .admin_key(new_admin_key.public_key())
        .freeze_with(&client)?
        .sign(old_admin_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, .. })
    );

    // Clean up with the old key (rotation did not succeed)
    delete_registered_node(&client, &old_admin_key, registered_node_id).await?;

    Ok(())
}

// ─── Delete transaction tests ───

#[tokio::test]
async fn can_delete_registered_node() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };
    set_address_book_operator(&client);

    let admin_key = PrivateKey::generate_ed25519();

    let registered_node_id = create_registered_node(
        &client,
        &admin_key,
        "delete test",
        vec![make_block_node_endpoint()],
    )
    .await?;

    delete_registered_node(&client, &admin_key, registered_node_id).await?;

    Ok(())
}

#[tokio::test]
async fn delete_registered_node_fails_with_invalid_id() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };
    set_address_book_operator(&client);

    let admin_key = PrivateKey::generate_ed25519();

    let res = RegisteredNodeDeleteTransaction::new()
        .registered_node_id(999_999_999)
        .freeze_with(&client)?
        .sign(admin_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidRegisteredNodeId, .. })
    );

    Ok(())
}

#[tokio::test]
async fn delete_already_deleted_registered_node_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };
    set_address_book_operator(&client);

    let admin_key = PrivateKey::generate_ed25519();

    let registered_node_id = create_registered_node(
        &client,
        &admin_key,
        "double delete test",
        vec![make_block_node_endpoint()],
    )
    .await?;

    // First delete succeeds
    delete_registered_node(&client, &admin_key, registered_node_id).await?;

    // Second delete should fail
    let res = RegisteredNodeDeleteTransaction::new()
        .registered_node_id(registered_node_id)
        .freeze_with(&client)?
        .sign(admin_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidRegisteredNodeId, .. })
    );

    Ok(())
}

// ─── Full lifecycle test ───

#[tokio::test]
async fn registered_node_full_lifecycle() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };
    set_address_book_operator(&client);

    let admin_key = PrivateKey::generate_ed25519();

    // Step 1: Create with block node endpoint
    let registered_node_id = create_registered_node(
        &client,
        &admin_key,
        "lifecycle test node",
        vec![make_block_node_endpoint()],
    )
    .await?;

    // Step 2: Update description
    RegisteredNodeUpdateTransaction::new()
        .registered_node_id(registered_node_id)
        .description("updated lifecycle node")
        .freeze_with(&client)?
        .sign(admin_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Step 3: Update endpoints — add multiple types
    RegisteredNodeUpdateTransaction::new()
        .registered_node_id(registered_node_id)
        .service_endpoints(vec![
            make_block_node_endpoint(),
            make_mirror_node_endpoint(),
            make_rpc_relay_endpoint(),
            make_general_service_endpoint(),
        ])
        .freeze_with(&client)?
        .sign(admin_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Step 4: Rotate admin key
    let new_admin_key = PrivateKey::generate_ed25519();

    RegisteredNodeUpdateTransaction::new()
        .registered_node_id(registered_node_id)
        .admin_key(new_admin_key.public_key())
        .freeze_with(&client)?
        .sign(admin_key)
        .sign(new_admin_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Step 5: Delete with new admin key
    delete_registered_node(&client, &new_admin_key, registered_node_id).await?;

    Ok(())
}
