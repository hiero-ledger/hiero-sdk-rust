mod associate;
mod burn;
mod create;
mod delete;
mod dissociate;
mod fee_schedule_update;
mod freeze;
mod grant_kyc;
mod info;
mod mint;
mod nft_info;
mod nft_transfer;
mod nft_update;
mod pause;
mod reject;
mod reject_flow;

mod airdrop;
mod cancel_airdrop;
mod claim_airdrop;
mod revoke_kyc;
mod transfer;
mod transfer_with_hooks;
mod unfreeze;
mod unpause;
mod update;
mod wipe;

use hedera::{
    Client,
    PublicKey,
    TokenBurnTransaction,
    TokenCreateTransaction,
    TokenDeleteTransaction,
    TokenId,
    TokenMintTransaction,
    TransactionResponse,
};
use time::{
    Duration,
    OffsetDateTime,
};
use tokio::task::JoinSet;

use crate::account::Account;
use crate::common::{
    setup_global,
    Operator,
    TestEnvironment,
};

pub(crate) enum Key {
    Owner,
    Custom(PublicKey),
}

pub(crate) struct TokenKeys {
    pub(crate) admin: Option<Key>,
    pub(crate) freeze: Option<Key>,
    pub(crate) wipe: Option<Key>,
    pub(crate) kyc: Option<Key>,
    pub(crate) supply: Option<Key>,
    pub(crate) fee_schedule: Option<Key>,
    pub(crate) pause: Option<Key>,
}

impl TokenKeys {
    const NONE: Self = Self {
        admin: None,
        freeze: None,
        wipe: None,
        kyc: None,
        supply: None,
        fee_schedule: None,
        pause: None,
    };

    const DEFAULT: Self = Self { admin: Some(Key::Owner), ..Self::NONE };

    const ALL_OWNER: Self = Self {
        admin: Some(Key::Owner),
        freeze: Some(Key::Owner),
        wipe: Some(Key::Owner),
        kyc: Some(Key::Owner),
        supply: Some(Key::Owner),
        fee_schedule: Some(Key::Owner),
        pause: Some(Key::Owner),
    };
}

impl Default for TokenKeys {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Default)]
pub(crate) struct CreateFungibleToken {
    initial_supply: u64,
    keys: TokenKeys,
}

pub(crate) struct FungibleToken {
    pub(crate) id: TokenId,
    pub(crate) owner: Account,
}

pub(crate) const TEST_FUNGIBLE_INITIAL_BALANCE: &u64 = &1_000_000;
pub(crate) const TEST_MINTED_NFTS: &u64 = &10;
pub(crate) const TEST_AMOUNT: i64 = 100;

impl FungibleToken {
    pub(crate) async fn create(
        client: &Client,
        owner: &Account,
        params: CreateFungibleToken,
    ) -> hedera::Result<Self> {
        let owner_public_key = owner.key.public_key();

        let token_id = {
            let mut tx = TokenCreateTransaction::new();
            tx.name("ffff")
                .symbol("F")
                .decimals(3)
                .treasury_account_id(owner.id)
                .initial_supply(params.initial_supply);

            let keys = params.keys;

            if let Some(it) = keys.admin {
                tx.admin_key(match it {
                    Key::Owner => owner_public_key,
                    Key::Custom(key) => key,
                });
            }

            if let Some(it) = keys.freeze {
                tx.freeze_key(match it {
                    Key::Owner => owner_public_key,
                    Key::Custom(key) => key,
                });
            }

            if let Some(it) = keys.wipe {
                tx.wipe_key(match it {
                    Key::Owner => owner_public_key,
                    Key::Custom(key) => key,
                });
            }

            if let Some(it) = keys.kyc {
                tx.kyc_key(match it {
                    Key::Owner => owner_public_key,
                    Key::Custom(key) => key,
                });
            }

            if let Some(it) = keys.supply {
                tx.supply_key(match it {
                    Key::Owner => owner_public_key,
                    Key::Custom(key) => key,
                });
            }

            if let Some(it) = keys.fee_schedule {
                tx.fee_schedule_key(match it {
                    Key::Owner => owner_public_key,
                    Key::Custom(key) => key,
                });
            }

            if let Some(it) = keys.pause {
                tx.pause_key(match it {
                    Key::Owner => owner_public_key,
                    Key::Custom(key) => key,
                });
            }

            tx.freeze_default(false)
                .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
                .sign(owner.key.clone())
                .execute(client)
                .await?
                .get_receipt(client)
                .await?
                .token_id
                .unwrap()
        };

        Ok(Self { id: token_id, owner: owner.clone() })
    }

    async fn create_ft(
        client: &Client,
        owner: &Account,
        decimals: u32,
    ) -> anyhow::Result<FungibleToken> {
        let id = TokenCreateTransaction::new()
            .name("ffff".to_string())
            .symbol("F".to_string())
            .token_memo("memo".to_string())
            .decimals(decimals)
            .initial_supply(1_000_000)
            .max_supply(1_000_000)
            .treasury_account_id(owner.id)
            .token_supply_type(hedera::TokenSupplyType::Finite)
            .admin_key(owner.key.public_key())
            .freeze_key(owner.key.public_key())
            .wipe_key(owner.key.public_key())
            .supply_key(owner.key.public_key())
            .metadata_key(owner.key.public_key())
            .pause_key(owner.key.public_key())
            .execute(&client)
            .await?
            .get_receipt(&client)
            .await?
            .token_id
            .unwrap();

        Ok(FungibleToken { id, owner: owner.clone() })
    }

    async fn burn(&self, client: &Client, supply: u64) -> hedera::Result<()> {
        hedera::TokenBurnTransaction::new()
            .token_id(self.id)
            .amount(supply)
            .sign(self.owner.key.clone())
            .execute(client)
            .await?
            .get_receipt(client)
            .await?;

        Ok(())
    }

    async fn delete(self, client: &Client) -> hedera::Result<()> {
        TokenDeleteTransaction::new()
            .token_id(self.id)
            .sign(self.owner.key)
            .execute(client)
            .await?
            .get_receipt(client)
            .await?;

        Ok(())
    }
}

pub(crate) struct Nft {
    pub(crate) id: TokenId,
    pub(crate) owner: Account,
}

impl Nft {
    pub(crate) async fn create(client: &Client, owner: &Account) -> hedera::Result<Self> {
        let owner_public_key = owner.key.public_key();
        let token_id = TokenCreateTransaction::new()
            .name("ffff")
            .symbol("F")
            .token_type(hedera::TokenType::NonFungibleUnique)
            .treasury_account_id(owner.id)
            .admin_key(owner_public_key)
            .freeze_key(owner_public_key)
            .wipe_key(owner_public_key)
            .pause_key(owner_public_key)
            .supply_key(owner_public_key)
            .fee_schedule_key(owner_public_key)
            .freeze_default(false)
            .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
            .sign(owner.key.clone())
            .execute(client)
            .await?
            .get_receipt(client)
            .await?
            .token_id
            .unwrap();

        Ok(Self { id: token_id, owner: owner.clone() })
    }

    // fixme: find a better name
    async fn mint_incremental(
        &self,
        client: &Client,
        nfts_to_mint: u8,
    ) -> hedera::Result<Vec<i64>> {
        self.mint(client, (0..nfts_to_mint).map(|it| [it])).await
    }

    pub(crate) async fn mint<Bytes: AsRef<[u8]>>(
        &self,
        client: &Client,
        metadata: impl IntoIterator<Item = Bytes>,
    ) -> hedera::Result<Vec<i64>> {
        async fn inner(
            nft: &Nft,
            client: &Client,
            mut tx: TokenMintTransaction,
        ) -> hedera::Result<Vec<i64>> {
            let serials = tx
                .token_id(nft.id)
                .sign(nft.owner.key.clone())
                .execute(client)
                .await?
                .get_receipt(client)
                .await?
                .serials;

            Ok(serials)
        }

        let mut tx = TokenMintTransaction::new();

        tx.metadata(metadata);

        inner(self, client, tx).await
    }

    pub(crate) async fn burn(
        &self,
        client: &Client,
        serials: impl IntoIterator<Item = i64>,
    ) -> hedera::Result<()> {
        // non generic inner function to save generic instantiations... Not that that's a huge concern here.
        async fn inner(
            nft: &Nft,
            client: &Client,
            mut tx: TokenBurnTransaction,
        ) -> hedera::Result<()> {
            tx.token_id(nft.id)
                .sign(nft.owner.key.clone())
                .execute(client)
                .await?
                .get_receipt(client)
                .await
                .map(drop)
        }

        let mut tx = TokenBurnTransaction::new();

        tx.serials(serials);

        inner(self, client, tx).await
    }

    pub(crate) async fn delete(self, client: &Client) -> hedera::Result<()> {
        TokenDeleteTransaction::new()
            .token_id(self.id)
            .sign(self.owner.key)
            .execute(client)
            .await?
            .get_receipt(client)
            .await?;

        Ok(())
    }
}

#[tokio::test]
async fn mint_several_nfts_at_once() -> anyhow::Result<()> {
    async fn setup(op: &Operator, client: &Client) -> anyhow::Result<TokenId> {
        let token_id = TokenCreateTransaction::new()
            .name("sdk::rust::e2e::mint_many")
            .symbol("µ")
            .token_type(hedera::TokenType::NonFungibleUnique)
            .treasury_account_id(op.account_id)
            .admin_key(op.private_key.clone().public_key())
            .supply_key(op.private_key.clone().public_key())
            .expiration_time(OffsetDateTime::now_utc() + Duration::minutes(5))
            .freeze_default(false)
            .execute(client)
            .await?
            .get_receipt(client)
            .await?
            .token_id
            .ok_or_else(|| anyhow::anyhow!("Token creation failed"))?;

        log::info!("successfully created token {token_id}");

        Ok(token_id)
    }

    async fn teardown(client: &Client, token_id: TokenId) -> anyhow::Result<()> {
        TokenDeleteTransaction::new()
            .token_id(token_id)
            .execute(client)
            .await?
            .get_receipt(client)
            .await?;

        Ok(())
    }

    const MINT_TRANSACTIONS: usize = 5;
    // mint faster by using less transactions.
    const MAX_MINTS_PER_TX: usize = 10;

    let TestEnvironment { config, client } = setup_global();

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to lack of operator");
        return Ok(());
    };

    if !config.run_nonfree_tests {
        log::debug!("skipping non-free test");
        return Ok(());
    }

    let token_id = setup(op, &client).await?;

    let mut tasks = JoinSet::new();

    for _ in 0..MINT_TRANSACTIONS {
        // give the tasks a bit of time between spawning to avoid hammering the network.
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        tasks.spawn({
            let client = client.clone();
            async move { create_nft(&client, token_id, MAX_MINTS_PER_TX).await }
        });
    }

    let mut responses = Vec::with_capacity(MINT_TRANSACTIONS);

    // note: we collect the responses to test simultaniously waiting for multiple receipts next.
    while let Some(response) = tasks.join_next().await {
        let response = response??;

        responses.push(response);
    }

    let mut tasks = JoinSet::new();

    for response in responses {
        // give the tasks a bit of time between spawning to avoid hammering the network.
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;

        let client = client.clone();
        tasks.spawn(async move { response.get_receipt(&client).await });
    }

    while let Some(receipt) = tasks.join_next().await {
        // we error for status here.
        let _receipt = receipt??;
    }

    teardown(&client, token_id).await?;

    Ok(())
}

async fn create_nft(
    client: &Client,
    token_id: TokenId,
    nfts: usize,
) -> hedera::Result<TransactionResponse> {
    TokenMintTransaction::default()
        .token_id(token_id)
        .metadata(vec![Vec::from([0x12, 0x34]); nfts])
        .execute(client)
        .await
}
