use clap::Parser;
use hiero_sdk::{
    AccountCreateTransaction, AccountId, AccountInfoQuery, AccountUpdateTransaction, Hbar, PrivateKey
};

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
    let _ = dotenvy::dotenv().ok();

    let args = Args::parse();

    let client = hiero_sdk::Client::for_name(&args.hedera_network)?;

    client.set_operator(args.operator_account_id, args.operator_key.clone());

    // First, we create a new account so we don't affect our account

    let key1 = PrivateKey::generate_ecdsa();
    let key2 = PrivateKey::generate_ecdsa();

    let response = AccountCreateTransaction::new()
        .set_key_without_alias(key1.public_key())
        .initial_balance(Hbar::new(1))
        .execute(&client)
        .await?;

    println!("transaction id: {}", response.transaction_id);

    let account_id = response.get_receipt(&client).await?.account_id.unwrap();

    println!("new account id: `{account_id}`");
    println!("account key: `{}`", key1.public_key());

    println!(":: update public key of account `{account_id}`");
    println!("set key = `{}`", key2.public_key());

    // note that we have to sign with both the new key (key2) and the old key (key1).
    let response = AccountUpdateTransaction::new()
        .account_id(account_id)
        .key(key2.public_key())
        .sign(key1.clone())
        .sign(key2.clone())
        .execute(&client)
        .await?;

    println!("transaction id: {}", response.transaction_id);

    let _ = response.get_receipt(&client).await?;

    println!(":: run AccountInfoQuery and check our current key");

    let info = AccountInfoQuery::new()
        .account_id(account_id)
        .execute(&client)
        .await?;

    println!("key: {:?}", info.key);

    Ok(())
}
