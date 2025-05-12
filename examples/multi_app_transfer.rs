// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hedera::{
    AccountBalanceQuery, AccountCreateTransaction, AccountId, Client, Hbar, PrivateKey, Transaction, TransferTransaction
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

    // the exchange should possess this key, we're only generating it for demonstration purposes
    let exchange_key = PrivateKey::generate_ed25519();
    // this is the only key we should actually possess
    let user_key = PrivateKey::generate_ed25519();

    // the exchange creates an account for the user to transfer funds to
    let exchange_account_id = AccountCreateTransaction::new()
        // the exchange only accepts transfers that it validates through a side channel (e.g. REST API)
        .receiver_signature_required(true)
        .key(exchange_key.public_key())
        // The owner key has to sign this transaction
        // when receiver_signature_required is true
        .freeze_with(&client)?
        .sign(exchange_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    // for the purpose of this example we create an account for
    // the user with a balance of 5 h
    let user_account_id = AccountCreateTransaction::new()
        .initial_balance(Hbar::new(5))
        .key(user_key.public_key())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    // next we make a transfer from the user account to the
    // exchange account, this requires signing by both parties
    let mut transfer_txn = TransferTransaction::new();

    transfer_txn
        .hbar_transfer(user_account_id, Hbar::new(-2))
        .hbar_transfer(exchange_account_id, Hbar::new(2))
        // the exchange-provided memo required to validate the transaction
        .transaction_memo("https://some-exchange.com/user1/account1")
        // NOTE: to manually sign, you must freeze the Transaction first
        .freeze_with(&client)?
        .sign(user_key);

    // the exchange must sign the transaction in order for it to be accepted by the network
    // assume this is some REST call to the exchange API server
    let signed_txn_bytes = exchange_signs_transaction(exchange_key, &transfer_txn.to_bytes()?)?;

    // parse the transaction bytes returned from the exchange
    let mut signed_transfer_txn = Transaction::from_bytes(&signed_txn_bytes)?
        .downcast::<TransferTransaction>()
        .unwrap();

    // get the amount we are about to transfer
    // we built this with +2, -2 (which we might see in any order)
    let transfer_amount = signed_transfer_txn
        .get_hbar_transfers()
        .values()
        .copied()
        .next()
        .map(|it| if Hbar::ZERO >= it { it } else { -it });

    println!("about to transfer {transfer_amount:?}...");

    // we now execute the signed transaction and wait for it to be accepted
    let transaction_response = signed_transfer_txn.execute(&client).await?;

    // (important!) wait for consensus by querying for the receipt
    transaction_response.get_receipt(&client).await?;

    let sender_balance_after = AccountBalanceQuery::new()
        .account_id(user_account_id)
        .execute(&client)
        .await?
        .hbars;

    let receipt_balance_after = AccountBalanceQuery::new()
        .account_id(exchange_account_id)
        .execute(&client)
        .await?
        .hbars;

    println!("{user_account_id} balance = {sender_balance_after}");
    println!("{exchange_account_id} balance = {receipt_balance_after}");

    Ok(())
}

fn exchange_signs_transaction(
    exchange_key: PrivateKey,
    transaction_data: &[u8],
) -> hedera::Result<Vec<u8>> {
    Transaction::from_bytes(transaction_data)?
        .sign(exchange_key)
        .to_bytes()
}
