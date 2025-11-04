use hedera::{
    AccountCreateTransaction,
    AccountDeleteTransaction,
    AccountId,
    Hbar,
    NodeUpdateTransaction,
    PrivateKey,
    Status,
};

use crate::common::{
    setup_nonfree,
    setup_nonfree_local_two_nodes as _,
    TestEnvironment,
};

#[tokio::test]
async fn can_change_node_account_id_to_the_same_account() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let receipt = NodeUpdateTransaction::new()
        .node_id(0)
        .description("testUpdated")
        .account_id(AccountId::new(0, 0, 3))
        .node_account_ids(vec![AccountId::new(0, 0, 3)])
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    assert_eq!(receipt.status, Status::Success);

    Ok(())
}

// #[tokio::test]
// async fn can_change_node_account_id() -> anyhow::Result<()> {
//     let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
//         return Ok(());
//     };
//
//     let receipt = NodeUpdateTransaction::new()
//         .node_id(0)
//         .description("testUpdated")
//         .account_id(AccountId::new(0, 0, 3))
//         .node_account_ids(vec![AccountId::new(0, 0, 3)])
//         .execute(&client)
//         .await?
//         .get_receipt(&client)
//         .await?;
//
//     assert_eq!(receipt.status, Status::Success);
//
//     Ok(())
// }

#[tokio::test]
async fn can_change_node_account_id_invalid_signature() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let new_operator_key = PrivateKey::generate_ed25519();
    let new_balance = Hbar::new(2);

    let operator = AccountCreateTransaction::new()
        .set_key_without_alias(new_operator_key.public_key())
        .initial_balance(new_balance)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    client.set_operator(operator, new_operator_key);

    let res = NodeUpdateTransaction::new()
        .node_id(0)
        .description("testUpdated")
        .account_id(AccountId::new(0, 0, 3))
        .node_account_ids(vec![AccountId::new(0, 0, 3)])
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
async fn can_change_node_account_id_to_non_existent_account_id() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let res = NodeUpdateTransaction::new()
        .node_id(0)
        .description("testUpdated")
        .account_id(AccountId::new(0, 0, 9999999))
        .node_account_ids(vec![AccountId::new(0, 0, 3)])
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches::assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::InvalidNodeAccountId, .. })
    );

    Ok(())
}

#[tokio::test]
#[ignore = "TODO: unskip when services implements check for this"]
async fn can_change_node_account_id_to_deleted_account_id() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let new_account_key = PrivateKey::generate_ed25519();

    let new_account = AccountCreateTransaction::new()
        .set_key_without_alias(new_account_key.public_key())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    AccountDeleteTransaction::new()
        .account_id(new_account)
        .transfer_account_id(client.get_operator_account_id().unwrap())
        .freeze_with(&client)?
        .sign(new_account_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let res = NodeUpdateTransaction::new()
        .node_id(0)
        .description("testUpdated")
        .account_id(new_account)
        .node_account_ids(vec![AccountId::new(0, 0, 3)])
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

// #[tokio::test]
// async fn can_change_node_account_no_balance() -> anyhow::Result<()> {
//     let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
//         return Ok(());
//     };
//
//     let new_account_key = PrivateKey::generate_ed25519();
//
//     let new_account = AccountCreateTransaction::new()
//         .set_key_without_alias(new_account_key.public_key())
//         .execute(&client)
//         .await?
//         .get_receipt(&client)
//         .await?
//         .account_id
//         .unwrap();
//
//     let res = NodeUpdateTransaction::new()
//         .node_id(0)
//         .description("testUpdated")
//         .account_id(new_account)
//         .node_account_ids(vec![AccountId::new(0, 0, 3)])
//         .freeze_with(&client)?
//         .sign(new_account_key)
//         .execute(&client)
//         .await?
//         .get_receipt(&client)
//         .await;
//
//     assert_matches::assert_matches!(
//         res,
//         Err(hedera::Error::ReceiptStatus { status: Status::InvalidNodeAccount, .. })
//     );
//
//     Ok(())
// }

// #[tokio::test]
// async fn can_change_node_account_update_addressbook_and_retry() -> anyhow::Result<()> {
//     let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
//         return Ok(());
//     };
//
//     // create the account that will be the node account id
//     let new_node_account_id = AccountCreateTransaction::new()
//         .set_key_without_alias(client.get_operator_public_key().unwrap())
//         .initial_balance(Hbar::new(1))
//         .node_account_ids(vec![AccountId::new(0, 0, 3)])
//         .execute(&client)
//         .await?
//         .get_receipt(&client)
//         .await?
//         .account_id
//         .unwrap();
//
//     // update node account id
//     // 0.0.3 -> new account id
//     NodeUpdateTransaction::new()
//         .node_id(0)
//         .description("testUpdated")
//         .account_id(new_node_account_id)
//         .node_account_ids(vec![AccountId::new(0, 0, 3)])
//         .execute(&client)
//         .await?
//         .get_receipt(&client)
//         .await?;
//
//     // wait for mirror node to import data
//     tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
//
//     let new_account_key = PrivateKey::generate_ed25519();
//
//     // submit to node 3 and node 4, node 3 fails, node 4 succeeds
//     AccountCreateTransaction::new()
//         .set_key_without_alias(new_account_key.public_key())
//         .node_account_ids(vec![AccountId::new(0, 0, 3), AccountId::new(0, 0, 4)])
//         .execute(&client)
//         .await?
//         .get_receipt(&client)
//         .await?;
//
//     // verify address book has been updated
//     // let network = client.network();
//     // assert_eq!(network.get("127.0.0.1:50211"), Some(&new_node_account_id));
//     // assert_eq!(network.get("127.0.0.1:51211"), Some(&AccountId::new(0, 0, 4)));
//
//     // this transaction should succeed
//     AccountCreateTransaction::new()
//         .set_key_without_alias(new_account_key.public_key())
//         .node_account_ids(vec![new_node_account_id])
//         .execute(&client)
//         .await?
//         .get_receipt(&client)
//         .await?;
//
//     // revert the node account id
//     NodeUpdateTransaction::new()
//         .node_id(0)
//         .node_account_ids(vec![new_node_account_id])
//         .description("testUpdated")
//         .account_id(AccountId::new(0, 0, 3))
//         .execute(&client)
//         .await?
//         .get_receipt(&client)
//         .await?;
//
//     Ok(())
// }
