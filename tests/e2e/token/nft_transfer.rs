use assert_matches::assert_matches;
use hiero_sdk::{
    Hbar,
    Status,
    TokenAssociateTransaction,
    TransferTransaction,
};

use crate::account::Account;
use crate::common::{
    setup_nonfree,
    TestEnvironment,
};
use crate::token::Nft;

#[tokio::test]
async fn basic() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let (alice, bob) = tokio::try_join!(
        Account::create(Hbar::new(0), &client),
        Account::create(Hbar::new(0), &client)
    )?;

    let token = Nft::create(&client, &alice).await?;

    let associate_fut = async {
        TokenAssociateTransaction::new()
            .account_id(bob.id)
            .token_ids([token.id])
            .sign(bob.key.clone())
            .execute(&client)
            .await?
            .get_receipt(&client)
            .await?;

        Ok(())
    };

    let (serials, _) = tokio::try_join!(token.mint_incremental(&client, 10), associate_fut)?;

    let mut transfer_tx = TransferTransaction::new();

    let serials_to_transfer = &serials[..4];

    for &serial in serials_to_transfer {
        transfer_tx.nft_transfer(token.id.nft(serial as u64), alice.id, bob.id);
    }

    transfer_tx.sign(alice.key.clone()).execute(&client).await?.get_receipt(&client).await?;

    let mut transfer_tx = TransferTransaction::new();

    for &serial in serials_to_transfer {
        transfer_tx.nft_transfer(token.id.nft(serial as u64), bob.id, alice.id);
    }

    transfer_tx.sign(bob.key.clone()).execute(&client).await?.get_receipt(&client).await?;

    token.burn(&client, serials).await?;
    token.delete(&client).await?;

    tokio::try_join!(alice.delete(&client), bob.delete(&client))?;

    Ok(())
}

#[tokio::test]

async fn unowned_nfts_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else {
        return Ok(());
    };

    let (alice, bob) = tokio::try_join!(
        Account::create(Hbar::new(0), &client),
        Account::create(Hbar::new(0), &client)
    )?;

    let token = Nft::create(&client, &alice).await?;

    let associate_fut = async {
        TokenAssociateTransaction::new()
            .account_id(bob.id)
            .token_ids([token.id])
            .sign(bob.key.clone())
            .execute(&client)
            .await?
            .get_receipt(&client)
            .await?;

        Ok(())
    };

    let (serials, _) = tokio::try_join!(token.mint_incremental(&client, 10), associate_fut)?;

    let mut transfer_tx = TransferTransaction::new();

    let serials_to_transfer = &serials[..4];

    // try to transfer in the wrong direction
    for &serial in serials_to_transfer {
        transfer_tx.nft_transfer(token.id.nft(serial as u64), bob.id, alice.id);
    }

    let res = transfer_tx.sign(bob.key.clone()).execute(&client).await?.get_receipt(&client).await;

    assert_matches!(
        res,
        Err(hiero_sdk::Error::ReceiptStatus { status: Status::SenderDoesNotOwnNftSerialNo, .. })
    );

    token.burn(&client, serials).await?;
    token.delete(&client).await?;

    tokio::try_join!(alice.delete(&client), bob.delete(&client))?;

    Ok(())
}
