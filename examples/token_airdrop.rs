// SPDX-License-Identifier: Apache-2.0

use std::iter::repeat;

use clap::Parser;
use hedera::{
    AccountBalanceQuery, AccountCreateTransaction, AccountId, Client, Hbar, PrivateKey, TokenAirdropTransaction, TokenCancelAirdropTransaction, TokenClaimAirdropTransaction, TokenCreateTransaction, TokenMintTransaction, TokenRejectTransaction
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
    let private_key_1 = PrivateKey::generate_ecdsa();
    let alice_id = AccountCreateTransaction::new()
        .key(private_key_1.public_key())
        .initial_balance(Hbar::new(10))
        .max_automatic_token_associations(-1)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    let private_key_2 = PrivateKey::generate_ecdsa();
    let bob_id = AccountCreateTransaction::new()
        .key(private_key_2.public_key())
        .max_automatic_token_associations(1)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    let private_key_3 = PrivateKey::generate_ecdsa();
    let carol_id = AccountCreateTransaction::new()
        .key(private_key_3.public_key())
        .max_automatic_token_associations(0)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    let treasury_key = PrivateKey::generate_ecdsa();
    let treasury_account_id = AccountCreateTransaction::new()
        .key(treasury_key.public_key())
        .initial_balance(Hbar::new(10))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .account_id
        .unwrap();

    /*
     * Step 2:
     * Create FT and NFT and mint
     */
    let token_id = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .decimals(3)
        .initial_supply(100)
        .max_supply(100)
        .treasury_account_id(treasury_account_id)
        .token_supply_type(hedera::TokenSupplyType::Finite)
        .admin_key(operator_key.clone().public_key())
        .freeze_key(operator_key.clone().public_key())
        .supply_key(operator_key.clone().public_key())
        .pause_key(operator_key.clone().public_key())
        .expiration_time(OffsetDateTime::now_utc() + Duration::hours(2))
        .freeze_with(&client)?
        .sign(treasury_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    let nft_id = TokenCreateTransaction::new()
        .name("example NFT")
        .symbol("F")
        .max_supply(10)
        .treasury_account_id(treasury_account_id)
        .token_supply_type(hedera::TokenSupplyType::Finite)
        .token_type(hedera::TokenType::NonFungibleUnique)
        .admin_key(operator_key.clone().public_key())
        .freeze_key(operator_key.clone().public_key())
        .supply_key(operator_key.clone().public_key())
        .pause_key(operator_key.clone().public_key())
        .expiration_time(OffsetDateTime::now_utc() + Duration::hours(2))
        .freeze_with(&client)?
        .sign(treasury_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    _ = TokenMintTransaction::new()
        .token_id(nft_id)
        .metadata(repeat(vec![9, 1, 6]).take(4).collect::<Vec<Vec<_>>>())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    /*
     * Step 3:
     * Airdrop fungible tokens to all 3 accounts
     */
    println!("Airdropping tokens to all accounts");

    let tx_record = TokenAirdropTransaction::new()
        .token_transfer(token_id, alice_id, 10)
        .token_transfer(token_id, treasury_account_id, -10)
        .token_transfer(token_id, bob_id, 10)
        .token_transfer(token_id, treasury_account_id, -10)
        .token_transfer(token_id, carol_id, 10)
        .token_transfer(token_id, treasury_account_id, -10)
        .freeze_with(&client)?
        .sign(treasury_key.clone())
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    /*
     * Step 4:
     * Get the transaction record and see one pending airdrop (for carol)
     */
    println!(
        "Pending airdrop length: {}",
        tx_record.pending_airdrop_records.len()
    );
    println!(
        "Pending airdrops: {:?}",
        tx_record.pending_airdrop_records.get(0)
    );

    /*
     * Step 5:
     * Query to verify alice and bob received the airdrops and carol did not
     */
    let alice_balance = AccountBalanceQuery::new()
        .account_id(alice_id)
        .execute(&client)
        .await?;

    let bob_balance = AccountBalanceQuery::new()
        .account_id(bob_id)
        .execute(&client)
        .await?;

    let carol_balance = AccountBalanceQuery::new()
        .account_id(carol_id)
        .execute(&client)
        .await?;

    println!(
        "Alice ft balance after airdrop: {}",
        alice_balance.tokens.get(&token_id).unwrap()
    );
    println!(
        "Bob ft balance after airdrop: {}",
        bob_balance.tokens.get(&token_id).unwrap()
    );
    println!(
        "Carol ft balance after airdrop: {:?}",
        carol_balance.tokens.get(&token_id)
    );

    /*
     * Step 6:
     * Claim the airdrop for carol
     */
    println!("Claiming ft with Carol");

    _ = TokenClaimAirdropTransaction::new()
        .add_pending_airdrop_id(
            tx_record
                .pending_airdrop_records
                .get(0)
                .unwrap()
                .pending_airdrop_id,
        )
        .freeze_with(&client)?
        .sign(private_key_3.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let carol_balance = AccountBalanceQuery::new()
        .account_id(carol_id)
        .execute(&client)
        .await?;

    println!(
        "Carol ft balance after airdrop: {}",
        carol_balance.tokens.get(&token_id).unwrap()
    );

    /*
     * Step 7:
     * Airdrop the NFTs to all three accounts
     */
    println!("Airdropping nfts");
    let tx_record = TokenAirdropTransaction::new()
        .nft_transfer(nft_id.nft(1), treasury_account_id, alice_id)
        .nft_transfer(nft_id.nft(2), treasury_account_id, bob_id)
        .nft_transfer(nft_id.nft(3), treasury_account_id, carol_id)
        .freeze_with(&client)?
        .sign(treasury_key.clone())
        .execute(&client)
        .await?
        .get_record(&client)
        .await?;

    /*
     * Step 8:
     * Get the transaction record and verify two pending airdrops (for bob & carol)
     */
    println!(
        "Pending airdrops length: {}",
        tx_record.pending_airdrop_records.len()
    );
    println!(
        "Pending airdrops for Bob: {}",
        tx_record.pending_airdrop_records.get(0).unwrap()
    );
    println!(
        "Pending airdrops for Carol: {}",
        tx_record.pending_airdrop_records.get(1).unwrap()
    );

    /*
     * Step 9:
     * Query to verify alice received the airdrop and bob and carol did not
     */
    let alice_balance = AccountBalanceQuery::new()
        .account_id(alice_id)
        .execute(&client)
        .await?;

    let bob_balance = AccountBalanceQuery::new()
        .account_id(bob_id)
        .execute(&client)
        .await?;

    let carol_balance = AccountBalanceQuery::new()
        .account_id(carol_id)
        .execute(&client)
        .await?;

    println!(
        "Alice nft balance after airdrop: {}",
        alice_balance.tokens.get(&nft_id).unwrap()
    );

    println!(
        "Bob nft balance after airdrop: {:?}",
        bob_balance.tokens.get(&nft_id)
    );

    println!(
        "Carol nft balance after airdrop: {:?}",
        carol_balance.tokens.get(&nft_id)
    );

    /*
     * Step 10:
     * Claim the airdrop for bob
     */
    println!("Claiming nft with Bob");
    _ = TokenClaimAirdropTransaction::new()
        .add_pending_airdrop_id(
            tx_record
                .pending_airdrop_records
                .get(0)
                .unwrap()
                .pending_airdrop_id,
        )
        .freeze_with(&client)?
        .sign(private_key_2.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let bob_balance = AccountBalanceQuery::new()
        .account_id(bob_id)
        .execute(&client)
        .await?;

    println!(
        "Bob nft balance after claim: {}",
        bob_balance.tokens.get(&nft_id).unwrap()
    );

    /*
     * Step 11:
     * Cancel the airdrop for carol
     */
    println!("Cancelling nft for Carol");

    _ = TokenCancelAirdropTransaction::new()
        .add_pending_airdrop_id(
            tx_record
                .pending_airdrop_records
                .get(1)
                .unwrap()
                .pending_airdrop_id,
        )
        .freeze_with(&client)?
        .sign(treasury_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let carol_balance = AccountBalanceQuery::new()
        .account_id(carol_id)
        .execute(&client)
        .await?;

    println!(
        "Carol nft balance after cancel: {:?}",
        carol_balance.tokens.get(&nft_id)
    );

    /*
     * Step 12:
     * Reject the NFT for bob
     */
    println!("Rejecting nft with Bob");

    _ = TokenRejectTransaction::new()
        .owner(bob_id)
        .add_nft_id(nft_id.nft(2))
        .freeze_with(&client)?
        .sign(private_key_2)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    /*
     * Step 13:
     * Query to verify bob no longer has the NFT
     */
    let bob_balance = AccountBalanceQuery::new()
        .account_id(bob_id)
        .execute(&client)
        .await?;

    println!(
        "Bob nft balance after reject: {}",
        bob_balance.tokens.get(&nft_id).unwrap()
    );

    /*
     * Step 13:
     * Query to verify the NFT was returned to the Treasury
     */
    let treasury_balance = AccountBalanceQuery::new()
        .account_id(treasury_account_id)
        .execute(&client)
        .await?;

    println!(
        "Treasury nft balance after reject: {}",
        treasury_balance.tokens.get(&nft_id).unwrap()
    );

    /*
     * Step 14:
     * Reject the Fungible token for carol
     */
    println!("Rejecting ft with Carol");

    _ = TokenRejectTransaction::new()
        .owner(carol_id)
        .add_token_id(token_id)
        .freeze_with(&client)?
        .sign(private_key_3.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    /*
     * Step 14:
     * Query to verify Carol no longer has the fungible tokens
     */
    let carol_balance = AccountBalanceQuery::new()
        .account_id(carol_id)
        .execute(&client)
        .await?;

    println!(
        "Carol ft balance after reject: {}",
        carol_balance.tokens.get(&token_id).unwrap()
    );

    /*
     * Step 15:
     * Query to verify Treasury received the rejected fungible tokens
     */
    let treasury_balance = AccountBalanceQuery::new()
        .account_id(treasury_account_id)
        .execute(&client)
        .await?;

    println!(
        "Treasury ft balance after reject: {}",
        treasury_balance.tokens.get(&token_id).unwrap()
    );

    println!("Token airdrop example completed successfully");
    Ok(())
}
