use assert_matches::assert_matches;
use hedera::{
    AccountCreateTransaction,
    ContractCreateTransaction,
    ContractId,
    EvmHookSpec,
    Hbar,
    HookCreationDetails,
    HookEntityId,
    HookExtensionPoint,
    HookId,
    LambdaEvmHook,
    LambdaSStoreTransaction,
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

async fn create_account_with_hook(
    client: &hedera::Client,
    contract_id: ContractId,
) -> anyhow::Result<(hedera::AccountId, PrivateKey, HookId)> {
    let account_key = PrivateKey::generate_ed25519();

    // Create initial storage slot (use minimal representation - no leading zeros)
    let storage_slot = LambdaStorageSlot::new(vec![0x01, 0x02, 0x03, 0x04], vec![0x05, 0x06, 0x07, 0x08]);

    // Create lambda hook with storage
    let spec = EvmHookSpec::new(Some(contract_id));
    let lambda_hook = LambdaEvmHook::new(spec, vec![LambdaStorageUpdate::StorageSlot(storage_slot)]);

    let hook_details =
        HookCreationDetails::new(HookExtensionPoint::AccountAllowanceHook, 1, Some(lambda_hook));

    // Create account with the hook
    let receipt = AccountCreateTransaction::new()
        .key(account_key.public_key())
        .initial_balance(Hbar::new(1))
        .add_hook(hook_details)
        .freeze_with(client)?
        .sign(account_key.clone())
        .execute(client)
        .await?
        .get_receipt(client)
        .await?;

    let account_id = receipt.account_id.unwrap();

    // Create hook ID
    let entity_id = HookEntityId::new(Some(account_id));
    let hook_id = HookId::new(Some(entity_id), 1);

    Ok((account_id, account_key, hook_id))
}

#[tokio::test]
async fn can_update_storage_slots_with_valid_signatures() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let contract_id = create_hook_contract(&client).await?;
    let (_account_id, account_key, hook_id) = create_account_with_hook(&client, contract_id).await?;

    // Create new storage update (use minimal representation - no leading zeros)
    let new_storage_slot = LambdaStorageSlot::new(vec![0x09, 0x0a, 0x0b, 0x0c], vec![0x0d, 0x0e, 0x0f, 0x10]);
    let storage_update = LambdaStorageUpdate::StorageSlot(new_storage_slot);

    // Update storage slots
    let receipt = LambdaSStoreTransaction::new()
        .hook_id(hook_id)
        .add_storage_update(storage_update)
        .freeze_with(&client)?
        .sign(account_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    assert_matches!(receipt.status, Status::Success);

    Ok(())
}

#[tokio::test]
async fn cannot_update_more_than_256_storage_slots() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let contract_id = create_hook_contract(&client).await?;
    let (_account_id, account_key, hook_id) = create_account_with_hook(&client, contract_id).await?;

    // Create 257 storage slots (exceeds limit)
    let storage_slot = LambdaStorageSlot::new(vec![0x01, 0x02, 0x03, 0x04], vec![0x05, 0x06, 0x07, 0x08]);
    let storage_updates: Vec<LambdaStorageUpdate> =
        (0..257).map(|_| LambdaStorageUpdate::StorageSlot(storage_slot.clone())).collect();

    let result = LambdaSStoreTransaction::new()
        .hook_id(hook_id)
        .storage_updates(storage_updates)
        .freeze_with(&client)?
        .sign(account_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert!(result.is_err());
    if let Err(err) = result {
        let err_str = err.to_string();
        assert!(
            err_str.contains("TOO_MANY_LAMBDA_STORAGE_UPDATES")
                || err_str.contains("TooManyLambdaStorageUpdates"),
            "Expected TOO_MANY_LAMBDA_STORAGE_UPDATES error, got: {}",
            err_str
        );
    }

    Ok(())
}

#[tokio::test]
async fn cannot_update_storage_with_invalid_signature() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let contract_id = create_hook_contract(&client).await?;
    let (_account_id, _account_key, hook_id) = create_account_with_hook(&client, contract_id).await?;

    // Use wrong key
    let invalid_key = PrivateKey::generate_ed25519();

    let storage_slot = LambdaStorageSlot::new(vec![0x31, 0x32, 0x33, 0x34], vec![0x35, 0x36, 0x37, 0x38]);
    let storage_update = LambdaStorageUpdate::StorageSlot(storage_slot);

    let result = LambdaSStoreTransaction::new()
        .hook_id(hook_id)
        .add_storage_update(storage_update)
        .freeze_with(&client)?
        .sign(invalid_key)
        .execute(&client)
        .await;

    assert!(result.is_err());
    if let Err(err) = result {
        let err_str = err.to_string();
        assert!(
            err_str.contains("INVALID_SIGNATURE") || err_str.contains("InvalidSignature"),
            "Expected INVALID_SIGNATURE error, got: {}",
            err_str
        );
    }

    Ok(())
}

#[tokio::test]
async fn cannot_update_storage_for_nonexistent_hook() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let contract_id = create_hook_contract(&client).await?;
    let (account_id, account_key, _hook_id) = create_account_with_hook(&client, contract_id).await?;

    // Use non-existent hook ID
    let entity_id = HookEntityId::new(Some(account_id));
    let nonexistent_hook_id = HookId::new(Some(entity_id), 999);

    let storage_slot = LambdaStorageSlot::new(vec![0x41, 0x42, 0x43, 0x44], vec![0x45, 0x46, 0x47, 0x48]);
    let storage_update = LambdaStorageUpdate::StorageSlot(storage_slot);

    let result = LambdaSStoreTransaction::new()
        .hook_id(nonexistent_hook_id)
        .add_storage_update(storage_update)
        .freeze_with(&client)?
        .sign(account_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert!(result.is_err());
    if let Err(err) = result {
        let err_str = err.to_string();
        assert!(
            err_str.contains("HOOK_NOT_FOUND") || err_str.contains("HookNotFound"),
            "Expected HOOK_NOT_FOUND error, got: {}",
            err_str
        );
    }

    Ok(())
}

#[tokio::test]
async fn can_update_multiple_storage_slots() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let contract_id = create_hook_contract(&client).await?;
    let (_account_id, account_key, hook_id) = create_account_with_hook(&client, contract_id).await?;

    // Create multiple storage updates
    let storage_slot1 =
        LambdaStorageSlot::new(vec![0x11, 0x12, 0x13, 0x14], vec![0x15, 0x16, 0x17, 0x18]);
    let storage_slot2 =
        LambdaStorageSlot::new(vec![0x21, 0x22, 0x23, 0x24], vec![0x25, 0x26, 0x27, 0x28]);
    let storage_slot3 =
        LambdaStorageSlot::new(vec![0x31, 0x32, 0x33, 0x34], vec![0x35, 0x36, 0x37, 0x38]);

    let storage_updates = vec![
        LambdaStorageUpdate::StorageSlot(storage_slot1),
        LambdaStorageUpdate::StorageSlot(storage_slot2),
        LambdaStorageUpdate::StorageSlot(storage_slot3),
    ];

    // Update multiple storage slots at once
    let receipt = LambdaSStoreTransaction::new()
        .hook_id(hook_id)
        .storage_updates(storage_updates)
        .freeze_with(&client)?
        .sign(account_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    assert_matches!(receipt.status, Status::Success);

    Ok(())
}
