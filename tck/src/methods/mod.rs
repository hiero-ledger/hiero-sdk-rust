use std::collections::HashMap;

use jsonrpsee::core::async_trait;
use jsonrpsee::types::ErrorObjectOwned;
use serde_json::Value;

use crate::responses::{
    AccountCreateResponse,
    AccountUpdateResponse,
    GenerateKeyResponse,
    TokenBurnResponse,
    TokenMintResponse,
    TokenResponse,
};

pub mod account;
pub mod common;
pub mod token;
pub mod traits;
pub mod utility;

// Re-export traits for convenience
pub use traits::{
    AccountRpcServer,
    TokenRpcServer,
    UtilityRpcServer,
};

pub struct RpcServerImpl;

// Implement UtilityRpcServer for RpcServerImpl
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
        utility::setup(
            operator_account_id,
            operator_private_key,
            node_ip,
            node_account_id,
            mirror_network_ip,
        )
    }

    fn reset(&self) -> Result<HashMap<String, String>, ErrorObjectOwned> {
        utility::reset()
    }

    fn generate_key(
        &self,
        _type: String,
        from_key: Option<String>,
        threshold: Option<i32>,
        keys: Option<Value>,
    ) -> Result<GenerateKeyResponse, ErrorObjectOwned> {
        utility::generate_key(_type, from_key, threshold, keys)
    }
}

// Implement AccountRpcServer for RpcServerImpl
#[async_trait]
impl AccountRpcServer for RpcServerImpl {
    async fn create_account(
        &self,
        key: Option<String>,
        initial_balance: Option<i64>,
        receiver_signature_required: Option<bool>,
        auto_renew_period: Option<i64>,
        memo: Option<String>,
        max_auto_token_associations: Option<i64>,
        staked_account_id: Option<String>,
        staked_node_id: Option<i64>,
        decline_staking_reward: Option<bool>,
        alias: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<AccountCreateResponse, ErrorObjectOwned> {
        account::create_account(
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
        staked_node_id: Option<i64>,
        decline_staking_reward: Option<bool>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<AccountUpdateResponse, ErrorObjectOwned> {
        account::update_account(
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
}

// Implement TokenRpcServer for RpcServerImpl
#[async_trait]
impl TokenRpcServer for RpcServerImpl {
    async fn token_claim(
        &self,
        pending_airdrop_ids: Vec<HashMap<String, String>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<HashMap<String, String>, ErrorObjectOwned> {
        token::token_claim(pending_airdrop_ids, common_transaction_params).await
    }

    async fn token_cancel(
        &self,
        pending_airdrop_ids: Vec<HashMap<String, String>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<HashMap<String, String>, ErrorObjectOwned> {
        token::token_cancel(pending_airdrop_ids, common_transaction_params).await
    }

    async fn create_token(
        &self,
        name: Option<String>,
        symbol: Option<String>,
        decimals: Option<i64>,
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
    ) -> Result<TokenResponse, ErrorObjectOwned> {
        token::create_token(
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

    async fn burn_token(
        &self,
        token_id: Option<String>,
        amount: Option<u64>,
        serials: Option<Vec<i64>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenBurnResponse, ErrorObjectOwned> {
        token::burn_token(token_id, amount, serials, common_transaction_params).await
    }

    async fn pause_token(
        &self,
        token_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned> {
        token::pause_token(token_id, common_transaction_params).await
    }

    async fn unpause_token(
        &self,
        token_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned> {
        token::unpause_token(token_id, common_transaction_params).await
    }

    async fn freeze_token(
        &self,
        token_id: Option<String>,
        account_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned> {
        token::freeze_token(token_id, account_id, common_transaction_params).await
    }

    async fn unfreeze_token(
        &self,
        token_id: Option<String>,
        account_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned> {
        token::unfreeze_token(token_id, account_id, common_transaction_params).await
    }

    async fn associate_token(
        &self,
        account_id: Option<String>,
        token_ids: Option<Vec<String>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned> {
        token::associate_token(account_id, token_ids, common_transaction_params).await
    }

    async fn dissociate_token(
        &self,
        account_id: Option<String>,
        token_ids: Option<Vec<String>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned> {
        token::dissociate_token(account_id, token_ids, common_transaction_params).await
    }

    async fn delete_token(
        &self,
        token_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned> {
        token::delete_token(token_id, common_transaction_params).await
    }

    async fn update_token(
        &self,
        token_id: Option<String>,
        symbol: Option<String>,
        name: Option<String>,
        treasury_account_id: Option<String>,
        admin_key: Option<String>,
        kyc_key: Option<String>,
        freeze_key: Option<String>,
        wipe_key: Option<String>,
        supply_key: Option<String>,
        auto_renew_account_id: Option<String>,
        auto_renew_period: Option<String>,
        expiration_time: Option<String>,
        memo: Option<String>,
        fee_schedule_key: Option<String>,
        pause_key: Option<String>,
        metadata: Option<String>,
        metadata_key: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned> {
        token::update_token(
            token_id,
            symbol,
            name,
            treasury_account_id,
            admin_key,
            kyc_key,
            freeze_key,
            wipe_key,
            supply_key,
            auto_renew_account_id,
            auto_renew_period,
            expiration_time,
            memo,
            fee_schedule_key,
            pause_key,
            metadata,
            metadata_key,
            common_transaction_params,
        )
        .await
    }

    async fn update_token_fee_schedule(
        &self,
        token_id: Option<String>,
        custom_fees: Option<Value>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned> {
        token::update_token_fee_schedule(token_id, custom_fees, common_transaction_params).await
    }

    async fn grant_token_kyc(
        &self,
        token_id: Option<String>,
        account_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned> {
        token::grant_token_kyc(token_id, account_id, common_transaction_params).await
    }

    async fn revoke_token_kyc(
        &self,
        token_id: Option<String>,
        account_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned> {
        token::revoke_token_kyc(token_id, account_id, common_transaction_params).await
    }

    async fn mint_token(
        &self,
        token_id: Option<String>,
        amount: Option<String>,
        metadata: Option<Vec<String>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenMintResponse, ErrorObjectOwned> {
        token::mint_token(token_id, amount, metadata, common_transaction_params).await
    }

    async fn wipe_token(
        &self,
        token_id: Option<String>,
        account_id: Option<String>,
        amount: Option<String>,
        serial_numbers: Option<Vec<i64>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned> {
        token::wipe_token(token_id, account_id, amount, serial_numbers, common_transaction_params        )
            .await
    }
}
