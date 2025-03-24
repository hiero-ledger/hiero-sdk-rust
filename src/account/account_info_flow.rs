// SPDX-License-Identifier: Apache-2.0

use crate::transaction::TransactionExecute;
use crate::{
    AccountId,
    AccountInfoQuery,
    Client,
    Error,
    Key,
    PublicKey,
    Transaction,
};

async fn query_pk(client: &Client, account_id: AccountId) -> crate::Result<PublicKey> {
    let key = AccountInfoQuery::new().account_id(account_id).execute(client).await?.key;

    match key {
        Key::Single(it) => Ok(it),
        _ => {
            Err(Error::signature_verify("`{account_id}`: unsupported key kind: {key:?}".to_owned()))
        }
    }
}

/// Verify the `signature` for `msg` via the given account's public key.
///
/// # Errors
/// - [`Error::SignatureVerify`] if the signature algorithm doesn't match the account's public key.
/// - [`Error::SignatureVerify`] if the signature is invalid for the account's public key.
/// - See [`AccountInfoQuery::execute`]
pub async fn verify_signature(
    client: &Client,
    account_id: AccountId,
    msg: &[u8],
    signature: &[u8],
) -> crate::Result<()> {
    let key = query_pk(client, account_id).await?;

    key.verify(msg, signature)
}

/// Returns `Ok(())` if the given account's public key has signed the given transaction.
/// # Errors
/// - [`Error::SignatureVerify`] if the private key associated with the account's public key did _not_ sign this transaction,
///   or the signature associated was invalid.
/// - See [`AccountInfoQuery::execute`]
pub async fn verify_transaction_signature<D: TransactionExecute>(
    client: &Client,
    account_id: AccountId,
    transaction: &mut Transaction<D>,
) -> crate::Result<()> {
    let key = query_pk(client, account_id).await?;

    key.verify_transaction(transaction)
}
