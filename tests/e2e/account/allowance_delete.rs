use hiero_sdk::{
    AccountAllowanceApproveTransaction,
    AccountAllowanceDeleteTransaction,
    Hbar,
    TokenAssociateTransaction,
    TokenNftInfoQuery,
    TransactionId,
    TransferTransaction,
};

use crate::account::Account;
use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

#[tokio::test]
async fn transfer_after_allowance_remove_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let treasury = Account::create(Hbar::new(0), &client).await?;

    let (spender, receiver) = tokio::try_join!(
        Account::create(Hbar::new(1), &client),
        Account::create(Hbar::new(0), &client),
    )?;

    let nft_collection = crate::token::Nft::create(&client, &treasury).await?;

    TokenAssociateTransaction::new()
        .account_id(receiver.id)
        .token_ids([nft_collection.id])
        .sign(receiver.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let serials = nft_collection.mint(&client, [b"nft1", b"nft2"]).await?;

    let nft1 = nft_collection.id.nft(serials[0] as u64);
    let nft2 = nft_collection.id.nft(serials[1] as u64);

    AccountAllowanceApproveTransaction::new()
        .approve_token_nft_allowance(nft1, treasury.id, spender.id)
        .approve_token_nft_allowance(nft2, treasury.id, spender.id)
        .sign(treasury.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    AccountAllowanceDeleteTransaction::new()
        .delete_all_token_nft_allowances(nft2, treasury.id)
        .sign(treasury.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    TransferTransaction::new()
        .approved_nft_transfer(nft1, treasury.id, receiver.id)
        .transaction_id(TransactionId::generate(spender.id))
        .sign(spender.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info = TokenNftInfoQuery::new().nft_id(nft1).execute(&client).await?;

    assert_eq!(info.account_id, receiver.id);

    let res = TransferTransaction::new()
        .approved_nft_transfer(nft2, treasury.id, receiver.id)
        .transaction_id(TransactionId::generate(spender.id))
        .sign(spender.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches::assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus {
            status: hiero_sdk::Status::SpenderDoesNotHaveAllowance,
            ..
        })
    );

    TransferTransaction::new()
        .nft_transfer(nft1, receiver.id, treasury.id)
        .sign(receiver.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    nft_collection.burn(&client, serials).await?;
    nft_collection.delete(&client).await?;

    let _ = tokio::try_join!(
        treasury.delete(&client),
        spender.delete(&client),
        receiver.delete(&client),
    )?;

    Ok(())
}

#[tokio::test]
async fn remove_single_serial_when_allowance_is_given_for_all_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let treasury = Account::create(Hbar::new(0), &client).await?;

    let (spender, receiver) = tokio::try_join!(
        Account::create(Hbar::new(1), &client),
        Account::create(Hbar::new(0), &client),
    )?;

    let nft_collection = crate::token::Nft::create(&client, &treasury).await?;

    TokenAssociateTransaction::new()
        .account_id(receiver.id)
        .token_ids([nft_collection.id])
        .sign(receiver.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let serials = nft_collection.mint(&client, [b"nft1", b"nft2"]).await?;

    let nft1 = nft_collection.id.nft(serials[0] as u64);
    let nft2 = nft_collection.id.nft(serials[1] as u64);

    AccountAllowanceApproveTransaction::new()
        .approve_token_nft_allowance_all_serials(nft_collection.id, treasury.id, spender.id)
        .sign(treasury.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    TransferTransaction::new()
        .approved_nft_transfer(nft1, treasury.id, receiver.id)
        .transaction_id(TransactionId::generate(spender.id))
        .sign(spender.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    // hopefully in the future this should end up with a precheck error provided from services
    AccountAllowanceDeleteTransaction::new()
        .delete_all_token_nft_allowances(nft2, treasury.id)
        .sign(treasury.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    TransferTransaction::new()
        .approved_nft_transfer(nft2, treasury.id, receiver.id)
        .transaction_id(TransactionId::generate(spender.id))
        .sign(spender.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let info_nft_1 = TokenNftInfoQuery::new().nft_id(nft1).execute(&client).await?;
    let info_nft_2 = TokenNftInfoQuery::new().nft_id(nft2).execute(&client).await?;

    assert_eq!(info_nft_1.account_id, receiver.id);
    assert_eq!(info_nft_2.account_id, receiver.id);

    TransferTransaction::new()
        .nft_transfer(nft1, receiver.id, treasury.id)
        .nft_transfer(nft2, receiver.id, treasury.id)
        .sign(receiver.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    nft_collection.burn(&client, serials).await?;
    nft_collection.delete(&client).await?;

    receiver.delete(&client).await?;
    spender.delete(&client).await?;
    treasury.delete(&client).await?;

    Ok(())
}
