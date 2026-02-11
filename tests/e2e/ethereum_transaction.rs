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
        .hbar_transfer(op.account_id, Hbar::new(-10))
        .hbar_transfer(new_alias_id, Hbar::new(10))
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
        .gas(300_000)
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
    let nonce = hex::decode("").unwrap();
    let max_priority_gas = hex::decode("").unwrap();
    let max_gas = hex::decode("d1385c7bf0").unwrap();
    let gas_limit = hex::decode("0249f0").unwrap();
    let to = hex::decode(contract_id.to_solidity_address().unwrap()).unwrap();
    let value = hex::decode("").unwrap();
    let call_data: Vec<u8> = ContractExecuteTransaction::new()
        .function_with_parameters(
            "setMessage",
            ContractFunctionParameters::new().add_string("new message"),
        )
        .get_function_parameters()
        .try_into()
        .unwrap();

    let rec_id = hex::decode("").map_err(|e| e)?;

    // First: create RLP list with 9 items (without signature)
    let mut rlp_bytes = BytesMut::new();
    rlp_bytes.put_u8(0x02);
    
    let mut list = RlpStream::new_list_with_buffer(rlp_bytes, 9);
    let empty_access_list: Vec<Vec<u8>> = vec![];
    list.append(&chain_id)
        .append(&nonce)
        .append(&max_priority_gas)
        .append(&max_gas)
        .append(&gas_limit)
        .append(&to)
        .append(&value)
        .append(&call_data)
        .append_list::<Vec<u8>, _>(&empty_access_list); // Empty access list

    // Get the bytes to sign (0x02 + RLP with 9 items)
    let sequence = list.out().to_vec();
    
    let signed_bytes = private_key.sign(&sequence);
    let r = &signed_bytes[0..32];
    let s = &signed_bytes[32..64];

    // Second: create new RLP list with 12 items (including signature)
    let mut rlp_bytes_final = BytesMut::new();
    rlp_bytes_final.put_u8(0x02);
    
    let mut list_final = RlpStream::new_list_with_buffer(rlp_bytes_final, 12);
    list_final
        .append(&chain_id)
        .append(&nonce)
        .append(&max_priority_gas)
        .append(&max_gas)
        .append(&gas_limit)
        .append(&to)
        .append(&value)
        .append(&call_data)
        .append_list::<Vec<u8>, _>(&empty_access_list)
        .append(&rec_id)
        .append(&r)
        .append(&s);

    let rlp_bytes = list_final.out().to_vec();
    println!("RLP bytes (hex): {}", hex::encode(&rlp_bytes));

    let eth_resp =
        EthereumTransaction::new().ethereum_data(rlp_bytes).execute(&client).await?;

    let eth_record = eth_resp.get_record(&client).await?;

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

#[tokio::test]
#[ignore = "Enable when pectra rolls out"]
async fn eip7702_ethereum_transaction() -> anyhow::Result<()> {
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
        .gas(1_000_000)
        .constructor_parameters(
            ContractFunctionParameters::new().add_string("hello from hiero").to_bytes(None),
        )
        .bytecode_file_id(file_id)
        .contract_memo("[e2e::EIP7702Transaction]")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .contract_id
        .unwrap();

    let chain_id = hex::decode("012a").unwrap();
    let nonce = hex::decode("").unwrap();
    let max_priority_gas = hex::decode("").unwrap();
    let max_gas = hex::decode("d1385c7bf0").unwrap();
    let gas_limit = hex::decode("0249f0").unwrap();
    let to = hex::decode(contract_id.to_solidity_address().unwrap()).unwrap();
    let value = hex::decode("").unwrap();
    let call_data: Vec<u8> = ContractExecuteTransaction::new()
        .function_with_parameters(
            "setMessage",
            ContractFunctionParameters::new().add_string("new message"),
        )
        .get_function_parameters()
        .try_into()
        .unwrap();

    let rec_id = hex::decode("").map_err(|e| e)?;
    
    // Empty access list for EIP-7702
    let empty_access_list: Vec<Vec<u8>> = vec![];

    // Create authorization for EIP-7702
    // Authorization message: keccak256(0x05 || rlp([chain_id, address, nonce]))
    let eip_7702_magic = vec![0x05u8];
    let contract_address_for_auth = to.clone();

    // RLP encode [chainId, contractAddress, nonce] for authorization
    let mut auth_rlp = RlpStream::new_list(3);
    auth_rlp
        .append(&chain_id)
        .append(&contract_address_for_auth)
        .append(&nonce);
    let auth_rlp_bytes = auth_rlp.out().to_vec();

    // Create authorization preimage: 0x05 || rlp([chainId, address, nonce])
    let mut auth_preimage = eip_7702_magic.clone();
    auth_preimage.extend_from_slice(&auth_rlp_bytes);

    // Hash with keccak256
    use sha3::Digest;
    let auth_message = sha3::Keccak256::digest(&auth_preimage);

    // Sign the authorization message
    let auth_signed_bytes = private_key.sign(&auth_message);
    let auth_r = &auth_signed_bytes[0..32];
    let auth_s = &auth_signed_bytes[32..64];
    
    // For simplicity, use empty yParity (recovery ID 0)
    // In production, you would calculate the actual recovery ID
    let auth_y_parity = vec![];

    // First: create RLP list with 10 items (without signature) for EIP-7702
    let mut rlp_bytes = BytesMut::new();
    rlp_bytes.put_u8(0x04); // EIP-7702 transaction type

    let mut list = RlpStream::new_list_with_buffer(rlp_bytes, 10);
    
    // Build authorization list: [[chainId, address, nonce, yParity, r, s]]
    let mut auth_list_rlp = RlpStream::new_list(1);
    let mut auth_tuple_rlp = RlpStream::new_list(6);
    auth_tuple_rlp
        .append(&chain_id)
        .append(&contract_address_for_auth)
        .append(&nonce)
        .append(&auth_y_parity)
        .append(&auth_r)
        .append(&auth_s);
    
    let auth_tuple_bytes = auth_tuple_rlp.out().to_vec();
    auth_list_rlp.append_raw(&auth_tuple_bytes, 1);
    let auth_list_bytes = auth_list_rlp.out().to_vec();
    
    list.append(&chain_id)
        .append(&nonce)
        .append(&max_priority_gas)
        .append(&max_gas)
        .append(&gas_limit)
        .append(&to)
        .append(&value)
        .append(&call_data)
        .append_list::<Vec<u8>, _>(&empty_access_list); // Empty access list
    
    // Append the authorization list as raw bytes
    list.append_raw(&auth_list_bytes, 1);

    // Get the bytes to sign (0x04 + RLP with 10 items)
    let sequence = list.out().to_vec();
    
    let signed_bytes = private_key.sign(&sequence);
    let r = &signed_bytes[0..32];
    let s = &signed_bytes[32..64];

    // Second: create new RLP list with 13 items (including signature)
    let mut rlp_bytes_final = BytesMut::new();
    rlp_bytes_final.put_u8(0x04);
    
    let mut list_final = RlpStream::new_list_with_buffer(rlp_bytes_final, 13);
    list_final
        .append(&chain_id)
        .append(&nonce)
        .append(&max_priority_gas)
        .append(&max_gas)
        .append(&gas_limit)
        .append(&to)
        .append(&value)
        .append(&call_data)
        .append_list::<Vec<u8>, _>(&empty_access_list);
    
    // Append authorization list again
    list_final.append_raw(&auth_list_bytes, 1);
    
    // Add signature
    list_final
        .append(&rec_id)
        .append(&r)
        .append(&s);

    let rlp_bytes = list_final.out().to_vec();
    println!("EIP-7702 RLP bytes (hex): {}", hex::encode(&rlp_bytes));

    let eth_resp =
        EthereumTransaction::new().ethereum_data(rlp_bytes).execute(&client).await?;

    let eth_record = eth_resp.get_record(&client).await?;

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
