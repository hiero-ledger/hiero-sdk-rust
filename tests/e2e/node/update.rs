use std::collections::HashMap;

use hedera::{
    AccountCreateTransaction,
    AccountDeleteTransaction,
    AccountId,
    Client,
    Hbar,
    NodeAddressBookQuery,
    NodeUpdateTransaction,
    PrivateKey,
    ServiceEndpoint,
    Status,
};

/// Helper function to setup client with local DAB tests configuration
fn setup_dab_tests_client() -> Client {
    let mut network = HashMap::new();
    network.insert("127.0.0.1:50211".to_string(), AccountId::new(0, 0, 3));
    network.insert("127.0.0.1:51211".to_string(), AccountId::new(0, 0, 4));

    let client = Client::for_network(network).unwrap();
    client.set_mirror_network(vec!["127.0.0.1:5600".to_string()]);

    // Set the operator to be account 0.0.2
    let operator_account_id = AccountId::new(0, 0, 2);
    let operator_key = PrivateKey::from_str_der(
        "302e020100300506032b65700422042091132178e72057a1d7528025956fe39b0b847f200ab59b2fdd367017f3087137",
    )
    .unwrap();

    client.set_operator(operator_account_id, operator_key);
    client
}

#[tokio::test]
async fn can_execute_node_update_transaction() -> anyhow::Result<()> {
    let client = setup_dab_tests_client();

    let response = NodeUpdateTransaction::new()
        .node_id(0)
        .node_account_ids(vec![AccountId::new(0, 0, 4)])
        .description("testUpdated")
        .decline_reward(true)
        .grpc_proxy_endpoint(ServiceEndpoint {
            ip_address_v4: None,
            port: 123456,
            domain_name: "testWebUpdatedsdfsdfsdfsdf.com".to_owned(),
        })
        .execute(&client)
        .await?;

    let receipt = response.get_receipt(&client).await?;
    assert_eq!(receipt.status, Status::Success);

    Ok(())
}

#[tokio::test]
async fn can_delete_grpc_web_proxy_endpoint() -> anyhow::Result<()> {
    let client = setup_dab_tests_client();

    let response = NodeUpdateTransaction::new()
        .node_id(0)
        .node_account_ids(vec![AccountId::new(0, 0, 4)])
        .delete_grpc_proxy_endpoint()
        .execute(&client)
        .await?;

    let receipt = response.get_receipt(&client).await?;
    assert_eq!(receipt.status, Status::Success);

    Ok(())
}

#[tokio::test]
async fn can_change_node_account_id_and_revert_back() -> anyhow::Result<()> {
    let client = setup_dab_tests_client();

    // Change node account ID from 0.0.8 to 0.0.2
    let response1 = NodeUpdateTransaction::new()
        .node_id(0)
        .node_account_ids(vec![AccountId::new(0, 0, 4)])
        .account_id(AccountId::new(0, 0, 2))
        .execute(&client)
        .await?;

    let receipt1 = response1.get_receipt(&client).await?;
    assert_eq!(receipt1.status, Status::Success);

    // Revert the ID back to 0.0.8
    let response2 = NodeUpdateTransaction::new()
        .node_id(0)
        .node_account_ids(vec![AccountId::new(0, 0, 4)])
        .account_id(AccountId::new(0, 0, 3))
        .execute(&client)
        .await?;

    let receipt2 = response2.get_receipt(&client).await?;
    assert_eq!(receipt2.status, Status::Success);

    Ok(())
}

#[tokio::test]
async fn fails_with_invalid_signature_when_updating_without_admin_key() -> anyhow::Result<()> {
    let client = setup_dab_tests_client();

    // Create a new account to be the operator
    let new_operator_key = PrivateKey::generate_ed25519();
    let create_resp = AccountCreateTransaction::new()
        .set_key_without_alias(new_operator_key.public_key())
        .node_account_ids(vec![AccountId::new(0, 0, 4)])
        .initial_balance(Hbar::new(2))
        .execute(&client)
        .await?;

    let create_receipt = create_resp.get_receipt(&client).await?;
    let new_operator = create_receipt.account_id.unwrap();

    // Set the new account as operator
    client.set_operator(new_operator, new_operator_key);

    // Try to update node account ID without admin key signature
    let res = NodeUpdateTransaction::new()
        .node_id(0)
        .node_account_ids(vec![AccountId::new(0, 0, 4)])
        .description("testUpdated")
        .account_id(AccountId::new(0, 0, 50))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches::assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::InvalidSignature, .. })
    );

    Ok(())
}

#[tokio::test]
async fn can_change_node_account_id_to_the_same_account() -> anyhow::Result<()> {
    let client = setup_dab_tests_client();

    let response = NodeUpdateTransaction::new()
        .node_id(1)
        .node_account_ids(vec![AccountId::new(0, 0, 3)])
        .account_id(AccountId::new(0, 0, 4))
        .execute(&client)
        .await?;

    let receipt = response.get_receipt(&client).await?;
    assert_eq!(receipt.status, Status::Success);

    Ok(())
}

#[tokio::test]
async fn fails_when_changing_to_non_existent_account_id() -> anyhow::Result<()> {
    let client = setup_dab_tests_client();

    let res = NodeUpdateTransaction::new()
        .node_id(0)
        .node_account_ids(vec![AccountId::new(0, 0, 4)])
        .description("testUpdated")
        .account_id(AccountId::new(0, 0, 999999999))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches::assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::InvalidSignature, .. })
    );

    Ok(())
}

#[tokio::test]
async fn fails_when_changing_node_account_id_without_account_key() -> anyhow::Result<()> {
    let client = setup_dab_tests_client();

    // Create a new account
    let new_key = PrivateKey::generate_ed25519();
    let create_resp = AccountCreateTransaction::new()
        .set_key_without_alias(new_key.public_key())
        .node_account_ids(vec![AccountId::new(0, 0, 4)])
        .initial_balance(Hbar::new(2))
        .execute(&client)
        .await?;

    let create_receipt = create_resp.get_receipt(&client).await?;
    let new_node_account_id = create_receipt.account_id.unwrap();

    // Try to set node account ID to new account without signing with new account's key
    let res = NodeUpdateTransaction::new()
        .node_id(0)
        .node_account_ids(vec![AccountId::new(0, 0, 4)])
        .description("testUpdated")
        .account_id(new_node_account_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches::assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::InvalidSignature, .. })
    );

    Ok(())
}

#[tokio::test]
async fn fails_when_changing_to_deleted_account_id() -> anyhow::Result<()> {
    let client = setup_dab_tests_client();

    // Create a new account
    let new_account_key = PrivateKey::generate_ed25519();
    let create_resp = AccountCreateTransaction::new()
        .set_key_without_alias(new_account_key.public_key())
        .node_account_ids(vec![AccountId::new(0, 0, 4)])
        .execute(&client)
        .await?;

    let create_receipt = create_resp.get_receipt(&client).await?;
    let new_account = create_receipt.account_id.unwrap();

    // Delete the account
    let delete_resp = AccountDeleteTransaction::new()
        .account_id(new_account)
        .transfer_account_id(client.get_operator_account_id().unwrap())
        .freeze_with(&client)?
        .sign(new_account_key.clone())
        .execute(&client)
        .await?;

    delete_resp.get_receipt(&client).await?;

    // Try to set node account ID to deleted account
    let res = NodeUpdateTransaction::new()
        .node_id(0)
        .node_account_ids(vec![AccountId::new(0, 0, 4)])
        .description("testUpdated")
        .account_id(new_account)
        .freeze_with(&client)?
        .sign(new_account_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches::assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::AccountDeleted, .. })
    );

    Ok(())
}

#[tokio::test]
async fn fails_when_new_node_account_has_zero_balance() -> anyhow::Result<()> {
    let client = setup_dab_tests_client();

    // Create a new account with zero balance
    let new_account_key = PrivateKey::generate_ed25519();
    let create_resp = AccountCreateTransaction::new()
        .set_key_without_alias(new_account_key.public_key())
        .node_account_ids(vec![AccountId::new(0, 0, 4)])
        .execute(&client)
        .await?;

    let create_receipt = create_resp.get_receipt(&client).await?;
    let new_account = create_receipt.account_id.unwrap();

    // Try to set node account ID to account with zero balance
    let res = NodeUpdateTransaction::new()
        .node_id(0)
        .description("testUpdated")
        .account_id(new_account)
        .node_account_ids(vec![AccountId::new(0, 0, 4)])
        .freeze_with(&client)?
        .sign(new_account_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    // Should fail with status code 526 (NODE_ACCOUNT_HAS_ZERO_BALANCE)
    assert_matches::assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::NodeAccountHasZeroBalance, .. })
    );

    Ok(())
}

#[tokio::test]
async fn updates_addressbook_and_retries_after_node_account_id_change() -> anyhow::Result<()> {
    let client = setup_dab_tests_client();

    // Create the account that will be the new node account ID
    let new_account_key = PrivateKey::generate_ed25519();
    let create_resp = AccountCreateTransaction::new()
        .set_key_without_alias(new_account_key.public_key())
        .initial_balance(Hbar::new(1))
        .execute(&client)
        .await?;

    let create_receipt = create_resp.get_receipt(&client).await?;
    let new_node_account_id = create_receipt.account_id.unwrap();

    // Update node account ID (0.0.4 -> new_node_account_id)
    let update_resp = NodeUpdateTransaction::new()
        .node_id(1)
        .account_id(new_node_account_id)
        .node_account_ids(vec![AccountId::new(0, 0, 3)])
        .freeze_with(&client)?
        .sign(new_account_key)
        .execute(&client)
        .await?;

    update_resp.get_receipt(&client).await?;

    // Wait for mirror node to import data
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    let another_new_key = PrivateKey::generate_ed25519();

    // Submit to the updated node - should trigger addressbook refresh
    let test_resp = AccountCreateTransaction::new()
        .set_key_without_alias(another_new_key.public_key())
        .node_account_ids(vec![AccountId::new(0, 0, 4), AccountId::new(0, 0, 3)])
        .execute(&client)
        .await?;

    let test_receipt = test_resp.get_receipt(&client).await?;
    assert_eq!(test_receipt.status, Status::Success);

    // Verify address book has been updated
    let network = client.network();
    let has_new_node_account =
        network.values().any(|account_id| *account_id == new_node_account_id);

        println!("network: {:?}", network);
    assert!(has_new_node_account, "Address book should contain the new node account ID");

    // Check if the new node account has the specific address and apply workaround if needed
    let network = client.network();
    let node_address = network
        .iter()
        .find(|(_, account_id)| **account_id == new_node_account_id)
        .map(|(address, _)| address);

    if let Some(address) = node_address {
        assert_eq!(
            address,
            "network-node2-svc.solo.svc.cluster.local:50211",
            "Expected node with account {} to have address network-node2-svc.solo.svc.cluster.local:50211",
            new_node_account_id
        );

        // Apply workaround: change port from 50211 to 51211
        let mut updated_network = HashMap::new();
        for (addr, acc_id) in network.iter() {
            if *acc_id == new_node_account_id {
                updated_network
                    .insert("network-node2-svc.solo.svc.cluster.local:51211".to_string(), *acc_id);
            } else {
                updated_network.insert(addr.clone(), *acc_id);
            }
        }
        client.set_network(updated_network)?;
    }

    // This transaction should succeed with the new node account ID
    let final_resp = AccountCreateTransaction::new()
        .set_key_without_alias(another_new_key.public_key())
        .node_account_ids(vec![new_node_account_id])
        .execute(&client)
        .await?;

    let final_receipt = final_resp.get_receipt(&client).await?;
    assert_eq!(final_receipt.status, Status::Success);

    // Revert the node account ID
    let revert_resp = NodeUpdateTransaction::new()
        .node_id(1)
        .node_account_ids(vec![new_node_account_id])
        .account_id(AccountId::new(0, 0, 4))
        .execute(&client)
        .await?;

    revert_resp.get_receipt(&client).await?;

    Ok(())
}

#[tokio::test]
async fn handles_node_account_id_change_without_mirror_node_setup() -> anyhow::Result<()> {
    // Create a client without mirror network
    let network_client = setup_dab_tests_client();

    // Remove mirror network
    network_client.set_mirror_network(vec![]);

    // Create the account that will be the new node account ID
    let new_account_key = PrivateKey::generate_ed25519();
    let create_resp = AccountCreateTransaction::new()
        .set_key_without_alias(new_account_key.public_key())
        .node_account_ids(vec![AccountId::new(0, 0, 3), AccountId::new(0, 0, 4)])
        .initial_balance(Hbar::new(1))
        .execute(&network_client)
        .await?;

    let create_receipt = create_resp.get_receipt(&network_client).await?;
    let new_node_account_id = create_receipt.account_id.unwrap();

    // Update node account ID
    let update_resp = NodeUpdateTransaction::new()
        .node_id(1)
        .account_id(new_node_account_id)
        .node_account_ids(vec![AccountId::new(0, 0, 3)])
        .freeze_with(&network_client)?
        .sign(new_account_key)
        .execute(&network_client)
        .await?;

    update_resp.get_receipt(&network_client).await?;

    // Wait for changes to propagate
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    let another_new_key = PrivateKey::generate_ed25519();

    // Submit transaction - should retry since no mirror node to update addressbook
    let test_resp = AccountCreateTransaction::new()
        .set_key_without_alias(another_new_key.public_key())
        .node_account_ids(vec![AccountId::new(0, 0, 4), AccountId::new(0, 0, 3)])
        .execute(&network_client)
        .await?;

    let test_receipt = test_resp.get_receipt(&network_client).await?;
    assert_eq!(test_receipt.status, Status::Success);

    // Verify address book has NOT been updated (no mirror node)
    let network = network_client.network();
    let node1 = network.iter().find(|(_, account_id)| **account_id == AccountId::new(0, 0, 3));
    let node2 = network.iter().find(|(_, account_id)| **account_id == AccountId::new(0, 0, 4));

    assert!(node1.is_some(), "Node 0.0.3 should still be in the network state");
    assert!(node2.is_some(), "Node 0.0.4 should still be in the network state");

    // This transaction should succeed with retries
    let final_resp = AccountCreateTransaction::new()
        .set_key_without_alias(another_new_key.public_key())
        .node_account_ids(vec![AccountId::new(0, 0, 4), AccountId::new(0, 0, 3)])
        .execute(&network_client)
        .await?;

    let final_receipt = final_resp.get_receipt(&network_client).await?;
    assert_eq!(final_receipt.status, Status::Success);

    // Revert the node account ID
    let revert_resp = NodeUpdateTransaction::new()
        .node_id(1)
        .node_account_ids(vec![ AccountId::new(0, 0, 3)])
        .account_id(AccountId::new(0, 0, 4))
        .execute(&network_client)
        .await?;

    revert_resp.get_receipt(&network_client).await?;
    // Wait for changes to propagate
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    Ok(())
}
