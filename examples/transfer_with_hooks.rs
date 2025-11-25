// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::Path;

use clap::Parser;
use hedera::{
    AccountCreateTransaction, AccountId, Client, ContractCreateTransaction, ContractId, EvmHookCall, EvmHookSpec, FungibleHookCall, FungibleHookType, Hbar, HookCall, HookCreationDetails, HookExtensionPoint, LambdaEvmHook, NftHookCall, NftHookType, PrivateKey, TokenCreateTransaction, TokenMintTransaction, TokenType, TransferTransaction
};

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, env, default_value = "0.0.2")]
    operator_account_id: AccountId,

    #[clap(
        long,
        env,
        default_value = "302e020100300506032b65700422042091132178e72057a1d7528025956fe39b0b847f200ab59b2fdd367017f3087137"
    )]
    operator_key: PrivateKey,

    #[clap(long, env, default_value = "testnet")]
    hedera_network: String,
}

const HOOK_BYTECODE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/transfer_hook.hex");

fn load_hook_bytecode() -> anyhow::Result<Vec<u8>> {
    let bytecode_hex = fs::read_to_string(Path::new(HOOK_BYTECODE_PATH))?;
    let cleaned_hex: String = bytecode_hex.split_whitespace().collect();
    let bytecode = hex::decode(cleaned_hex)?;
    Ok(bytecode)
}

async fn create_hook_contract(
    client: &Client,
    operator_key: &PrivateKey,
) -> anyhow::Result<ContractId> {
    let bytecode = load_hook_bytecode()?;

    let receipt = ContractCreateTransaction::new()
        .admin_key(operator_key.public_key())
        .gas(1_000_000)
        .bytecode(bytecode)
        .freeze_with(client)?
        .sign(operator_key.clone())
        .execute(client)
        .await?
        .get_receipt(client)
        .await?;

    Ok(receipt.contract_id.unwrap())
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

    println!("Transfer Transaction Hooks Example Start!");

    /*
     * Step 1: Set up prerequisites - create tokens and NFTs
     */
    println!("Setting up prerequisites...");

    let hook_contract_id = create_hook_contract(&client, &operator_key).await?;
    println!("Hook contract created with ID: {hook_contract_id}");

    let hook_id = 1;
    let spec = EvmHookSpec::new(Some(hook_contract_id));
    let lambda_hook = LambdaEvmHook::new(spec, vec![]);
    let hook_details = HookCreationDetails::new(
        HookExtensionPoint::AccountAllowanceHook,
        hook_id,
        Some(lambda_hook),
    );

    let sender_key = PrivateKey::generate_ecdsa();
    let sender_receipt = AccountCreateTransaction::new()
        .set_key_without_alias(sender_key.public_key())
        .initial_balance(Hbar::new(10))
        .add_hook(hook_details.clone())
        .freeze_with(&client)?
        .sign(sender_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let sender_account_id = sender_receipt.account_id.unwrap();

    let receiver_receipt = AccountCreateTransaction::new()
        .set_key_without_alias(PrivateKey::generate_ecdsa().public_key())
        .max_automatic_token_associations(-1)
        .initial_balance(Hbar::new(10))
        .add_hook(hook_details)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let receiver_account_id = receiver_receipt.account_id.unwrap();

    println!("Creating fungible token...");
    let fungible_token_receipt = TokenCreateTransaction::new()
        .name("Example Fungible Token")
        .symbol("EFT")
        .token_type(TokenType::FungibleCommon)
        .decimals(2)
        .initial_supply(10_000)
        .treasury_account_id(sender_account_id)
        .admin_key(sender_key.public_key())
        .supply_key(sender_key.public_key())
        .freeze_with(&client)?
        .sign(sender_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let fungible_token_id = fungible_token_receipt.token_id.unwrap();
    println!("Created fungible token with ID: {fungible_token_id}");

    println!("Creating NFT token...");
    let nft_token_receipt = TokenCreateTransaction::new()
        .name("Example NFT Token")
        .symbol("ENT")
        .token_type(TokenType::NonFungibleUnique)
        .treasury_account_id(sender_account_id)
        .admin_key(sender_key.public_key())
        .supply_key(sender_key.public_key())
        .freeze_with(&client)?
        .sign(sender_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let nft_token_id = nft_token_receipt.token_id.unwrap();
    println!("Created NFT token with ID: {nft_token_id}");

    println!("Minting NFT...");
    let metadata = b"Example NFT Metadata".to_vec();
    let mint_receipt = TokenMintTransaction::new()
        .token_id(nft_token_id)
        .metadata(vec![metadata])
        .freeze_with(&client)?
        .sign(sender_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let nft_id = nft_token_id.nft(mint_receipt.serials[0] as u64);
    println!("Minted NFT with ID: {nft_id}");

    /*
     * Step 2: Demonstrate TransferTransaction API with hooks (demonstration only)
     */
    println!("\n=== TransferTransaction with Hooks API Demonstration ===");

    // Create different hooks for different transfer types (for demonstration)
    println!("Creating hook call objects (demonstration)...");

    // HBAR transfer with pre-tx allowance hook
    let hbar_hook = FungibleHookCall {
        hook_call: HookCall::new(Some(hook_id), {
            let mut evm_call = EvmHookCall::new(Some(vec![0x01, 0x02]));
            evm_call.set_gas_limit(20_000);
            Some(evm_call)
        }),
        hook_type: FungibleHookType::PreTxAllowanceHook,
    };

    // NFT sender hook (pre-hook)
    let nft_sender_hook = NftHookCall {
        hook_call: HookCall::new(Some(hook_id), {
            let mut evm_call = EvmHookCall::new(Some(vec![0x03, 0x04]));
            evm_call.set_gas_limit(20_000);
            Some(evm_call)
        }),
        hook_type: NftHookType::PreHookSender,
    };

    // NFT receiver hook (pre-hook)
    let nft_receiver_hook = NftHookCall {
        hook_call: HookCall::new(Some(hook_id), {
            let mut evm_call = EvmHookCall::new(Some(vec![0x05, 0x06]));
            evm_call.set_gas_limit(20_000);
            Some(evm_call)
        }),
        hook_type: NftHookType::PreHookReceiver,
    };

    // Fungible token transfer with pre-post allowance hook
    let fungible_token_hook = FungibleHookCall {
        hook_call: HookCall::new(Some(hook_id), {
            let mut evm_call = EvmHookCall::new(Some(vec![0x07, 0x08]));
            evm_call.set_gas_limit(20_000);
            Some(evm_call)
        }),
        hook_type: FungibleHookType::PrePostTxAllowanceHook,
    };

    // Build separate TransferTransactions with hooks (demonstration)
    println!("Building separate TransferTransactions with hooks...");

    // Transaction 1: HBAR transfers with hook
    println!("\n1. Building HBAR TransferTransaction with hook...");
    TransferTransaction::new()
        .add_hbar_transfer_with_hook(sender_account_id, Hbar::from_tinybars(-1), hbar_hook)
        .hbar_transfer(receiver_account_id, Hbar::from_tinybars(1))
        .freeze_with(&client)?
        .sign(sender_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Transaction 2: NFT transfer with sender and receiver hooks
    println!("\n2. Building NFT TransferTransaction with hooks...");
    TransferTransaction::new()
        .add_nft_transfer_with_hook(
            nft_id,
            sender_account_id,
            receiver_account_id,
            nft_sender_hook,
            nft_receiver_hook,
        )
        .freeze_with(&client)?
        .sign(sender_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Transaction 3: Fungible token transfers with hook
    println!("\n3. Building Fungible Token TransferTransaction with hook...");
    TransferTransaction::new()
        .add_token_transfer_with_hook(
            fungible_token_id,
            sender_account_id,
            -1_000,
            fungible_token_hook,
        )
        .token_transfer(fungible_token_id, receiver_account_id, 1_000)
        .freeze_with(&client)?
        .sign(sender_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    println!("\nAll TransferTransactions executed successfully with the following hook calls:");
    println!("  - Transaction 1: HBAR transfer with pre-tx allowance hook");
    println!("  - Transaction 2: NFT transfer with sender and receiver hooks");
    println!("  - Transaction 3: Fungible token transfer with pre-post allowance hook");

    println!("Transfer Transaction Hooks Example Complete!");

    Ok(())
}
