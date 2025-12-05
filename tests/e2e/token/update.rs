use assert_matches::assert_matches;
use hiero_sdk::{
    Client,
    Hbar,
    Key,
    KeyList,
    PrivateKey,
    PublicKey,
    Status,
    TokenCreateTransaction,
    TokenDeleteTransaction,
    TokenId,
    TokenInfoQuery,
    TokenKeyValidation,
    TokenType,
    TokenUpdateTransaction,
};
use time::{
    Duration,
    OffsetDateTime,
};

use super::FungibleToken;
use crate::account::Account;
use crate::common::{
    setup_nonfree,
    TestEnvironment,
};
use crate::token::{
    CreateFungibleToken,
    TokenKeys,
};

#[tokio::test]
async fn basic() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;
    let token = FungibleToken::create(
        &client,
        &account,
        CreateFungibleToken { initial_supply: 0, keys: TokenKeys::ALL_OWNER },
    )
    .await?;

    TokenUpdateTransaction::new()
        .token_id(token.id)
        .token_name("aaaa")
        .token_symbol("A")
        .sign(account.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info = TokenInfoQuery::new().token_id(token.id).execute(&client).await?;

    assert_eq!(info.token_id, token.id);
    assert_eq!(info.name, "aaaa");
    assert_eq!(info.symbol, "A");
    assert_eq!(info.decimals, 3);
    assert_eq!(info.treasury_account_id, account.id);
    assert_eq!(info.admin_key, Some(Key::Single(account.key.public_key())));
    assert_eq!(info.freeze_key, Some(Key::Single(account.key.public_key())));
    assert_eq!(info.wipe_key, Some(Key::Single(account.key.public_key())));
    assert_eq!(info.kyc_key, Some(Key::Single(account.key.public_key())));
    assert_eq!(info.supply_key, Some(Key::Single(account.key.public_key())));
    assert_eq!(info.pause_key, Some(Key::Single(account.key.public_key())));
    assert_eq!(info.fee_schedule_key, Some(Key::Single(account.key.public_key())));
    assert_eq!(info.default_freeze_status, Some(false));
    assert_eq!(info.default_kyc_status, Some(false));

    token.delete(&client).await?;
    account.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn immutable_token_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let account = Account::create(Hbar::new(0), &client).await?;

    let token = FungibleToken::create(
        &client,
        &account,
        CreateFungibleToken { initial_supply: 0, keys: TokenKeys::NONE },
    )
    .await?;

    let res = TokenUpdateTransaction::new()
        .token_id(token.id)
        .token_name("aaaa")
        .token_symbol("A")
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::TokenIsImmutable, .. })
    );

    // can't delete the account because the token still exists, can't delete the token because there's no admin key.

    Ok(())
}

#[tokio::test]
async fn update_immutable_token_metadata() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };
    let initial_metadata = vec![1];
    let updated_metadata = vec![1, 2];
    let metadata_key = PrivateKey::generate_ed25519();

    // Create the Fungible Token with metadata key.
    let token_id = TokenCreateTransaction::new()
        .name("ffff")
        .symbol("F")
        .token_type(TokenType::FungibleCommon)
        .decimals(3)
        .initial_supply(100000)
        .metadata(initial_metadata.clone())
        .treasury_account_id(client.get_operator_account_id().unwrap())
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .admin_key(client.get_operator_public_key().unwrap())
        .metadata_key(metadata_key.public_key())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    let token_info = TokenInfoQuery::new().token_id(token_id).execute(&client).await?;

    assert_eq!(&token_info.metadata, &initial_metadata);
    assert_eq!(token_info.metadata_key, Some(Key::Single(metadata_key.public_key())));

    // Update token with metadata key.
    _ = TokenUpdateTransaction::new()
        .token_id(token_id)
        .metadata(updated_metadata.clone())
        .freeze_with(&client)?
        .sign(metadata_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    let token_info = TokenInfoQuery::new().token_id(token_id).execute(&client).await?;

    assert_eq!(token_info.metadata, updated_metadata);

    Ok(())
}

// HIP-540 (https://hips.hedera.com/hip/hip-540)
// Make a token immutable when updating keys to an empty KeyList, signing with an Admin Key,
// and setting the key verification mode to NO_VALIDATION
#[tokio::test]
async fn update_keys_with_admin_sig() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let admin_key = PrivateKey::generate_ed25519();

    // Admin, Freeze, Wipe, Kyc, Supply, Pause, Fee Schedule, and Metadata keys.
    let keys = generate_keys(Some(admin_key));

    // Create the token with all keys.
    let token_id = create_token_with_keys(&client, &keys).await?;

    let empty_keylist = KeyList::new();

    // Update all lower-privilege keys for token with empty key list,
    // signing with admin key, and verifying with no validation.
    _ = TokenUpdateTransaction::new()
        .token_id(token_id)
        .admin_key(empty_keylist.clone())
        .wipe_key(empty_keylist.clone())
        .freeze_key(empty_keylist.clone())
        .kyc_key(empty_keylist.clone())
        .supply_key(empty_keylist.clone())
        .pause_key(empty_keylist.clone())
        .fee_schedule_key(empty_keylist.clone())
        .metadata_key(empty_keylist.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .freeze_with(&client)?
        .sign(keys.admin_key.unwrap().to_owned())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let token_info = TokenInfoQuery::new().token_id(token_id).execute(&client).await?;

    assert_eq!(token_info.admin_key, None);
    assert_eq!(token_info.freeze_key, None);
    assert_eq!(token_info.wipe_key, None);
    assert_eq!(token_info.kyc_key, None);
    assert_eq!(token_info.supply_key, None);
    assert_eq!(token_info.pause_key, None);
    assert_eq!(token_info.fee_schedule_key, None);
    assert_eq!(token_info.metadata_key, None);

    _ = TokenDeleteTransaction::new().token_id(token_id).execute(&client).await?;

    Ok(())
}

// HIP-540 (https://hips.hedera.com/hip/hip-540)
// Can remove all of token’s lower-privilege keys when updating keys to an empty KeyList,
// signing with an Admin Key, and setting the key verification mode to FULL_VALIDATION
#[tokio::test]
async fn remove_keys_with_admin_sig() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let admin_key = PrivateKey::generate_ed25519();

    // Admin, Freeze, Wipe, Kyc, Supply, Pause, Fee Schedule, and Metadata keys.
    let keys = generate_keys(Some(admin_key));

    // Create the token with all keys.
    let token_id = create_token_with_keys(&client, &keys).await?;

    let empty_keylist = KeyList::new();

    // Update all lower-privilege keys for token with empty key list,
    // signing with admin key, and verifying with full validation.
    _ = TokenUpdateTransaction::new()
        .token_id(token_id)
        .admin_key(empty_keylist.clone())
        .wipe_key(empty_keylist.clone())
        .freeze_key(empty_keylist.clone())
        .kyc_key(empty_keylist.clone())
        .supply_key(empty_keylist.clone())
        .pause_key(empty_keylist.clone())
        .fee_schedule_key(empty_keylist.clone())
        .metadata_key(empty_keylist.clone())
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.admin_key.unwrap())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let token_info = TokenInfoQuery::new().token_id(token_id).execute(&client).await?;

    assert_eq!(token_info.admin_key, None);
    assert_eq!(token_info.freeze_key, None);
    assert_eq!(token_info.wipe_key, None);
    assert_eq!(token_info.kyc_key, None);
    assert_eq!(token_info.supply_key, None);
    assert_eq!(token_info.pause_key, None);
    assert_eq!(token_info.fee_schedule_key, None);
    assert_eq!(token_info.metadata_key, None);

    _ = TokenDeleteTransaction::new().token_id(token_id).execute(&client).await?;
    Ok(())
}

// HIP-540 (https://hips.hedera.com/hip/hip-540)
// Can update all of token’s lower-privilege keys to an unusable key (i.e. all-zeros key),
// when signing with an Admin Key, and setting the key verification mode to FULL_VALIDATION, and then revert previous keys
#[tokio::test]
async fn revert_keys_with_admin_sig() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let admin_key = PrivateKey::generate_ed25519();

    // Admin, Freeze, Wipe, Kyc, Supply, Pause, Fee Schedule, and Metadata keys.
    let keys = generate_keys(Some(admin_key));

    // Create the token with all keys.
    let token_id = create_token_with_keys(&client, &keys).await?;

    // Generate an unusable key to update the supply key.
    let unusable_key = PublicKey::from_str_ed25519(
        "0x0000000000000000000000000000000000000000000000000000000000000000",
    )
    .unwrap();

    // Update all lower-privilege keys for token with invalid zeros key,
    // signing with admin key, and verifying with full validation.
    _ = TokenUpdateTransaction::new()
        .token_id(token_id)
        .wipe_key(unusable_key)
        .freeze_key(unusable_key)
        .kyc_key(unusable_key)
        .supply_key(unusable_key)
        .pause_key(unusable_key)
        .fee_schedule_key(unusable_key)
        .metadata_key(unusable_key)
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.admin_key.as_ref().unwrap().clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let token_info = TokenInfoQuery::new().token_id(token_id).execute(&client).await?;

    assert_eq!(token_info.freeze_key, Some(Key::Single(unusable_key)));
    assert_eq!(token_info.wipe_key, Some(Key::Single(unusable_key)));
    assert_eq!(token_info.kyc_key, Some(Key::Single(unusable_key)));
    assert_eq!(token_info.supply_key, Some(Key::Single(unusable_key)));
    assert_eq!(token_info.pause_key, Some(Key::Single(unusable_key)));
    assert_eq!(token_info.fee_schedule_key, Some(Key::Single(unusable_key)));
    assert_eq!(token_info.metadata_key, Some(Key::Single(unusable_key)));

    // Set all lower-privilege keys back to their original values,
    // signing with admin key, and verifying with no validation.
    _ = TokenUpdateTransaction::new()
        .token_id(token_id)
        .wipe_key(keys.wipe_key.public_key())
        .freeze_key(keys.freeze_key.public_key())
        .kyc_key(keys.kyc_key.public_key())
        .supply_key(keys.supply_key.public_key())
        .pause_key(keys.pause_key.public_key())
        .fee_schedule_key(keys.fee_schedule_key.public_key())
        .metadata_key(keys.metadata_key.public_key())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .freeze_with(&client)?
        .sign(keys.admin_key.as_ref().unwrap().clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let token_info = TokenInfoQuery::new().token_id(token_id).execute(&client).await?;

    assert_eq!(
        token_info.admin_key,
        Some(Key::Single(keys.admin_key.unwrap().clone().public_key()))
    );
    assert_eq!(token_info.freeze_key, Some(Key::Single(keys.freeze_key.public_key())));
    assert_eq!(token_info.wipe_key, Some(Key::Single(keys.wipe_key.public_key())));
    assert_eq!(token_info.kyc_key, Some(Key::Single(keys.kyc_key.public_key())));
    assert_eq!(token_info.supply_key, Some(Key::Single(keys.supply_key.public_key())));
    assert_eq!(token_info.pause_key, Some(Key::Single(keys.pause_key.public_key())));
    assert_eq!(token_info.fee_schedule_key, Some(Key::Single(keys.fee_schedule_key.public_key())));
    assert_eq!(token_info.metadata_key, Some(Key::Single(keys.metadata_key.public_key())));

    _ = TokenDeleteTransaction::new().token_id(token_id).execute(&client).await?;
    Ok(())
}

// HIP-540 (https://hips.hedera.com/hip/hip-540)
// Can update all of token’s lower-privilege keys when signing with an Admin Key
// and new respective lower-privilege key, and setting key verification mode to FULL_VALIDATION
#[tokio::test]
async fn update_low_privilege_keys_with_admin_sig() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let admin_key = PrivateKey::generate_ed25519();

    // Freeze, Wipe, Kyc, Supply, Pause, Fee Schedule, and Metadata keys.
    let keys = generate_keys(Some(admin_key));

    // New Freeze, Wipe, Kyc, Supply, Pause, Fee Schedule, and Metadata keys.
    let new_keys = generate_keys(None);

    // Create the NFT with all of token’s lower-privilege keys.
    let token_id = create_token_with_keys(&client, &keys).await?;

    // Update all lower-privilege keys for token with new lower-privilege keys,
    // signing with admin key and new lower-privilege keys, and verifying with full validation.
    _ = TokenUpdateTransaction::new()
        .token_id(token_id)
        .wipe_key(new_keys.wipe_key.public_key())
        .freeze_key(new_keys.freeze_key.public_key())
        .kyc_key(new_keys.kyc_key.public_key())
        .supply_key(new_keys.supply_key.public_key())
        .pause_key(new_keys.pause_key.public_key())
        .fee_schedule_key(new_keys.fee_schedule_key.public_key())
        .metadata_key(new_keys.metadata_key.public_key())
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.admin_key.unwrap())
        .sign(new_keys.wipe_key.clone())
        .sign(new_keys.freeze_key.clone())
        .sign(new_keys.kyc_key.clone())
        .sign(new_keys.supply_key.clone())
        .sign(new_keys.pause_key.clone())
        .sign(new_keys.fee_schedule_key.clone())
        .sign(new_keys.metadata_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let token_info = TokenInfoQuery::new().token_id(token_id).execute(&client).await?;

    assert_eq!(token_info.freeze_key, Some(Key::Single(new_keys.freeze_key.public_key())));
    assert_eq!(token_info.wipe_key, Some(Key::Single(new_keys.wipe_key.public_key())));
    assert_eq!(token_info.kyc_key, Some(Key::Single(new_keys.kyc_key.public_key())));
    assert_eq!(token_info.supply_key, Some(Key::Single(new_keys.supply_key.public_key())));
    assert_eq!(token_info.pause_key, Some(Key::Single(new_keys.pause_key.public_key())));
    assert_eq!(
        token_info.fee_schedule_key,
        Some(Key::Single(new_keys.fee_schedule_key.public_key()))
    );
    assert_eq!(token_info.metadata_key, Some(Key::Single(new_keys.metadata_key.public_key())));

    _ = TokenDeleteTransaction::new().token_id(token_id).execute(&client).await?;

    _ = TokenDeleteTransaction::new().token_id(token_id).execute(&client).await?;
    Ok(())
}

// HIP-540 (https://hips.hedera.com/hip/hip-540)
// Cannot make a token immutable when updating keys to an empty KeyList,
// signing with a key that is different from an Admin Key, and setting the key verification mode to NO_VALIDATION
#[tokio::test]
async fn update_keys_empty_keylist_without_admin_sig_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let admin_key = PrivateKey::generate_ed25519();

    // Admin (if required), Freeze, Wipe, Kyc, Supply, Pause, Fee Schedule, and Metadata keys.
    let keys = generate_keys(Some(admin_key));

    // Create the token with all keys.
    let token_id = create_token_with_keys(&client, &keys).await?;

    let empty_keylist = KeyList::new();

    // Fails to update the immutable token keys to empty keylist without admin signature (sign implicitly with operator key).
    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .wipe_key(empty_keylist.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .kyc_key(empty_keylist.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .freeze_key(empty_keylist.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .pause_key(empty_keylist.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .supply_key(empty_keylist.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .fee_schedule_key(empty_keylist.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .metadata_key(empty_keylist.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .admin_key(empty_keylist.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    _ = TokenDeleteTransaction::new().token_id(token_id).execute(&client).await?;
    Ok(())
}

// HIP-540 (https://hips.hedera.com/hip/hip-540)
// Cannot make a token immutable when updating keys to an unusable key (i.e. all-zeros key),
// signing with a key that is different from an Admin Key, and setting the key verification mode to NO_VALIDATION
#[tokio::test]
async fn update_keys_unusable_key_without_admin_sig_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let admin_key = PrivateKey::generate_ed25519();

    // Admin, Freeze, Wipe, Kyc, Supply, Pause, Fee Schedule, and Metadata keys.
    let keys = generate_keys(Some(admin_key));

    // Create the token with all keys.
    let token_id = create_token_with_keys(&client, &keys).await?;

    // Generate an unusable key.
    let unusable_key = PublicKey::from_str_ed25519(
        "0x0000000000000000000000000000000000000000000000000000000000000000",
    )
    .unwrap();

    // Fails to update the immutable token keys to unusable key  without admin signature (sign implicitly with operator key).
    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .wipe_key(unusable_key.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .kyc_key(unusable_key.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .freeze_key(unusable_key.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .pause_key(unusable_key.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .supply_key(unusable_key.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .fee_schedule_key(unusable_key.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .metadata_key(unusable_key.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .admin_key(unusable_key.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    _ = TokenDeleteTransaction::new().token_id(token_id).execute(&client).await?;
    Ok(())
}

// HIP-540 (https://hips.hedera.com/hip/hip-540)
// Cannot update the Admin Key to an unusable key (i.e. all-zeros key),
// signing with an Admin Key, and setting the key verification mode to NO_VALIDATION
#[tokio::test]
async fn update_admin_key_to_usuable_key_fail() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Admin and Freeze keys
    let admin_key = PrivateKey::generate_ed25519();
    let supply_key = PrivateKey::generate_ed25519();

    // Create the NFT with admin and supply key.
    let token_id = TokenCreateTransaction::new()
        .name("Test NFT")
        .symbol("TNFT")
        .token_type(TokenType::NonFungibleUnique)
        .treasury_account_id(client.get_operator_account_id().unwrap())
        .admin_key(admin_key.public_key())
        .supply_key(supply_key.public_key())
        .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
        .freeze_with(&client)?
        .sign(admin_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?
        .token_id
        .unwrap();

    let token_info = TokenInfoQuery::new().token_id(token_id).execute(&client).await?;

    assert_eq!(token_info.admin_key, Some(Key::Single(admin_key.public_key())));

    // Generate an unusable key.
    let unusable_key = PublicKey::from_str_ed25519(
        "0x0000000000000000000000000000000000000000000000000000000000000000",
    )
    .unwrap();

    // Update the Admin Key to an unusable key (i.e., all-zeros key),
    // signing with an Admin Key, and setting the key verification mode to NO_VALIDATION
    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .admin_key(unusable_key.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .freeze_with(&client)?
        .sign(admin_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    _ = TokenDeleteTransaction::new().token_id(token_id).execute(&client).await?;
    Ok(())
}

// HIP-540 (https://hips.hedera.com/hip/hip-540)
// Can update all of token’s lower-privilege keys to an unusable key (i.e. all-zeros key),
// when signing with a respective lower-privilege key, and setting the key verification mode to NO_VALIDATION
#[tokio::test]
async fn update_keys_with_lower_privilege_keys_sigs() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };
    // Freeze, Wipe, Kyc, Supply, Pause, Fee Schedule, and Metadata keys.
    let keys = generate_keys(None);

    // Create the NFT with all of token’s lower-privilege keys.
    let token_id = create_token_with_keys(&client, &keys).await?;

    // Generate an unusable key.
    let unusable_key = PublicKey::from_str_ed25519(
        "0x0000000000000000000000000000000000000000000000000000000000000000",
    )
    .unwrap();

    // Update all of token’s lower-privilege keys to an unusable key (i.e., all-zeros key),
    // when signing with a respective lower-privilege key,
    // and setting the key verification mode to NO_VALIDATION
    let _ = TokenUpdateTransaction::new()
        .token_id(token_id)
        .freeze_key(unusable_key)
        .wipe_key(unusable_key)
        .kyc_key(unusable_key)
        .supply_key(unusable_key)
        .pause_key(unusable_key)
        .fee_schedule_key(unusable_key)
        .metadata_key(unusable_key)
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .freeze_with(&client)?
        .sign(keys.freeze_key.clone())
        .sign(keys.wipe_key.clone())
        .sign(keys.kyc_key.clone())
        .sign(keys.supply_key.clone())
        .sign(keys.pause_key.clone())
        .sign(keys.fee_schedule_key.clone())
        .sign(keys.metadata_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    let token_info = TokenInfoQuery::new().token_id(token_id).execute(&client).await?;

    assert_eq!(token_info.freeze_key, Some(Key::Single(unusable_key)));
    assert_eq!(token_info.wipe_key, Some(Key::Single(unusable_key)));
    assert_eq!(token_info.kyc_key, Some(Key::Single(unusable_key)));
    assert_eq!(token_info.supply_key, Some(Key::Single(unusable_key)));
    assert_eq!(token_info.pause_key, Some(Key::Single(unusable_key)));
    assert_eq!(token_info.fee_schedule_key, Some(Key::Single(unusable_key)));
    assert_eq!(token_info.metadata_key, Some(Key::Single(unusable_key)));

    _ = TokenDeleteTransaction::new().token_id(token_id).execute(&client).await?;
    Ok(())
}

// HIP-540 (https://hips.hedera.com/hip/hip-540)
// Can update all of token’s lower-privilege keys when signing with an old lower-privilege key
// and with a new lower-privilege key, and setting key verification mode to FULL_VALIDATION
#[tokio::test]
async fn update_keys_with_new_and_old_lower_privilege_keys_sigs() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Freeze, Wipe, Kyc, Supply, Pause, Fee Schedule, and Metadata keys.
    let keys = generate_keys(None);

    // New Freeze, Wipe, Kyc, Supply, Pause, Fee Schedule, and Metadata keys.
    let new_keys = generate_keys(None);

    // Create the NFT with all of token’s lower-privilege keys.
    let token_id = create_token_with_keys(&client, &keys).await?;

    // Update all of token’s lower-privilege keys when signing with an old respective lower-privilege key,
    // and setting key verification mode to NO_VALIDATION
    let _ = TokenUpdateTransaction::new()
        .token_id(token_id)
        .freeze_key(new_keys.freeze_key.public_key())
        .wipe_key(new_keys.wipe_key.public_key())
        .kyc_key(new_keys.kyc_key.public_key())
        .supply_key(new_keys.supply_key.public_key())
        .pause_key(new_keys.pause_key.public_key())
        .fee_schedule_key(new_keys.fee_schedule_key.public_key())
        .metadata_key(new_keys.metadata_key.public_key())
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.freeze_key.clone())
        .sign(keys.kyc_key.clone())
        .sign(keys.wipe_key.clone())
        .sign(keys.supply_key.clone())
        .sign(keys.pause_key.clone())
        .sign(keys.fee_schedule_key.clone())
        .sign(keys.metadata_key.clone())
        .sign(new_keys.freeze_key.clone())
        .sign(new_keys.kyc_key.clone())
        .sign(new_keys.wipe_key.clone())
        .sign(new_keys.supply_key.clone())
        .sign(new_keys.pause_key.clone())
        .sign(new_keys.fee_schedule_key.clone())
        .sign(new_keys.metadata_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    let token_info = TokenInfoQuery::new().token_id(token_id).execute(&client).await?;

    assert_eq!(token_info.freeze_key, Some(Key::Single(new_keys.freeze_key.public_key())));
    assert_eq!(token_info.wipe_key, Some(Key::Single(new_keys.wipe_key.public_key())));
    assert_eq!(token_info.kyc_key, Some(Key::Single(new_keys.kyc_key.public_key())));
    assert_eq!(token_info.supply_key, Some(Key::Single(new_keys.supply_key.public_key())));
    assert_eq!(token_info.pause_key, Some(Key::Single(new_keys.pause_key.public_key())));
    assert_eq!(
        token_info.fee_schedule_key,
        Some(Key::Single(new_keys.fee_schedule_key.public_key()))
    );
    assert_eq!(token_info.metadata_key, Some(Key::Single(new_keys.metadata_key.public_key())));

    _ = TokenDeleteTransaction::new().token_id(token_id).execute(&client).await?;
    Ok(())
}

// HIP-540 (https://hips.hedera.com/hip/hip-540)
// Can update all of token’s lower-privilege keys when signing ONLY with an old lower-privilege key,
// and setting key verification mode to NO_VALIDATION
#[tokio::test]
async fn update_keys_with_all_old_lower_privilege_keys_sigs() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Freeze, Wipe, Kyc, Supply, Pause, Fee Schedule, and Metadata keys.
    let keys = generate_keys(None);

    // New Freeze, Wipe, Kyc, Supply, Pause, Fee Schedule, and Metadata keys.
    let new_keys = generate_keys(None);

    // Create the NFT with all of token’s lower-privilege keys.
    let token_id = create_token_with_keys(&client, &keys).await?;

    // Update all of token’s lower-privilege keys when signing with all older respective lower-privilege keys,
    // and setting key verification mode to NO_VALIDATION
    let _ = TokenUpdateTransaction::new()
        .token_id(token_id)
        .freeze_key(new_keys.freeze_key.public_key())
        .wipe_key(new_keys.wipe_key.public_key())
        .kyc_key(new_keys.kyc_key.public_key())
        .supply_key(new_keys.supply_key.public_key())
        .pause_key(new_keys.pause_key.public_key())
        .fee_schedule_key(new_keys.fee_schedule_key.public_key())
        .metadata_key(new_keys.metadata_key.public_key())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .freeze_with(&client)?
        .sign(keys.freeze_key.clone())
        .sign(keys.kyc_key.clone())
        .sign(keys.wipe_key.clone())
        .sign(keys.supply_key.clone())
        .sign(keys.pause_key.clone())
        .sign(keys.fee_schedule_key.clone())
        .sign(keys.metadata_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    let token_info = TokenInfoQuery::new().token_id(token_id).execute(&client).await?;

    assert_eq!(token_info.freeze_key, Some(Key::Single(new_keys.freeze_key.public_key())));
    assert_eq!(token_info.wipe_key, Some(Key::Single(new_keys.wipe_key.public_key())));
    assert_eq!(token_info.kyc_key, Some(Key::Single(new_keys.kyc_key.public_key())));
    assert_eq!(token_info.supply_key, Some(Key::Single(new_keys.supply_key.public_key())));
    assert_eq!(token_info.pause_key, Some(Key::Single(new_keys.pause_key.public_key())));
    assert_eq!(
        token_info.fee_schedule_key,
        Some(Key::Single(new_keys.fee_schedule_key.public_key()))
    );
    assert_eq!(token_info.metadata_key, Some(Key::Single(new_keys.metadata_key.public_key())));

    _ = TokenDeleteTransaction::new().token_id(token_id).execute(&client).await?;
    Ok(())
}

// HIP-540 (https://hips.hedera.com/hip/hip-540)
// Cannot remove all of token’s lower-privilege keys when updating them to an empty KeyList,
// signing with a respective lower-privilege key, and setting the key verification mode to NO_VALIDATION
#[tokio::test]
async fn remove_empty_keylist_keys_lower_privilege_keys_sigs_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Freeze, Wipe, Kyc, Supply, Pause, Fee Schedule, and Metadata keys.
    let keys = generate_keys(None);

    // Create the NFT with all of token’s lower-privilege keys.
    let token_id = create_token_with_keys(&client, &keys).await?;

    let empty_keylist = KeyList::new();

    // Remove all of token’s lower-privilege keys
    // when updating them to an empty KeyList (trying to remove keys one by one to check all errors),
    // signing with a respective lower-privilege key,
    // and setting the key verification mode to NO_VALIDATION
    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .wipe_key(empty_keylist.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .freeze_with(&client)?
        .sign(keys.wipe_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::TokenIsImmutable, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .kyc_key(empty_keylist.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .freeze_with(&client)?
        .sign(keys.kyc_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::TokenIsImmutable, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .freeze_key(empty_keylist.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .freeze_with(&client)?
        .sign(keys.freeze_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::TokenIsImmutable, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .pause_key(empty_keylist.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .freeze_with(&client)?
        .sign(keys.pause_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::TokenIsImmutable, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .supply_key(empty_keylist.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .freeze_with(&client)?
        .sign(keys.supply_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::TokenIsImmutable, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .fee_schedule_key(empty_keylist.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .freeze_with(&client)?
        .sign(keys.fee_schedule_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::TokenIsImmutable, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .metadata_key(empty_keylist.clone())
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .freeze_with(&client)?
        .sign(keys.metadata_key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::TokenIsImmutable, transaction_id: _ })
    );

    _ = TokenDeleteTransaction::new().token_id(token_id).execute(&client).await?;
    Ok(())
}

// HIP-540 (https://hips.hedera.com/hip/hip-540)
// Cannot update all of token’s lower-privilege keys to an unusable key (i.e. all-zeros key),
// when signing with a key that is different from a respective lower-privilege key, and setting
// the key verification mode to NO_VALIDATION
#[tokio::test]
async fn update_keys_unusable_key_different_key_sig_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Freeze, Wipe, Kyc, Supply, Pause, Fee Schedule, and Metadata keys.
    let keys = generate_keys(None);

    // Create the NFT with all of token’s lower-privilege keys.
    let token_id = create_token_with_keys(&client, &keys).await?;

    // Generate an unusable key.
    let unusable_key = PublicKey::from_str_ed25519(
        "0x0000000000000000000000000000000000000000000000000000000000000000",
    )
    .unwrap();

    // Update all of token’s lower-privilege keys to an unusable key (i.e. all-zeros key)
    // (trying to remove keys one by one to check all errors),
    // signing with a key that is different from a respective lower-privilege key (implicitly with an operator key),
    // and setting the key verification mode to NO_VALIDATION
    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .wipe_key(unusable_key)
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .kyc_key(unusable_key)
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .freeze_key(unusable_key)
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .pause_key(unusable_key)
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .supply_key(unusable_key)
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .fee_schedule_key(unusable_key)
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .metadata_key(unusable_key)
        .key_verification_mode(TokenKeyValidation::NoValidation)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    _ = TokenDeleteTransaction::new().token_id(token_id).execute(&client).await?;
    Ok(())
}

// HIP-540 (https://hips.hedera.com/hip/hip-540)
// Cannot update all of token’s lower-privilege keys to an unusable key (i.e. all-zeros key),
// when signing ONLY with an old respective lower-privilege key, and setting the key
// verification mode to FULL_VALIDATION
#[tokio::test]
async fn update_with_unusable_key_with_old_key_sig_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Freeze, Wipe, Kyc, Supply, Pause, Fee Schedule, and Metadata keys.
    let keys = generate_keys(None);

    // Create the NFT with all of token’s lower-privilege keys.
    let token_id = create_token_with_keys(&client, &keys).await?;

    // Generate an unusable key.
    let unusable_key = PublicKey::from_str_ed25519(
        "0x0000000000000000000000000000000000000000000000000000000000000000",
    )
    .unwrap();

    // Update all of token’s lower-privilege keys to an unusable key (i.e., all-zeros key)
    // (trying to remove keys one by one to check all errors),
    // signing ONLY with an old respective lower-privilege key,
    // and setting the key verification mode to FULL_VALIDATION
    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .wipe_key(unusable_key)
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.wipe_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .kyc_key(unusable_key)
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.kyc_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .freeze_key(unusable_key)
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.freeze_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .pause_key(unusable_key)
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.pause_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .supply_key(unusable_key)
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.supply_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .fee_schedule_key(unusable_key)
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.fee_schedule_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .metadata_key(unusable_key)
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.metadata_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    _ = TokenDeleteTransaction::new().token_id(token_id).execute(&client).await?;
    Ok(())
}

// HIP-540 (https://hips.hedera.com/hip/hip-540)
// Cannot update all of token’s lower-privilege keys to an unusable key (i.e. all-zeros key),
// when signing with an old respective lower-privilege key and new respective lower-privilege key,
// and setting the key verification mode to FULL_VALIDATION
#[tokio::test]
async fn update_unusable_key_old_new_key_sig_full_validation_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Freeze, Wipe, Kyc, Supply, Pause, Fee Schedule, and Metadata keys.
    let keys = generate_keys(None);

    // New Freeze, Wipe, Kyc, Supply, Pause, Fee Schedule, and Metadata keys.
    let new_keys = generate_keys(None);

    // Create the NFT with all of token’s lower-privilege keys.
    let token_id = create_token_with_keys(&client, &keys).await?;

    // Generate an unusable key.
    let unusable_key = PublicKey::from_str_ed25519(
        "0x0000000000000000000000000000000000000000000000000000000000000000",
    )
    .unwrap();

    // Update all of token’s lower-privilege keys to an unusable key (i.e., all-zeros key)
    // (trying to remove keys one by one to check all errors),
    // signing with an old respective lower-privilege key and new respective lower-privilege key,
    // and setting the key verification mode to FULL_VALIDATION
    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .wipe_key(unusable_key)
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.wipe_key)
        .sign(new_keys.wipe_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .kyc_key(unusable_key)
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.kyc_key)
        .sign(new_keys.kyc_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .freeze_key(unusable_key)
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.freeze_key)
        .sign(new_keys.freeze_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .pause_key(unusable_key)
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.pause_key)
        .sign(new_keys.pause_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .supply_key(unusable_key)
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.supply_key)
        .sign(new_keys.supply_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .fee_schedule_key(unusable_key)
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.fee_schedule_key)
        .sign(new_keys.fee_schedule_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .metadata_key(unusable_key)
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.metadata_key)
        .sign(new_keys.metadata_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    _ = TokenDeleteTransaction::new().token_id(token_id).execute(&client).await?;
    Ok(())
}

// HIP-540 (https://hips.hedera.com/hip/hip-540)
// Cannot update all of token’s lower-privilege keys, when signing ONLY with an
// old respective lower-privilege key, and setting the key verification mode to
// FULL_VALIDATION
#[tokio::test]
async fn update_keys_old_key_sig_full_validation_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Freeze, Wipe, Kyc, Supply, Pause, Fee Schedule, and Metadata keys.
    let keys = generate_keys(None);

    // New Freeze, Wipe, Kyc, Supply, Pause, Fee Schedule, and Metadata keys.
    let new_keys = generate_keys(None);

    // Create the NFT with all of token’s lower-privilege keys.
    let token_id = create_token_with_keys(&client, &keys).await?;

    // Update all of token’s lower-privilege keys
    // (trying to update keys one by one to check all errors),
    // signing ONLY with an old respective lower-privilege key,
    // and setting the key verification mode to FULL_VALIDATION
    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .wipe_key(new_keys.wipe_key.public_key())
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.wipe_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .kyc_key(new_keys.kyc_key.public_key())
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.kyc_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .freeze_key(new_keys.freeze_key.public_key())
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.freeze_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .pause_key(new_keys.pause_key.public_key())
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.pause_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .supply_key(new_keys.supply_key.public_key())
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.supply_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .fee_schedule_key(new_keys.fee_schedule_key.public_key())
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.fee_schedule_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    let tx = TokenUpdateTransaction::new()
        .token_id(token_id)
        .metadata_key(new_keys.metadata_key.public_key())
        .key_verification_mode(TokenKeyValidation::FullValidation)
        .freeze_with(&client)?
        .sign(keys.metadata_key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        tx,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    _ = TokenDeleteTransaction::new().token_id(token_id).execute(&client).await?;
    Ok(())
}

struct Keys {
    admin_key: Option<PrivateKey>,
    wipe_key: PrivateKey,
    kyc_key: PrivateKey,
    freeze_key: PrivateKey,
    pause_key: PrivateKey,
    supply_key: PrivateKey,
    fee_schedule_key: PrivateKey,
    metadata_key: PrivateKey,
}

fn generate_keys(admin_key: Option<PrivateKey>) -> Keys {
    Keys {
        admin_key,
        wipe_key: PrivateKey::generate_ed25519(),
        kyc_key: PrivateKey::generate_ed25519(),
        freeze_key: PrivateKey::generate_ed25519(),
        pause_key: PrivateKey::generate_ed25519(),
        supply_key: PrivateKey::generate_ed25519(),
        fee_schedule_key: PrivateKey::generate_ed25519(),
        metadata_key: PrivateKey::generate_ed25519(),
    }
}

async fn create_token_with_keys(client: &Client, keys: &Keys) -> anyhow::Result<TokenId> {
    // Create the NFT with all of token’s lower-privilege keys.
    let token_id = {
        let mut tx = TokenCreateTransaction::new();

        tx.name("Test NFT")
            .symbol("TNFT")
            .token_type(TokenType::NonFungibleUnique)
            .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
            .treasury_account_id(client.get_operator_account_id().unwrap())
            .freeze_key(keys.freeze_key.public_key())
            .supply_key(keys.supply_key.public_key())
            .wipe_key(keys.wipe_key.public_key())
            .kyc_key(keys.kyc_key.public_key())
            .pause_key(keys.pause_key.public_key())
            .fee_schedule_key(keys.fee_schedule_key.public_key())
            .metadata_key(keys.metadata_key.public_key());

        if let Some(admin_key) = &keys.admin_key {
            tx.admin_key(admin_key.public_key()).freeze_with(client)?.sign(admin_key.clone());
        }

        tx.execute(&client).await?.get_receipt(&client).await?.token_id.unwrap()
    };

    let token_info: hiero_sdk::TokenInfo =
        TokenInfoQuery::new().token_id(token_id).execute(&client).await?;

    if let Some(admin_key) = &keys.admin_key {
        assert_eq!(token_info.admin_key, Some(Key::Single(admin_key.public_key())));
    } else {
        assert_eq!(token_info.admin_key, None);
    }

    assert_eq!(token_info.freeze_key, Some(Key::Single(keys.freeze_key.public_key())));
    assert_eq!(token_info.wipe_key, Some(Key::Single(keys.wipe_key.public_key())));
    assert_eq!(token_info.kyc_key, Some(Key::Single(keys.kyc_key.public_key())));
    assert_eq!(token_info.supply_key, Some(Key::Single(keys.supply_key.public_key())));
    assert_eq!(token_info.pause_key, Some(Key::Single(keys.pause_key.public_key())));
    assert_eq!(token_info.fee_schedule_key, Some(Key::Single(keys.fee_schedule_key.public_key())));
    assert_eq!(token_info.metadata_key, Some(Key::Single(keys.metadata_key.public_key())));

    Ok(token_id)
}
