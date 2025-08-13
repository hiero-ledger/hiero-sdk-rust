use std::str::FromStr;

use hedera::{
    AccountCreateTransaction,
    AccountId,
    BatchTransaction,
    Hbar,
    PrivateKey,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

#[tokio::test]
async fn can_execute_batch_transaction() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    // Given
    let operator_id = AccountId::new(0, 0, 2);
    let operator_key = PrivateKey::from_str(
        "302e020100300506032b65700422042091132178e72057a1d7528025956fe39b0b847f200ab59b2fdd367017f3087137"
    )?;

    let account_key = PrivateKey::generate_ed25519();

    let mut inner_transaction = AccountCreateTransaction::new();
    inner_transaction.set_key_without_alias(account_key.public_key()).initial_balance(Hbar::new(1));

    // Use batchify to prepare the transaction
    inner_transaction.batchify(&client, operator_key.public_key().into())?;

    // When / Then
    let mut batch_transaction = BatchTransaction::new();
    batch_transaction.add_inner_transaction(inner_transaction.into())?;

    client.set_operator(operator_id, operator_key);

    let tx_response = batch_transaction.execute(&client).await?;
    let _tx_receipt = tx_response.get_receipt(&client).await?;

    Ok(())
}
