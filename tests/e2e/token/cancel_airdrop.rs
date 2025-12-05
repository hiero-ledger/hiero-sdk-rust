use std::iter::repeat;

use anyhow::anyhow;
use assert_matches::assert_matches;
use hiero_sdk::{
    AccountBalanceQuery,
    PrivateKey,
    Status,
    TokenAirdropTransaction,
    TokenAssociateTransaction,
    TokenCancelAirdropTransaction,
    TokenFreezeTransaction,
    TokenMintTransaction,
    TokenPauseTransaction,
    TransactionId,
};

use crate::account::Account;
use crate::common::{
    setup_nonfree,
    Config,
    TestEnvironment,
};
use crate::token::{
    FungibleToken,
    Nft,
    TEST_AMOUNT,
    TEST_FUNGIBLE_INITIAL_BALANCE,
    TEST_MINTED_NFTS,
};

#[tokio::test]
async fn basic() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;

    // Create a receiver account with unlimited auto associations
    let receiver_key = PrivateKey::generate_ed25519();
    let receiver_account = Account::create_with_max_associations(0, &receiver_key, &client).await?;

    // Create a token and an NFT
    let token = FungibleToken::create_ft(&client, &operator_account, 3).await?;
    let nft = Nft::create(&client, &operator_account).await?;

    // Mint Nfts
    let mint_receipt = TokenMintTransaction::new()
        .token_id(nft.id)
        .metadata(repeat(vec![9, 1, 6]).take(10).collect::<Vec<Vec<u8>>>())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let nft_serials = mint_receipt.serials;

    // Airdrop tokens
    let record = TokenAirdropTransaction::new()
        .nft_transfer(
            nft.id.nft(nft_serials[0].try_into().unwrap()),
            operator_account.id,
            receiver_account.id,
        )
        .nft_transfer(
            nft.id.nft(nft_serials[1].try_into().unwrap()),
            operator_account.id,
            receiver_account.id,
        )
        .token_transfer(token.id, receiver_account.id, TEST_AMOUNT)
        .token_transfer(token.id, operator_account.id, -TEST_AMOUNT)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    let record = TokenCancelAirdropTransaction::new()
        .add_pending_airdrop_id(record.pending_airdrop_records[0].pending_airdrop_id)
        .add_pending_airdrop_id(record.pending_airdrop_records[1].pending_airdrop_id)
        .add_pending_airdrop_id(record.pending_airdrop_records[2].pending_airdrop_id)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    // Verify transaction record
    assert_eq!(record.pending_airdrop_records.len(), 0);

    // Verify the receiver holds the tokens via query
    let receiver_account_balance =
        AccountBalanceQuery::new().account_id(receiver_account.id).execute(&client).await?;

    assert_eq!(receiver_account_balance.tokens.get(&token.id), None);
    assert_eq!(receiver_account_balance.tokens.get(&nft.id), None);

    // Verify the operator does not hold the tokens
    let operator_account_balance =
        AccountBalanceQuery::new().account_id(operator_account.id).execute(&client).await?;

    assert_eq!(operator_account_balance.tokens.get(&token.id), Some(TEST_FUNGIBLE_INITIAL_BALANCE));
    assert_eq!(operator_account_balance.tokens.get(&nft.id), Some(TEST_MINTED_NFTS));

    Ok(())
}

#[tokio::test]
async fn cancel_frozen_tokens() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;

    // Create a receiver with 0 auto associations
    let receiver_key = PrivateKey::generate_ed25519();
    let receiver_account = Account::create_with_max_associations(0, &receiver_key, &client).await?;

    // Create a token
    let token = FungibleToken::create_ft(&client, &operator_account, 3).await?;

    // Airdrop the tokens to both
    let record = TokenAirdropTransaction::new()
        .token_transfer(token.id, receiver_account.id, TEST_AMOUNT)
        .token_transfer(token.id, operator_account.id, -TEST_AMOUNT)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    // Associate the token
    let _ = TokenAssociateTransaction::new()
        .account_id(receiver_account.id)
        .token_ids([token.id])
        .freeze_with(&client)?
        .sign(receiver_account.key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Freeze the token
    let _ = TokenFreezeTransaction::new()
        .token_id(token.id)
        .account_id(receiver_account.id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Cancel the airdrop
    let _ = TokenCancelAirdropTransaction::new()
        .add_pending_airdrop_id(record.pending_airdrop_records[0].pending_airdrop_id)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    Ok(())
}

#[tokio::test]
async fn cancel_paused_tokens() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;

    // Create a receiver with 0 auto associations
    let receiver_key = PrivateKey::generate_ed25519();
    let receiver_account = Account::create_with_max_associations(0, &receiver_key, &client).await?;

    // Create a token
    let token = FungibleToken::create_ft(&client, &operator_account, 3).await?;

    // Airdrop the tokens to both
    let record = TokenAirdropTransaction::new()
        .token_transfer(token.id, receiver_account.id, TEST_AMOUNT)
        .token_transfer(token.id, operator_account.id, -TEST_AMOUNT)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    // Pause the token
    let _ = TokenPauseTransaction::new()
        .token_id(token.id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Cancel the airdrop
    let _ = TokenCancelAirdropTransaction::new()
        .add_pending_airdrop_id(record.pending_airdrop_records[0].pending_airdrop_id)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    Ok(())
}

#[tokio::test]
async fn cancel_to_multiple_receivers() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;

    // Create a receiver1 with 0 auto associations
    let receiver_key_1 = PrivateKey::generate_ed25519();
    let receiver_account_1 =
        Account::create_with_max_associations(0, &receiver_key_1, &client).await?;

    // Create a receiver2 with 0 auto associations
    let receiver_key_2 = PrivateKey::generate_ed25519();
    let receiver_account_2 =
        Account::create_with_max_associations(0, &receiver_key_2, &client).await?;

    // Create a token and an NFT
    let token = FungibleToken::create_ft(&client, &operator_account, 3).await?;
    let nft = Nft::create(&client, &operator_account).await?;

    // Mint Nfts
    let mint_receipt = TokenMintTransaction::new()
        .token_id(nft.id)
        .metadata(repeat(vec![9, 1, 6]).take(10).collect::<Vec<Vec<u8>>>())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let nft_serials = mint_receipt.serials;

    // Airdrop the tokens to both
    let mut binding = TokenAirdropTransaction::new();
    let record = binding
        .nft_transfer(
            nft.id.nft(nft_serials[0].try_into().unwrap()),
            operator_account.id,
            receiver_account_1.id,
        )
        .nft_transfer(
            nft.id.nft(nft_serials[1].try_into().unwrap()),
            operator_account.id,
            receiver_account_1.id,
        )
        .token_transfer(token.id, receiver_account_1.id, TEST_AMOUNT)
        .token_transfer(token.id, operator_account.id, -TEST_AMOUNT)
        .nft_transfer(
            nft.id.nft(nft_serials[2].try_into().unwrap()),
            operator_account.id,
            receiver_account_2.id,
        )
        .nft_transfer(
            nft.id.nft(nft_serials[3].try_into().unwrap()),
            operator_account.id,
            receiver_account_2.id,
        )
        .token_transfer(token.id, receiver_account_2.id, TEST_AMOUNT)
        .token_transfer(token.id, operator_account.id, -TEST_AMOUNT)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    // panic!("here");

    // Verify the txn record
    assert_eq!(record.pending_airdrop_records.len(), 6);

    // Cancel the tokens signing with receiver1 and receiver2
    let pending_airdrop_ids = record
        .pending_airdrop_records
        .iter()
        .map(|record| record.pending_airdrop_id)
        .collect::<Vec<_>>();

    let record = TokenCancelAirdropTransaction::new()
        .pending_airdrop_ids(pending_airdrop_ids)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    // verify in the transaction record the pending airdrop ids for nft and ft - should no longer exist
    assert_eq!(record.pending_airdrop_records.len(), 0);

    // Verify the receiver1 holds the tokens via query
    let receiver_account_balance =
        AccountBalanceQuery::new().account_id(receiver_account_1.id).execute(&client).await?;

    assert_eq!(receiver_account_balance.tokens.get(&token.id), None);
    assert_eq!(receiver_account_balance.tokens.get(&nft.id), None);

    // Verify the receiver2 holds the tokens via query
    let receiver_account_balance_2 =
        AccountBalanceQuery::new().account_id(receiver_account_2.id).execute(&client).await?;

    assert_eq!(receiver_account_balance_2.tokens.get(&token.id), None);
    assert_eq!(receiver_account_balance_2.tokens.get(&nft.id), None);

    // Verify the operator does not hold the tokens
    let operator_balance =
        AccountBalanceQuery::new().account_id(operator_account.id).execute(&client).await?;

    assert_eq!(operator_balance.tokens.get(&token.id), Some(TEST_FUNGIBLE_INITIAL_BALANCE));
    assert_eq!(operator_balance.tokens.get(&nft.id), Some(TEST_MINTED_NFTS));

    Ok(())
}

#[tokio::test]
async fn cancel_from_multiple_airdrops() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;

    // Create a receiver1 with 0 auto associations
    let receiver_key = PrivateKey::generate_ed25519();
    let receiver_account = Account::create_with_max_associations(0, &receiver_key, &client).await?;

    // Create a token and an NFT
    let token = FungibleToken::create_ft(&client, &operator_account, 3).await?;
    let nft = Nft::create(&client, &operator_account).await?;

    // Mint Nfts
    let mint_receipt = TokenMintTransaction::new()
        .token_id(nft.id)
        .metadata(repeat(vec![9, 1, 6]).take(10).collect::<Vec<Vec<_>>>())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let nft_serials = mint_receipt.serials;

    // Airdrop the tokens to both
    let record1 = TokenAirdropTransaction::new()
        .nft_transfer(
            nft.id.nft(nft_serials[0].try_into().unwrap()),
            operator_account.id,
            receiver_account.id,
        )
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    let record2 = TokenAirdropTransaction::new()
        .nft_transfer(
            nft.id.nft(nft_serials[1].try_into().unwrap()),
            operator_account.id,
            receiver_account.id,
        )
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    let record3 = TokenAirdropTransaction::new()
        .token_transfer(token.id, receiver_account.id, TEST_AMOUNT)
        .token_transfer(token.id, operator_account.id, -TEST_AMOUNT)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    let pending_airdrop_ids = vec![
        record1.pending_airdrop_records[0].pending_airdrop_id,
        record2.pending_airdrop_records[0].pending_airdrop_id,
        record3.pending_airdrop_records[0].pending_airdrop_id,
    ];

    // Cancel the all the tokens with the receiver
    let record = TokenCancelAirdropTransaction::new()
        .pending_airdrop_ids(pending_airdrop_ids)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    // verify in the transaction record the pending airdrop ids for nft and ft - should no longer exist
    assert_eq!(record.pending_airdrop_records.len(), 0);

    // Verify the receiver1 holds the tokens via query
    let receiver_account_balance =
        AccountBalanceQuery::new().account_id(receiver_account.id).execute(&client).await?;

    assert_eq!(receiver_account_balance.tokens.get(&token.id).as_deref(), None);
    assert_eq!(receiver_account_balance.tokens.get(&nft.id), None);

    // Verify the operator does not hold the tokens
    let operator_balance =
        AccountBalanceQuery::new().account_id(operator_account.id).execute(&client).await?;

    assert_eq!(operator_balance.tokens.get(&token.id), Some(TEST_FUNGIBLE_INITIAL_BALANCE));
    assert_eq!(operator_balance.tokens.get(&nft.id), Some(TEST_MINTED_NFTS));

    Ok(())
}

#[tokio::test]
async fn cannot_cancel_nonexisting_airdrops_fail() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;

    // Create a receiver with 0 auto associations
    let receiver_key = PrivateKey::generate_ed25519();
    let receiver_account = Account::create_with_max_associations(0, &receiver_key, &client).await?;

    // Create a token
    let token = FungibleToken::create_ft(&client, &operator_account, 3).await?;

    // Airdrop the tokens to both
    let record = TokenAirdropTransaction::new()
        .token_transfer(token.id, receiver_account.id, TEST_AMOUNT)
        .token_transfer(token.id, operator_account.id, -TEST_AMOUNT)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    let random_account_key = PrivateKey::generate_ed25519();
    let random_account =
        Account::create_with_max_associations(0, &random_account_key, &client).await?;

    // Cancel the tokens with the random account which has not created pending airdrops
    // fails with INVALID_SIGNATURE
    let res = TokenCancelAirdropTransaction::new()
        .transaction_id(TransactionId::generate(random_account.id))
        .add_pending_airdrop_id(record.pending_airdrop_records[0].pending_airdrop_id)
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::TransactionPreCheckStatus { status: Status::InvalidSignature, .. })
    );

    Ok(())
}

#[tokio::test]
async fn cannot_cancel_canceled_airdrops_fail() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;

    // Create a receiver account with 0 auto associations
    let receiver_key = PrivateKey::generate_ed25519();
    let receiver_account = Account::create_with_max_associations(0, &receiver_key, &client).await?;

    // Create a token
    let token = FungibleToken::create_ft(&client, &operator_account, 3).await?;

    // Airdrop tokens from the sender to the receiver
    let record = TokenAirdropTransaction::new()
        .token_transfer(token.id, receiver_account.id, TEST_AMOUNT)
        .token_transfer(token.id, operator_account.id, -TEST_AMOUNT)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    // Cancel the tokens with the receiver
    _ = TokenCancelAirdropTransaction::new()
        .add_pending_airdrop_id(record.pending_airdrop_records[0].pending_airdrop_id)
        .execute(&client)
        .await?
        .get_record(&client)
        .await;

    // Cancel the tokens with the receiver again
    // fails with INVALID_PENDING_AIRDROP_ID
    let res = TokenCancelAirdropTransaction::new()
        .add_pending_airdrop_id(record.pending_airdrop_records[0].pending_airdrop_id)
        .execute(&client)
        .await?
        .get_record(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidPendingAirdropId, .. })
    );

    Ok(())
}

#[tokio::test]
async fn cannot_cancel_empty_airdrop_list_fail() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Cancel the tokens with the receiver without setting pendingAirdropIds
    // fails with EMPTY_PENDING_AIRDROP_ID_LIST
    let res = TokenCancelAirdropTransaction::new().execute(&client).await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::TransactionPreCheckStatus {
            status: Status::EmptyPendingAirdropIdList,
            ..
        })
    );

    Ok(())
}

#[tokio::test]
async fn cannot_cancel_duplicated_entries_fail() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;

    // Create a receiver account with 0 auto associations
    let receiver_key = PrivateKey::generate_ed25519();
    let receiver_account = Account::create_with_max_associations(0, &receiver_key, &client).await?;

    // Create a token
    let token = FungibleToken::create_ft(&client, &operator_account, 3).await?;

    // Airdrop tokens
    let record = TokenAirdropTransaction::new()
        .token_transfer(token.id, receiver_account.id, TEST_AMOUNT)
        .token_transfer(token.id, operator_account.id, -TEST_AMOUNT)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    // Cancel the tokens with the receiver again
    // fails with INVALID_PENDING_AIRDROP_ID
    let res = TokenCancelAirdropTransaction::new()
        .add_pending_airdrop_id(record.pending_airdrop_records[0].pending_airdrop_id)
        .add_pending_airdrop_id(record.pending_airdrop_records[0].pending_airdrop_id)
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::TransactionPreCheckStatus {
            status: Status::PendingAirdropIdRepeated,
            ..
        })
    );

    Ok(())
}

async fn test_operator_account(config: &Config) -> anyhow::Result<Account> {
    if let Some(operator) = config.operator.clone() {
        Ok(Account { key: operator.private_key, id: operator.account_id })
    } else {
        return Err(anyhow!("no operator configured"));
    }
}
