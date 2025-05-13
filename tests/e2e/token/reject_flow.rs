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
    TokenAssociateTransaction,
    TokenCreateTransaction,
    TokenMintTransaction,
    TokenNftInfoQuery,
    TokenRejectFlow,
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
async fn basic_flow_fungible_token() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;
    let ft = create_ft(&client, &operator_account, 3).await?;
    let receiver_account = create_receiver_account(0, &operator_account.key, &client).await?;

    _ = TokenAssociateTransaction::new()
        .account_id(receiver_account.id)
        .token_ids(vec![ft.id])
        .freeze_with(&client)?
        .sign(receiver_account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    _ = TransferTransaction::new()
        .token_transfer(ft.id, operator_account.id, -10)
        .token_transfer(ft.id, receiver_account.id, 10)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    _ = TokenRejectFlow::new()
        .owner(receiver_account.id)
        .add_token_id(ft.id)
        .freeze_with(client.clone())
        .sign(receiver_account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let treasury_account_balance =
        AccountBalanceQuery::new().account_id(operator_account.id).execute(&client).await?;

    assert_eq!(treasury_account_balance.tokens.get(&ft.id), Some(&(1_000_000 as u64)));

    let res = TransferTransaction::new()
        .token_transfer(ft.id, operator_account.id, -10)
        .token_transfer(ft.id, receiver_account.id, 10)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::TokenNotAssociatedToAccount, .. })
    );

    ft.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn basic_flow_nft() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let operator_account = test_operator_account(&config).await?;
    let nft = Nft::create(&client, &operator_account).await?;
    let receiver_account = create_receiver_account(0, &operator_account.key, &client).await?;

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

    _ = TokenAssociateTransaction::new()
        .account_id(receiver_account.id)
        .token_ids([nft.id])
        .freeze_with(&client)?
        .sign(receiver_account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    _ = TransferTransaction::new()
        .nft_transfer(nft.id.nft(nft_serials[0]), operator_account.id, receiver_account.id)
        .nft_transfer(nft.id.nft(nft_serials[1]), operator_account.id, receiver_account.id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    _ = TokenRejectFlow::new()
        .owner(receiver_account.id)
        .nft_ids([nft.id.nft(nft_serials[0]), nft.id.nft(nft_serials[1])])
        .freeze_with(client.clone())
        .sign(receiver_account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    println!("here3");

    let nft_info =
        TokenNftInfoQuery::new().nft_id(nft.id.nft(nft_serials[1])).execute(&client).await?;

    assert_eq!(nft_info.account_id, operator_account.id);

    let res = TransferTransaction::new()
        .nft_transfer(nft.id.nft(nft_serials[1]), operator_account.id, receiver_account.id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::TokenNotAssociatedToAccount, .. })
    );

    nft.delete(&client).await?;

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
        .initial_balance(Hbar::new(1))
        .max_automatic_token_associations(max_automatic_token_associations)
        .execute(client)
        .await?
        .get_receipt(client)
        .await?;

    let account_id = receipt.account_id.unwrap();

    Ok(Account { key: account_key.clone(), id: account_id })
}
