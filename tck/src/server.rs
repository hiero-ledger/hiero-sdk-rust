use std::collections::HashMap;
use std::str::FromStr;

use hiero_sdk::{
    AccountId,
    EvmAddress,
    Hbar,
    NftId,
    TokenId,
    TransferTransaction,
};
use jsonrpsee::core::async_trait;
use jsonrpsee::types::error::INTERNAL_ERROR_CODE;
use jsonrpsee::types::{
    ErrorObject,
    ErrorObjectOwned,
};
use serde_json::Value;

use crate::methods::account::AccountRpcServer;
use crate::methods::contract::ContractRpcServer;
use crate::methods::file::FileRpcServer;
use crate::methods::schedule::ScheduleRpcServer;
use crate::methods::token::TokenRpcServer;
use crate::methods::topic::TopicRpcServer;
use crate::methods::utility::UtilityRpcServer;
use crate::responses::{
    AccountBalanceResponse,
    AccountCreateResponse,
    AccountUpdateResponse,
    ContractByteCodeResponse,
    ContractCallResponse,
    ContractInfoResponse,
    ContractResponse,
    FileResponse,
    GenerateKeyResponse,
    ScheduleInfoResponse,
    ScheduleResponse,
    TopicInfoResponse,
    TopicResponse,
};

pub struct RpcServerImpl;

#[async_trait]
impl UtilityRpcServer for RpcServerImpl {
    fn setup(
        &self,
        operator_account_id: Option<String>,
        operator_private_key: Option<String>,
        node_ip: Option<String>,
        node_account_id: Option<String>,
        mirror_network_ip: Option<String>,
    ) -> Result<String, ErrorObjectOwned> {
        crate::methods::utility::setup(
            operator_account_id,
            operator_private_key,
            node_ip,
            node_account_id,
            mirror_network_ip,
        )
    }

    fn reset(&self) -> Result<HashMap<String, String>, ErrorObjectOwned> {
        crate::methods::utility::reset()
    }

    fn generate_key(
        &self,
        _type: String,
        from_key: Option<String>,
        threshold: Option<i32>,
        keys: Option<Value>,
    ) -> Result<GenerateKeyResponse, ErrorObjectOwned> {
        crate::methods::utility::generate_key(_type, from_key, threshold, keys)
    }

    fn set_operator(
        &self,
        operator_account_id: String,
        operator_private_key: String,
    ) -> Result<HashMap<String, String>, ErrorObjectOwned> {
        crate::methods::utility::set_operator(operator_account_id, operator_private_key)
    }
}

#[async_trait]
impl AccountRpcServer for RpcServerImpl {
    async fn create_account(
        &self,
        key: Option<String>,
        initial_balance: Option<String>,
        receiver_signature_required: Option<bool>,
        auto_renew_period: Option<String>,
        memo: Option<String>,
        max_auto_token_associations: Option<i64>,
        staked_account_id: Option<String>,
        staked_node_id: Option<String>,
        decline_staking_reward: Option<bool>,
        alias: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<AccountCreateResponse, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::account::create_account(
            &client,
            key,
            initial_balance,
            receiver_signature_required,
            auto_renew_period,
            memo,
            max_auto_token_associations,
            staked_account_id,
            staked_node_id,
            decline_staking_reward,
            alias,
            common_transaction_params,
        )
        .await
    }

    async fn update_account(
        &self,
        account_id: Option<String>,
        key: Option<String>,
        auto_renew_period: Option<i64>,
        expiration_time: Option<i64>,
        receiver_signature_required: Option<bool>,
        memo: Option<String>,
        max_auto_token_associations: Option<i64>,
        staked_account_id: Option<String>,
        staked_node_id: Option<String>,
        decline_staking_reward: Option<bool>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<AccountUpdateResponse, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::account::update_account(
            &client,
            account_id,
            key,
            auto_renew_period,
            expiration_time,
            receiver_signature_required,
            memo,
            max_auto_token_associations,
            staked_account_id,
            staked_node_id,
            decline_staking_reward,
            common_transaction_params,
        )
        .await
    }

    async fn get_account_balance(
        &self,
        account_id: Option<String>,
        contract_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<AccountBalanceResponse, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::account::get_account_balance(
            &client,
            account_id,
            contract_id,
            common_transaction_params,
        )
        .await
    }

    async fn delete_account(
        &self,
        delete_account_id: Option<String>,
        transfer_account_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<AccountUpdateResponse, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::account::delete_account(
            &client,
            delete_account_id,
            transfer_account_id,
            common_transaction_params,
        )
        .await
    }

    async fn transfer_crypto(
        &self,
        transfers: Option<Vec<Value>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<AccountUpdateResponse, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::account::transfer_crypto(&client, transfers, common_transaction_params)
            .await
    }
}

#[async_trait]
impl ContractRpcServer for RpcServerImpl {
    async fn create_contract(
        &self,
        admin_key: Option<String>,
        auto_renew_period: Option<String>,
        auto_renew_account_id: Option<String>,
        initial_balance: Option<String>,
        bytecode_file_id: Option<String>,
        initcode: Option<String>,
        staked_account_id: Option<String>,
        staked_node_id: Option<String>,
        gas: Option<String>,
        decline_staking_reward: Option<bool>,
        memo: Option<String>,
        max_automatic_token_associations: Option<i32>,
        constructor_parameters: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ContractResponse, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::contract::create_contract(
            &client,
            admin_key,
            auto_renew_period,
            auto_renew_account_id,
            initial_balance,
            bytecode_file_id,
            initcode,
            staked_account_id,
            staked_node_id,
            gas,
            decline_staking_reward,
            memo,
            max_automatic_token_associations,
            constructor_parameters,
            common_transaction_params,
        )
        .await
    }

    async fn update_contract(
        &self,
        contract_id: Option<String>,
        admin_key: Option<String>,
        auto_renew_period: Option<String>,
        auto_renew_account_id: Option<String>,
        staked_account_id: Option<String>,
        staked_node_id: Option<String>,
        decline_staking_reward: Option<bool>,
        memo: Option<String>,
        max_automatic_token_associations: Option<i32>,
        expiration_time: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ContractResponse, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::contract::update_contract(
            &client,
            contract_id,
            admin_key,
            auto_renew_period,
            auto_renew_account_id,
            staked_account_id,
            staked_node_id,
            decline_staking_reward,
            memo,
            max_automatic_token_associations,
            expiration_time,
            common_transaction_params,
        )
        .await
    }

    async fn delete_contract(
        &self,
        contract_id: Option<String>,
        transfer_account_id: Option<String>,
        transfer_contract_id: Option<String>,
        permanent_removal: Option<bool>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ContractResponse, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::contract::delete_contract(
            &client,
            contract_id,
            transfer_account_id,
            transfer_contract_id,
            permanent_removal,
            common_transaction_params,
        )
        .await
    }

    async fn execute_contract(
        &self,
        contract_id: Option<String>,
        gas: Option<String>,
        amount: Option<String>,
        function_parameters: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ContractResponse, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::contract::execute_contract(
            &client,
            contract_id,
            gas,
            amount,
            function_parameters,
            common_transaction_params,
        )
        .await
    }

    async fn contract_call_query(
        &self,
        contract_id: Option<String>,
        gas: Option<String>,
        function_name: Option<String>,
        function_parameters: Option<String>,
        sender_account_id: Option<String>,
        max_query_payment: Option<String>,
    ) -> Result<ContractCallResponse, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::contract::contract_call_query(
            &client,
            contract_id,
            gas,
            function_name,
            function_parameters,
            sender_account_id,
            max_query_payment,
        )
        .await
    }

    async fn contract_bytecode_query(
        &self,
        contract_id: Option<String>,
        query_payment: Option<String>,
        max_query_payment: Option<String>,
    ) -> Result<ContractByteCodeResponse, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::contract::contract_bytecode_query(
            &client,
            contract_id,
            query_payment,
            max_query_payment,
        )
        .await
    }

    async fn contract_info_query(
        &self,
        contract_id: Option<String>,
        query_payment: Option<String>,
        max_query_payment: Option<String>,
    ) -> Result<ContractInfoResponse, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::contract::contract_info_query(
            &client,
            contract_id,
            query_payment,
            max_query_payment,
        )
        .await
    }
}

#[async_trait]
impl FileRpcServer for RpcServerImpl {
    async fn create_file(
        &self,
        keys: Option<Vec<String>>,
        contents: Option<String>,
        expiration_time: Option<String>,
        memo: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<FileResponse, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::file::create_file(
            &client,
            keys,
            contents,
            expiration_time,
            memo,
            common_transaction_params,
        )
        .await
    }

    async fn append_file(
        &self,
        file_id: Option<String>,
        contents: Option<String>,
        max_chunks: Option<usize>,
        chunk_size: Option<usize>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<FileResponse, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::file::append_file(
            &client,
            file_id,
            contents,
            max_chunks,
            chunk_size,
            common_transaction_params,
        )
        .await
    }

    async fn delete_file(
        &self,
        file_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<FileResponse, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::file::delete_file(&client, file_id, common_transaction_params).await
    }
}

#[async_trait]
impl TokenRpcServer for RpcServerImpl {
    async fn create_token(
        &self,
        name: Option<String>,
        symbol: Option<String>,
        decimals: Option<u32>,
        initial_supply: Option<String>,
        treasury_account_id: Option<String>,
        admin_key: Option<String>,
        kyc_key: Option<String>,
        freeze_key: Option<String>,
        wipe_key: Option<String>,
        supply_key: Option<String>,
        freeze_default: Option<bool>,
        expiration_time: Option<String>,
        auto_renew_period: Option<String>,
        auto_renew_account_id: Option<String>,
        memo: Option<String>,
        token_type: Option<String>,
        supply_type: Option<String>,
        max_supply: Option<String>,
        fee_schedule_key: Option<String>,
        custom_fees: Option<Value>,
        pause_key: Option<String>,
        metadata: Option<String>,
        metadata_key: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<HashMap<String, String>, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::token::create_token(
            &client,
            name,
            symbol,
            decimals,
            initial_supply,
            treasury_account_id,
            admin_key,
            kyc_key,
            freeze_key,
            wipe_key,
            supply_key,
            freeze_default,
            expiration_time,
            auto_renew_period,
            auto_renew_account_id,
            memo,
            token_type,
            supply_type,
            max_supply,
            fee_schedule_key,
            custom_fees,
            pause_key,
            metadata,
            metadata_key,
            common_transaction_params,
        )
        .await
    }

    async fn delete_token(
        &self,
        token_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<HashMap<String, String>, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::token::delete_token(&client, token_id, common_transaction_params).await
    }

    async fn token_claim(
        &self,
        pending_airdrop_ids: Vec<HashMap<String, String>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<HashMap<String, String>, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::token::token_claim(&client, pending_airdrop_ids, common_transaction_params)
            .await
    }

    async fn token_cancel(
        &self,
        pending_airdrop_ids: Vec<HashMap<String, String>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<HashMap<String, String>, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::token::token_cancel(&client, pending_airdrop_ids, common_transaction_params)
            .await
    }

    async fn associate_token(
        &self,
        account_id: Option<String>,
        token_ids: Option<Vec<String>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<HashMap<String, String>, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::token::associate_token(
            &client,
            account_id,
            token_ids,
            common_transaction_params,
        )
        .await
    }
}

#[async_trait]
impl TopicRpcServer for RpcServerImpl {
    async fn create_topic(
        &self,
        memo: Option<String>,
        admin_key: Option<String>,
        submit_key: Option<String>,
        auto_renew_period: Option<String>,
        auto_renew_account_id: Option<String>,
        fee_schedule_key: Option<String>,
        fee_exempt_keys: Option<Vec<String>>,
        custom_fees: Option<Value>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TopicResponse, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::topic::create_topic(
            &client,
            memo,
            admin_key,
            submit_key,
            auto_renew_period,
            auto_renew_account_id,
            fee_schedule_key,
            fee_exempt_keys,
            custom_fees,
            common_transaction_params,
        )
        .await
    }

    async fn update_topic(
        &self,
        topic_id: Option<String>,
        memo: Option<String>,
        admin_key: Option<String>,
        submit_key: Option<String>,
        auto_renew_period: Option<String>,
        auto_renew_account_id: Option<String>,
        expiration_time: Option<String>,
        fee_schedule_key: Option<String>,
        fee_exempt_keys: Option<Vec<String>>,
        custom_fees: Option<Value>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TopicResponse, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::topic::update_topic(
            &client,
            topic_id,
            memo,
            admin_key,
            submit_key,
            auto_renew_period,
            auto_renew_account_id,
            expiration_time,
            fee_schedule_key,
            fee_exempt_keys,
            custom_fees,
            common_transaction_params,
        )
        .await
    }

    async fn delete_topic(
        &self,
        topic_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TopicResponse, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::topic::delete_topic(&client, topic_id, common_transaction_params).await
    }

    async fn submit_topic_message(
        &self,
        topic_id: Option<String>,
        message: Option<String>,
        max_chunks: Option<i64>,
        chunk_size: Option<i64>,
        custom_fee_limits: Option<Value>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TopicResponse, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::topic::submit_topic_message(
            &client,
            topic_id,
            message,
            max_chunks,
            chunk_size,
            custom_fee_limits,
            common_transaction_params,
        )
        .await
    }

    async fn get_topic_info(
        &self,
        topic_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TopicInfoResponse, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::topic::get_topic_info(&client, topic_id, common_transaction_params).await
    }
}

#[async_trait]
impl ScheduleRpcServer for RpcServerImpl {
    async fn create_schedule(
        &self,
        scheduled_transaction: Option<Value>,
        memo: Option<String>,
        admin_key: Option<String>,
        payer_account_id: Option<String>,
        expiration_time: Option<String>,
        wait_for_expiry: Option<bool>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ScheduleResponse, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::schedule::create_schedule(
            &client,
            scheduled_transaction,
            memo,
            admin_key,
            payer_account_id,
            expiration_time,
            wait_for_expiry,
            common_transaction_params,
        )
        .await
    }

    async fn sign_schedule(
        &self,
        schedule_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ScheduleResponse, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::schedule::sign_schedule(&client, schedule_id, common_transaction_params)
            .await
    }

    async fn delete_schedule(
        &self,
        schedule_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ScheduleResponse, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::schedule::delete_schedule(&client, schedule_id, common_transaction_params)
            .await
    }

    async fn get_schedule_info(
        &self,
        schedule_id: Option<String>,
        query_payment: Option<String>,
        max_query_payment: Option<String>,
        get_cost: Option<bool>,
    ) -> Result<ScheduleInfoResponse, ErrorObjectOwned> {
        let client = crate::methods::utility::get_client()?;
        crate::methods::schedule::get_schedule_info(
            &client,
            schedule_id,
            query_payment,
            max_query_payment,
            get_cost,
        )
        .await
    }
}

pub(crate) fn build_transfer_tx_from_value(
    params: &Value,
) -> Result<TransferTransaction, ErrorObjectOwned> {
    let mut tx = TransferTransaction::new();

    let transfers = params.get("transfers").and_then(Value::as_array).ok_or_else(|| {
        ErrorObject::owned(
            INTERNAL_ERROR_CODE,
            "transfers array is required for transferCrypto".to_string(),
            None::<()>,
        )
    })?;

    if transfers.is_empty() {
        return Err(ErrorObject::owned(
            INTERNAL_ERROR_CODE,
            "No transfers provided".to_string(),
            None::<()>,
        ));
    }

    for t in transfers {
        if let Value::Object(obj) = t {
            let is_approved = obj.get("approved").and_then(Value::as_bool).unwrap_or(false);

            // Handle HBAR transfers
            if let Some(hbar_obj) = obj.get("hbar").and_then(Value::as_object) {
                let amount_i64 = hbar_obj
                    .get("amount")
                    .and_then(|v| v.as_i64().or_else(|| v.as_str()?.parse::<i64>().ok()))
                    .ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "amount is required in hbar transfers".to_string(),
                            None::<()>,
                        )
                    })?;
                let amount = Hbar::from_tinybars(amount_i64);

                if let Some(account_id_str) = hbar_obj.get("accountId").and_then(Value::as_str) {
                    let account_id = AccountId::from_str(account_id_str).map_err(|e| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            format!("Invalid account ID: {e}"),
                            None::<()>,
                        )
                    })?;

                    if is_approved {
                        tx.approved_hbar_transfer(account_id, amount);
                    } else {
                        tx.hbar_transfer(account_id, amount);
                    }
                } else if let Some(evm_address_str) =
                    hbar_obj.get("evmAddress").and_then(Value::as_str)
                {
                    let evm_address = EvmAddress::from_str(evm_address_str).map_err(|e| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            format!("Invalid EVM address: {e}"),
                            None::<()>,
                        )
                    })?;
                    let account_id = AccountId::from_evm_address(&evm_address, 0, 0);

                    if is_approved {
                        tx.approved_hbar_transfer(account_id, amount);
                    } else {
                        tx.hbar_transfer(account_id, amount);
                    }
                } else {
                    return Err(ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Either accountId or evmAddress is required in hbar transfers".to_string(),
                        None::<()>,
                    ));
                }
            }
            // Handle token transfers
            else if let Some(token_obj) = obj.get("token").and_then(Value::as_object) {
                let account_id_str =
                    token_obj.get("accountId").and_then(Value::as_str).ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "accountId is required in token transfers".to_string(),
                            None::<()>,
                        )
                    })?;
                let account_id = AccountId::from_str(account_id_str).map_err(|e| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        format!("Invalid account ID: {e}"),
                        None::<()>,
                    )
                })?;

                let token_id_str =
                    token_obj.get("tokenId").and_then(Value::as_str).ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "tokenId is required in token transfers".to_string(),
                            None::<()>,
                        )
                    })?;
                let token_id = TokenId::from_str(token_id_str).map_err(|e| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        format!("Invalid token ID: {e}"),
                        None::<()>,
                    )
                })?;

                let amount = token_obj
                    .get("amount")
                    .and_then(|v| v.as_i64().or_else(|| v.as_str()?.parse::<i64>().ok()))
                    .ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "amount is required in token transfers".to_string(),
                            None::<()>,
                        )
                    })?;

                let decimals = token_obj.get("decimals").and_then(Value::as_u64).map(|d| d as u32);

                if is_approved {
                    tx.approved_token_transfer(token_id, account_id, amount);
                } else if let Some(dec) = decimals {
                    tx.token_transfer_with_decimals(token_id, account_id, amount, dec);
                } else {
                    tx.token_transfer(token_id, account_id, amount);
                }
            }
            // Handle NFT transfers
            else if let Some(nft_obj) = obj.get("nft").and_then(Value::as_object) {
                let sender_account_id_str =
                    nft_obj.get("senderAccountId").and_then(Value::as_str).ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "senderAccountId is required in NFT transfers".to_string(),
                            None::<()>,
                        )
                    })?;
                let sender_account_id =
                    AccountId::from_str(sender_account_id_str).map_err(|e| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            format!("Invalid sender account ID: {e}"),
                            None::<()>,
                        )
                    })?;

                let receiver_account_id_str =
                    nft_obj.get("receiverAccountId").and_then(Value::as_str).ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "receiverAccountId is required in NFT transfers".to_string(),
                            None::<()>,
                        )
                    })?;
                let receiver_account_id =
                    AccountId::from_str(receiver_account_id_str).map_err(|e| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            format!("Invalid receiver account ID: {e}"),
                            None::<()>,
                        )
                    })?;

                let token_id_str =
                    nft_obj.get("tokenId").and_then(Value::as_str).ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "tokenId is required in NFT transfers".to_string(),
                            None::<()>,
                        )
                    })?;
                let token_id = TokenId::from_str(token_id_str).map_err(|e| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        format!("Invalid token ID: {e}"),
                        None::<()>,
                    )
                })?;

                let serial_number = nft_obj
                    .get("serialNumber")
                    .and_then(|v| v.as_i64().or_else(|| v.as_str()?.parse::<i64>().ok()))
                    .ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "serialNumber is required in NFT transfers".to_string(),
                            None::<()>,
                        )
                    })?;

                let nft_id = NftId::from((token_id, serial_number as u64));

                if is_approved {
                    tx.approved_nft_transfer(nft_id, sender_account_id, receiver_account_id);
                } else {
                    tx.nft_transfer(nft_id, sender_account_id, receiver_account_id);
                }
            }
        }
    }

    Ok(tx)
}
