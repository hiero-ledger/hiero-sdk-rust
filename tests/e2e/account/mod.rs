mod allowance_approve;
mod allowance_delete;
mod balance;
mod create;
mod delete;
mod info;
mod update;

use hiero_sdk::{
    AccountId,
    Hbar,
    PrivateKey,
};

#[derive(Clone)]
pub struct Account {
    pub key: PrivateKey,
    pub id: AccountId,
}

impl Account {
    pub async fn create(
        initial_balance: Hbar,
        client: &hiero_sdk::Client,
    ) -> hiero_sdk::Result<Self> {
        let key = PrivateKey::generate_ed25519();

        let receipt = hiero_sdk::AccountCreateTransaction::new()
            .set_key_without_alias(key.public_key())
            .initial_balance(initial_balance)
            .execute(client)
            .await?
            .get_receipt(client)
            .await?;

        let account_id = receipt.account_id.unwrap();

        Ok(Self { key, id: account_id })
    }

    pub async fn delete(self, client: &hiero_sdk::Client) -> hiero_sdk::Result<()> {
        hiero_sdk::AccountDeleteTransaction::new()
            .account_id(self.id)
            .transfer_account_id(client.get_operator_account_id().unwrap())
            .freeze_with(client)?
            .sign(self.key)
            .execute(client)
            .await?
            .get_receipt(client)
            .await?;

        Ok(())
    }

    pub async fn create_with_max_associations(
        max_automatic_token_associations: i32,
        account_key: &PrivateKey,
        client: &hiero_sdk::Client,
    ) -> hiero_sdk::Result<Self> {
        let receipt = hiero_sdk::AccountCreateTransaction::new()
            .set_key_without_alias(account_key.public_key())
            .initial_balance(Hbar::new(10))
            .max_automatic_token_associations(max_automatic_token_associations)
            .execute(client)
            .await?
            .get_receipt(client)
            .await?;

        let account_id = receipt.account_id.unwrap();

        Ok(Account { key: account_key.clone(), id: account_id })
    }
}
