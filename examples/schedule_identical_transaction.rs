// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hedera::{
    AccountCreateTransaction, AccountDeleteTransaction, AccountId, Client, Hbar, Key, KeyList, PrivateKey, ScheduleCreateTransaction, ScheduleId, ScheduleInfoQuery, ScheduleSignTransaction, Status, TransactionReceiptQuery, TransferTransaction
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

    client.set_operator(args.operator_account_id, args.operator_key.clone());

    println!("threshold key example");
    println!("keys:");

    let mut private_keys = Vec::with_capacity(3);
    let mut public_keys = Vec::with_capacity(3);
    let mut clients = Vec::with_capacity(3);
    let mut accounts = Vec::with_capacity(3);

    for i in 0..3 {
        let private_key = PrivateKey::generate_ed25519();
        let public_key = private_key.public_key();

        println!("key #{i}");
        println!("private key: {private_key}");
        println!("public key: {public_key}");

        let receipt = AccountCreateTransaction::new()
            .key(public_key)
            .initial_balance(Hbar::new(1))
            .execute(&client)
            .await?
            .get_receipt(&client)
            .await?;

        let account_id = receipt.account_id.unwrap();

        let client = Client::for_name(&args.hedera_network)?;

        client.set_operator(account_id, private_key.clone());

        private_keys.push(private_key);
        public_keys.push(public_key);
        clients.push(client);
        accounts.push(account_id);
        println!("account = {account_id}");
    }

    let key_list = KeyList {
        keys: public_keys.iter().copied().map(Key::from).collect(),
        threshold: Some(2),
    };

    // We are using all of these keys, so the scheduled transaction doesn't automatically go through
    // It works perfectly fine with just one key
    // The key that must sign each transfer out of the account. If receiverSigRequired is true, then
    // it must also sign any transfer into the account.
    let threshold_account = AccountCreateTransaction::new()
        .key(key_list.clone())
        .initial_balance(Hbar::new(10))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    println!("threshold account = {threshold_account}");

    let mut schedule_id: Option<ScheduleId> = None;

    for (loop_client, operator_id) in clients.iter().zip(&accounts) {
        // Each loopClient creates an identical transaction, sending 1 hbar to each of the created accounts,
        // sent from the threshold Account
        let mut tx = TransferTransaction::new();
        for account in &accounts {
            tx.hbar_transfer(*account, Hbar::new(1));
        }

        tx.hbar_transfer(threshold_account, -Hbar::new(3));

        let mut scheduled_tx = ScheduleCreateTransaction::new();

        scheduled_tx.scheduled_transaction(tx);

        scheduled_tx.payer_account_id(threshold_account);

        let response = scheduled_tx.execute(loop_client).await?;

        let loop_receipt = TransactionReceiptQuery::new()
            .transaction_id(response.transaction_id)
            .node_account_ids([response.node_account_id])
            .execute(loop_client)
            .await?;

        println!(
            "operator [{operator_id}]: schedule_id = {:?}",
            loop_receipt.schedule_id
        );

        // Save the schedule ID, so that it can be asserted for each loopClient submission
        let schedule_id = &*schedule_id.get_or_insert_with(|| loop_receipt.schedule_id.unwrap());

        if Some(schedule_id) != loop_receipt.schedule_id.as_ref() {
            println!(
                "invalid generated schedule id, expected {schedule_id}, got {:?}",
                loop_receipt.schedule_id
            );
        }

        // If the status return by the receipt is related to already created, execute a schedule sign transaction
        if loop_receipt.status == Status::IdenticalScheduleAlreadyCreated {
            let sign_response = ScheduleSignTransaction::new()
                .schedule_id(*schedule_id)
                .node_account_ids([response.node_account_id])
                .execute(loop_client)
                .await?;

            let sign_receipt = TransactionReceiptQuery::new()
                .transaction_id(sign_response.transaction_id)
                .execute(&client)
                .await?;

            if !matches!(
                sign_receipt.status,
                Status::Success | Status::ScheduleAlreadyExecuted
            ) {
                println!(
                    "Bad status while getting receipt of schedule sign with operator {operator_id}: {:?}"
, sign_receipt.status,
                );
                return Ok(());
            }
        }
    }

    println!(
        "{:?}",
        ScheduleInfoQuery::new()
            .schedule_id(schedule_id.unwrap())
            .execute(&client)
            .await?
    );

    let mut threshold_delete_tx = AccountDeleteTransaction::new();

    threshold_delete_tx
        .account_id(threshold_account)
        .transfer_account_id(args.operator_account_id)
        .freeze_with(&client)?;

    for (key, account) in private_keys.into_iter().zip(accounts) {
        threshold_delete_tx.sign(key.clone());

        AccountDeleteTransaction::new()
            .account_id(account)
            .transfer_account_id(args.operator_account_id)
            .freeze_with(&client)?
            .sign(key)
            .execute(&client)
            .await?
            .get_receipt(&client)
            .await?;
    }

    threshold_delete_tx
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}
