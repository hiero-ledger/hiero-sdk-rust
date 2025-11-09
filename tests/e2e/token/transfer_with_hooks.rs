use assert_matches::assert_matches;
use hedera::{
    AccountCreateTransaction,
    ContractCreateTransaction,
    ContractId,
    EvmHookCall,
    EvmHookSpec,
    FungibleHookCall,
    FungibleHookType,
    Hbar,
    HookCall,
    HookCreationDetails,
    HookExtensionPoint,
    LambdaEvmHook,
    NftHookCall,
    NftHookType,
    PrivateKey,
    Status,
    TokenCreateTransaction,
    TokenMintTransaction,
    TokenSupplyType,
    TokenType,
    TransferTransaction,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

const HOOK_BYTECODE: &str = "6080604052348015600e575f5ffd5b506107d18061001c5f395ff3fe608060405260043610610033575f3560e01c8063124d8b301461003757806394112e2f14610067578063bd0dd0b614610097575b5f5ffd5b610051600480360381019061004c91906106f2565b6100c7565b60405161005e9190610782565b60405180910390f35b610081600480360381019061007c91906106f2565b6100d2565b60405161008e9190610782565b60405180910390f35b6100b160048036038101906100ac91906106f2565b6100dd565b6040516100be9190610782565b60405180910390f35b5f6001905092915050565b5f6001905092915050565b5f6001905092915050565b5f604051905090565b5f5ffd5b5f5ffd5b5f5ffd5b5f60a08284031215610112576101116100f9565b5b81905092915050565b5f5ffd5b5f601f19601f8301169050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b6101658261011f565b810181811067ffffffffffffffff821117156101845761018361012f565b5b80604052505050565b5f6101966100e8565b90506101a2828261015c565b919050565b5f5ffd5b5f5ffd5b5f67ffffffffffffffff8211156101c9576101c861012f565b5b602082029050602081019050919050565b5f5ffd5b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f610207826101de565b9050919050565b610217816101fd565b8114610221575f5ffd5b50565b5f813590506102328161020e565b92915050565b5f8160070b9050919050565b61024d81610238565b8114610257575f5ffd5b50565b5f8135905061026881610244565b92915050565b5f604082840312156102835761028261011b565b5b61028d604061018d565b90505f61029c84828501610224565b5f8301525060206102af8482850161025a565b60208301525092915050565b5f6102cd6102c8846101af565b61018d565b905080838252602082019050604084028301858111156102f0576102ef6101da565b5b835b818110156103195780610305888261026e565b8452602084019350506040810190506102f2565b5050509392505050565b5f82601f830112610337576103366101ab565b5b81356103478482602086016102bb565b91505092915050565b5f67ffffffffffffffff82111561036a5761036961012f565b5b602082029050602081019050919050565b5f67ffffffffffffffff8211156103955761039461012f565b5b602082029050602081019050919050565b5f606082840312156103bb576103ba61011b565b5b6103c5606061018d565b90505f6103d484828501610224565b5f8301525060206103e784828501610224565b60208301525060406103fb8482850161025a565b60408301525092915050565b5f6104196104148461037b565b61018d565b9050808382526020820190506060840283018581111561043c5761043b6101da565b5b835b81811015610465578061045188826103a6565b84526020840193505060608101905061043e565b5050509392505050565b5f82601f830112610483576104826101ab565b5b8135610493848260208601610407565b91505092915050565b5f606082840312156104b1576104b061011b565b5b6104bb606061018d565b90505f6104ca84828501610224565b5f83015250602082013567ffffffffffffffff8111156104ed576104ec6101a7565b5b6104f984828501610323565b602083015250604082013567ffffffffffffffff81111561051d5761051c6101a7565b5b6105298482850161046f565b60408301525092915050565b5f61054761054284610350565b61018d565b9050808382526020820190506020840283018581111561056a576105696101da565b5b835b818110156105b157803567ffffffffffffffff81111561058f5761058e6101ab565b5b80860161059c898261049c565b8552602085019450505060208101905061056c565b5050509392505050565b5f82601f8301126105cf576105ce6101ab565b5b81356105df848260208601610535565b91505092915050565b5f604082840312156105fd576105fc61011b565b5b610607604061018d565b90505f82013567ffffffffffffffff811115610626576106256101a7565b5b61063284828501610323565b5f83015250602082013567ffffffffffffffff811115610655576106546101a7565b5b610661848285016105bb565b60208301525092915050565b5f604082840312156106825761068161011b565b5b61068c604061018d565b90505f82013567ffffffffffffffff8111156106ab576106aa6101a7565b5b6106b7848285016105e8565b5f83015250602082013567ffffffffffffffff8111156106da576106d96101a7565b5b6106e6848285016105e8565b60208301525092915050565b5f5f60408385031215610708576107076100f1565b5b5f83013567ffffffffffffffff811115610725576107246100f5565b5b610731858286016100fd565b925050602083013567ffffffffffffffff811115610752576107516100f5565b5b61075e8582860161066d565b9150509250929050565b5f8115159050919050565b61077c81610768565b82525050565b5f6020820190506107955f830184610773565b9291505056fea26469706673582212207dfe7723f6d6869419b1cb0619758b439da0cf4ffd9520997c40a3946299d4dc64736f6c634300081e0033";

async fn create_hook_contract(client: &hedera::Client) -> anyhow::Result<ContractId> {
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

#[tokio::test]
async fn can_transfer_hbar_with_pre_tx_allowance_hook() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let contract_id = create_hook_contract(&client).await?;

    let sender_key = PrivateKey::generate_ed25519();
    let receiver_key = PrivateKey::generate_ed25519();

    // Create sender account with hook
    let spec = EvmHookSpec::new(Some(contract_id));
    let lambda_hook = LambdaEvmHook::new(spec, vec![]);
    let hook_details =
        HookCreationDetails::new(HookExtensionPoint::AccountAllowanceHook, 1, Some(lambda_hook));

    let sender_receipt = AccountCreateTransaction::new()
        .key(sender_key.public_key())
        .initial_balance(Hbar::new(10))
        .add_hook(hook_details)
        .freeze_with(&client)?
        .sign(sender_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let sender_id = sender_receipt.account_id.unwrap();

    // Create receiver account
    let receiver_receipt = AccountCreateTransaction::new()
        .key(receiver_key.public_key())
        .initial_balance(Hbar::new(1))
        .max_automatic_token_associations(-1)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let receiver_id = receiver_receipt.account_id.unwrap();

    // Create hook call for the transfer
    let mut evm_hook_call = EvmHookCall::new(Some(vec![]));
    evm_hook_call.set_gas_limit(25_000);
    let hook_call = HookCall::new(Some(1), Some(evm_hook_call));
    let fungible_hook_call =
        FungibleHookCall { hook_call, hook_type: FungibleHookType::PreTxAllowanceHook };

    // Perform transfer with hook
    let transfer_receipt = TransferTransaction::new()
        .hbar_transfer_with_hook(sender_id, Hbar::from_tinybars(-100), fungible_hook_call)
        .hbar_transfer(receiver_id, Hbar::from_tinybars(100))
        .freeze_with(&client)?
        .sign(sender_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    assert_matches!(transfer_receipt.status, Status::Success);

    Ok(())
}

#[tokio::test]
async fn can_transfer_hbar_with_pre_post_tx_allowance_hook() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let contract_id = create_hook_contract(&client).await?;

    let sender_key = PrivateKey::generate_ed25519();
    let receiver_key = PrivateKey::generate_ed25519();

    // Create sender account with hook
    let spec = EvmHookSpec::new(Some(contract_id));
    let lambda_hook = LambdaEvmHook::new(spec, vec![]);
    let hook_details =
        HookCreationDetails::new(HookExtensionPoint::AccountAllowanceHook, 1, Some(lambda_hook));

    let sender_receipt = AccountCreateTransaction::new()
        .key(sender_key.public_key())
        .initial_balance(Hbar::new(10))
        .add_hook(hook_details)
        .freeze_with(&client)?
        .sign(sender_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let sender_id = sender_receipt.account_id.unwrap();

    // Create receiver account
    let receiver_receipt = AccountCreateTransaction::new()
        .key(receiver_key.public_key())
        .initial_balance(Hbar::new(1))
        .max_automatic_token_associations(-1)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let receiver_id = receiver_receipt.account_id.unwrap();

    // Create hook call for the transfer
    let mut evm_hook_call = EvmHookCall::new(Some(vec![]));
    evm_hook_call.set_gas_limit(25_000);
    let hook_call = HookCall::new(Some(1), Some(evm_hook_call));
    let fungible_hook_call =
        FungibleHookCall { hook_call, hook_type: FungibleHookType::PrePostTxAllowanceHook };

    // Perform transfer with hook
    let transfer_receipt = TransferTransaction::new()
        .hbar_transfer_with_hook(sender_id, Hbar::from_tinybars(-100), fungible_hook_call)
        .hbar_transfer(receiver_id, Hbar::from_tinybars(100))
        .freeze_with(&client)?
        .sign(sender_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    assert_matches!(transfer_receipt.status, Status::Success);

    Ok(())
}

#[tokio::test]
async fn can_transfer_fungible_token_with_hook() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let contract_id = create_hook_contract(&client).await?;

    let treasury_key = PrivateKey::generate_ed25519();
    let sender_key = PrivateKey::generate_ed25519();
    let receiver_key = PrivateKey::generate_ed25519();

    // Create sender account with hook
    let spec = EvmHookSpec::new(Some(contract_id));
    let lambda_hook = LambdaEvmHook::new(spec, vec![]);
    let hook_details =
        HookCreationDetails::new(HookExtensionPoint::AccountAllowanceHook, 1, Some(lambda_hook));

    let sender_receipt = AccountCreateTransaction::new()
        .key(sender_key.public_key())
        .initial_balance(Hbar::new(10))
        .add_hook(hook_details)
        .freeze_with(&client)?
        .sign(sender_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let sender_id = sender_receipt.account_id.unwrap();

    // Create receiver account
    let receiver_receipt = AccountCreateTransaction::new()
        .key(receiver_key.public_key())
        .initial_balance(Hbar::new(1))
        .max_automatic_token_associations(-1)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let receiver_id = receiver_receipt.account_id.unwrap();

    // Create fungible token
    let token_receipt = TokenCreateTransaction::new()
        .name("Test Token")
        .symbol("TEST")
        .decimals(2)
        .initial_supply(1000)
        .treasury_account_id(sender_id)
        .admin_key(treasury_key.public_key())
        .supply_key(treasury_key.public_key())
        .token_type(TokenType::FungibleCommon)
        .token_supply_type(TokenSupplyType::Infinite)
        .freeze_with(&client)?
        .sign(sender_key.clone())
        .sign(treasury_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let token_id = token_receipt.token_id.unwrap();

    // Create hook call for the transfer
    let mut evm_hook_call = EvmHookCall::new(Some(vec![]));
    evm_hook_call.set_gas_limit(25_000);
    let hook_call = HookCall::new(Some(1), Some(evm_hook_call));
    let fungible_hook_call =
        FungibleHookCall { hook_call, hook_type: FungibleHookType::PreTxAllowanceHook };

    // Perform token transfer with hook
    let transfer_receipt = TransferTransaction::new()
        .token_transfer_with_hook(token_id, sender_id, -10, fungible_hook_call)
        .token_transfer(token_id, receiver_id, 10)
        .freeze_with(&client)?
        .sign(sender_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    assert_matches!(transfer_receipt.status, Status::Success);

    Ok(())
}

#[tokio::test]
async fn can_transfer_nft_with_sender_hook() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let contract_id = create_hook_contract(&client).await?;

    let treasury_key = PrivateKey::generate_ed25519();
    let sender_key = PrivateKey::generate_ed25519();
    let receiver_key = PrivateKey::generate_ed25519();

    // Create sender account with hook
    let spec = EvmHookSpec::new(Some(contract_id));
    let lambda_hook = LambdaEvmHook::new(spec, vec![]);
    let hook_details =
        HookCreationDetails::new(HookExtensionPoint::AccountAllowanceHook, 1, Some(lambda_hook));

    let sender_receipt = AccountCreateTransaction::new()
        .key(sender_key.public_key())
        .initial_balance(Hbar::new(10))
        .add_hook(hook_details)
        .freeze_with(&client)?
        .sign(sender_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let sender_id = sender_receipt.account_id.unwrap();

    // Create receiver account
    let receiver_receipt = AccountCreateTransaction::new()
        .key(receiver_key.public_key())
        .initial_balance(Hbar::new(1))
        .max_automatic_token_associations(-1)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let receiver_id = receiver_receipt.account_id.unwrap();

    // Create NFT token
    let token_receipt = TokenCreateTransaction::new()
        .name("Test NFT")
        .symbol("TNFT")
        .treasury_account_id(sender_id)
        .admin_key(treasury_key.public_key())
        .supply_key(treasury_key.public_key())
        .token_type(TokenType::NonFungibleUnique)
        .token_supply_type(TokenSupplyType::Infinite)
        .freeze_with(&client)?
        .sign(sender_key.clone())
        .sign(treasury_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let token_id = token_receipt.token_id.unwrap();

    // Mint NFT
    let mint_receipt = TokenMintTransaction::new()
        .token_id(token_id)
        .metadata(vec![vec![1, 2, 3]])
        .freeze_with(&client)?
        .sign(treasury_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let serial_number = mint_receipt.serials[0] as u64;

    // Create hook call for sender
    let mut evm_hook_call = EvmHookCall::new(Some(vec![]));
    evm_hook_call.set_gas_limit(25_000);
    let hook_call = HookCall::new(Some(1), Some(evm_hook_call));
    let sender_nft_hook_call = NftHookCall { hook_call, hook_type: NftHookType::PreHookSender };

    // Perform NFT transfer with sender hook
    let transfer_receipt = TransferTransaction::new()
        .nft_transfer_with_sender_hook(
            token_id.nft(serial_number),
            sender_id,
            receiver_id,
            sender_nft_hook_call,
        )
        .freeze_with(&client)?
        .sign(sender_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    assert_matches!(transfer_receipt.status, Status::Success);

    Ok(())
}

#[tokio::test]
async fn can_transfer_nft_with_receiver_hook() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let contract_id = create_hook_contract(&client).await?;

    let treasury_key = PrivateKey::generate_ed25519();
    let sender_key = PrivateKey::generate_ed25519();
    let receiver_key = PrivateKey::generate_ed25519();

    // Create sender account
    let sender_receipt = AccountCreateTransaction::new()
        .key(sender_key.public_key())
        .initial_balance(Hbar::new(10))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let sender_id = sender_receipt.account_id.unwrap();

    // Create receiver account with hook
    let spec = EvmHookSpec::new(Some(contract_id));
    let lambda_hook = LambdaEvmHook::new(spec, vec![]);
    let hook_details =
        HookCreationDetails::new(HookExtensionPoint::AccountAllowanceHook, 1, Some(lambda_hook));

    let receiver_receipt = AccountCreateTransaction::new()
        .key(receiver_key.public_key())
        .initial_balance(Hbar::new(1))
        .max_automatic_token_associations(-1)
        .add_hook(hook_details)
        .freeze_with(&client)?
        .sign(receiver_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let receiver_id = receiver_receipt.account_id.unwrap();

    // Create NFT token
    let token_receipt = TokenCreateTransaction::new()
        .name("Test NFT")
        .symbol("TNFT")
        .treasury_account_id(sender_id)
        .admin_key(treasury_key.public_key())
        .supply_key(treasury_key.public_key())
        .token_type(TokenType::NonFungibleUnique)
        .token_supply_type(TokenSupplyType::Infinite)
        .freeze_with(&client)?
        .sign(sender_key.clone())
        .sign(treasury_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let token_id = token_receipt.token_id.unwrap();

    // Mint NFT
    let mint_receipt = TokenMintTransaction::new()
        .token_id(token_id)
        .metadata(vec![vec![1, 2, 3]])
        .freeze_with(&client)?
        .sign(treasury_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let serial_number = mint_receipt.serials[0] as u64;

    // Create hook call for receiver
    let mut evm_hook_call = EvmHookCall::new(Some(vec![]));
    evm_hook_call.set_gas_limit(25_000);
    let hook_call = HookCall::new(Some(1), Some(evm_hook_call));
    let receiver_nft_hook_call = NftHookCall { hook_call, hook_type: NftHookType::PreHookReceiver };

    // Perform NFT transfer with receiver hook
    let transfer_receipt = TransferTransaction::new()
        .nft_transfer_with_receiver_hook(
            token_id.nft(serial_number),
            sender_id,
            receiver_id,
            receiver_nft_hook_call,
        )
        .freeze_with(&client)?
        .sign(sender_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    assert_matches!(transfer_receipt.status, Status::Success);

    Ok(())
}

#[tokio::test]
async fn can_transfer_nft_with_both_sender_and_receiver_hooks() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let contract_id = create_hook_contract(&client).await?;

    let treasury_key = PrivateKey::generate_ed25519();
    let sender_key = PrivateKey::generate_ed25519();
    let receiver_key = PrivateKey::generate_ed25519();

    let spec = EvmHookSpec::new(Some(contract_id));
    let lambda_hook = LambdaEvmHook::new(spec, vec![]);
    let hook_details =
        HookCreationDetails::new(HookExtensionPoint::AccountAllowanceHook, 1, Some(lambda_hook));

    // Create sender account with hook
    let sender_receipt = AccountCreateTransaction::new()
        .key(sender_key.public_key())
        .initial_balance(Hbar::new(10))
        .add_hook(hook_details.clone())
        .freeze_with(&client)?
        .sign(sender_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let sender_id = sender_receipt.account_id.unwrap();

    // Create receiver account with hook
    let receiver_receipt = AccountCreateTransaction::new()
        .key(receiver_key.public_key())
        .initial_balance(Hbar::new(1))
        .max_automatic_token_associations(-1)
        .add_hook(hook_details)
        .freeze_with(&client)?
        .sign(receiver_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let receiver_id = receiver_receipt.account_id.unwrap();

    // Create NFT token
    let token_receipt = TokenCreateTransaction::new()
        .name("Test NFT")
        .symbol("TNFT")
        .treasury_account_id(sender_id)
        .admin_key(treasury_key.public_key())
        .supply_key(treasury_key.public_key())
        .token_type(TokenType::NonFungibleUnique)
        .token_supply_type(TokenSupplyType::Infinite)
        .freeze_with(&client)?
        .sign(sender_key.clone())
        .sign(treasury_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let token_id = token_receipt.token_id.unwrap();

    // Mint NFT
    let mint_receipt = TokenMintTransaction::new()
        .token_id(token_id)
        .metadata(vec![vec![1, 2, 3]])
        .freeze_with(&client)?
        .sign(treasury_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let serial_number = mint_receipt.serials[0] as u64;

    // Create hook calls
    let mut sender_evm_hook_call = EvmHookCall::new(Some(vec![]));
    sender_evm_hook_call.set_gas_limit(25_000);
    let sender_hook_call = HookCall::new(Some(1), Some(sender_evm_hook_call));
    let sender_nft_hook_call =
        NftHookCall { hook_call: sender_hook_call, hook_type: NftHookType::PrePostHookSender };

    let mut receiver_evm_hook_call = EvmHookCall::new(Some(vec![]));
    receiver_evm_hook_call.set_gas_limit(25_000);
    let receiver_hook_call = HookCall::new(Some(1), Some(receiver_evm_hook_call));
    let receiver_nft_hook_call =
        NftHookCall { hook_call: receiver_hook_call, hook_type: NftHookType::PrePostHookReceiver };

    // Perform NFT transfer with both hooks
    let transfer_receipt = TransferTransaction::new()
        .nft_transfer_with_both_hooks(
            token_id.nft(serial_number),
            sender_id,
            receiver_id,
            sender_nft_hook_call,
            receiver_nft_hook_call,
        )
        .freeze_with(&client)?
        .sign(sender_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    assert_matches!(transfer_receipt.status, Status::Success);

    Ok(())
}
