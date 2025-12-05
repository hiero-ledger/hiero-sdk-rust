use std::iter::repeat;

use anyhow::anyhow;
use assert_matches::assert_matches;
use hiero_sdk::{
    AccountBalanceQuery,
    PrivateKey,
    Status,
    TokenAirdropTransaction,
    TokenAssociateTransaction,
    TokenClaimAirdropTransaction,
    TokenDeleteTransaction,
    TokenFreezeTransaction,
    TokenMintTransaction,
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
    let mut binding = TokenAirdropTransaction::new();
    let record = binding
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
        .token_transfer(token.id, receiver_account.id, 100)
        .token_transfer(token.id, operator_account.id, -100)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    // Verify transaction record
    assert_eq!(record.pending_airdrop_records.len(), 3);
    assert_eq!(record.pending_airdrop_records.get(0).unwrap().pending_airdrop_value, Some(100));
    assert_eq!(
        record.pending_airdrop_records.get(0).unwrap().pending_airdrop_id.token_id,
        Some(token.id)
    );
    assert_eq!(record.pending_airdrop_records.get(0).unwrap().pending_airdrop_id.nft_id, None);

    assert_eq!(record.pending_airdrop_records.get(1).unwrap().pending_airdrop_value, None);
    assert_eq!(
        record.pending_airdrop_records.get(1).unwrap().pending_airdrop_id.nft_id,
        Some(nft.id.nft(1))
    );
    assert_eq!(record.pending_airdrop_records.get(1).unwrap().pending_airdrop_id.token_id, None);

    assert_eq!(record.pending_airdrop_records.get(2).unwrap().pending_airdrop_value, None);
    assert_eq!(
        record.pending_airdrop_records.get(2).unwrap().pending_airdrop_id.nft_id,
        Some(nft.id.nft(2))
    );
    assert_eq!(record.pending_airdrop_records.get(2).unwrap().pending_airdrop_id.token_id, None);

    let record = TokenClaimAirdropTransaction::new()
        .add_pending_airdrop_id(record.pending_airdrop_records.get(0).unwrap().pending_airdrop_id)
        .add_pending_airdrop_id(record.pending_airdrop_records.get(1).unwrap().pending_airdrop_id)
        .add_pending_airdrop_id(record.pending_airdrop_records.get(2).unwrap().pending_airdrop_id)
        .freeze_with(&client)?
        .sign(receiver_key)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    assert_eq!(record.pending_airdrop_records.len(), 0);

    // Verify the receiver holds the tokens via query
    let receiver_account_balance =
        AccountBalanceQuery::new().account_id(receiver_account.id).execute(&client).await?;

    assert_eq!(receiver_account_balance.tokens.get(&token.id), Some(&(100 as u64)));
    assert_eq!(receiver_account_balance.tokens.get(&nft.id), Some(&(2 as u64)));

    // Verify the operator does not hold the tokens
    let operator_account_balance =
        AccountBalanceQuery::new().account_id(operator_account.id).execute(&client).await?;

    assert_eq!(operator_account_balance.tokens.get(&token.id), Some(&(999_900 as u64)));
    assert_eq!(operator_account_balance.tokens.get(&nft.id), Some(&(8 as u64)));

    Ok(())
}

#[tokio::test]
async fn claim_to_multiple_receivers() -> anyhow::Result<()> {
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
        .metadata(repeat(vec![9, 1, 6]).take(10).collect::<Vec<Vec<_>>>())
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
        .token_transfer(token.id, receiver_account_1.id, 100)
        .token_transfer(token.id, operator_account.id, -100)
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
        .token_transfer(token.id, receiver_account_2.id, 100)
        .token_transfer(token.id, operator_account.id, -100)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    // Verify the txn record
    assert_eq!(record.pending_airdrop_records.len(), 6);

    // Claim the tokens signing with receiver1 and receiver2
    let pending_airdrop_ids = record
        .pending_airdrop_records
        .iter()
        .map(|record| record.pending_airdrop_id)
        .collect::<Vec<_>>();

    let record = TokenClaimAirdropTransaction::new()
        .pending_airdrop_ids(pending_airdrop_ids)
        .freeze_with(&client)?
        .sign(receiver_key_1)
        .sign(receiver_key_2)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    // verify in the transaction record the pending airdrop ids for nft and ft - should no longer exist
    assert_eq!(record.pending_airdrop_records.len(), 0);

    // Verify the receiver1 holds the tokens via query
    let receiver_account_balance =
        AccountBalanceQuery::new().account_id(receiver_account_1.id).execute(&client).await?;

    assert_eq!(receiver_account_balance.tokens.get(&token.id).as_deref(), Some(&(100 as u64)));
    assert_eq!(receiver_account_balance.tokens.get(&nft.id), Some(&(2 as u64)));

    // Verify the receiver2 holds the tokens via query
    let receiver_account_balance_2 =
        AccountBalanceQuery::new().account_id(receiver_account_2.id).execute(&client).await?;

    assert_eq!(receiver_account_balance_2.tokens.get(&token.id).as_deref(), Some(&(100 as u64)));
    assert_eq!(receiver_account_balance_2.tokens.get(&nft.id), Some(&(2 as u64)));

    // Verify the operator does not hold the tokens
    let operator_balance =
        AccountBalanceQuery::new().account_id(operator_account.id).execute(&client).await?;

    assert_eq!(operator_balance.tokens.get(&token.id), Some(&((1_000_000 - 100 * 2) as u64)));
    assert_eq!(operator_balance.tokens.get(&nft.id), Some(&(6 as u64)));

    Ok(())
}

#[tokio::test]
async fn claim_from_multiple_airdrops() -> anyhow::Result<()> {
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
        .token_transfer(token.id, receiver_account.id, 100)
        .token_transfer(token.id, operator_account.id, -100)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    let pending_airdrop_ids = vec![
        record1.pending_airdrop_records[0].pending_airdrop_id,
        record2.pending_airdrop_records[0].pending_airdrop_id,
        record3.pending_airdrop_records[0].pending_airdrop_id,
    ];

    let record = TokenClaimAirdropTransaction::new()
        .pending_airdrop_ids(pending_airdrop_ids)
        .freeze_with(&client)?
        .sign(receiver_key)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    // verify in the transaction record the pending airdrop ids for nft and ft - should no longer exist
    assert_eq!(record.pending_airdrop_records.len(), 0);

    // Verify the receiver1 holds the tokens via query
    let receiver_account_balance =
        AccountBalanceQuery::new().account_id(receiver_account.id).execute(&client).await?;

    assert_eq!(receiver_account_balance.tokens.get(&token.id).as_deref(), Some(&(100 as u64)));
    assert_eq!(receiver_account_balance.tokens.get(&nft.id), Some(&(2 as u64)));

    // Verify the operator does not hold the tokens
    let operator_balance =
        AccountBalanceQuery::new().account_id(operator_account.id).execute(&client).await?;

    assert_eq!(operator_balance.tokens.get(&token.id), Some(&((1_000_000 - 100) as u64)));
    assert_eq!(operator_balance.tokens.get(&nft.id), Some(&(8 as u64)));

    Ok(())
}

#[tokio::test]
async fn cannot_claim_nonexisting_tokens_fail() -> anyhow::Result<()> {
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
        .token_transfer(token.id, receiver_account.id, 100)
        .token_transfer(token.id, operator_account.id, -100)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    // Claim the tokens with the operator which does not have pending airdrops
    // fails with INVALID_SIGNATURE
    let res = TokenClaimAirdropTransaction::new()
        .pending_airdrop_ids(vec![record.pending_airdrop_records[0].pending_airdrop_id])
        .execute(&client)
        .await?
        .get_record(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, .. })
    );

    Ok(())
}

#[tokio::test]
async fn cannot_claim_already_claimed_airdrop_fail() -> anyhow::Result<()> {
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
        .token_transfer(token.id, receiver_account.id, 100)
        .token_transfer(token.id, operator_account.id, -100)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    // Claim the tokens with the receiver
    let _ = TokenClaimAirdropTransaction::new()
        .pending_airdrop_ids(vec![record.pending_airdrop_records[0].pending_airdrop_id])
        .freeze_with(&client)?
        .sign(receiver_account.key)
        .execute(&client)
        .await?
        .get_record(&client)
        .await;

    // Claim the tokens with the operator which does not have pending airdrops
    // fails with INVALID_PENDING_AIRDROP_ID
    let res = TokenClaimAirdropTransaction::new()
        .pending_airdrop_ids(vec![record.pending_airdrop_records[0].pending_airdrop_id])
        .freeze_with(&client)?
        .sign(receiver_key)
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
async fn cannot_claim_empty_pending_airdrops_fail() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Claim the tokens with the receiver without setting pendingAirdropIds
    // fails with EMPTY_PENDING_AIRDROP_ID_LIST
    let res = TokenClaimAirdropTransaction::new().execute(&client).await;

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
async fn cannot_claim_duplicate_entries_fail() -> anyhow::Result<()> {
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
        .token_transfer(token.id, receiver_account.id, 100)
        .token_transfer(token.id, operator_account.id, -100)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    // Claim the tokens with duplicate pending airdrop token ids
    // fails with PENDING_AIRDROP_ID_REPEATED
    let res = TokenClaimAirdropTransaction::new()
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

#[tokio::test]
async fn cannot_claim_deleted_tokens_fail() -> anyhow::Result<()> {
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
        .token_transfer(token.id, receiver_account.id, 100)
        .token_transfer(token.id, operator_account.id, -100)
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    // Delete the token
    let _ = TokenDeleteTransaction::new()
        .token_id(token.id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Claim the tokens with receiver
    // fails with TOKEN_IS_DELETED
    let res = TokenClaimAirdropTransaction::new()
        .add_pending_airdrop_id(record.pending_airdrop_records[0].pending_airdrop_id)
        .freeze_with(&client)?
        .sign(receiver_key)
        .execute(&client)
        .await?
        .get_record(&client)
        .await;

    assert_matches!(res, Err(hiero_sdk::Error::ReceiptStatus { status: Status::TokenWasDeleted, .. }));

    Ok(())
}

#[tokio::test]
async fn cannot_claim_frozen_token_fail() -> anyhow::Result<()> {
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
        .token_transfer(token.id, receiver_account.id, 100)
        .token_transfer(token.id, operator_account.id, -100)
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

    // Claim the tokens with receiver
    // fails with ACCOUNT_FROZEN_FOR_TOKEN
    let res = TokenClaimAirdropTransaction::new()
        .add_pending_airdrop_id(record.pending_airdrop_records[0].pending_airdrop_id)
        .freeze_with(&client)?
        .sign(receiver_key)
        .execute(&client)
        .await?
        .get_record(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::AccountFrozenForToken, .. })
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
