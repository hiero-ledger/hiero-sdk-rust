// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::token_service_client::TokenServiceClient;
use time::{
    Duration,
    OffsetDateTime,
};
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};
use crate::token::token_key_validation_type::TokenKeyValidation;
use crate::transaction::{
    AnyTransactionData,
    ChunkInfo,
    ToSchedulableTransactionDataProtobuf,
    ToTransactionDataProtobuf,
    TransactionData,
    TransactionExecute,
};
use crate::{
    AccountId,
    BoxGrpcFuture,
    Error,
    Key,
    TokenId,
    Transaction,
    ValidateChecksums,
};

/// At consensus, updates an already created token to the given values.
///
/// If no value is given for a field, that field is left unchanged. For an immutable token (that is,
/// a token without an admin key), only the expiry may be updated. Setting any other field in that
/// case will cause the transaction status to resolve to `TokenIsImmutable`.
///
/// ### --- Signing Requirements ---
/// 1. Whether or not a token has an admin key, its expiry can be extended with only the transaction
///    payer's signature.
/// 2. Updating any other field of a mutable token requires the admin key's signature.
/// 3. If a new admin key is set, this new key must sign **unless** it is exactly an empty
///    `KeyList`. This special sentinel key removes the existing admin key and causes the
///    token to become immutable. (Other [`Key`][Key] structures without a constituent
///    `Ed25519` key will be rejected with `InvalidAdminKey`.
/// 4. If a new treasury is set, the new treasury account's key must sign the transaction.
///
/// ### --- Nft Requirements ---
/// 1. If a non fungible token has a positive treasury balance, the operation will abort with
///    `CurrentTreasuryStillOwnsNfts`.
pub type TokenUpdateTransaction = Transaction<TokenUpdateTransactionData>;

#[derive(Debug, Clone, Default)]
pub struct TokenUpdateTransactionData {
    /// The token to be updated.
    token_id: Option<TokenId>,

    /// The publicly visible name of the token.
    token_name: String,

    /// The publicly visible token symbol.
    token_symbol: String,

    /// The account which will act as a treasury for the token.
    treasury_account_id: Option<AccountId>,

    /// The key which can perform update/delete operations on the token.
    admin_key: Option<Key>,

    /// The key which can grant or revoke KYC of an account for the token's transactions.
    kyc_key: Option<Key>,

    /// The key which can sign to freeze or unfreeze an account for token transactions.
    freeze_key: Option<Key>,

    /// The key which can wipe the token balance of an account.
    wipe_key: Option<Key>,

    /// The key which can change the supply of a token.
    supply_key: Option<Key>,

    /// An account which will be automatically charged to renew the token's expiration, at
    /// autoRenewPeriod interval
    auto_renew_account_id: Option<AccountId>,

    /// The interval at which the auto-renew account will be charged to extend the token's expiry
    auto_renew_period: Option<Duration>,

    /// Sets the time at which the token should expire.
    expiration_time: Option<OffsetDateTime>,

    /// The memo associated with the token (UTF-8 encoding max 100 bytes)
    token_memo: Option<String>,

    /// The key which can change the token's custom fee schedule; must sign a TokenFeeScheduleUpdate
    /// transaction
    fee_schedule_key: Option<Key>,

    /// The Key which can pause and unpause the Token.
    /// If Empty the token pause status defaults to PauseNotApplicable, otherwise Unpaused.
    pause_key: Option<Key>,

    /// Metadata of the created token definition.
    metadata: Option<Vec<u8>>,

    /// The key which can change the metadata of a token
    /// (token definition, partition definition, and individual NFTs).
    metadata_key: Option<Key>,

    /// Determines whether the system should check the validity of the passed keys for update.
    key_verification_mode: TokenKeyValidation,
}

impl TokenUpdateTransaction {
    /// Returns the token to be updated.
    #[must_use]
    pub fn get_token_id(&self) -> Option<TokenId> {
        self.data().token_id
    }

    /// Sets the token to be updated.
    pub fn token_id(&mut self, token_id: impl Into<TokenId>) -> &mut Self {
        self.data_mut().token_id = Some(token_id.into());
        self
    }

    /// Returns the new publicly visible name of the token.
    #[must_use]
    pub fn get_token_name(&self) -> &str {
        &self.data().token_name
    }

    /// Sets the new publicly visible name of the token.
    ///
    /// Maximum 100 characters.
    pub fn token_name(&mut self, token_name: impl Into<String>) -> &mut Self {
        self.data_mut().token_name = token_name.into();
        self
    }

    ///Returns the new publicly visible token symbol.
    #[must_use]
    pub fn get_token_symbol(&self) -> &str {
        &self.data().token_symbol
    }

    /// Sets the new publicly visible token symbol.
    ///
    /// Maximum 100 characters.
    pub fn token_symbol(&mut self, token_symbol: impl Into<String>) -> &mut Self {
        self.data_mut().token_symbol = token_symbol.into();
        self
    }

    /// Returns the new account which will act as a treasury for the token.
    #[must_use]
    pub fn get_treasury_account_id(&self) -> Option<AccountId> {
        self.data().treasury_account_id
    }

    /// Sets the new account which will act as a treasury for the token.
    ///
    /// If the provided `treasury_account_id` does not exist or has been deleted, the response
    /// will be `InvalidTreasuryAccountForToken`.
    ///
    /// If successful, the token balance held in the previous treasury account is transferred to the
    /// new one.
    pub fn treasury_account_id(&mut self, treasury_account_id: AccountId) -> &mut Self {
        self.data_mut().treasury_account_id = Some(treasury_account_id);
        self
    }

    /// Returns the new key which can perform update/delete operations on the token.
    #[must_use]
    pub fn get_admin_key(&self) -> Option<&Key> {
        self.data().admin_key.as_ref()
    }

    /// Sets the new key which can perform update/delete operations on the token.
    ///
    /// If the token is immutable, transaction will resolve to `TokenIsImmutable`.
    pub fn admin_key(&mut self, admin_key: impl Into<Key>) -> &mut Self {
        self.data_mut().admin_key = Some(admin_key.into());
        self
    }

    /// Returns the new key which can grant or revoke KYC of an account for the token's transactions.
    #[must_use]
    pub fn get_kyc_key(&self) -> Option<&Key> {
        self.data().kyc_key.as_ref()
    }

    /// Sets the new key which can grant or revoke KYC of an account for the token's transactions.
    ///
    /// If the token does not currently have a KYC key, transaction will resolve to `TokenHasNoKycKey`.
    pub fn kyc_key(&mut self, kyc_key: impl Into<Key>) -> &mut Self {
        self.data_mut().kyc_key = Some(kyc_key.into());
        self
    }

    /// Returns the new key which can sign to freeze or unfreeze an account for token transactions.
    #[must_use]
    pub fn get_freeze_key(&self) -> Option<&Key> {
        self.data().freeze_key.as_ref()
    }

    /// Sets the new key which can sign to freeze or unfreeze an account for token transactions.
    ///
    /// If the token does not currently have a Freeze key, transaction will resolve to `TokenHasNoFreezeKey`.
    pub fn freeze_key(&mut self, freeze_key: impl Into<Key>) -> &mut Self {
        self.data_mut().freeze_key = Some(freeze_key.into());
        self
    }

    /// Returns the new key which can wipe the token balance of an account.
    #[must_use]
    pub fn get_wipe_key(&self) -> Option<&Key> {
        self.data().wipe_key.as_ref()
    }

    /// Sets the new key which can wipe the token balance of an account.
    ///
    /// If the token does not currently have a Wipe key, transaction will resolve to `TokenHasNoWipeKey`.
    pub fn wipe_key(&mut self, wipe_key: impl Into<Key>) -> &mut Self {
        self.data_mut().wipe_key = Some(wipe_key.into());
        self
    }

    /// Returns the new key which can change the supply of a token.
    #[must_use]
    pub fn get_supply_key(&self) -> Option<&Key> {
        self.data().supply_key.as_ref()
    }

    /// Sets the new key which can change the supply of a token.
    ///
    /// If the token does not currently have a Supply key, transaction will resolve to `TokenHasNoSupplyKey`.
    pub fn supply_key(&mut self, supply_key: impl Into<Key>) -> &mut Self {
        self.data_mut().supply_key = Some(supply_key.into());
        self
    }

    /// Returns the new account which will be automatically charged to renew the token's expiration.
    #[must_use]
    pub fn get_auto_renew_account_id(&self) -> Option<AccountId> {
        self.data().auto_renew_account_id
    }

    /// Sets the new account which will be automatically charged to renew the token's expiration.
    pub fn auto_renew_account_id(&mut self, auto_renew_account_id: AccountId) -> &mut Self {
        self.data_mut().auto_renew_account_id = Some(auto_renew_account_id);
        self
    }

    /// Returns the new interval at which the auto renew account will be charged to extend the token's expiry.
    #[must_use]
    pub fn get_auto_renew_period(&self) -> Option<Duration> {
        self.data().auto_renew_period
    }

    /// Sets the new interval at which the auto renew account will be charged to extend
    /// the token's expiry.
    pub fn auto_renew_period(&mut self, auto_renew_period: Duration) -> &mut Self {
        self.data_mut().auto_renew_period = Some(auto_renew_period);
        self
    }

    /// Returns the new time at which the token should expire.
    #[must_use]
    pub fn get_expiration_time(&self) -> Option<OffsetDateTime> {
        self.data().expiration_time
    }

    /// Sets the new time at which the token should expire.
    ///
    /// If the new expiration time is earlier than the current expiration time, transaction
    /// will resolve to `InvalidExpirationTime`.
    pub fn expiration_time(&mut self, expiration_time: OffsetDateTime) -> &mut Self {
        let data = self.data_mut();
        data.expiration_time = Some(expiration_time);
        data.auto_renew_period = None;

        self
    }

    /// Returns the new memo associated with the token.
    #[must_use]
    pub fn get_token_memo(&self) -> Option<&str> {
        self.data().token_memo.as_deref()
    }

    /// Sets the new memo associated with the token.
    ///
    /// Maximum of 100 bytes.
    pub fn token_memo(&mut self, memo: Option<impl Into<String>>) -> &mut Self {
        self.data_mut().token_memo = memo.map(|m| m.into());
        self
    }

    /// Returns the new key which can change the token's custom fee schedule.
    #[must_use]
    pub fn get_fee_schedule_key(&self) -> Option<&Key> {
        self.data().fee_schedule_key.as_ref()
    }

    /// Sets the new key which can change the token's custom fee schedule.
    ///
    /// If the token does not currently have a fee schedule key, transaction will resolve to
    /// `TokenHasNoFeeScheduleKey`.
    pub fn fee_schedule_key(&mut self, fee_schedule_key: impl Into<Key>) -> &mut Self {
        self.data_mut().fee_schedule_key = Some(fee_schedule_key.into());
        self
    }

    /// Returns the new key which can pause and unpause the token.
    #[must_use]
    pub fn get_pause_key(&self) -> Option<&Key> {
        self.data().pause_key.as_ref()
    }

    /// Sets the new key which can pause and unpause the Token.
    ///
    /// If the token does not currently have a pause key, transaction will resolve to `TokenHasNoPauseKey`.
    pub fn pause_key(&mut self, pause_key: impl Into<Key>) -> &mut Self {
        self.data_mut().pause_key = Some(pause_key.into());
        self
    }

    /// Returns the new metadata of the created token definition.
    #[must_use]
    pub fn get_metadata(&self) -> Option<Vec<u8>> {
        self.data().metadata.clone()
    }

    /// Sets the new metadata of the token definition.
    pub fn metadata(&mut self, metadata: Vec<u8>) -> &mut Self {
        self.data_mut().metadata = Some(metadata);
        self
    }

    /// Returns the new key which can change the metadata of a token.
    #[must_use]
    pub fn get_metadata_key(&self) -> Option<&Key> {
        self.data().metadata_key.as_ref()
    }

    /// Sets the new key which can change the metadata of a token.
    pub fn metadata_key(&mut self, metadata_key: impl Into<Key>) -> &mut Self {
        self.data_mut().metadata_key = Some(metadata_key.into());
        self
    }

    /// Returns key verification mode.
    #[must_use]
    pub fn get_key_verification_mode(&self) -> TokenKeyValidation {
        self.data().key_verification_mode
    }

    /// Assignss key verification mode.
    pub fn key_verification_mode(
        &mut self,
        key_verification_mode: TokenKeyValidation,
    ) -> &mut Self {
        self.data_mut().key_verification_mode = key_verification_mode;
        self
    }
}

impl TransactionData for TokenUpdateTransactionData {}

impl TransactionExecute for TokenUpdateTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { TokenServiceClient::new(channel).update_token(request).await })
    }
}

impl ValidateChecksums for TokenUpdateTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.token_id.validate_checksums(ledger_id)?;
        self.auto_renew_account_id.validate_checksums(ledger_id)?;
        self.treasury_account_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for TokenUpdateTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::TokenUpdate(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for TokenUpdateTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::TokenUpdate(self.to_protobuf())
    }
}

impl From<TokenUpdateTransactionData> for AnyTransactionData {
    fn from(transaction: TokenUpdateTransactionData) -> Self {
        Self::TokenUpdate(transaction)
    }
}

impl FromProtobuf<services::TokenUpdateTransactionBody> for TokenUpdateTransactionData {
    fn from_protobuf(pb: services::TokenUpdateTransactionBody) -> crate::Result<Self> {
        let key_verification_mode =
            services::TokenKeyValidation::try_from(pb.key_verification_mode as i32)
                .unwrap_or_default();

        Ok(Self {
            token_id: Option::from_protobuf(pb.token)?,
            token_name: pb.name,
            token_symbol: pb.symbol,
            treasury_account_id: Option::from_protobuf(pb.treasury)?,
            admin_key: Option::from_protobuf(pb.admin_key)?,
            kyc_key: Option::from_protobuf(pb.kyc_key)?,
            freeze_key: Option::from_protobuf(pb.freeze_key)?,
            wipe_key: Option::from_protobuf(pb.wipe_key)?,
            supply_key: Option::from_protobuf(pb.supply_key)?,
            auto_renew_account_id: Option::from_protobuf(pb.auto_renew_account)?,
            auto_renew_period: pb.auto_renew_period.map(Into::into),
            expiration_time: pb.expiry.map(Into::into),
            token_memo: pb.memo,
            fee_schedule_key: Option::from_protobuf(pb.fee_schedule_key)?,
            pause_key: Option::from_protobuf(pb.pause_key)?,
            metadata: pb.metadata,
            metadata_key: Option::from_protobuf(pb.metadata_key)?,
            key_verification_mode: TokenKeyValidation::from_protobuf(key_verification_mode)?,
        })
    }
}

impl ToProtobuf for TokenUpdateTransactionData {
    type Protobuf = services::TokenUpdateTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::TokenUpdateTransactionBody {
            token: self.token_id.to_protobuf(),
            name: self.token_name.clone(),
            symbol: self.token_symbol.clone(),
            treasury: self.treasury_account_id.to_protobuf(),
            admin_key: self.admin_key.to_protobuf(),
            kyc_key: self.kyc_key.to_protobuf(),
            freeze_key: self.freeze_key.to_protobuf(),
            wipe_key: self.wipe_key.to_protobuf(),
            supply_key: self.supply_key.to_protobuf(),
            expiry: self.expiration_time.map(Into::into),
            auto_renew_account: self.auto_renew_account_id.to_protobuf(),
            auto_renew_period: self.auto_renew_period.map(Into::into),
            memo: self.token_memo.clone(),
            fee_schedule_key: self.fee_schedule_key.to_protobuf(),
            pause_key: self.pause_key.to_protobuf(),
            metadata: self.metadata.clone(),
            metadata_key: self.metadata_key.to_protobuf(),
            key_verification_mode: self.key_verification_mode.to_protobuf().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use expect_test::expect_file;
    use hedera_proto::services;
    use time::{
        Duration,
        OffsetDateTime,
    };

    use super::TokenUpdateTransactionData;
    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::token::token_key_validation_type::TokenKeyValidation;
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
        VALID_START,
    };
    use crate::{
        AccountId,
        AnyTransaction,
        PrivateKey,
        PublicKey,
        TokenId,
        TokenUpdateTransaction,
    };

    fn test_admin_key() -> PublicKey {
        PrivateKey::from_str(
            "302e020100300506032b657004220420db484b828e64b2d8f12ce3c0a0e93a0b8cce7af1bb8f39c97732394482538e11").unwrap().public_key()
    }

    fn test_kyc_key() -> PublicKey {
        PrivateKey::from_str(
            "302e020100300506032b657004220420db484b828e64b2d8f12ce3c0a0e93a0b8cce7af1bb8f39c97732394482538e12").unwrap().public_key()
    }

    fn test_freeze_key() -> PublicKey {
        PrivateKey::from_str(
            "302e020100300506032b657004220420db484b828e64b2d8f12ce3c0a0e93a0b8cce7af1bb8f39c97732394482538e13").unwrap().public_key()
    }

    fn test_wipe_key() -> PublicKey {
        PrivateKey::from_str(
            "302e020100300506032b657004220420db484b828e64b2d8f12ce3c0a0e93a0b8cce7af1bb8f39c97732394482538e14").unwrap().public_key()
    }

    fn test_supply_key() -> PublicKey {
        PrivateKey::from_str(
            "302e020100300506032b657004220420db484b828e64b2d8f12ce3c0a0e93a0b8cce7af1bb8f39c97732394482538e15").unwrap().public_key()
    }

    fn test_fee_schedule_key() -> PublicKey {
        PrivateKey::from_str(
            "302e020100300506032b657004220420db484b828e64b2d8f12ce3c0a0e93a0b8cce7af1bb8f39c97732394482538e16").unwrap().public_key()
    }

    fn test_pause_key() -> PublicKey {
        PrivateKey::from_str(
            "302e020100300506032b657004220420db484b828e64b2d8f12ce3c0a0e93a0b8cce7af1bb8f39c97732394482538e17").unwrap().public_key()
    }

    fn test_metadata_key() -> PublicKey {
        PrivateKey::from_str(
            "302e020100300506032b657004220420db484b828e64b2d8f12ce3c0a0e93a0b8cce7af1bb8f39c97732394482538e18").unwrap().public_key()
    }

    const TEST_TREASURY_ACCOUNT_ID: AccountId = AccountId::new(7, 7, 7);
    const TEST_AUTO_RENEW_ACCOUNT_ID: AccountId = AccountId::new(8, 8, 8);
    const TEST_TOKEN_NAME: &str = "test name";
    const TEST_TOKEN_SYMBOL: &str = "test symbol";
    const TEST_TOKEN_MEMO: &str = "test memo";
    const TEST_TOKEN_ID: TokenId = TokenId::new(4, 2, 0);
    const TEST_AUTO_RENEW_PERIOD: Duration = Duration::hours(10);
    const TEST_EXPIRATION_TIME: OffsetDateTime = VALID_START;
    const TEST_METADATA: &str = "Token Metadata";
    const TEST_KEY_VERIFICATION_MODE: TokenKeyValidation = TokenKeyValidation::FullValidation;

    fn make_transaction() -> TokenUpdateTransaction {
        let mut tx = TokenUpdateTransaction::new_for_tests();

        tx.token_id(TEST_TOKEN_ID)
            .fee_schedule_key(test_fee_schedule_key())
            .supply_key(test_supply_key())
            .admin_key(test_admin_key())
            .auto_renew_account_id(TEST_AUTO_RENEW_ACCOUNT_ID)
            .auto_renew_period(TEST_AUTO_RENEW_PERIOD)
            .freeze_key(test_freeze_key())
            .wipe_key(test_wipe_key())
            .token_symbol(TEST_TOKEN_SYMBOL)
            .kyc_key(test_kyc_key())
            .pause_key(test_pause_key())
            .expiration_time(TEST_EXPIRATION_TIME)
            .treasury_account_id(TEST_TREASURY_ACCOUNT_ID)
            .token_name(TEST_TOKEN_NAME)
            .token_memo(Some(TEST_TOKEN_MEMO))
            .metadata(TEST_METADATA.as_bytes().to_vec())
            .metadata_key(test_metadata_key())
            .key_verification_mode(TokenKeyValidation::NoValidation)
            .freeze()
            .unwrap();

        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect_file!["./snapshots/token_update_transaction/serialize.txt"].assert_debug_eq(&tx);
    }

    #[test]
    fn to_from_bytes() {
        let tx = make_transaction();

        let tx2 = AnyTransaction::from_bytes(&tx.to_bytes().unwrap()).unwrap();

        let tx = transaction_body(tx);
        let tx2 = transaction_body(tx2);

        assert_eq!(tx, tx2);
    }

    #[test]
    fn from_proto_body() {
        let tx = services::TokenUpdateTransactionBody {
            token: Some(TEST_TOKEN_ID.to_protobuf()),
            symbol: TEST_TOKEN_SYMBOL.to_owned(),
            name: TEST_TOKEN_NAME.to_owned(),
            treasury: Some(TEST_TREASURY_ACCOUNT_ID.to_protobuf()),
            admin_key: Some(test_admin_key().to_protobuf()),
            kyc_key: Some(test_kyc_key().to_protobuf()),
            freeze_key: Some(test_freeze_key().to_protobuf()),
            wipe_key: Some(test_wipe_key().to_protobuf()),
            supply_key: Some(test_supply_key().to_protobuf()),
            auto_renew_account: Some(TEST_AUTO_RENEW_ACCOUNT_ID.to_protobuf()),
            auto_renew_period: Some(TEST_AUTO_RENEW_PERIOD.to_protobuf()),
            expiry: Some(TEST_EXPIRATION_TIME.to_protobuf()),
            memo: Some(TEST_TOKEN_MEMO.to_owned()),
            fee_schedule_key: Some(test_fee_schedule_key().to_protobuf()),
            pause_key: Some(test_pause_key().to_protobuf()),
            metadata: Some(TEST_METADATA.to_owned().into()),
            metadata_key: Some(test_metadata_key().to_protobuf()),
            key_verification_mode: TEST_KEY_VERIFICATION_MODE.to_protobuf().into(),
        };

        let tx = TokenUpdateTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(tx.token_id, Some(TEST_TOKEN_ID));
        assert_eq!(tx.token_name, TEST_TOKEN_NAME);
        assert_eq!(tx.token_symbol, TEST_TOKEN_SYMBOL);
        assert_eq!(tx.treasury_account_id, Some(TEST_TREASURY_ACCOUNT_ID));
        assert_eq!(tx.admin_key, Some(test_admin_key().into()));
        assert_eq!(tx.kyc_key, Some(test_kyc_key().into()));
        assert_eq!(tx.freeze_key, Some(test_freeze_key().into()));
        assert_eq!(tx.wipe_key, Some(test_wipe_key().into()));
        assert_eq!(tx.supply_key, Some(test_supply_key().into()));
        assert_eq!(tx.auto_renew_account_id, Some(TEST_AUTO_RENEW_ACCOUNT_ID));
        assert_eq!(tx.auto_renew_period, Some(TEST_AUTO_RENEW_PERIOD));
        assert_eq!(tx.expiration_time, Some(TEST_EXPIRATION_TIME));
        assert_eq!(tx.token_memo, Some(TEST_TOKEN_MEMO.into()));
        assert_eq!(tx.fee_schedule_key, Some(test_fee_schedule_key().into()));
        assert_eq!(tx.pause_key, Some(test_pause_key().into()));
        assert_eq!(tx.metadata, Some(TEST_METADATA.as_bytes().into()));
        assert_eq!(tx.metadata_key, Some(test_metadata_key().into()));
        assert_eq!(tx.key_verification_mode, TEST_KEY_VERIFICATION_MODE);
    }

    #[test]
    fn get_set_token_id() {
        let mut tx = TokenUpdateTransaction::new();
        tx.token_id(TEST_TOKEN_ID);
        assert_eq!(tx.get_token_id(), Some(TEST_TOKEN_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_token_id_frozen_panic() {
        let mut tx = make_transaction();
        tx.token_id(TEST_TOKEN_ID);
    }

    #[test]
    fn get_set_name() {
        let mut tx = TokenUpdateTransaction::new();
        tx.token_name(TEST_TOKEN_NAME);
        assert_eq!(tx.get_token_name(), TEST_TOKEN_NAME);
    }

    #[test]
    #[should_panic]
    fn get_set_name_frozen_panic() {
        let mut tx = make_transaction();
        tx.token_name(TEST_TOKEN_NAME);
    }

    #[test]
    fn get_set_symbol() {
        let mut tx = TokenUpdateTransaction::new();
        tx.token_symbol(TEST_TOKEN_SYMBOL);
        assert_eq!(tx.get_token_symbol(), TEST_TOKEN_SYMBOL);
    }

    #[test]
    #[should_panic]
    fn get_set_symbol_frozen_panic() {
        let mut tx = make_transaction();
        tx.token_symbol(TEST_TOKEN_SYMBOL);
    }

    #[test]
    fn get_set_treasury_account_id() {
        let mut tx = TokenUpdateTransaction::new();
        tx.treasury_account_id(TEST_TREASURY_ACCOUNT_ID);
        assert_eq!(tx.get_treasury_account_id(), Some(TEST_TREASURY_ACCOUNT_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_treasury_account_id_frozen_panic() {
        let mut tx = make_transaction();
        tx.treasury_account_id(TEST_TREASURY_ACCOUNT_ID);
    }

    #[test]
    fn get_set_admin_key() {
        let mut tx = TokenUpdateTransaction::new();
        tx.admin_key(test_admin_key());
        assert_eq!(tx.get_admin_key(), Some(&test_admin_key().into()));
    }

    #[test]
    #[should_panic]
    fn get_set_admin_key_frozen_panic() {
        let mut tx = make_transaction();
        tx.admin_key(test_admin_key());
    }

    #[test]
    fn get_set_kyc_key() {
        let mut tx = TokenUpdateTransaction::new();
        tx.kyc_key(test_kyc_key());
        assert_eq!(tx.get_kyc_key(), Some(&test_kyc_key().into()));
    }

    #[test]
    #[should_panic]
    fn get_set_kyc_key_frozen_panic() {
        let mut tx = make_transaction();
        tx.kyc_key(test_kyc_key());
    }

    #[test]
    fn get_set_freeze_key() {
        let mut tx = TokenUpdateTransaction::new();
        tx.freeze_key(test_freeze_key());
        assert_eq!(tx.get_freeze_key(), Some(&test_freeze_key().into()));
    }

    #[test]
    #[should_panic]
    fn get_set_freeze_key_frozen_panic() {
        let mut tx = make_transaction();
        tx.freeze_key(test_freeze_key());
    }

    #[test]
    fn get_set_wipe_key() {
        let mut tx = TokenUpdateTransaction::new();
        tx.wipe_key(test_wipe_key());
        assert_eq!(tx.get_wipe_key(), Some(&test_wipe_key().into()));
    }

    #[test]
    #[should_panic]
    fn get_set_wipe_key_frozen_panic() {
        let mut tx = make_transaction();
        tx.wipe_key(test_wipe_key());
    }

    #[test]
    fn get_set_supply_key() {
        let mut tx = TokenUpdateTransaction::new();
        tx.supply_key(test_supply_key());
        assert_eq!(tx.get_supply_key(), Some(&test_supply_key().into()));
    }

    #[test]
    #[should_panic]
    fn get_set_supply_key_frozen_panic() {
        let mut tx = make_transaction();
        tx.supply_key(test_supply_key());
    }

    #[test]
    fn get_set_auto_renew_account_id() {
        let mut tx = TokenUpdateTransaction::new();
        tx.auto_renew_account_id(TEST_AUTO_RENEW_ACCOUNT_ID);
        assert_eq!(tx.get_auto_renew_account_id(), Some(TEST_AUTO_RENEW_ACCOUNT_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_auto_renew_account_id_frozen_panic() {
        let mut tx = make_transaction();
        tx.auto_renew_account_id(TEST_AUTO_RENEW_ACCOUNT_ID);
    }

    #[test]
    fn get_set_auto_renew_period() {
        let mut tx = TokenUpdateTransaction::new();
        tx.auto_renew_period(TEST_AUTO_RENEW_PERIOD);
        assert_eq!(tx.get_auto_renew_period(), Some(TEST_AUTO_RENEW_PERIOD));
    }

    #[test]
    #[should_panic]
    fn get_set_auto_renew_period_frozen_panic() {
        let mut tx = make_transaction();
        tx.auto_renew_period(TEST_AUTO_RENEW_PERIOD);
    }

    #[test]
    fn get_set_expiration_time() {
        let mut tx = TokenUpdateTransaction::new();
        tx.expiration_time(TEST_EXPIRATION_TIME);
        assert_eq!(tx.get_expiration_time(), Some(TEST_EXPIRATION_TIME));
    }

    #[test]
    #[should_panic]
    fn get_set_expiration_time_frozen_panic() {
        let mut tx = make_transaction();
        tx.expiration_time(TEST_EXPIRATION_TIME);
    }

    #[test]
    fn get_set_token_memo() {
        let mut tx = TokenUpdateTransaction::new();
        tx.token_memo(Some(TEST_TOKEN_MEMO));
        assert_eq!(tx.get_token_memo(), Some(TEST_TOKEN_MEMO));
    }

    #[test]
    #[should_panic]
    fn get_set_token_memo_frozen_panic() {
        let mut tx = make_transaction();
        tx.token_memo(Some(TEST_TOKEN_MEMO));
    }

    #[test]
    fn get_set_fee_schedule_key() {
        let mut tx = TokenUpdateTransaction::new();
        tx.fee_schedule_key(test_fee_schedule_key());
        assert_eq!(tx.get_fee_schedule_key(), Some(&test_fee_schedule_key().into()));
    }

    #[test]
    #[should_panic]
    fn get_set_fee_schedule_key_frozen_panic() {
        let mut tx = make_transaction();
        tx.fee_schedule_key(test_fee_schedule_key());
    }

    #[test]
    fn get_set_pause_key() {
        let mut tx = TokenUpdateTransaction::new();
        tx.pause_key(test_pause_key());
        assert_eq!(tx.get_pause_key(), Some(&test_pause_key().into()));
    }

    #[test]
    #[should_panic]
    fn get_set_pause_key_frozen_panic() {
        let mut tx = make_transaction();
        tx.pause_key(test_pause_key());
    }

    #[test]
    fn get_set_metadata() {
        let mut tx = TokenUpdateTransaction::new();
        tx.metadata(TEST_METADATA.as_bytes().to_vec());
        assert_eq!(tx.get_metadata(), Some(TEST_METADATA.as_bytes().to_vec()));
    }

    #[test]
    #[should_panic]
    fn get_set_metadata_frozen_panic() {
        let mut tx = make_transaction();
        tx.metadata(TEST_METADATA.as_bytes().to_vec());
    }

    #[test]
    fn get_set_metadata_key() {
        let mut tx = TokenUpdateTransaction::new();
        tx.metadata_key(test_metadata_key());
        assert_eq!(tx.get_metadata_key(), Some(&test_metadata_key().into()));
    }

    #[test]
    #[should_panic]
    fn get_set_metadata_key_frozen_panic() {
        let mut tx = make_transaction();
        tx.metadata_key(test_metadata_key());
    }

    #[test]
    fn get_set_key_verification_mode() {
        let mut tx = TokenUpdateTransaction::new();
        tx.key_verification_mode(TokenKeyValidation::NoValidation);
        assert_eq!(tx.get_key_verification_mode(), TokenKeyValidation::NoValidation);
    }

    #[test]
    #[should_panic]
    fn get_set_key_verification_mode_frozen_panic() {
        let mut tx = make_transaction();
        tx.key_verification_mode(TokenKeyValidation::NoValidation);
    }
}
