// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hedera::{
    AccountCreateTransaction, AccountId, Client, Hbar, KeyList, PrivateKey, Transaction, TransferTransaction
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

    println!("private key for user 1 = {user1_key}");
    println!("public key for user 1 = {}", user1_key.public_key());
    println!("private key for user 2 = {user2_key}");
    println!("public key for user 2 = {}", user2_key.public_key());

    // create a multi-sig account
    let keylist = KeyList::from([user1_key.public_key(), user2_key.public_key()]);

    let create_account_transaction = AccountCreateTransaction::new()
        .initial_balance(Hbar::new(2))
        .key(keylist)
        .execute(&client)
        .await?;

    let receipt = create_account_transaction.get_receipt(&client).await?;

    let account_id = receipt.account_id.unwrap();

    println!("account id = {account_id}");

    // create a transfer from new account to 0.0.3
    let mut transfer_transaction = TransferTransaction::new();

    transfer_transaction
        .node_account_ids([(AccountId::from(3))])
        .hbar_transfer(account_id, Hbar::new(-1))
        .hbar_transfer(AccountId::from(3), Hbar::new(1))
        .freeze_with(&client)?;

    // convert transaction to bytes to send to signatories
    let transaction_bytes = transfer_transaction.to_bytes()?;
    let mut transaction_to_execute = Transaction::from_bytes(&transaction_bytes)?;

    // ask users to sign and return signature
    let user1_signature =
        user1_key.sign_transaction(&mut Transaction::from_bytes(&transaction_bytes)?)?;
    let user2_signature =
        user2_key.sign_transaction(&mut Transaction::from_bytes(&transaction_bytes)?)?;

    // recreate the transaction from bytes
    transaction_to_execute.sign_with_operator(&client)?;
    transaction_to_execute.add_signature(user1_key.public_key(), user1_signature);
    transaction_to_execute.add_signature(user2_key.public_key(), user2_signature);

    let result = transaction_to_execute.execute(&client).await?;
    let receipt = result.get_receipt(&client).await?;
    println!("{:?}", receipt.status);

    Ok(())
}
