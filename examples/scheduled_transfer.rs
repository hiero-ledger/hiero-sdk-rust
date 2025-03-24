// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hedera::{
    AccountBalanceQuery, AccountCreateTransaction, AccountDeleteTransaction, AccountId, Client, Hbar, PrivateKey, ScheduleCreateTransaction, ScheduleInfoQuery, ScheduleSignTransaction, TransferTransaction
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

    env_logger::init();

    let args = Args::parse();

    let client = Client::for_name(&args.hedera_network)?;

    client.set_operator(args.operator_account_id, args.operator_key.clone());

    // A scheduled transaction is a transaction that has been proposed by an account,
    // but which requires more signatures before it will actually execute on the Hedera network.
    //
    // For example, if Alice wants to transfer an amount of Hbar to Bob, and Bob has
    // receiverSignatureRequired set to true, then that transaction must be signed by
    // both Alice and Bob before the transaction will be executed.
    //
    // To solve this problem, Alice can propose the transaction by creating a scheduled
    // transaction on the Hedera network which, if executed, would transfer Hbar from
    // Alice to Bob.  That scheduled transaction will have a ScheduleId by which we can
    // refer to that scheduled transaction.  Alice can communicate the ScheduleId to Bob, and
    // then Bob can use a ScheduleSignTransaction to sign that scheduled transaction.
    //
    // Bob has a 30 minute window in which to sign the scheduled transaction, starting at the
    // moment that Alice creates the scheduled transaction.  If a scheduled transaction
    // is not signed by all of the necessary signatories within the 30 minute window,
    // that scheduled transaction will expire, and will not be executed.
    //
    // Once a scheduled transaction has all of the signatures necessary to execute, it will
    // be executed on the Hedera network automatically.  If you create a scheduled transaction
    // on the Hedera network, but that transaction only requires your signature in order to
    // execute and no one else's, that scheduled transaction will be automatically
    // executed immediately.
    let bobs_key = PrivateKey::generate_ed25519();

    let bobs_id = AccountCreateTransaction::new()
        .receiver_signature_required(true)
        .key(bobs_key.public_key())
        .initial_balance(Hbar::new(10))
        .freeze_with(&client)?
        .sign(bobs_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    println!(
        "Alice's ID: {}",
        args.operator_account_id.to_string_with_checksum(&client)?
    );
    println!("Bob's ID: {}", bobs_id.to_string_with_checksum(&client)?);

    let bobs_initial_balance = AccountBalanceQuery::new()
        .account_id(bobs_id)
        .execute(&client)
        .await?;

    println!("Bob's initial balance:");
    println!("{bobs_initial_balance:?}");

    let mut transfer_to_schedule = TransferTransaction::new();

    transfer_to_schedule
        .hbar_transfer(args.operator_account_id, Hbar::new(-10))
        .hbar_transfer(bobs_id, Hbar::new(10));

    println!("Transfer to be scheduled:");
    println!("{transfer_to_schedule:?}");

    // The `payer_account_id` is the account that will be charged the fee
    // for executing the scheduled transaction if/when it is executed.
    // That fee is separate from the fee that we will pay to execute the
    // ScheduleCreateTransaction itself.
    //
    // To clarify: Alice pays a fee to execute the ScheduleCreateTransaction,
    // which creates the scheduled transaction on the Hedera network.
    // She specifies when creating the scheduled transaction that Bob will pay
    // the fee for the scheduled transaction when it is executed.
    //
    // If `payer_account_id` is not specified, the account who creates the scheduled transaction
    // will be charged for executing the scheduled transaction.
    let schedule_id = ScheduleCreateTransaction::new()
        .scheduled_transaction(transfer_to_schedule)
        .payer_account_id(bobs_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .schedule_id
        .unwrap();

    println!(
        "The schedule_id is: {}",
        schedule_id.to_string_with_checksum(&client)
    );

    // Bob's balance should be unchanged.  The transfer has been scheduled, but it hasn't been executed yet
    // because it requires Bob's signature.
    let bobs_balance_after_schedule = AccountBalanceQuery::new()
        .account_id(bobs_id)
        .execute(&client)
        .await?;

    println!("Bob's balance after scheduling the transfer (should be unchanged):");
    println!("{bobs_balance_after_schedule:?}");

    // Once Alice has communicated the scheduleId to Bob, Bob can query for information about the
    // scheduled transaction.
    let scheduled_transaction_info = ScheduleInfoQuery::new()
        .schedule_id(schedule_id)
        .execute(&client)
        .await?;

    println!("Info about scheduled transaction:");
    println!("{scheduled_transaction_info:?}");

    // getScheduledTransaction() will return an SDK Transaction object identical to the transaction
    // that was scheduled, which Bob can then inspect like a normal transaction.
    let scheduled_transaction = scheduled_transaction_info.scheduled_transaction()?;

    // We happen to know that this transaction is (or certainly ought to be) a TransferTransaction
    let Ok(scheduled_transfer) = scheduled_transaction.downcast::<TransferTransaction>() else {
        anyhow::bail!("scheduled transaction was not a transfer transaction");
    };

    println!("The scheduled transfer transaction from Bob's POV:");
    println!("{scheduled_transfer:?}");

    ScheduleSignTransaction::new()
        .schedule_id(schedule_id)
        .freeze_with(&client)?
        .sign(bobs_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let balance_after_signing = AccountBalanceQuery::new()
        .account_id(bobs_id)
        .execute(&client)
        .await?;

    println!("Bob's balance after signing the scheduled transaction:");
    println!("{balance_after_signing:?}");

    let post_transaction_info = ScheduleInfoQuery::new()
        .schedule_id(schedule_id)
        .execute(&client)
        .await?;

    println!("Info on the scheduled transaction, executed_at should no longer be null:");
    println!("{post_transaction_info:?}");

    // Clean up
    AccountDeleteTransaction::new()
        .transfer_account_id(args.operator_account_id)
        .account_id(bobs_id)
        .freeze_with(&client)?
        .sign(bobs_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}
