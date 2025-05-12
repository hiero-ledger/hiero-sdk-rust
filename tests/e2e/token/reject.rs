// SPDX-License-Identifier: Apache-2.0

use std::iter::repeat;

use anyhow::anyhow;
use assert_matches::assert_matches;
use hedera::{
    AccountBalanceQuery,
    Client,
    Hbar,
    PrivateKey,
    Status,
    TokenCreateTransaction,
    TokenFreezeTransaction,
    TokenMintTransaction,
    TokenNftInfoQuery,
    TokenPauseTransaction,
    TokenRejectTransaction,
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
};

#[tokio::test]
async fn basic_fungible_token() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;
    let ft1 = create_ft(&client, &operator_account, 3).await?;
    let ft2 = create_ft(&client, &operator_account, 3).await?;
    let receiver_account = create_receiver_account(100, &operator_account.key, &client).await?;

    _ = TransferTransaction::new()
        .token_transfer(ft1.id, operator_account.id, -10)
        .token_transfer(ft1.id, receiver_account.id, 10)
        .token_transfer(ft2.id, operator_account.id, -10)
        .token_transfer(ft2.id, receiver_account.id, 10)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    _ = TokenRejectTransaction::new()
        .owner(receiver_account.id)
        .token_ids(vec![ft1.id, ft2.id])
        .freeze_with(&client)?
        .sign(receiver_account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let receiver_balance =
        AccountBalanceQuery::new().account_id(receiver_account.id).execute(&client).await?;

    assert_eq!(receiver_balance.tokens.get(&ft1.id), Some(&(0 as u64)));
    assert_eq!(receiver_balance.tokens.get(&ft2.id), Some(&(0 as u64)));

    let treasury_account_balance =
        AccountBalanceQuery::new().account_id(operator_account.id).execute(&client).await?;

    assert_eq!(treasury_account_balance.tokens.get(&ft1.id), Some(&(1_000_000 as u64)));
    assert_eq!(treasury_account_balance.tokens.get(&ft2.id), Some(&(1_000_000 as u64)));

    ft1.delete(&client).await?;
    ft2.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn basic_nft() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;
    let nft1 = Nft::create(&client, &operator_account).await?;
    let nft2 = Nft::create(&client, &operator_account).await?;
    let receiver_account_key = PrivateKey::generate_ed25519();
    let receiver_account = create_receiver_account(100, &receiver_account_key, &client).await?;

    _ = TokenMintTransaction::new()
        .token_id(nft1.id)
        .metadata(repeat(vec![3, 6, 9]).take(5))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let nft_serials = TokenMintTransaction::new()
        .token_id(nft2.id)
        .metadata(repeat(vec![9, 1, 6]).take(5))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .serials
        .into_iter()
        .map(|it| it as u64)
        .collect::<Vec<u64>>();

    _ = TransferTransaction::new()
        .nft_transfer(nft1.id.nft(nft_serials[0]), operator_account.id, receiver_account.id)
        .nft_transfer(nft1.id.nft(nft_serials[1]), operator_account.id, receiver_account.id)
        .nft_transfer(nft2.id.nft(nft_serials[0]), operator_account.id, receiver_account.id)
        .nft_transfer(nft2.id.nft(nft_serials[1]), operator_account.id, receiver_account.id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    _ = TokenRejectTransaction::new()
        .owner(receiver_account.id)
        .nft_ids(vec![nft1.id.nft(nft_serials[1]), nft2.id.nft(nft_serials[1])])
        .freeze_with(&client)?
        .sign(receiver_account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let nft_info =
        TokenNftInfoQuery::new().nft_id(nft1.id.nft(nft_serials[1])).execute(&client).await?;

    assert_eq!(nft_info.account_id, operator_account.id);

    let nft_info =
        TokenNftInfoQuery::new().nft_id(nft2.id.nft(nft_serials[1])).execute(&client).await?;

    assert_eq!(nft_info.account_id, operator_account.id);

    nft1.delete(&client).await?;
    nft2.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn ft_and_nft_reject() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;
    let ft1 = create_ft(&client, &operator_account, 3).await?;
    let ft2 = create_ft(&client, &operator_account, 3).await?;
    let nft1 = Nft::create(&client, &operator_account).await?;
    let nft2 = Nft::create(&client, &operator_account).await?;
    let receiver_account_key = PrivateKey::generate_ed25519();
    let receiver_account = create_receiver_account(100, &receiver_account_key, &client).await?;

    _ = TokenMintTransaction::new()
        .token_id(nft1.id)
        .metadata(repeat(vec![3, 6, 9]).take(5))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let nft_serials = TokenMintTransaction::new()
        .token_id(nft2.id)
        .metadata(repeat(vec![9, 1, 6]).take(5))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .serials
        .into_iter()
        .map(|it| it as u64)
        .collect::<Vec<u64>>();

    _ = TransferTransaction::new()
        .token_transfer(ft1.id, operator_account.id, -10)
        .token_transfer(ft1.id, receiver_account.id, 10)
        .token_transfer(ft2.id, operator_account.id, -10)
        .token_transfer(ft2.id, receiver_account.id, 10)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    _ = TransferTransaction::new()
        .nft_transfer(nft1.id.nft(nft_serials[0]), operator_account.id, receiver_account.id)
        .nft_transfer(nft1.id.nft(nft_serials[1]), operator_account.id, receiver_account.id)
        .nft_transfer(nft2.id.nft(nft_serials[0]), operator_account.id, receiver_account.id)
        .nft_transfer(nft2.id.nft(nft_serials[1]), operator_account.id, receiver_account.id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    _ = TokenRejectTransaction::new()
        .owner(receiver_account.id)
        .token_ids(vec![ft1.id, ft2.id])
        .nft_ids(vec![nft1.id.nft(nft_serials[1]), nft2.id.nft(nft_serials[1])])
        .freeze_with(&client)?
        .sign(receiver_account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let receiver_account_balance =
        AccountBalanceQuery::new().account_id(receiver_account.id).execute(&client).await?;

    assert_eq!(receiver_account_balance.tokens[&ft1.id], 0);
    assert_eq!(receiver_account_balance.tokens[&ft2.id], 0);
    assert_eq!(receiver_account_balance.tokens[&nft1.id], 1);
    assert_eq!(receiver_account_balance.tokens[&nft2.id], 1);

    let treasury_account_balance =
        AccountBalanceQuery::new().account_id(operator_account.id).execute(&client).await?;

    assert_eq!(treasury_account_balance.tokens[&ft1.id], 1_000_000);
    assert_eq!(treasury_account_balance.tokens[&ft2.id], 1_000_000);

    let nft_info =
        TokenNftInfoQuery::new().nft_id(nft1.id.nft(nft_serials[1])).execute(&client).await?;

    assert_eq!(nft_info.account_id, operator_account.id);

    let nft_info =
        TokenNftInfoQuery::new().nft_id(nft2.id.nft(nft_serials[1])).execute(&client).await?;

    assert_eq!(nft_info.account_id, operator_account.id);

    nft1.delete(&client).await?;
    nft2.delete(&client).await?;
    ft1.delete(&client).await?;
    ft2.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn ft_and_nft_freeze_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;
    let ft = create_ft(&client, &operator_account, 18).await?;
    let nft = Nft::create(&client, &operator_account).await?;
    let receiver_account_key = PrivateKey::generate_ed25519();
    let receiver_account = create_receiver_account(100, &receiver_account_key, &client).await?;

    _ = TransferTransaction::new()
        .token_transfer(ft.id, operator_account.id, -10)
        .token_transfer(ft.id, receiver_account.id, 10)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    _ = TokenFreezeTransaction::new()
        .token_id(ft.id)
        .account_id(receiver_account.id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let res = TokenRejectTransaction::new()
        .owner(receiver_account.id)
        .add_token_id(ft.id)
        .freeze_with(&client)?
        .sign(receiver_account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::AccountFrozenForToken, .. })
    );

    let nft_serials = TokenMintTransaction::new()
        .token_id(nft.id)
        .metadata(repeat(vec![9, 1, 6]).take(5))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .serials
        .into_iter()
        .map(|it| it as u64)
        .collect::<Vec<u64>>();

    _ = TransferTransaction::new()
        .nft_transfer(nft.id.nft(nft_serials[0]), operator_account.id, receiver_account.id)
        .nft_transfer(nft.id.nft(nft_serials[1]), operator_account.id, receiver_account.id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    _ = TokenFreezeTransaction::new()
        .token_id(nft.id)
        .account_id(receiver_account.id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let res = TokenRejectTransaction::new()
        .owner(receiver_account.id)
        .add_nft_id(nft.id.nft(nft_serials[1]))
        .freeze_with(&client)?
        .sign(receiver_account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::AccountFrozenForToken, .. })
    );

    nft.delete(&client).await?;
    ft.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn ft_and_nft_paused_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;
    let ft = create_ft(&client, &operator_account, 3).await?;
    let nft = Nft::create(&client, &operator_account).await?;
    let receiver_account_key = PrivateKey::generate_ed25519();
    let receiver_account = create_receiver_account(100, &receiver_account_key, &client).await?;

    _ = TransferTransaction::new()
        .token_transfer(ft.id, operator_account.id, -10)
        .token_transfer(ft.id, receiver_account.id, 10)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    _ = TokenPauseTransaction::new()
        .token_id(ft.id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let res = TokenRejectTransaction::new()
        .owner(receiver_account.id)
        .add_token_id(ft.id)
        .freeze_with(&client)?
        .sign(receiver_account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(res, Err(hedera::Error::ReceiptStatus { status: Status::TokenIsPaused, .. }));

    let nft_serials = TokenMintTransaction::new()
        .token_id(nft.id)
        .metadata(repeat(vec![9, 1, 6]).take(5))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .serials
        .into_iter()
        .map(|it| it as u64)
        .collect::<Vec<u64>>();

    _ = TransferTransaction::new()
        .nft_transfer(nft.id.nft(nft_serials[0]), operator_account.id, receiver_account.id)
        .nft_transfer(nft.id.nft(nft_serials[1]), operator_account.id, receiver_account.id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    _ = TokenPauseTransaction::new()
        .token_id(nft.id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let res = TokenRejectTransaction::new()
        .owner(receiver_account.id)
        .add_nft_id(nft.id.nft(nft_serials[1]))
        .freeze_with(&client)?
        .sign(receiver_account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(res, Err(hedera::Error::ReceiptStatus { status: Status::TokenIsPaused, .. }));

    Ok(())
}

#[tokio::test]
async fn add_or_set_nft_token_id_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;
    let nft = Nft::create(&client, &operator_account).await?;
    let receiver_account_key = PrivateKey::generate_ed25519();
    let receiver_account = create_receiver_account(100, &receiver_account_key, &client).await?;

    let nft_serials = TokenMintTransaction::new()
        .token_id(nft.id)
        .metadata(repeat(vec![9, 1, 6]).take(5))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .serials
        .into_iter()
        .map(|it| it as u64)
        .collect::<Vec<u64>>();

    _ = TransferTransaction::new()
        .nft_transfer(nft.id.nft(nft_serials[0]), operator_account.id, receiver_account.id)
        .nft_transfer(nft.id.nft(nft_serials[1]), operator_account.id, receiver_account.id)
        .nft_transfer(nft.id.nft(nft_serials[2]), operator_account.id, receiver_account.id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let res = TokenRejectTransaction::new()
        .owner(receiver_account.id)
        .add_token_id(nft.id)
        .nft_ids([nft.id.nft(nft_serials[1]), nft.id.nft(nft_serials[2])])
        .freeze_with(&client)?
        .sign(receiver_account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus {
            status: Status::AccountAmountTransfersOnlyAllowedForFungibleCommon,
            ..
        })
    );

    let res = TokenRejectTransaction::new()
        .owner(receiver_account.id)
        .token_ids([nft.id])
        .freeze_with(&client)?
        .sign(receiver_account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus {
            status: Status::AccountAmountTransfersOnlyAllowedForFungibleCommon,
            ..
        })
    );

    nft.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn treasury_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;
    let ft = create_ft(&client, &operator_account, 3).await?;
    let nft = Nft::create(&client, &operator_account).await?;

    let res = TokenRejectTransaction::new()
        .owner(operator_account.id)
        .add_token_id(ft.id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::AccountIsTreasury, .. })
    );

    let nft_serials = TokenMintTransaction::new()
        .token_id(nft.id)
        .metadata(repeat(vec![9, 1, 6]).take(5))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .serials
        .into_iter()
        .map(|it| it as u64)
        .collect::<Vec<u64>>();

    let res = TokenRejectTransaction::new()
        .owner(operator_account.id)
        .add_nft_id(nft.id.nft(nft_serials[0]))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::AccountIsTreasury, .. })
    );

    Ok(())
}

#[tokio::test]
async fn invalid_sig_fail() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;
    let random_key = PrivateKey::generate_ed25519();
    let ft = create_ft(&client, &operator_account, 3).await?;
    let receiver_account_key = PrivateKey::generate_ed25519();
    let receiver_account = create_receiver_account(100, &receiver_account_key, &client).await?;

    _ = TransferTransaction::new()
        .token_transfer(ft.id, operator_account.id, -10)
        .token_transfer(ft.id, receiver_account.id, 10)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let res = TokenRejectTransaction::new()
        .owner(receiver_account.id)
        .add_token_id(ft.id)
        .freeze_with(&client)?
        .sign(random_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::InvalidSignature, .. })
    );

    ft.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn missing_token_fail() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;
    let res = TokenRejectTransaction::new().owner(operator_account.id).execute(&client).await;

    assert_matches!(
        res,
        Err(hedera::Error::TransactionPreCheckStatus {
            status: Status::EmptyTokenReferenceList,
            ..
        })
    );

    Ok(())
}

#[tokio::test]
async fn token_reference_list_size_exceeded_fail() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;
    let ft = create_ft(&client, &operator_account, 3).await?;
    let nft = Nft::create(&client, &operator_account).await?;
    let receiver_account_key = PrivateKey::generate_ed25519();
    let receiver_account = create_receiver_account(-1, &receiver_account_key, &client).await?;

    let nft_serials = TokenMintTransaction::new()
        .token_id(nft.id)
        .metadata(repeat(vec![9, 1, 6]).take(10))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .serials
        .into_iter()
        .map(|it| it as u64)
        .collect::<Vec<u64>>();

    _ = TransferTransaction::new()
        .token_transfer(ft.id, operator_account.id, -10)
        .token_transfer(ft.id, receiver_account.id, 10)
        .nft_transfer(nft.id.nft(nft_serials[0]), operator_account.id, receiver_account.id)
        .nft_transfer(nft.id.nft(nft_serials[1]), operator_account.id, receiver_account.id)
        .nft_transfer(nft.id.nft(nft_serials[2]), operator_account.id, receiver_account.id)
        .nft_transfer(nft.id.nft(nft_serials[3]), operator_account.id, receiver_account.id)
        .nft_transfer(nft.id.nft(nft_serials[4]), operator_account.id, receiver_account.id)
        .nft_transfer(nft.id.nft(nft_serials[5]), operator_account.id, receiver_account.id)
        .nft_transfer(nft.id.nft(nft_serials[6]), operator_account.id, receiver_account.id)
        .nft_transfer(nft.id.nft(nft_serials[7]), operator_account.id, receiver_account.id)
        .nft_transfer(nft.id.nft(nft_serials[8]), operator_account.id, receiver_account.id)
        .nft_transfer(nft.id.nft(nft_serials[9]), operator_account.id, receiver_account.id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let res = TokenRejectTransaction::new()
        .owner(receiver_account.id)
        .add_token_id(ft.id)
        .nft_ids([
            nft.id.nft(nft_serials[0]),
            nft.id.nft(nft_serials[1]),
            nft.id.nft(nft_serials[2]),
            nft.id.nft(nft_serials[3]),
            nft.id.nft(nft_serials[4]),
            nft.id.nft(nft_serials[5]),
            nft.id.nft(nft_serials[6]),
            nft.id.nft(nft_serials[7]),
            nft.id.nft(nft_serials[8]),
            nft.id.nft(nft_serials[9]),
        ])
        .freeze_with(&client)?
        .sign(receiver_account.key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus {
            status: Status::TokenReferenceListSizeLimitExceeded,
            ..
        })
    );

    nft.delete(&client).await?;
    ft.delete(&client).await?;

    Ok(())
}

async fn create_ft(
    client: &Client,
    owner: &Account,
    decimals: u32,
) -> anyhow::Result<FungibleToken> {
    let id = TokenCreateTransaction::new()
        .name("ffff".to_string())
        .symbol("F".to_string())
        .token_memo("memo".to_string())
        .decimals(decimals)
        .initial_supply(1_000_000)
        .max_supply(1_000_000)
        .treasury_account_id(owner.id)
        .token_supply_type(hedera::TokenSupplyType::Finite)
        .admin_key(owner.key.public_key())
        .freeze_key(owner.key.public_key())
        .wipe_key(owner.key.public_key())
        .supply_key(owner.key.public_key())
        .metadata_key(owner.key.public_key())
        .pause_key(owner.key.public_key())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    Ok(FungibleToken { id, owner: owner.clone() })
}

async fn test_operator_account(config: &Config) -> anyhow::Result<Account> {
    if let Some(operator) = config.operator.clone() {
        Ok(Account { key: operator.private_key, id: operator.account_id })
    } else {
        return Err(anyhow!("no operator configured"));
    }
}

async fn create_receiver_account(
    max_automatic_token_associations: i32,
    account_key: &PrivateKey,
    client: &hedera::Client,
) -> hedera::Result<Account> {
    let receipt = hedera::AccountCreateTransaction::new()
        .key(account_key.public_key())
        .initial_balance(Hbar::new(10))
        .max_automatic_token_associations(max_automatic_token_associations)
        .execute(client)
        .await?
        .get_receipt(client)
        .await?;

    let account_id = receipt.account_id.unwrap();

    Ok(Account { key: account_key.clone(), id: account_id })
}
