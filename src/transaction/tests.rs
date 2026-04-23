use std::collections::HashSet;
use std::time::Duration as StdDuration;

use assert_matches::assert_matches;
use hex_literal::hex;
use time::OffsetDateTime;

use crate::client::DEFAULT_GRPC_DEADLINE;
use crate::transaction::test_helpers::{
    unused_private_key,
    TEST_NODE_ACCOUNT_IDS,
    TEST_TX_ID,
};
use crate::transaction::AnyTransactionData;
use crate::{
    AnyTransaction,
    Client,
    FileAppendTransaction,
    Hbar,
    PrivateKey,
    TopicMessageSubmitTransaction,
    Transaction,
    TransactionId,
    TransferTransaction,
};

#[test]
fn to_bytes_from_bytes() -> crate::Result<()> {
    let mut tx = TransferTransaction::new();

    let bytes = tx
        .max_transaction_fee(Hbar::new(10))
        .transaction_valid_duration(time::Duration::seconds(119))
        .transaction_memo("hi hashgraph")
        .hbar_transfer(2.into(), Hbar::new(2))
        .hbar_transfer(101.into(), Hbar::new(-2))
        .transaction_id(TransactionId {
            account_id: 101.into(),
            valid_start: OffsetDateTime::now_utc(),
            nonce: None,
            scheduled: false,
        })
        .node_account_ids([6.into(), 7.into()])
        .freeze()?
        .to_bytes()?;

    let tx = tx;

    let lhs = tx.data();

    let tx2 = AnyTransaction::from_bytes(&bytes)?;

    let rhs = assert_matches!(tx2.data(), AnyTransactionData::Transfer(it) => it);

    assert_eq!(tx.get_max_transaction_fee(), tx2.get_max_transaction_fee());

    // note: they have *no* guaranteed order, so we have to convert to a `HashSet`...
    // `HashSet` makes this hard on us.
    {
        let lhs: Option<HashSet<_>> = tx.get_node_account_ids().map(|it| it.iter().collect());
        let rhs: Option<HashSet<_>> = tx2.get_node_account_ids().map(|it| it.iter().collect());
        assert_eq!(lhs, rhs);
    }

    assert_eq!(tx.get_transaction_id(), tx2.get_transaction_id());
    assert_eq!(tx.get_transaction_memo(), tx2.get_transaction_memo());
    assert_eq!(tx.get_transaction_valid_duration(), tx2.get_transaction_valid_duration());
    assert_eq!(lhs, rhs);
    assert!(tx2.sources.is_some());

    Ok(())
}

#[test]
fn signed_to_bytes_from_bytes_preserves_signatures() -> crate::Result<()> {
    let mut tx = TransferTransaction::new();

    // Build a minimal, frozen transaction (no network dependency)
    let mut tx = tx
        .max_transaction_fee(Hbar::new(10))
        .transaction_valid_duration(time::Duration::seconds(119))
        .transaction_memo("signed-preserve-test")
        .hbar_transfer(2.into(), Hbar::new(2))
        .hbar_transfer(101.into(), Hbar::new(-2))
        .transaction_id(TransactionId {
            account_id: 101.into(),
            valid_start: OffsetDateTime::now_utc(),
            nonce: None,
            scheduled: false,
        })
        .node_account_ids([6.into(), 7.into()])
        .freeze()?;

    // Sign with an arbitrary key
    let key: PrivateKey = "302e020100300506032b657004220420e40d4241d093b22910c78135e0501b137cd9205bbb9c0153c5adf2c65e7dc95a"
        .parse()
        .unwrap();
    tx.sign(key);

    // Serialize, then deserialize, then serialize again
    let bytes_before = tx.to_bytes()?;
    let tx2 = AnyTransaction::from_bytes(&bytes_before)?;
    let bytes_after = tx2.to_bytes()?;

    // If signatures are preserved, bytes should match
    assert_eq!(bytes_before, bytes_after);

    Ok(())
}

#[test]
fn from_bytes_sign_to_bytes() -> crate::Result<()> {
    let mut tx = TransferTransaction::new();

    let bytes = tx
        .max_transaction_fee(Hbar::new(10))
        .transaction_valid_duration(time::Duration::seconds(119))
        .transaction_memo("hi hashgraph")
        .hbar_transfer(2.into(), Hbar::new(2))
        .hbar_transfer(101.into(), Hbar::new(-2))
        .transaction_id(TransactionId {
            account_id: 101.into(),
            valid_start: OffsetDateTime::now_utc(),
            nonce: None,
            scheduled: false,
        })
        .node_account_ids([6.into(), 7.into()])
        .freeze()?
        .to_bytes()?;

    let mut tx2 = AnyTransaction::from_bytes(&bytes)?;

    tx2.sign(PrivateKey::from_bytes(&hex!("302e020100300506032b657004220420e40d4241d093b22910c78135e0501b137cd9205bbb9c0153c5adf2c65e7dc95a")).unwrap());

    let _bytes2 = tx2.to_bytes()?;

    // todo: check properties (but what properties?)

    Ok(())
}

#[tokio::test]
async fn chunked_to_from_bytes() -> crate::Result<()> {
    let client = Client::for_testnet();
    client.set_operator(0.into(), PrivateKey::generate_ed25519());

    let bytes = TopicMessageSubmitTransaction::new()
        .topic_id(314)
        .message(b"Hello, world!".to_vec())
        .chunk_size(8)
        .max_chunks(2)
        .transaction_id(TransactionId {
            account_id: 101.into(),
            valid_start: OffsetDateTime::now_utc(),
            nonce: None,
            scheduled: false,
        })
        .node_account_ids([6.into(), 7.into()])
        .freeze_with(&client)?
        .to_bytes()?;

    let _tx2 = AnyTransaction::from_bytes(&bytes)?;

    // todo: check properties

    Ok(())
}

#[tokio::test]
async fn test_client_grpc_deadline() {
    // Test 1: Client defaults to DEFAULT_GRPC_DEADLINE (10 seconds)
    let client = Client::for_testnet();
    assert_eq!(client.grpc_deadline(), DEFAULT_GRPC_DEADLINE);

    // Test 2: Can set client's grpc_deadline
    let custom_deadline = StdDuration::from_secs(5);
    client.set_grpc_deadline(custom_deadline);
    assert_eq!(client.grpc_deadline(), custom_deadline);

    // Test 3: Can change client's grpc_deadline multiple times
    let another_deadline = StdDuration::from_secs(2);
    client.set_grpc_deadline(another_deadline);
    assert_eq!(client.grpc_deadline(), another_deadline);
}

#[test]
fn test_transaction_grpc_deadline() {
    // Test 1: New transaction has no grpc_deadline set (None)
    let mut tx = TransferTransaction::new();
    assert_eq!(tx.get_grpc_deadline(), None);

    // Test 2: Can set transaction's grpc_deadline
    let deadline = StdDuration::from_secs(3);
    tx.grpc_deadline(deadline);
    assert_eq!(tx.get_grpc_deadline(), Some(deadline));

    // Test 3: Can change transaction's grpc_deadline
    let new_deadline = StdDuration::from_secs(7);
    tx.grpc_deadline(new_deadline);
    assert_eq!(tx.get_grpc_deadline(), Some(new_deadline));

    // Test 4: Transaction's grpc_deadline is independent per transaction
    let mut tx2 = TransferTransaction::new();
    assert_eq!(tx2.get_grpc_deadline(), None);

    tx2.grpc_deadline(StdDuration::from_secs(1));
    assert_eq!(tx2.get_grpc_deadline(), Some(StdDuration::from_secs(1)));
    // Original transaction still has its own deadline
    assert_eq!(tx.get_grpc_deadline(), Some(new_deadline));
}

#[tokio::test]
async fn test_transaction_grpc_deadline_precedence() {
    // Test that transaction's grpc_deadline can override client's default
    // This is tested through the Execute trait implementation

    let client = Client::for_testnet();
    client.set_grpc_deadline(StdDuration::from_secs(10));

    let mut tx = TransferTransaction::new();

    // Transaction without grpc_deadline should use client's default when executed
    // (We can't actually execute here, but we can verify the getter returns None)
    assert_eq!(tx.get_grpc_deadline(), None);

    // Transaction with grpc_deadline set should override client's default
    // Set the deadline before importing Execute trait to avoid method name conflict
    {
        // Call setter in a block without Execute trait in scope
        let deadline = StdDuration::from_secs(2);
        tx.grpc_deadline(deadline);
    }
    assert_eq!(tx.get_grpc_deadline(), Some(StdDuration::from_secs(2)));

    // Verify the transaction's deadline is accessible through the Execute trait
    {
        use crate::execute::Execute;
        let tx_ref = &tx;
        assert_eq!(Execute::grpc_deadline(tx_ref), Some(StdDuration::from_secs(2)));
    }
}

#[test]
fn test_grpc_deadline_preserved_through_clone() {
    let mut tx1 = TransferTransaction::new();
    tx1.grpc_deadline(StdDuration::from_secs(5));

    // Clone should preserve grpc_deadline
    let tx2 = tx1.clone();
    assert_eq!(tx2.get_grpc_deadline(), Some(StdDuration::from_secs(5)));

    // Modifying one shouldn't affect the other
    let mut tx3 = tx2.clone();
    tx3.grpc_deadline(StdDuration::from_secs(8));
    assert_eq!(tx1.get_grpc_deadline(), Some(StdDuration::from_secs(5)));
    assert_eq!(tx2.get_grpc_deadline(), Some(StdDuration::from_secs(5)));
    assert_eq!(tx3.get_grpc_deadline(), Some(StdDuration::from_secs(8)));
}

#[test]
fn test_offline_transaction_signing() -> crate::Result<()> {
    let mut tx = TransferTransaction::new();

    tx.node_account_ids(TEST_NODE_ACCOUNT_IDS)
        .transaction_id(TEST_TX_ID)
        .max_transaction_fee(Hbar::new(2))
        .freeze()?;

    let tx_bytes = tx.to_bytes()?;

    let private_key = unused_private_key();
    let signatures = private_key.sign_transaction_map(&mut tx)?;
    private_key.public_key().verify_transaction(&mut tx)?;

    let mut tx = Transaction::from_bytes(&tx_bytes)?;
    tx.freeze()?;
    tx.add_signature_map(signatures);
    private_key.public_key().verify_transaction(&mut tx)?;

    Ok(())
}

#[test]
fn test_offline_transaction_signing_with_chunks() -> crate::Result<()> {
    let mut tx = FileAppendTransaction::new();

    tx.node_account_ids(TEST_NODE_ACCOUNT_IDS)
        .transaction_id(TEST_TX_ID)
        .max_transaction_fee(Hbar::new(2))
        .contents(vec![0; 8000]) // contents that will require chunking
        .freeze()?;

    let tx_bytes = tx.to_bytes()?;

    let private_key = unused_private_key();
    let signatures = private_key.sign_transaction_map(&mut tx)?;

    private_key.public_key().verify_transaction(&mut tx)?;

    let mut tx = Transaction::from_bytes(&tx_bytes)?;
    tx.freeze()?;
    tx.add_signature_map(signatures);
    private_key.public_key().verify_transaction(&mut tx)?;

    Ok(())
}
