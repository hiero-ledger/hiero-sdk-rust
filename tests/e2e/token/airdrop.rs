use std::iter::repeat;

use anyhow::anyhow;
use assert_matches::assert_matches;
use hedera::{
    AccountAllowanceApproveTransaction,
    AccountBalanceQuery,
    AccountCreateTransaction,
    FixedFee,
    FixedFeeData,
    Hbar,
    PrivateKey,
    Status,
    TokenAirdropTransaction,
    TokenAssociateTransaction,
    TokenCreateTransaction,
    TokenMintTransaction,
    TokenSupplyType,
    TransactionId,
    TransferTransaction,
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
};

#[tokio::test]
async fn airdrop_associated_tokens() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let metadata: Vec<Vec<u8>> = repeat(vec![9, 1, 6]).take(10).collect();

    let operator_account = test_operator_account(&config).await?;

    // Create a receiver account with unlimited auto associations
    let receiver_key = PrivateKey::generate_ed25519();
    let receiver_account =
        Account::create_with_max_associations(-1, &receiver_key, &client).await?;

    // Create a token and an NFT
    let token = FungibleToken::create_ft(&client, &operator_account, 3).await?;
    let nft = Nft::create(&client, &operator_account).await?;

    // Mint Nfts
    let mint_receipt = TokenMintTransaction::new()
        .token_id(nft.id)
        .metadata(metadata)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let nft_serials = mint_receipt.serials;

    // Airdrop tokens
    _ = TokenAirdropTransaction::new()
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
        .get_receipt(&client)
        .await?;

    // Verify the receiver holds the tokens via query
    let receiver_account_balance =
        AccountBalanceQuery::new().account_id(receiver_account.id).execute(&client).await?;

    assert_eq!(
        receiver_account_balance.tokens.get(&token.id).as_deref(),
        Some(&(TEST_AMOUNT as u64))
    );
    assert_eq!(receiver_account_balance.tokens.get(&nft.id).as_deref(), Some(&(2 as u64)));

    // Verify the operator does not hold the tokens
    let operator_balance =
        AccountBalanceQuery::new().account_id(operator_account.id).execute(&client).await?;

    assert_eq!(operator_balance.tokens.get(&token.id), Some(&(999_900 as u64)));
    assert_eq!(operator_balance.tokens.get(&nft.id), Some(&(8 as u64)));

    Ok(())
}

#[tokio::test]
async fn airdrop_non_associated_tokens() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let metadata: Vec<Vec<u8>> = repeat(vec![9, 1, 6]).take(10).collect();

    let operator_account = test_operator_account(&config).await?;

    // Create a receiver account with 0 auto associations
    let receiver_key = PrivateKey::generate_ed25519();
    let receiver_account = Account::create_with_max_associations(0, &receiver_key, &client).await?;

    // Create a token and an NFT
    let token = FungibleToken::create_ft(&client, &operator_account, 3).await?;
    let nft = Nft::create(&client, &operator_account).await?;

    // Mint Nfts
    let mint_receipt = TokenMintTransaction::new()
        .token_id(nft.id)
        .metadata(metadata)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let nft_serials = mint_receipt.serials;

    // Airdrop tokens
    let mut tx = TokenAirdropTransaction::new()
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
        .await?;
    tx.validate_status(true).get_receipt(&client).await?;
    let record = tx.get_record(&client).await?;

    // Verify pending airdrops in the transaction record
    assert_eq!(record.pending_airdrop_records.is_empty(), false);

    // Verify the receiver holds the tokens via query
    let receiver_account_balance =
        AccountBalanceQuery::new().account_id(receiver_account.id).execute(&client).await?;

    assert_eq!(receiver_account_balance.tokens.get(&token.id).as_deref(), None,);
    assert_eq!(receiver_account_balance.tokens.get(&nft.id).as_deref(), None);

    // Verify the operator does not hold the tokens
    let operator_balance =
        AccountBalanceQuery::new().account_id(operator_account.id).execute(&client).await?;

    assert_eq!(operator_balance.tokens.get(&token.id), Some(TEST_FUNGIBLE_INITIAL_BALANCE));
    assert_eq!(operator_balance.tokens.get(&nft.id), Some(&(10 as u64)));

    Ok(())
}

#[tokio::test]
async fn airdrop_to_alias() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let metadata: Vec<Vec<u8>> = repeat(vec![9, 1, 6]).take(10).collect();

    let operator_account = test_operator_account(&config).await?;

    // Create a token and an NFT
    let token = FungibleToken::create_ft(&client, &operator_account, 3).await?;
    let nft = Nft::create(&client, &operator_account).await?;

    // Mint Nfts
    let mint_receipt = TokenMintTransaction::new()
        .token_id(nft.id)
        .metadata(metadata)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let nft_serials = mint_receipt.serials;

    // Airdrop tokens to an alias
    let private_key = PrivateKey::generate_ed25519();
    let public_key = private_key.public_key();

    let alias_account_id = public_key.to_account_id(0, 0);

    // Airdrop tokens
    _ = TokenAirdropTransaction::new()
        .nft_transfer(
            nft.id.nft(nft_serials[0].try_into().unwrap()),
            operator_account.id,
            alias_account_id,
        )
        .nft_transfer(
            nft.id.nft(nft_serials[1].try_into().unwrap()),
            operator_account.id,
            alias_account_id,
        )
        .token_transfer(token.id, alias_account_id, TEST_AMOUNT)
        .token_transfer(token.id, operator_account.id, -TEST_AMOUNT)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Verify the receiver holds the tokens via query
    let receiver_account_balance =
        AccountBalanceQuery::new().account_id(alias_account_id).execute(&client).await?;

    assert_eq!(
        receiver_account_balance.tokens.get(&token.id).as_deref(),
        Some(&(TEST_AMOUNT as u64))
    );
    assert_eq!(receiver_account_balance.tokens.get(&nft.id).as_deref(), Some(&(2 as u64)));

    // Verify the operator does not hold the tokens
    let operator_balance =
        AccountBalanceQuery::new().account_id(operator_account.id).execute(&client).await?;

    assert_eq!(operator_balance.tokens.get(&token.id), Some(&(999_900 as u64)));
    assert_eq!(operator_balance.tokens.get(&nft.id), Some(&(8 as u64)));

    Ok(())
}

#[tokio::test]
async fn airdrop_with_custom_fees() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;

    // Create a receiver account with 0 auto associations
    let receiver_key = PrivateKey::generate_ed25519();
    let receiver_account =
        Account::create_with_max_associations(-1, &receiver_key, &client).await?;

    // Create a token and an NFT
    let custom_fee_token = FungibleToken::create_ft(&client, &operator_account, 3).await?;

    let fee = FixedFee {
        fee: FixedFeeData { amount: 1, denominating_token_id: Some(custom_fee_token.id) },
        fee_collector_account_id: Some(operator_account.id),
        all_collectors_are_exempt: true,
    };

    let token_id = TokenCreateTransaction::new()
        .name("Test Fungible Token")
        .symbol("TFT")
        .token_memo("I was created for integration tests")
        .decimals(3)
        .initial_supply(1_000_000)
        .max_supply(1_000_000)
        .treasury_account_id(operator_account.id)
        .token_supply_type(TokenSupplyType::Finite)
        .admin_key(operator_account.key.public_key())
        .freeze_key(operator_account.key.public_key())
        .supply_key(operator_account.key.public_key())
        .metadata_key(operator_account.key.public_key())
        .pause_key(operator_account.key.public_key())
        .custom_fees([fee.into()])
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    // Create sender account with unlimited associations and send some tokens to it
    let sender_key = PrivateKey::generate_ed25519();
    let sender_account = Account::create_with_max_associations(-1, &sender_key, &client).await?;

    // Associate the token to the sender
    _ = TokenAssociateTransaction::new()
        .account_id(sender_account.id)
        .token_ids([custom_fee_token.id])
        .freeze_with(&client)?
        .sign(sender_account.key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Send tokens to the sender
    _ = TransferTransaction::new()
        .token_transfer(custom_fee_token.id, operator_account.id, -TEST_AMOUNT)
        .token_transfer(custom_fee_token.id, sender_account.id, TEST_AMOUNT)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    _ = TransferTransaction::new()
        .token_transfer(token_id, operator_account.id, -TEST_AMOUNT)
        .token_transfer(token_id, sender_account.id, TEST_AMOUNT)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Airdrop tokens from the sender to the receiver
    _ = TokenAirdropTransaction::new()
        .token_transfer(token_id, receiver_account.id, TEST_AMOUNT)
        .token_transfer(token_id, sender_account.id, -TEST_AMOUNT)
        .freeze_with(&client)?
        .sign(sender_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Verify the custom fee has been paid by the sender to collector
    let receiver_account_balance =
        AccountBalanceQuery::new().account_id(receiver_account.id).execute(&client).await?;

    assert_eq!(
        receiver_account_balance.tokens.get(&token_id).as_deref(),
        Some(&(TEST_AMOUNT as u64))
    );

    let sender_account_balance =
        AccountBalanceQuery::new().account_id(sender_account.id).execute(&client).await?;

    assert_eq!(sender_account_balance.tokens.get(&token_id).as_deref(), Some(&(0 as u64)));
    assert_eq!(sender_account_balance.tokens.get(&custom_fee_token.id), Some(&(99 as u64)));

    // Verify the operator does not hold the tokens
    let operator_balance =
        AccountBalanceQuery::new().account_id(operator_account.id).execute(&client).await?;

    assert_eq!(operator_balance.tokens.get(&token_id), Some(&(999_900 as u64)));
    assert_eq!(operator_balance.tokens.get(&custom_fee_token.id), Some(&(999_901 as u64)));

    Ok(())
}

#[tokio::test]
async fn airdrop_tokens_w_receiver_sig() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;

    let token = FungibleToken::create_ft(&client, &operator_account, 3).await?;

    // Create a receiver account with 0 auto associations
    let receiver_key = PrivateKey::generate_ed25519();
    let receiver_account_id = AccountCreateTransaction::new()
        .set_key_without_alias(receiver_key.public_key())
        .initial_balance(Hbar::new(1))
        .receiver_signature_required(true)
        .max_automatic_token_associations(-1)
        .freeze_with(&client)?
        .sign(receiver_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    // Airdrop tokens
    _ = TokenAirdropTransaction::new()
        .token_transfer(token.id, receiver_account_id, TEST_AMOUNT)
        .token_transfer(token.id, operator_account.id, -TEST_AMOUNT)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}

#[tokio::test]
async fn airdrop_nfts_w_receiver_sig() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };
    let operator_account = test_operator_account(&config).await?;

    // Create Nfts
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

    // Create receiver with unlimited auto associations and receiver_sig = true
    let receiver_key = PrivateKey::generate_ed25519();
    let receiver_account_id = AccountCreateTransaction::new()
        .set_key_without_alias(receiver_key.public_key())
        .initial_balance(Hbar::new(1))
        .receiver_signature_required(true)
        .max_automatic_token_associations(-1)
        .freeze_with(&client)?
        .sign(receiver_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    // Airdrop tokens
    _ = TokenAirdropTransaction::new()
        .nft_transfer(
            nft.id.nft(nft_serials[0].try_into().unwrap()),
            operator_account.id,
            receiver_account_id,
        )
        .nft_transfer(
            nft.id.nft(nft_serials[1].try_into().unwrap()),
            operator_account.id,
            receiver_account_id,
        )
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}

#[tokio::test]
async fn token_allowance_and_no_balance_ft_fail() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;

    // Create a spender account with -1 auto associations
    let spender_key = PrivateKey::generate_ed25519();
    let spender_account = Account::create_with_max_associations(-1, &spender_key, &client).await?;

    // Create a receiver account with -1 auto associations
    let sender_key = PrivateKey::generate_ed25519();
    let sender_account = Account::create_with_max_associations(-1, &sender_key, &client).await?;

    // Create a token and an NFT
    let token = FungibleToken::create_ft(&client, &operator_account, 3).await?;

    _ = TransferTransaction::new()
        .token_transfer(token.id, operator_account.id, -TEST_AMOUNT)
        .token_transfer(token.id, sender_account.id, TEST_AMOUNT)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Approve allowance to the spender
    _ = AccountAllowanceApproveTransaction::new()
        .approve_token_allowance(
            token.id,
            sender_account.id,
            spender_account.id,
            TEST_AMOUNT as u64,
        )
        .freeze_with(&client)?
        .sign(sender_account.key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Airdrop the tokens from the sender to the spender via approval
    // Fails with not supported status
    let res = TokenAirdropTransaction::new()
        .token_transfer(token.id, spender_account.id, TEST_AMOUNT)
        .approved_token_transfer(token.id, spender_account.id, -TEST_AMOUNT)
        .transaction_id(TransactionId::generate(spender_account.id))
        .freeze_with(&client)?
        .sign(spender_account.key)
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::TransactionPreCheckStatus { status: Status::NotSupported, .. })
    );

    Ok(())
}

#[tokio::test]
async fn token_allowance_and_no_balance_nft_fail() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;

    // Create a spender account with -1 auto associations
    let spender_key = PrivateKey::generate_ed25519();
    let spender_account = Account::create_with_max_associations(-1, &spender_key, &client).await?;

    // Create a receiver account with -1 auto associations
    let sender_key = PrivateKey::generate_ed25519();
    let sender_account = Account::create_with_max_associations(-1, &sender_key, &client).await?;

    // Create Nft
    let nft = Nft::create(&client, &operator_account).await?;

    let mint_receipt = TokenMintTransaction::new()
        .token_id(nft.id)
        .metadata(repeat(vec![9, 1, 6]).take(10).collect::<Vec<Vec<u8>>>())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let nft_serials = mint_receipt.serials;

    _ = TransferTransaction::new()
        .nft_transfer(
            nft.id.nft(nft_serials[0].try_into().unwrap()),
            operator_account.id,
            sender_account.id,
        )
        .nft_transfer(
            nft.id.nft(nft_serials[1].try_into().unwrap()),
            operator_account.id,
            sender_account.id,
        )
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Approve allowance to the spender
    _ = AccountAllowanceApproveTransaction::new()
        .approve_token_nft_allowance(
            nft.id.nft(nft_serials[0].try_into().unwrap()),
            sender_account.id,
            spender_account.id,
        )
        .approve_token_nft_allowance(
            nft.id.nft(nft_serials[1].try_into().unwrap()),
            sender_account.id,
            spender_account.id,
        )
        .freeze_with(&client)?
        .sign(sender_account.key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Airdrop the tokens from the sender to the spender via approval
    // Fails with not supported status
    let res = TokenAirdropTransaction::new()
        .approved_nft_transfer(
            nft.id.nft(nft_serials[0].try_into().unwrap()),
            sender_account.id,
            spender_account.id,
        )
        .approved_nft_transfer(
            nft.id.nft(nft_serials[1].try_into().unwrap()),
            sender_account.id,
            spender_account.id,
        )
        .transaction_id(TransactionId::generate(spender_account.id))
        .freeze_with(&client)?
        .sign(spender_account.key)
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::TransactionPreCheckStatus { status: Status::NotSupported, .. })
    );

    Ok(())
}

#[tokio::test]
async fn invalid_body_fail() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;

    // Airdrop with no tokenID or NftID
    // fails with EMPTY_TOKEN_TRANSFER_BODY
    let res = TokenAirdropTransaction::new().execute(&client).await;

    assert_matches!(
        res,
        Err(hedera::Error::TransactionPreCheckStatus {
            status: Status::EmptyTokenTransferBody,
            ..
        })
    );

    // Create a fungible token
    let token = FungibleToken::create_ft(&client, &operator_account, 3).await?;

    // Airdrop with invalid transfers
    // fails with INVALID_TRANSACTION_BODY
    let res = TokenAirdropTransaction::new()
        .token_transfer(token.id, operator_account.id, TEST_AMOUNT)
        .token_transfer(token.id, operator_account.id, TEST_AMOUNT)
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::TransactionPreCheckStatus {
            status: Status::InvalidTransactionBody,
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
