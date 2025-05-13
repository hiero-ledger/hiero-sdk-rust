// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hedera::{
    AccountAllowanceApproveTransaction, AccountBalanceQuery, AccountCreateTransaction, AccountDeleteTransaction, AccountId, Client, Hbar, PrivateKey, TransactionId, TransferTransaction
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

#[derive(Clone, Debug)]
struct Account {
    key: PrivateKey,
    id: AccountId,
    name: &'static str,
}

async fn create_account(client: &Client, name: &'static str) -> hedera::Result<Account> {
    let key = PrivateKey::generate_ed25519();

    let reciept = AccountCreateTransaction::new()
        .key(key.public_key())
        .initial_balance(Hbar::new(5))
        .account_memo(format!("[sdk::rust::account_allowance_example::{name}]"))
        .execute(client)
        .await?
        .get_receipt(client)
        .await?;

    let account_id = reciept
        .account_id
        .expect("Created account but no account ID in receipt");

    Ok(Account {
        key,
        id: account_id,
        name,
    })
}

async fn create_accounts(client: &Client) -> anyhow::Result<[Account; 3]> {
    println!("Creating accounts");

    let (alice, bob, charlie) = tokio::try_join!(
        create_account(client, "Alice"),
        create_account(client, "Bob"),
        create_account(client, "Charlie"),
    )?;

    let accounts = [alice, bob, charlie];

    for account in &accounts {
        println!("{}'s ID: {}", account.name, account.id);
    }

    Ok(accounts)
}

// this needs to be a function because rust doesn't have try blocks.
/// Transfer from `alice` (0) to `charlie` (2) via `bob`'s allowance (1).
async fn transfer(client: &Client, accounts: &[Account; 3], value: Hbar) -> hedera::Result<()> {
    let [alice, bob, charlie] = accounts;
    // `approved_{hbar,token}_transfer()` means that the transfer has been approved by an allowance
    // The allowance spender must be pay the fee for the transaction.
    // use `transaction_id()` to set the account ID that will pay the fee for the transaction.
    let _ = TransferTransaction::new()
        .approved_hbar_transfer(alice.id, -value)
        .hbar_transfer(charlie.id, value)
        .transaction_id(TransactionId::generate(bob.id))
        .freeze_with(client)?
        .sign(bob.key.clone())
        .execute(client)
        .await?
        .get_receipt(client)
        .await?;

    Ok(())
}

async fn demonstrate_allowances(client: &Client, accounts: &[Account; 3]) -> anyhow::Result<()> {
    const FIRST_ALLOWANCE_VALUE: Hbar = Hbar::new(2);
    const FIRST_TRANSFER_VALUE: Hbar = Hbar::new(1);
    const SECOND_ALLOWANCE_VALUE: Hbar = Hbar::new(3);
    const SECOND_TRANSFER_VALUE: Hbar = Hbar::new(2);

    let [alice, bob, charlie] = accounts;

    println!(
        "Approving an allowance of {FIRST_ALLOWANCE_VALUE} with owner {} and spender {}",
        alice.name, bob.name
    );

    let _ = AccountAllowanceApproveTransaction::new()
        .approve_hbar_allowance(alice.id, bob.id, FIRST_ALLOWANCE_VALUE)
        .freeze_with(client)?
        .sign(alice.key.clone())
        .execute(client)
        .await?
        .get_receipt(client)
        .await?;

    print_balances(client, accounts).await?;

    println!(
        "Transferring {FIRST_TRANSFER_VALUE} from {alice} to {charlie}, but the transaction is signed only by {bob} ({bob} is dipping into their allowance from {alice})",
        alice=alice.name,
        bob=bob.name,
        charlie=charlie.name
    );

    transfer(client, accounts, FIRST_TRANSFER_VALUE).await?;

    let current_balance = FIRST_ALLOWANCE_VALUE - FIRST_TRANSFER_VALUE;

    println!(
        "Transfer succeeded. {bob} should now have {current_balance} left in their allowance.",
        bob = bob.name,
    );

    print_balances(client, accounts).await?;

    println!(
        "Attempting to transfer {SECOND_TRANSFER_VALUE} from {alice} to {charlie} using {bob}'s allowance.",
        alice=alice.name,
        bob=bob.name,
        charlie=charlie.name
    );
    println!(
        "This should fail, because there is only {current_balance} left in {bob}'s allowance.",
        bob = bob.name
    );

    match transfer(client, accounts, SECOND_TRANSFER_VALUE).await {
        Ok(()) => {
            println!("The transfer succeeded. This should not happen.");
        }

        Err(e) => {
            println!("The transfer failed as expected: {e:?}");
        }
    }

    println!(
        "Adjusting {bob}'s allowance to {SECOND_ALLOWANCE_VALUE}.",
        bob = bob.name
    );

    let _ = AccountAllowanceApproveTransaction::new()
        .approve_hbar_allowance(alice.id, bob.id, SECOND_ALLOWANCE_VALUE)
        .freeze_with(client)?
        .sign(alice.key.clone())
        .execute(client)
        .await?
        .get_receipt(client)
        .await?;

    println!(
        "Attempting to transfer {SECOND_TRANSFER_VALUE} from {alice} to {charlie} using {bob}'s allowance again.",
        alice=alice.name,
        bob=bob.name,
        charlie=charlie.name
    );
    println!("This time it should succeed.");

    transfer(client, accounts, SECOND_TRANSFER_VALUE).await?;

    println!("Transfer succeeded.");

    print_balances(client, accounts).await?;

    println!("Deleting {bob}'s allowance", bob = bob.name);

    let _ = AccountAllowanceApproveTransaction::new()
        .approve_hbar_allowance(alice.id, bob.id, Hbar::ZERO)
        .freeze_with(client)?
        .sign(alice.key.clone())
        .execute(client)
        .await?
        .get_receipt(client)
        .await?;

    Ok(())
}

async fn clean_up(
    client: &Client,
    operator_id: AccountId,
    accounts: [Account; 3],
) -> anyhow::Result<()> {
    println!("Cleaning up...");

    for account in accounts {
        let _ = AccountDeleteTransaction::new()
            .account_id(account.id)
            .transfer_account_id(operator_id)
            .freeze_with(client)?
            .sign(account.key)
            .execute(client)
            .await?
            .get_receipt(client)
            .await?;

        println!("Deleted `{}` ({})", account.name, account.id);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    let args = Args::parse();

    let client = Client::for_name(&args.hedera_network)?;

    client.set_operator(args.operator_account_id, args.operator_key);

    let accounts = create_accounts(&client).await?;

    print_balances(&client, &accounts).await?;

    demonstrate_allowances(&client, &accounts).await?;
    clean_up(&client, args.operator_account_id, accounts).await?;

    println!("End of example");

    Ok(())
}

async fn print_balances(client: &Client, accounts: &[Account; 3]) -> hedera::Result<()> {
    for account in accounts {
        let balance = AccountBalanceQuery::new()
            .account_id(account.id)
            .execute(client)
            .await?
            .hbars;

        println!("{name}'s balance: {balance}", name = account.name);
    }

    Ok(())
}
