// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hedera::{
    AccountCreateTransaction, AccountDeleteTransaction, AccountId, Client, Hbar, PrivateKey, TokenAssociateTransaction, TokenCreateTransaction, TokenDeleteTransaction, TokenGrantKycTransaction, TokenWipeTransaction, TransferTransaction
};
use time::{Duration, OffsetDateTime};

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, env)]
    operator_account_id: AccountId,

    #[clap(long, env)]
    operator_key: PrivateKey,

    #[clap(long, env, default_value = "testnet")]
    hedera_network: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let Args {
        operator_account_id,
        operator_key,
        hedera_network,
    } = Args::parse();

    let client = Client::for_name(&hedera_network)?;

    client.set_operator(operator_account_id, operator_key.clone());
    let (private_key1, account_id1) = create_account(&client, 1).await?;
    let (private_key2, account_id2) = create_account(&client, 2).await?;

    let token_id = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .decimals(3)
        .initial_supply(1_000_000)
        .treasury_account_id(operator_account_id)
        .admin_key(operator_key.clone().public_key())
        .freeze_key(operator_key.clone().public_key())
        .wipe_key(operator_key.clone().public_key())
        .kyc_key(operator_key.clone().public_key())
        .supply_key(operator_key.clone().public_key())
        .expiration_time(OffsetDateTime::now_utc() + Duration::hours(2))
        .freeze_default(false)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    println!("token = {token_id}");

    TokenAssociateTransaction::new()
        .account_id(account_id1)
        .token_ids([token_id])
        .freeze_with(&client)?
        .sign(private_key1.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    println!("Associated account {account_id1} with token {token_id}");

    TokenAssociateTransaction::new()
        .account_id(account_id2)
        .token_ids([token_id])
        .freeze_with(&client)?
        .sign(private_key2.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    println!("Associated account {account_id2} with token {token_id}");

    TokenGrantKycTransaction::new()
        .account_id(account_id1)
        .token_id(token_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    println!("Granted KYC for account {account_id1} on token {token_id}");

    TokenGrantKycTransaction::new()
        .account_id(account_id2)
        .token_id(token_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    println!("Granted KYC for account {account_id2} on token {token_id}");

    TransferTransaction::new()
        .token_transfer(token_id, operator_account_id, -10)
        .token_transfer(token_id, account_id1, 10)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    println!("Sent 10 tokens from account {operator_account_id} to account {account_id1} on token {token_id}");

    TransferTransaction::new()
        .token_transfer(token_id, account_id1, -10)
        .token_transfer(token_id, account_id2, 10)
        .freeze_with(&client)?
        .sign(private_key1.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    println!(
        "Sent 10 tokens from account {account_id1} to account {account_id2} on token {token_id}"
    );

    TransferTransaction::new()
        .token_transfer(token_id, account_id2, -10)
        .token_transfer(token_id, account_id1, 10)
        .freeze_with(&client)?
        .sign(private_key2.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    println!(
        "Sent 10 tokens from account {account_id2} to account {account_id1} on token {token_id}"
    );

    TokenWipeTransaction::new()
        .account_id(account_id1)
        .token_id(token_id)
        .amount(10_u64)
        .freeze_with(&client)?
        .sign(private_key1.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    println!("Wiped balance of token {token_id} from account {account_id1}");

    TokenDeleteTransaction::new()
        .token_id(token_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    println!("Deleted token {token_id}");

    delete_account(&client, operator_account_id, 1, account_id1, private_key1).await?;
    delete_account(&client, operator_account_id, 2, account_id2, private_key2).await?;

    Ok(())
}

async fn create_account(
    client: &Client,
    account_number: usize,
) -> anyhow::Result<(PrivateKey, AccountId)> {
    let private_key = PrivateKey::generate_ed25519();
    println!("private key  = {private_key}");
    println!("public key = {}", private_key.public_key());

    let receipt = AccountCreateTransaction::new()
        .key(private_key.public_key())
        .initial_balance(Hbar::from_tinybars(1000))
        .execute(client)
        .await?
        .get_receipt(client)
        .await?;

    let account_id = receipt.account_id.unwrap();
    println!("created account_id{account_number}: {account_id}");

    Ok((private_key, account_id))
}

async fn delete_account(
    client: &Client,
    operator_account_id: AccountId,
    account_number: usize,
    account_id: AccountId,
    private_key: PrivateKey,
) -> anyhow::Result<()> {
    AccountDeleteTransaction::new()
        .account_id(account_id)
        .transfer_account_id(operator_account_id)
        .freeze_with(client)?
        .sign(private_key)
        .execute(client)
        .await?
        .get_receipt(client)
        .await?;

    println!("deleted account_id{account_number}: {account_id}");

    Ok(())
}
