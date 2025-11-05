use assert_matches::assert_matches;
use hedera::{
    AccountCreateTransaction,
    AccountUpdateTransaction,
    ContractCreateTransaction,
    ContractId,
    EvmHookSpec,
    Hbar,
    HookCreationDetails,
    HookExtensionPoint,
    LambdaEvmHook,
    LambdaStorageSlot,
    LambdaStorageUpdate,
    PrivateKey,
    Status,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

const HOOK_BYTECODE: &str = "6080604052348015600e575f5ffd5b506103da8061001c5f395ff3fe60806040526004361061001d575f3560e01c80630b6c5c0414610021575b5f5ffd5b61003b6004803603810190610036919061021c565b610051565b60405161004891906102ed565b60405180910390f35b5f61016d73ffffffffffffffffffffffffffffffffffffffff163073ffffffffffffffffffffffffffffffffffffffff16146100c2576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016100b990610386565b60405180910390fd5b60019050979650505050505050565b5f5ffd5b5f5ffd5b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f610102826100d9565b9050919050565b610112816100f8565b811461011c575f5ffd5b50565b5f8135905061012d81610109565b92915050565b5f819050919050565b61014581610133565b811461014f575f5ffd5b50565b5f813590506101608161013c565b92915050565b5f5ffd5b5f5ffd5b5f5ffd5b5f5f83601f84011261018757610186610166565b5b8235905067ffffffffffffffff8111156101a4576101a361016a565b5b6020830191508360018202830111156101c0576101bf61016e565b5b9250929050565b5f5f83601f8401126101dc576101db610166565b5b8235905067ffffffffffffffff8111156101f9576101f861016a565b5b6020830191508360018202830111156102155761021461016e565b5b9250929050565b5f5f5f5f5f5f5f60a0888a031215610237576102366100d1565b5b5f6102448a828b0161011f565b97505060206102558a828b01610152565b96505060406102668a828b01610152565b955050606088013567ffffffffffffffff811115610287576102866100d5565b5b6102938a828b01610172565b9450945050608088013567ffffffffffffffff8111156102b6576102b56100d5565b5b6102c28a828b016101c7565b925092505092959891949750929550565b5f8115159050919050565b6102e7816102d3565b82525050565b5f6020820190506103005f8301846102de565b92915050565b5f82825260208201905092915050565b7f436f6e74726163742063616e206f6e6c792062652063616c6c656420617320615f8201527f20686f6f6b000000000000000000000000000000000000000000000000000000602082015250565b5f610370602583610306565b915061037b82610316565b604082019050919050565b5f6020820190508181035f83015261039d81610364565b905091905056fea2646970667358221220a8c76458204f8bb9a86f59ec2f0ccb7cbe8ae4dcb65700c4b6ee91a39404083a64736f6c634300081e0033";

async fn create_hook_contract(client: &hedera::Client) -> anyhow::Result<ContractId> {
    let bytecode = hex::decode(HOOK_BYTECODE)?;

    let receipt = ContractCreateTransaction::new()
        .bytecode(bytecode)
        .gas(300_000)
        .execute(client)
        .await?
        .get_receipt(client)
        .await?;

    Ok(receipt.contract_id.unwrap())
}

#[tokio::test]
async fn can_update_account_to_add_lambda_hook() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let contract_id = create_hook_contract(&client).await?;

    let key = PrivateKey::generate_ed25519();

    // First create an account without hooks
    let create_receipt = AccountCreateTransaction::new()
        .key(key.public_key())
        .initial_balance(Hbar::new(1))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let account_id = create_receipt.account_id.unwrap();

    // Now update it to add a hook
    let spec = EvmHookSpec::new(Some(contract_id));
    let lambda_hook = LambdaEvmHook::new(spec, vec![]);

    let hook_details =
        HookCreationDetails::new(HookExtensionPoint::AccountAllowanceHook, 1, Some(lambda_hook));

    let update_receipt = AccountUpdateTransaction::new()
        .account_id(account_id)
        .add_hook(hook_details)
        .freeze_with(&client)?
        .sign(key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    assert_matches!(update_receipt.status, Status::Success);

    Ok(())
}

#[tokio::test]
async fn can_update_account_with_multiple_hooks() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let contract_id = create_hook_contract(&client).await?;

    let key = PrivateKey::generate_ed25519();

    // Create account
    let create_receipt = AccountCreateTransaction::new()
        .key(key.public_key())
        .initial_balance(Hbar::new(1))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let account_id = create_receipt.account_id.unwrap();

    // Add multiple hooks
    let spec = EvmHookSpec::new(Some(contract_id));
    let lambda_hook1 = LambdaEvmHook::new(spec.clone(), vec![]);
    let lambda_hook2 = LambdaEvmHook::new(spec, vec![]);

    let hook_details1 =
        HookCreationDetails::new(HookExtensionPoint::AccountAllowanceHook, 1, Some(lambda_hook1));

    let hook_details2 =
        HookCreationDetails::new(HookExtensionPoint::AccountAllowanceHook, 2, Some(lambda_hook2));

    let update_receipt = AccountUpdateTransaction::new()
        .account_id(account_id)
        .add_hook(hook_details1)
        .add_hook(hook_details2)
        .freeze_with(&client)?
        .sign(key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    assert_matches!(update_receipt.status, Status::Success);

    Ok(())
}

#[tokio::test]
async fn can_update_account_hook_with_storage_updates() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let contract_id = create_hook_contract(&client).await?;

    let key = PrivateKey::generate_ed25519();

    // Create account
    let create_receipt = AccountCreateTransaction::new()
        .key(key.public_key())
        .initial_balance(Hbar::new(1))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let account_id = create_receipt.account_id.unwrap();

    // Add hook with storage updates
    let storage_slot =
        LambdaStorageSlot::new(vec![0x01, 0x02, 0x03, 0x04], vec![0xFF, 0xEE, 0xDD, 0xCC]);
    let storage_update = LambdaStorageUpdate::StorageSlot(storage_slot);

    let spec = EvmHookSpec::new(Some(contract_id));
    let lambda_hook = LambdaEvmHook::new(spec, vec![storage_update]);

    let hook_details =
        HookCreationDetails::new(HookExtensionPoint::AccountAllowanceHook, 1, Some(lambda_hook));

    let update_receipt = AccountUpdateTransaction::new()
        .account_id(account_id)
        .add_hook(hook_details)
        .freeze_with(&client)?
        .sign(key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    assert_matches!(update_receipt.status, Status::Success);

    Ok(())
}

#[tokio::test]
async fn cannot_update_account_with_duplicate_hook_ids() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let contract_id = create_hook_contract(&client).await?;

    let key = PrivateKey::generate_ed25519();

    // Create account
    let create_receipt = AccountCreateTransaction::new()
        .key(key.public_key())
        .initial_balance(Hbar::new(1))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let account_id = create_receipt.account_id.unwrap();

    // Try to add hooks with duplicate IDs
    let spec = EvmHookSpec::new(Some(contract_id));
    let lambda_hook1 = LambdaEvmHook::new(spec.clone(), vec![]);
    let lambda_hook2 = LambdaEvmHook::new(spec, vec![]);

    let hook_details1 =
        HookCreationDetails::new(HookExtensionPoint::AccountAllowanceHook, 1, Some(lambda_hook1));

    let hook_details2 = HookCreationDetails::new(
        HookExtensionPoint::AccountAllowanceHook,
        1, // Same ID - should fail
        Some(lambda_hook2),
    );

    let result = AccountUpdateTransaction::new()
        .account_id(account_id)
        .add_hook(hook_details1)
        .add_hook(hook_details2)
        .freeze_with(&client)?
        .sign(key)
        .execute(&client)
        .await;

    assert!(result.is_err());
    if let Err(err) = result {
        let err_str = err.to_string();
        assert!(
            err_str.contains("HOOK_ID_REPEATED_IN_CREATION_DETAILS")
                || err_str.contains("HookIdRepeatedInCreationDetails"),
            "Expected HOOK_ID_REPEATED_IN_CREATION_DETAILS error, got: {}",
            err_str
        );
    }

    Ok(())
}

#[tokio::test]
async fn can_update_account_hook_with_admin_key() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let contract_id = create_hook_contract(&client).await?;

    let account_key = PrivateKey::generate_ed25519();
    let hook_admin_key = PrivateKey::generate_ed25519();

    // Create account
    let create_receipt = AccountCreateTransaction::new()
        .key(account_key.public_key())
        .initial_balance(Hbar::new(1))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let account_id = create_receipt.account_id.unwrap();

    // Add hook with admin key
    let spec = EvmHookSpec::new(Some(contract_id));
    let lambda_hook = LambdaEvmHook::new(spec, vec![]);

    let mut hook_details =
        HookCreationDetails::new(HookExtensionPoint::AccountAllowanceHook, 1, Some(lambda_hook));
    hook_details.admin_key = Some(hook_admin_key.public_key().into());

    let update_receipt = AccountUpdateTransaction::new()
        .account_id(account_id)
        .add_hook(hook_details)
        .freeze_with(&client)?
        .sign(account_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    assert_matches!(update_receipt.status, Status::Success);

    Ok(())
}
