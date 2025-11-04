use assert_matches::assert_matches;
use hedera::{
    AccountId,
    ContractCreateTransaction,
    ContractDeleteTransaction,
    ContractFunctionParameters,
    ContractId,
    ContractInfoQuery,
    ContractUpdateTransaction,
    EvmHookSpec,
    FileCreateTransaction,
    FileDeleteTransaction,
    FileId,
    Hbar,
    HookCreationDetails,
    HookExtensionPoint,
    LambdaEvmHook,
    LambdaStorageSlot,
    LambdaStorageUpdate,
    PublicKey,
    Status,
};

use super::SMART_CONTRACT_BYTECODE;
use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

const HOOK_BYTECODE: &str = "6080604052348015600e575f5ffd5b506103da8061001c5f395ff3fe60806040526004361061001d575f3560e01c80630b6c5c0414610021575b5f5ffd5b61003b6004803603810190610036919061021c565b610051565b60405161004891906102ed565b60405180910390f35b5f61016d73ffffffffffffffffffffffffffffffffffffffff163073ffffffffffffffffffffffffffffffffffffffff16146100c2626040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016100b990610386565b60405180910390fd5b60019050979650505050505050565b5f5ffd5b5f5ffd5b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f610102826100d9565b9050919050565b610112816100f8565b811461011c575f5ffd5b50565b5f8135905061012d81610109565b92915050565b5f819050919050565b61014581610133565b811461014f575f5ffd5b50565b5f813590506101608161013c565b92915050565b5f5ffd5b5f5ffd5b5f5ffd5b5f5f83601f84011261018757610186610166565b5b8235905067ffffffffffffffff8111156101a4576101a361016a565b5b6020830191508360018202830111156101c0576101bf61016e565b5b9250929050565b5f5f83601f8401126101dc576101db610166565b5b8235905067ffffffffffffffff8111156101f9576101f861016a565b5b6020830191508360018202830111156102155761021461016e565b5b9250929050565b5f5f5f5f5f5f5f60a0888a031215610237576102366100d1565b5b5f6102448a828b0161011f565b97505060206102558a828b01610152565b96505060406102668a828b01610152565b955050606088013567ffffffffffffffff811115610287576102866100d5565b5b6102938a828b01610172565b9450945050608088013567ffffffffffffffff8111156102b6576102b56100d5565b5b6102c28a828b016101c7565b925092505092959891949750929550565b5f8115159050919050565b6102e7816102d3565b82525050565b5f6020820190506103005f8301846102de565b92915050565b5f82825260208201905092915050565b7f436f6e74726163742063616e206f6e6c792062652063616c6c656420617320615f8201527f20686f6f6b000000000000000000000000000000000000000000000000000000602082015250565b5f610370602583610306565b915061037b82610316565b604082019050919050565b5f6020820190508181035f83015261039d81610364565b905091905056fea2646970667358221220a8c76458204f8bb9a86f59ec2f0ccb7cbe8ae4dcb65700c4b6ee91a39404083a64736f6c634300081e0033";

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

async fn create_test_contract(
    client: &hedera::Client,
    operator_key: PublicKey,
    operator_account_id: AccountId,
) -> anyhow::Result<(ContractId, FileId)> {
    let bytecode = hex::decode(SMART_CONTRACT_BYTECODE)?;

    let file_id = FileCreateTransaction::new()
        .keys([operator_key])
        .contents(bytecode)
        .execute(client)
        .await?
        .get_receipt(client)
        .await?
        .file_id
        .unwrap();

    let contract_id = ContractCreateTransaction::new()
        .admin_key(operator_key)
        .gas(300_000)
        .constructor_parameters(
            ContractFunctionParameters::new().add_string("Hello from Hiero.").to_bytes(None),
        )
        .bytecode_file_id(file_id)
        .contract_memo("[e2e::ContractCreateTransaction]")
        .auto_renew_account_id(operator_account_id)
        .execute(client)
        .await?
        .get_receipt(client)
        .await?
        .contract_id
        .unwrap();

    Ok((contract_id, file_id))
}

#[tokio::test]
async fn basic_contract_update() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &client.operator() else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let operator_key = op.private_key.public_key();
    let (contract_id, _file_id) =
        create_test_contract(&client, operator_key, op.account_id).await?;

    let info_before = ContractInfoQuery::new()
        .contract_id(contract_id)
        .query_payment(Hbar::new(1))
        .execute(&client)
        .await?;

    assert_eq!(info_before.contract_id, contract_id);
    assert_eq!(info_before.contract_memo, "[e2e::ContractCreateTransaction]");

    ContractUpdateTransaction::new()
        .contract_id(contract_id)
        .contract_memo("[e2e::ContractUpdateTransaction]")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info_after = ContractInfoQuery::new()
        .contract_id(contract_id)
        .query_payment(Hbar::new(5))
        .execute(&client)
        .await?;

    assert_eq!(info_after.contract_id, contract_id);
    assert_eq!(info_after.contract_memo, "[e2e::ContractUpdateTransaction]");

    ContractDeleteTransaction::new()
        .contract_id(contract_id)
        .transfer_account_id(op.account_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}

#[tokio::test]
async fn contract_update_fails_when_contract_id_not_set() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &client.operator() else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let operator_key = op.private_key.public_key();
    let (_contract_id, _file_id) =
        create_test_contract(&client, operator_key, op.account_id).await?;

    let result = ContractUpdateTransaction::new()
        .contract_memo("[e2e::ContractUpdateTransaction]")
        .execute(&client)
        .await;

    assert_matches!(result, Err(hedera::Error::TransactionPreCheckStatus { status, .. }) if status == Status::InvalidContractId);

    Ok(())
}

#[tokio::test]
async fn contract_update_auto_renew_account_id_to_zero() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &client.operator() else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let operator_key = op.private_key.public_key();
    let (contract_id, _file_id) =
        create_test_contract(&client, operator_key, op.account_id).await?;

    let info_before = ContractInfoQuery::new()
        .contract_id(contract_id)
        .query_payment(Hbar::new(1))
        .execute(&client)
        .await?;

    assert_eq!(info_before.contract_id, contract_id);
    assert_eq!(info_before.auto_renew_account_id, Some(op.account_id));

    ContractUpdateTransaction::new()
        .contract_id(contract_id)
        .contract_memo("[e2e::ContractUpdateTransaction]")
        .auto_renew_account_id(AccountId::new(0, 0, 0))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info_after = ContractInfoQuery::new()
        .contract_id(contract_id)
        .query_payment(Hbar::new(5))
        .execute(&client)
        .await?;

    assert_eq!(info_after.contract_id, contract_id);
    assert_eq!(info_after.auto_renew_account_id, Some(AccountId::new(0, 0, 0)));

    ContractDeleteTransaction::new()
        .contract_id(contract_id)
        .transfer_account_id(op.account_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}

#[tokio::test]
async fn contract_update_adds_basic_lambda_evm_hook() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &client.operator() else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let operator_key = op.private_key.public_key();
    let (contract_id, _file_id) =
        create_test_contract(&client, operator_key, op.account_id).await?;
    let lambda_contract_id = create_hook_contract(&client).await?;

    let lambda_hook = LambdaEvmHook::new(EvmHookSpec::new(Some(lambda_contract_id)), vec![]);

    let hook_details =
        HookCreationDetails::new(HookExtensionPoint::AccountAllowanceHook, 1, Some(lambda_hook));

    ContractUpdateTransaction::new()
        .contract_id(contract_id)
        .add_hook(hook_details)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}

#[tokio::test]
async fn contract_update_fails_with_duplicate_hook_ids() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &client.operator() else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let operator_key = op.private_key.public_key();
    let (contract_id, _file_id) =
        create_test_contract(&client, operator_key, op.account_id).await?;
    let lambda_contract_id = create_hook_contract(&client).await?;

    let lambda_hook = LambdaEvmHook::new(EvmHookSpec::new(Some(lambda_contract_id)), vec![]);

    let hook_details1 = HookCreationDetails::new(
        HookExtensionPoint::AccountAllowanceHook,
        1,
        Some(lambda_hook.clone()),
    );

    let hook_details2 =
        HookCreationDetails::new(HookExtensionPoint::AccountAllowanceHook, 1, Some(lambda_hook));

    let result = ContractUpdateTransaction::new()
        .contract_id(contract_id)
        .add_hook(hook_details1)
        .add_hook(hook_details2)
        .execute(&client)
        .await;

    assert_matches!(result, Err(hedera::Error::TransactionPreCheckStatus { status, .. }) if status == Status::HookIdRepeatedInCreationDetails);

    Ok(())
}

#[tokio::test]
async fn contract_update_fails_when_hook_id_in_use() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &client.operator() else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let operator_key = op.private_key.public_key();
    let (contract_id, _file_id) =
        create_test_contract(&client, operator_key, op.account_id).await?;
    let lambda_contract_id = create_hook_contract(&client).await?;

    let lambda_hook = LambdaEvmHook::new(EvmHookSpec::new(Some(lambda_contract_id)), vec![]);

    let hook_details = HookCreationDetails::new(
        HookExtensionPoint::AccountAllowanceHook,
        1,
        Some(lambda_hook.clone()),
    );

    ContractUpdateTransaction::new()
        .contract_id(contract_id)
        .add_hook(hook_details)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let duplicate_hook_details =
        HookCreationDetails::new(HookExtensionPoint::AccountAllowanceHook, 1, Some(lambda_hook));

    let result = ContractUpdateTransaction::new()
        .contract_id(contract_id)
        .add_hook(duplicate_hook_details)
        .execute(&client)
        .await;

    assert_matches!(result, Err(hedera::Error::TransactionPreCheckStatus { status, .. }) if status == Status::HookIdInUse);

    Ok(())
}

#[tokio::test]
async fn contract_update_adds_lambda_evm_hook_with_storage_updates() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &client.operator() else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let operator_key = op.private_key.public_key();
    let (contract_id, _file_id) =
        create_test_contract(&client, operator_key, op.account_id).await?;
    let lambda_contract_id = create_hook_contract(&client).await?;

    let storage_slot = LambdaStorageSlot::new(vec![0x01, 0x02, 0x03, 0x04], vec![]);
    let storage_update = LambdaStorageUpdate::StorageSlot(storage_slot);

    let lambda_hook =
        LambdaEvmHook::new(EvmHookSpec::new(Some(lambda_contract_id)), vec![storage_update]);

    let hook_details =
        HookCreationDetails::new(HookExtensionPoint::AccountAllowanceHook, 1, Some(lambda_hook));

    ContractUpdateTransaction::new()
        .contract_id(contract_id)
        .add_hook(hook_details)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}

#[tokio::test]
async fn contract_update_deletes_hook_by_id() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &client.operator() else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let operator_key = op.private_key.public_key();
    let (contract_id, _file_id) =
        create_test_contract(&client, operator_key, op.account_id).await?;
    let lambda_contract_id = create_hook_contract(&client).await?;

    let lambda_hook = LambdaEvmHook::new(EvmHookSpec::new(Some(lambda_contract_id)), vec![]);

    let hook_details =
        HookCreationDetails::new(HookExtensionPoint::AccountAllowanceHook, 1, Some(lambda_hook));

    ContractUpdateTransaction::new()
        .contract_id(contract_id)
        .add_hook(hook_details)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let delete_receipt = ContractUpdateTransaction::new()
        .contract_id(contract_id)
        .delete_hook(1)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    assert_matches!(delete_receipt.status, Status::Success);

    Ok(())
}

#[tokio::test]
async fn contract_update_fails_when_deleting_nonexistent_hook() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &client.operator() else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let operator_key = op.private_key.public_key();
    let (contract_id, _file_id) =
        create_test_contract(&client, operator_key, op.account_id).await?;
    let lambda_contract_id = create_hook_contract(&client).await?;

    let lambda_hook = LambdaEvmHook::new(EvmHookSpec::new(Some(lambda_contract_id)), vec![]);

    let hook_details =
        HookCreationDetails::new(HookExtensionPoint::AccountAllowanceHook, 1, Some(lambda_hook));

    ContractUpdateTransaction::new()
        .contract_id(contract_id)
        .add_hook(hook_details)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let result = ContractUpdateTransaction::new()
        .contract_id(contract_id)
        .delete_hook(999)
        .execute(&client)
        .await;

    assert_matches!(result, Err(hedera::Error::TransactionPreCheckStatus { status, .. }) if status == Status::HookNotFound);

    Ok(())
}

#[tokio::test]
async fn contract_update_fails_when_adding_and_deleting_same_hook_id() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &client.operator() else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let operator_key = op.private_key.public_key();
    let (contract_id, _file_id) =
        create_test_contract(&client, operator_key, op.account_id).await?;
    let lambda_contract_id = create_hook_contract(&client).await?;

    let lambda_hook = LambdaEvmHook::new(EvmHookSpec::new(Some(lambda_contract_id)), vec![]);

    let hook_details =
        HookCreationDetails::new(HookExtensionPoint::AccountAllowanceHook, 1, Some(lambda_hook));

    let result = ContractUpdateTransaction::new()
        .contract_id(contract_id)
        .add_hook(hook_details)
        .delete_hook(1)
        .execute(&client)
        .await;

    assert_matches!(result, Err(hedera::Error::TransactionPreCheckStatus { status, .. }) if status == Status::HookNotFound);

    ContractDeleteTransaction::new()
        .contract_id(contract_id)
        .transfer_account_id(op.account_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}
