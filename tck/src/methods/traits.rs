use std::collections::HashMap;

use jsonrpsee::proc_macros::rpc;
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

// Utility RPC methods
#[rpc(server, client)]
pub trait UtilityRpc {
    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/utility.md#generateKey
    */
    #[method(name = "generateKey")]
    fn generate_key(
        &self,
        _type: String,
        from_key: Option<String>,
        threshold: Option<i32>,
        keys: Option<Value>,
    ) -> Result<GenerateKeyResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/utility.md#setup
    */
    #[method(name = "setup")]
    fn setup(
        &self,
        operator_account_id: Option<String>,
        operator_private_key: Option<String>,
        node_ip: Option<String>,
        node_account_id: Option<String>,
        mirror_network_ip: Option<String>,
    ) -> Result<String, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/utility.md#reset
    */
    #[method(name = "reset")]
    fn reset(&self) -> Result<HashMap<String, String>, ErrorObjectOwned>;
}

// Account RPC methods
#[rpc(server, client)]
pub trait AccountRpc {
    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/crypto-service/accountCreateTransaction.md#createAccount
    */
    #[method(name = "createAccount")]
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
    ) -> Result<AccountCreateResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/crypto-service/accountUpdateTransaction.md#updateAccount
    */
    #[method(name = "updateAccount")]
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
    ) -> Result<AccountUpdateResponse, ErrorObjectOwned>;
}

// Token RPC methods
#[rpc(server, client)]
pub trait TokenRpc {
    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenClaimAirdropTransaction.md#tokenClaim
    */
    #[method(name = "tokenClaim")]
    async fn token_claim(
        &self,
        pending_airdrop_ids: Vec<HashMap<String, String>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<HashMap<String, String>, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenCancelAirdropTransaction.md#tokenCancel
    */
    #[method(name = "tokenCancel")]
    async fn token_cancel(
        &self,
        pending_airdrop_ids: Vec<HashMap<String, String>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<HashMap<String, String>, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenCreateTransaction.md#createToken
    */
    #[method(name = "createToken")]
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
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenBurnTransaction.md#burnToken
    */
    #[method(name = "burnToken")]
    async fn burn_token(
        &self,
        token_id: Option<String>,
        amount: Option<u64>,
        serials: Option<Vec<i64>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenBurnResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenPauseTransaction.md#pauseToken
    */
    #[method(name = "pauseToken")]
    async fn pause_token(
        &self,
        token_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenUnpauseTransaction.md#unpauseToken
    */
    #[method(name = "unpauseToken")]
    async fn unpause_token(
        &self,
        token_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenFreezeTransaction.md#freezeToken
    */
    #[method(name = "freezeToken")]
    async fn freeze_token(
        &self,
        token_id: Option<String>,
        account_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenUnfreezeTransaction.md#unfreezeToken
    */
    #[method(name = "unfreezeToken")]
    async fn unfreeze_token(
        &self,
        token_id: Option<String>,
        account_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenAssociateTransaction.md#associateToken
    */
    #[method(name = "associateToken")]
    async fn associate_token(
        &self,
        account_id: Option<String>,
        token_ids: Option<Vec<String>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenDissociateTransaction.md#dissociateToken
    */
    #[method(name = "dissociateToken")]
    async fn dissociate_token(
        &self,
        account_id: Option<String>,
        token_ids: Option<Vec<String>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenDeleteTransaction.md#deleteToken
    */
    #[method(name = "deleteToken")]
    async fn delete_token(
        &self,
        token_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenUpdateTransaction.md#updateToken
    */
    #[method(name = "updateToken")]
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
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenFeeScheduleUpdateTransaction.md#updateTokenFeeSchedule
    */
    #[method(name = "updateTokenFeeSchedule")]
    async fn update_token_fee_schedule(
        &self,
        token_id: Option<String>,
        custom_fees: Option<Value>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenGrantKycTransaction.md#grantTokenKyc
    */
    #[method(name = "grantTokenKyc")]
    async fn grant_token_kyc(
        &self,
        token_id: Option<String>,
        account_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenRevokeKycTransaction.md#revokeTokenKyc
    */
    #[method(name = "revokeTokenKyc")]
    async fn revoke_token_kyc(
        &self,
        token_id: Option<String>,
        account_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenMintTransaction.md#mintToken
    */
    #[method(name = "mintToken")]
    async fn mint_token(
        &self,
        token_id: Option<String>,
        amount: Option<String>,
        metadata: Option<Vec<String>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenMintResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/token-service/tokenWipeTransaction.md#wipeToken
    */
    #[method(name = "wipeToken")]
    async fn wipe_token(
        &self,
        token_id: Option<String>,
        account_id: Option<String>,
        amount: Option<String>,
        serial_numbers: Option<Vec<i64>>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<TokenResponse, ErrorObjectOwned>;
}
