// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hedera::{
    AccountBalanceQuery, AccountId, AccountInfoQuery, Client, Hbar, PrivateKey, TransferTransaction
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

    // Hiero supports a form of auto account creation.
    //
    // You can "create" an account by generating a private key, and then deriving the public key,
    // without any need to interact with the Hiero network.  The public key more or less acts as the user's
    // account ID.  This public key is an account's alias_key: a public key that aliases (or will eventually alias)
    // to a Hiero account.
    //
    // An AccountId takes one of two forms: a normal `AccountId` with no `alias_key` takes the form 0.0.123,
    // while an account ID with an `alias_key` takes the form
    // 0.0.302a300506032b6570032100114e6abc371b82dab5c15ea149f02d34a012087b163516dd70f44acafabf7777
    // Note the prefix of "0.0." indicating the shard and realm.  Also note that the aliasKey is stringified
    // as a hex-encoded ASN1 DER representation of the key.
    //
    // An AccountId with an aliasKey can be used just like a normal AccountId for the purposes of queries and
    // transactions, however most queries and transactions involving such an AccountId won't work until Hbar has
    // been transferred to the alias_key account.
    //
    // There is no record in the Hiero network of an account associated with a given `alias_key`
    // until an amount of Hbar is transferred to the account.  The moment that Hbar is transferred to that `alias_key`
    // AccountId is the moment that that account actually begins to exist in the Hiero ledger.

    println!(r#""Creating" a new account"#);

    let private_key = PrivateKey::generate_ed25519();
    let public_key = private_key.public_key();

    // Assuming that the target shard and realm are known.
    // For now they are virtually always 0 and 0.
    let alias_account_id = public_key.to_account_id(0, 0);

    println!("New account ID: {alias_account_id}");
    println!("Just the aliasKey: {:?}", &alias_account_id.alias);

    // Note that no queries or transactions have taken place yet.
    // This account "creation" process is entirely local.
    //
    // AccountId::from_str can construct an AccountId with an alias_key.
    // It expects a string of the form 0.0.123 in the case of a normal AccountId, or of the form
    // 0.0.302a300506032b6570032100114e6abc371b82dab5c15ea149f02d34a012087b163516dd70f44acafabf7777
    // in the case of an AccountId with an alias.  Note the prefix of "0.0." to indicate the shard and realm.
    //
    // If the shard and realm are known, you may use PublicKey::from_str().to_account_id() to construct the
    // alias_key AccountId.

    println!("Transferring some Hbar to the new account");
    let _ = TransferTransaction::new()
        .hbar_transfer(args.operator_account_id, Hbar::new(-10))
        .hbar_transfer(alias_account_id, Hbar::new(10))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let balance = AccountBalanceQuery::new()
        .account_id(alias_account_id)
        .execute(&client)
        .await?;

    println!("Balances of the new account: {balance:?}");

    let info = AccountInfoQuery::new()
        .account_id(alias_account_id)
        .execute(&client)
        .await?;

    println!("Info about the new account: {info:?}");

    // Note that once an account exists in the ledger, it is assigned a normal AccountId, which can be retrieved
    // via an AccountInfoQuery.
    //
    // Users may continue to refer to the account by its alias_key AccountId, but they may also
    // now refer to it by its normal AccountId.

    println!("the normal account ID: {}", info.account_id);
    println!("the alias key: {:?}", info.alias_key);
    println!("Example complete!");

    Ok(())
}
