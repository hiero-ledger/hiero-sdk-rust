use hiero_sdk::{
    ContractCreateTransaction,
    ContractDeleteTransaction,
    FileCreateTransaction,
    FileDeleteTransaction,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

const SMART_CONTRACT_BYTECODE: &str = "6080604052348015600f57600080fd5b50604051601a90603b565b604051809103906000f0801580156035573d6000803e3d6000fd5b50506047565b605c8061009483390190565b603f806100556000396000f3fe6080604052600080fdfea2646970667358221220a20122cbad3457fedcc0600363d6e895f17048f5caa4afdab9e655123737567d64736f6c634300081200336080604052348015600f57600080fd5b50603f80601d6000396000f3fe6080604052600080fdfea264697066735822122053dfd8835e3dc6fedfb8b4806460b9b7163f8a7248bac510c6d6808d9da9d6d364736f6c63430008120033";

#[tokio::test]
async fn increment_nonce_through_contract_constructor() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let file_id = FileCreateTransaction::new()
        .keys([op.private_key.public_key()])
        .contents(SMART_CONTRACT_BYTECODE)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .file_id
        .unwrap();

    let response = ContractCreateTransaction::new()
        .admin_key(op.private_key.public_key())
        .gas(1000000)
        .bytecode_file_id(file_id)
        .contract_memo("[e2e::ContractADeploysContractBInConstructor]")
        .execute(&client)
        .await?;

    let record = response.get_record(&client).await?;
    let contract_a = record.receipt.contract_id.unwrap();
    let contract_function_result = record.contract_function_result.unwrap();

    assert_eq!(contract_function_result.contract_nonces.len(), 2);

    let contract_a_nonce_info = contract_function_result
        .contract_nonces
        .iter()
        .find(|it| it.contract_id == contract_a)
        .unwrap();
    let contract_b_nonce_info = contract_function_result
        .contract_nonces
        .iter()
        .find(|it| it.contract_id != contract_a)
        .unwrap();

    // A.nonce = 2
    assert_eq!(contract_a_nonce_info.nonce, 2);
    // B.nonce = 1
    assert_eq!(contract_b_nonce_info.nonce, 1);

    ContractDeleteTransaction::new()
        .transfer_account_id(op.account_id)
        .contract_id(contract_a)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    FileDeleteTransaction::new()
        .file_id(file_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    Ok(())
}
