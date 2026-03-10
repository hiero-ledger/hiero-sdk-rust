use bytes::{
    BufMut,
    BytesMut,
};
use hiero_sdk::{
    AccountInfoQuery,
    ContractCreateTransaction,
    ContractDeleteTransaction,
    ContractExecuteTransaction,
    ContractFunctionParameters,
    EthereumTransaction,
    FileCreateTransaction,
    FileDeleteTransaction,
    Hbar,
    PrivateKey,
    TransferTransaction,
};
use rlp::RlpStream;

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

const SMART_CONTRACT_BYTECODE: &str =
        "608060405234801561001057600080fd5b506040516104d73803806104d78339818101604052602081101561003357600080fd5b810190808051604051939291908464010000000082111561005357600080fd5b90830190602082018581111561006857600080fd5b825164010000000081118282018810171561008257600080fd5b82525081516020918201929091019080838360005b838110156100af578181015183820152602001610097565b50505050905090810190601f1680156100dc5780820380516001836020036101000a031916815260200191505b506040525050600080546001600160a01b0319163317905550805161010890600190602084019061010f565b50506101aa565b828054600181600116156101000203166002900490600052602060002090601f016020900481019282601f1061015057805160ff191683800117855561017d565b8280016001018555821561017d579182015b8281111561017d578251825591602001919060010190610162565b5061018992915061018d565b5090565b6101a791905b808211156101895760008155600101610193565b90565b61031e806101b96000396000f3fe608060405234801561001057600080fd5b50600436106100415760003560e01c8063368b87721461004657806341c0e1b5146100ee578063ce6d41de146100f6575b600080fd5b6100ec6004803603602081101561005c57600080fd5b81019060208101813564010000000081111561007757600080fd5b82018360208201111561008957600080fd5b803590602001918460018302840111640100000000831117156100ab57600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600092019190915250929550610173945050505050565b005b6100ec6101a2565b6100fe6101ba565b6040805160208082528351818301528351919283929083019185019080838360005b83811015610138578181015183820152602001610120565b50505050905090810190601f1680156101655780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b6000546001600160a01b0316331461018a5761019f565b805161019d906001906020840190610250565b505b50565b6000546001600160a01b03163314156101b85733ff5b565b60018054604080516020601f600260001961010087891615020190951694909404938401819004810282018101909252828152606093909290918301828280156102455780601f1061021a57610100808354040283529160200191610245565b820191906000526020600020905b81548152906001019060200180831161022857829003601f168201915b505050505090505b90565b828054600181600116156101000203166002900490600052602060002090601f016020900481019282601f1061029157805160ff19168380011785556102be565b828001600101855582156102be579182015b828111156102be5782518255916020019190600101906102a3565b506102ca9291506102ce565b5090565b61024d91905b808211156102ca57600081556001016102d456fea264697066735822122084964d4c3f6bc912a9d20e14e449721012d625aa3c8a12de41ae5519752fc89064736f6c63430006000033";

#[tokio::test]
#[ignore = "Temporarily disabled for incoming local-node fix"]
async fn signer_nonce_changed_on_ethereum_transaction() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(());
    };

    if !config.is_local {
        log::debug!("skipping test due to non-local");
        return Ok(());
    }

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(());
    };

    let private_key = PrivateKey::generate_ecdsa();
    let new_alias_id = private_key.to_account_id(0, 0);

    _ = TransferTransaction::new()
        .hbar_transfer(op.account_id, Hbar::new(-1))
        .hbar_transfer(new_alias_id, Hbar::new(1))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // Check if Alias Account has been auto-created;
    let _ = AccountInfoQuery::new().account_id(new_alias_id).execute(&client).await?;

    let file_id = FileCreateTransaction::new()
        .keys([op.private_key.public_key()])
        .contents(SMART_CONTRACT_BYTECODE)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .file_id
        .unwrap();

    let contract_id = ContractCreateTransaction::new()
        .admin_key(op.private_key.public_key())
        .gas(200_000)
        .constructor_parameters(
            ContractFunctionParameters::new().add_string("Hello from Hedera.").to_bytes(None),
        )
        .bytecode_file_id(file_id)
        .contract_memo("[e2e::ContractCreateTransaction]")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .contract_id
        .unwrap();

    let chain_id = hex::decode("012a").unwrap();
    let nonce = hex::decode("00").unwrap();
    let max_priority_gas = hex::decode("00").unwrap();
    let max_gas = hex::decode("d1385c7bf0").unwrap();
    let gas_limit = hex::decode("0249f0").unwrap();
    let to = hex::decode(contract_id.to_solidity_address().unwrap()).unwrap();
    let value = hex::decode("00").unwrap();
    let call_data: Vec<u8> = ContractExecuteTransaction::new()
        .function_with_parameters(
            "setMessage",
            ContractFunctionParameters::new().add_string("new message"),
        )
        .get_function_parameters()
        .try_into()
        .unwrap();

    let access_list: Vec<u8> = vec![];
    let rec_id = hex::decode("01").map_err(|e| e)?;

    let mut rlp_bytes = BytesMut::new();

    rlp_bytes.put_u8(0x02);

    // RLP encoding the transaction data
    let mut list = RlpStream::new_list_with_buffer(rlp_bytes, 12);
    list.append(&chain_id)
        .append(&nonce)
        .append(&max_priority_gas)
        .append(&max_gas)
        .append(&gas_limit) // Gas limit
        .append(&to)
        .append(&value) // Value
        .append(&call_data)
        .append(&access_list.as_slice());

    let sequence = list.as_raw();

    let signed_bytes = private_key.sign(&sequence);

    let r = &signed_bytes[0..32];
    let s = &signed_bytes[32..64];

    // Add recovery id, r, s values to the list
    list.append(&rec_id);
    list.append(&r);
    list.append(&s);

    let eth_resp =
        EthereumTransaction::new().ethereum_data(list.out().to_vec()).execute(&client).await?;

    // Local node fails to query the record after a successful ethereum transaction.
    // Note: This is a service related bug.
    let eth_record = eth_resp.get_record(&client).await.unwrap();

    let signer_nonce = eth_record.contract_function_result.unwrap().signer_nonce;

    assert_eq!(signer_nonce, Some(1));

    ContractDeleteTransaction::new()
        .transfer_account_id(op.account_id)
        .contract_id(contract_id)
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
