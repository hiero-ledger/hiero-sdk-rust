// SPDX-License-Identifier: Apache-2.0

use std::io::Write;

use clap::Parser;
use hedera::{AccountBalanceQuery, AccountId, Client};

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, env, default_value = "testnet")]
    hedera_network: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let args = Args::parse();

    let client = Client::for_name(&args.hedera_network)?;

    // we need to return _something_ to say if stdin has been EOFed on us.
    if manual_checksum_validation(&client).await?.is_none() {
        return Ok(());
    }

    // we need to return _something_ to say if stdin has been EOFed on us.
    if automatic_checksum_validation(&client).await?.is_none() {
        return Ok(());
    }

    Ok(())
}

async fn manual_checksum_validation(client: &Client) -> anyhow::Result<Option<AccountId>> {
    println!("Example for manual checksum validation");

    let account_id = loop {
        let Some(account_id) = parse_account_id()? else {
            return Ok(None);
        };

        match account_id.validate_checksum(client) {
            Ok(()) => {}
            Err(e) => {
                println!("{e}");
                if let hedera::Error::BadEntityId {
                    shard,
                    realm,
                    num,
                    present_checksum,
                    expected_checksum,
                } = e
                {
                    println!(
                        "You entered {shard}.{realm}.{num}-{present_checksum}, the expected checksum was {expected_checksum}"
                    );
                    continue;
                }

                return Err(e.into());
            }
        };

        break account_id;
    };

    let balance = AccountBalanceQuery::new()
        .account_id(account_id)
        .execute(client)
        .await?;

    println!("Balance for account {account_id}: {balance:?}");

    Ok(Some(account_id))
}

async fn automatic_checksum_validation(client: &Client) -> anyhow::Result<Option<AccountId>> {
    println!("Example for automatic checksum validation");

    client.set_auto_validate_checksums(true);

    let Some(account_id) = parse_account_id()? else {
        return Ok(None);
    };

    let balance = AccountBalanceQuery::new()
        .account_id(account_id)
        .execute(client)
        .await?;

    println!("Balance for account {account_id}: {balance:?}");

    Ok(Some(account_id))
}

fn parse_account_id() -> anyhow::Result<Option<AccountId>> {
    loop {
        print!("Enter an account ID with checksum: ");
        let _ = std::io::stdout().flush();
        let stdin = std::io::stdin();
        let line = {
            let mut line = String::new();
            match stdin.read_line(&mut line) {
                Ok(0) => return Ok(None),
                Ok(_) => line,
                Err(e) => return Err(e.into()),
            }
        };

        let line = line.trim();

        let account_id: AccountId = match line.parse() {
            Ok(account_id) => account_id,
            Err(err @ hedera::Error::BasicParse(_)) => {
                println!("{err:?}");
                continue;
            }
            Err(e) => return Err(e.into()),
        };

        let Some(checksum) = account_id.checksum else {
            println!("You must enter a checksum.");
            continue;
        };

        println!("The checksum entered was: {checksum}");

        return Ok(Some(account_id));
    }
}
