// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hedera::{
    AccountCreateTransaction, AccountId, Client, Hbar, KeyList, PrivateKey, TransferTransaction
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
    let _ = dotenvy::dotenv();

    let args = Args::parse();

    let client = Client::for_name(&args.hedera_network)?;

    client.set_operator(args.operator_account_id, args.operator_key);

    let user1_key = PrivateKey::generate_ed25519();
    let user2_key = PrivateKey::generate_ed25519();

    let keylist = KeyList::from([user1_key.public_key(), user2_key.public_key()]);

    let create_account_transaction = AccountCreateTransaction::new()
        .initial_balance(Hbar::new(2))
        .key(keylist)
        .execute(&client)
        .await?;

    let receipt = create_account_transaction.get_receipt(&client).await?;

    let account_id = receipt.account_id.unwrap();

    println!("account id = {account_id}");

    let mut transfer_transaction = TransferTransaction::new();

    transfer_transaction
        .node_account_ids([AccountId::from(3)])
        .hbar_transfer(account_id, Hbar::new(-1))
        .hbar_transfer(AccountId::from(3), Hbar::new(1))
        .freeze_with(&client)?;

    transfer_transaction.sign_with_operator(&client)?;
    user1_key.sign_transaction(&mut transfer_transaction)?;
    user2_key.sign_transaction(&mut transfer_transaction)?;

    let result = transfer_transaction.execute(&client).await?;
    let receipt = result.get_receipt(&client).await?;

    println!("{:?}", receipt.status);

    Ok(())
}
