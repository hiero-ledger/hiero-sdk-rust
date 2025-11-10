// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hedera::{
    AccountCreateTransaction, AccountId, Client, ContractCreateTransaction, ContractId, EvmHookCall, EvmHookSpec, FungibleHookCall, FungibleHookType, Hbar, HookCall, HookCreationDetails, HookExtensionPoint, LambdaEvmHook, NftHookCall, NftHookType, PrivateKey, TokenCreateTransaction, TokenMintTransaction, TokenSupplyType, TokenType, TransferTransaction
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

const HOOK_BYTECODE: &str = "6080604052348015600e575f5ffd5b506107d18061001c5f395ff3fe608060405260043610610033575f3560e01c8063124d8b301461003757806394112e2f14610067578063bd0dd0b614610097575b5f5ffd5b610051600480360381019061004c91906106f2565b6100c7565b60405161005e9190610782565b60405180910390f35b610081600480360381019061007c91906106f2565b6100d2565b60405161008e9190610782565b60405180910390f35b6100b160048036038101906100ac91906106f2565b6100dd565b6040516100be9190610782565b60405180910390f35b5f6001905092915050565b5f6001905092915050565b5f6001905092915050565b5f604051905090565b5f5ffd5b5f5ffd5b5f5ffd5b5f60a08284031215610112576101116100f9565b5b81905092915050565b5f5ffd5b5f601f19601f8301169050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b6101658261011f565b810181811067ffffffffffffffff821117156101845761018361012f565b5b80604052505050565b5f6101966100e8565b90506101a2828261015c565b919050565b5f5ffd5b5f5ffd5b5f67ffffffffffffffff8211156101c9576101c861012f565b5b602082029050602081019050919050565b5f5ffd5b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f610207826101de565b9050919050565b610217816101fd565b8114610221575f5ffd5b50565b5f813590506102328161020e565b92915050565b5f8160070b9050919050565b61024d81610238565b8114610257575f5ffd5b50565b5f8135905061026881610244565b92915050565b5f604082840312156102835761028261011b565b5b61028d604061018d565b90505f61029c84828501610224565b5f8301525060206102af8482850161025a565b60208301525092915050565b5f6102cd6102c8846101af565b61018d565b905080838252602082019050604084028301858111156102f0576102ef6101da565b5b835b818110156103195780610305888261026e565b8452602084019350506040810190506102f2565b5050509392505050565b5f82601f830112610337576103366101ab565b5b81356103478482602086016102bb565b91505092915050565b5f67ffffffffffffffff82111561036a5761036961012f565b5b602082029050602081019050919050565b5f67ffffffffffffffff8211156103955761039461012f565b5b602082029050602081019050919050565b5f606082840312156103bb576103ba61011b565b5b6103c5606061018d565b90505f6103d484828501610224565b5f8301525060206103e784828501610224565b60208301525060406103fb8482850161025a565b60408301525092915050565b5f6104196104148461037b565b61018d565b9050808382526020820190506060840283018581111561043c5761043b6101da565b5b835b81811015610465578061045188826103a6565b84526020840193505060608101905061043e565b5050509392505050565b5f82601f830112610483576104826101ab565b5b8135610493848260208601610407565b91505092915050565b5f606082840312156104b1576104b061011b565b5b6104bb606061018d565b90505f6104ca84828501610224565b5f83015250602082013567ffffffffffffffff8111156104ed576104ec6101a7565b5b6104f984828501610323565b602083015250604082013567ffffffffffffffff81111561051d5761051c6101a7565b5b6105298482850161046f565b60408301525092915050565b5f61054761054284610350565b61018d565b9050808382526020820190506020840283018581111561056a576105696101da565b5b835b818110156105b157803567ffffffffffffffff81111561058f5761058e6101ab565b5b80860161059c898261049c565b8552602085019450505060208101905061056c565b5050509392505050565b5f82601f8301126105cf576105ce6101ab565b5b81356105df848260208601610535565b91505092915050565b5f604082840312156105fd576105fc61011b565b5b610607604061018d565b90505f82013567ffffffffffffffff811115610626576106256101a7565b5b61063284828501610323565b5f83015250602082013567ffffffffffffffff811115610655576106546101a7565b5b610661848285016105bb565b60208301525092915050565b5f604082840312156106825761068161011b565b5b61068c604061018d565b90505f82013567ffffffffffffffff8111156106ab576106aa6101a7565b5b6106b7848285016105e8565b5f83015250602082013567ffffffffffffffff8111156106da576106d96101a7565b5b6106e6848285016105e8565b60208301525092915050565b5f5f60408385031215610708576107076100f1565b5b5f83013567ffffffffffffffff811115610725576107246100f5565b5b610731858286016100fd565b925050602083013567ffffffffffffffff811115610752576107516100f5565b5b61075e8582860161066d565b9150509250929050565b5f8115159050919050565b61077c81610768565b82525050565b5f6020820190506107955f830184610773565b9291505056fea26469706673582212207dfe7723f6d6869419b1cb0619758b439da0cf4ffd9520997c40a3946299d4dc64736f6c634300081e0033";

async fn create_hook_contract(client: &Client) -> anyhow::Result<ContractId> {
    let bytecode = hex::decode(HOOK_BYTECODE)?;

    let receipt = ContractCreateTransaction::new()
        .bytecode(bytecode)
        .gas(1_700_000)
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
    client.set_operator(operator_account_id, operator_key);

    println!("Transfer Transaction Hooks Example Start!");

    // Step 1: Set up prerequisites - create hook contract
    println!("Setting up prerequisites...");

    let hook_contract_id = create_hook_contract(&client).await?;
    println!("Created hook contract: {hook_contract_id}");

    // Create hook details
    let hook_id = 1;
    let spec = EvmHookSpec::new(Some(hook_contract_id));
    let lambda_hook = LambdaEvmHook::new(spec, vec![]);
    let hook_details = HookCreationDetails::new(
        HookExtensionPoint::AccountAllowanceHook,
        hook_id,
        Some(lambda_hook),
    );

    // Create sender account with hook
    let sender_key = PrivateKey::generate_ed25519();
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
    println!("Created sender account: {sender_account_id}");

    // Create receiver account with hook and unlimited token associations
    let receiver_key = PrivateKey::generate_ed25519();
    let receiver_receipt = AccountCreateTransaction::new()
        .set_key_without_alias(receiver_key.public_key())
        .initial_balance(Hbar::new(10))
        .max_automatic_token_associations(-1)
        .add_hook(hook_details)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let receiver_account_id = receiver_receipt.account_id.unwrap();
    println!("Created receiver account: {receiver_account_id}");

    // Create fungible token
    println!("Creating fungible token...");
    let fungible_token_id = TokenCreateTransaction::new()
        .name("Example Fungible Token")
        .symbol("EFT")
        .decimals(2)
        .initial_supply(10_000)
        .treasury_account_id(sender_account_id)
        .admin_key(sender_key.public_key())
        .supply_key(sender_key.public_key())
        .token_type(TokenType::FungibleCommon)
        .token_supply_type(TokenSupplyType::Infinite)
        .freeze_with(&client)?
        .sign(sender_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    println!("Created fungible token: {fungible_token_id}");

    // Create NFT token
    println!("Creating NFT token...");
    let nft_token_id = TokenCreateTransaction::new()
        .name("Example NFT Token")
        .symbol("ENT")
        .treasury_account_id(sender_account_id)
        .admin_key(sender_key.public_key())
        .supply_key(sender_key.public_key())
        .token_type(TokenType::NonFungibleUnique)
        .token_supply_type(TokenSupplyType::Infinite)
        .freeze_with(&client)?
        .sign(sender_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    println!("Created NFT token: {nft_token_id}");

    // Mint NFT
    println!("Minting NFT...");
    let nft_serial = TokenMintTransaction::new()
        .token_id(nft_token_id)
        .metadata(vec![b"Example NFT Metadata".to_vec()])
        .freeze_with(&client)?
        .sign(sender_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .serials[0] as u64;

    let nft_id = nft_token_id.nft(nft_serial);
    println!("Minted NFT: {nft_id}");

    // Step 2: Demonstrate TransferTransaction API with hooks
    println!("\n=== TransferTransaction with Hooks API Demonstration ===");

    // Create hook call objects
    println!("Creating hook call objects...");

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

    // Build separate TransferTransactions with hooks
    println!("Building separate TransferTransactions with hooks...");

    // Transaction 1: HBAR transfers with hook
    println!("\n1. Executing HBAR TransferTransaction with hook...");
    TransferTransaction::new()
        .add_hbar_transfer_with_hook(sender_account_id, Hbar::from_tinybars(-1), hbar_hook)
        .hbar_transfer(receiver_account_id, Hbar::from_tinybars(1))
        .freeze_with(&client)?
        .sign(sender_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;
    println!("   ✓ HBAR transfer with pre-tx allowance hook completed");

    // Transaction 2: NFT transfer with sender and receiver hooks
    println!("\n2. Executing NFT TransferTransaction with hooks...");
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
    println!("   ✓ NFT transfer with sender and receiver hooks completed");

    // Transaction 3: Fungible token transfers with hook
    println!("\n3. Executing Fungible Token TransferTransaction with hook...");
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
    println!("   ✓ Fungible token transfer with pre-post allowance hook completed");

    println!("\nAll TransferTransactions executed successfully with the following hook calls:");
    println!("  - Transaction 1: HBAR transfer with pre-tx allowance hook");
    println!("  - Transaction 2: NFT transfer with sender and receiver hooks");
    println!("  - Transaction 3: Fungible token transfer with pre-post allowance hook");

    println!("Transfer Transaction Hooks Example Complete!");

    Ok(())
}
