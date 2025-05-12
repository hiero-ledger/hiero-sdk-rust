// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;
use std::ops::Deref;

use super::{
    NftId,
    TokenDissociateTransaction,
    TokenId,
    TokenRejectTransaction,
};
use crate::signer::AnySigner;
use crate::{
    AccountId,
    Client,
    PrivateKey,
    PublicKey,
    TransactionResponse,
};

///  Reject undesired token(s) and dissociate in a single flow.
///
/// The operation of this flow is as follows:
/// 1. Execute a [`TokenRejectTransaction`] using the provided NFT IDs and the Token IDs
/// 2. Dissociate the rejected tokens from the owner account
#[derive(Default, Debug)]
pub struct TokenRejectFlow {
    node_account_ids: Option<Vec<AccountId>>,
    token_reject_data: TokenRejectData,
}

#[derive(Default, Debug)]
struct TokenRejectData {
    owner: Option<AccountId>,
    token_ids: Vec<TokenId>,
    nft_ids: Vec<NftId>,
    freeze_with_client: Option<Client>,
    signer: Option<AnySigner>,
}

impl TokenRejectFlow {
    /// Create a new `TokenRejectFlow`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the owner id of the token to be rejected.
    #[must_use]
    pub fn get_owner(&self) -> Option<AccountId> {
        self.token_reject_data.owner
    }

    /// Sets the owner id of the token to be rejected.
    pub fn owner(&mut self, owner: impl Into<AccountId>) -> &mut Self {
        self.token_reject_data.owner = Some(owner.into());
        self
    }

    /// Returns the account IDs of the nodes the transactions may be submitted to.
    #[must_use]
    pub fn get_node_account_ids(&self) -> Option<&[AccountId]> {
        self.node_account_ids.as_deref()
    }

    /// Sets the account IDs of the nodes the transactions may be submitted to.
    pub fn node_account_ids(
        &mut self,
        node_account_ids: impl IntoIterator<Item = AccountId>,
    ) -> &mut Self {
        self.node_account_ids = Some(node_account_ids.into_iter().collect());

        self
    }

    /// Returns the list of token IDs.
    #[must_use]
    pub fn get_token_ids(&self) -> &[TokenId] {
        self.token_reject_data.token_ids.deref()
    }

    /// Sets the list of token IDs.
    pub fn token_ids(&mut self, token_ids: impl IntoIterator<Item = TokenId>) -> &mut Self {
        self.token_reject_data.token_ids = token_ids.into_iter().collect();

        self
    }

    /// Adds a token ID to the list of token IDs.
    pub fn add_token_id(&mut self, token_id: TokenId) -> &mut Self {
        self.token_reject_data.token_ids.push(token_id);

        self
    }

    /// Returns the list of NFT IDs.
    #[must_use]
    pub fn get_nft_ids(&self) -> &[NftId] {
        self.token_reject_data.nft_ids.deref()
    }

    /// Sets the list of NFT IDs.
    pub fn nft_ids(&mut self, nft_ids: impl IntoIterator<Item = NftId>) -> &mut Self {
        self.token_reject_data.nft_ids = nft_ids.into_iter().collect();

        self
    }

    /// Adds an NFT ID to the list of NFT IDs.
    pub fn add_nft_id(&mut self, nft_id: NftId) -> &mut Self {
        self.token_reject_data.nft_ids.push(nft_id);

        self
    }

    /// Sets the client to use for freezing the generated *``TokenRejectTransaction``*.
    ///
    /// By default freezing will use the client provided to ``execute``.
    pub fn freeze_with(&mut self, client: Client) -> &mut Self {
        self.token_reject_data.freeze_with_client = Some(client);

        self
    }

    /// Sets the signer for use in the ``TokenRejectTransaction``
    ///
    /// Important: Only *one* signer is allowed.
    pub fn sign(&mut self, key: PrivateKey) -> &mut Self {
        self.token_reject_data.signer = Some(AnySigner::PrivateKey(key));

        self
    }

    /// Sets the signer for use in the ``TokenRejectTransaction``
    ///
    /// Important: Only *one* signer is allowed.
    pub fn sign_with<F: Fn(&[u8]) -> Vec<u8> + Send + Sync + 'static>(
        &mut self,
        public_key: PublicKey,
        signer: F,
    ) -> &mut Self {
        self.token_reject_data.signer = Some(AnySigner::arbitrary(Box::new(public_key), signer));

        self
    }

    /// Set the operator that this transaction will be signed with.
    pub fn sign_with_operator(&mut self, client: &Client) -> &mut Self {
        // todo: proper error
        let operator_key = client
            .load_operator()
            .as_deref()
            .map(|it| it.signer.clone())
            .expect("Must call `Client.set_operator` to use token reject flow");

        self.token_reject_data.signer = Some(operator_key);

        self
    }

    /// Generates the required transactions and executes them all.
    pub async fn execute(&self, client: &Client) -> crate::Result<TransactionResponse> {
        self.execute_with_optional_timeout(client, None).await
    }

    /// Generates the required transactions and executes them all.
    pub async fn execute_with_timeout(
        &self,
        client: &Client,
        timeout_per_transaction: std::time::Duration,
    ) -> crate::Result<TransactionResponse> {
        self.execute_with_optional_timeout(client, Some(timeout_per_transaction)).await
    }

    async fn execute_with_optional_timeout(
        &self,
        client: &Client,
        timeout_per_transaction: Option<std::time::Duration>,
    ) -> crate::Result<TransactionResponse> {
        let reject_response =
            make_token_reject_transaction(&self.token_reject_data, self.node_account_ids.clone())?
                .execute_with_optional_timeout(client, timeout_per_transaction)
                .await?;

        reject_response
            .get_receipt_query()
            .execute_with_optional_timeout(client, timeout_per_transaction)
            .await?;

        let dissociate_response = make_token_dissociate_transaction(
            &self.token_reject_data,
            self.node_account_ids.clone(),
        )?
        .execute_with_optional_timeout(client, timeout_per_transaction)
        .await?;

        dissociate_response
            .get_receipt_query()
            .execute_with_optional_timeout(client, timeout_per_transaction)
            .await?;

        Ok(reject_response)
    }
}

fn make_token_reject_transaction(
    data: &TokenRejectData,
    node_account_ids: Option<Vec<AccountId>>,
) -> crate::Result<TokenRejectTransaction> {
    let mut tmp = TokenRejectTransaction::new();

    if let Some(owner) = &data.owner {
        tmp.owner(owner.clone());
    }

    tmp.token_ids(data.token_ids.clone());

    tmp.nft_ids(data.nft_ids.clone());

    if let Some(node_account_ids) = node_account_ids {
        tmp.node_account_ids(node_account_ids);
    }

    if let Some(client) = &data.freeze_with_client {
        tmp.freeze_with(client)?;
    }

    if let Some(signer) = &data.signer {
        tmp.sign_signer(signer.clone());
    }

    Ok(tmp)
}

fn make_token_dissociate_transaction(
    data: &TokenRejectData,
    node_account_ids: Option<Vec<AccountId>>,
) -> crate::Result<TokenDissociateTransaction> {
    let mut token_ids = data.token_ids.clone();
    token_ids.extend(data.nft_ids.iter().map(|it| it.token_id));

    let unique_token_ids: Vec<_> =
        token_ids.into_iter().collect::<HashSet<_>>().into_iter().collect();

    let mut tmp = TokenDissociateTransaction::new();

    if let Some(owner) = data.owner {
        tmp.account_id(owner);
    }

    tmp.token_ids(unique_token_ids);

    if let Some(node_account_ids) = node_account_ids {
        tmp.node_account_ids(node_account_ids);
    }

    if let Some(client) = &data.freeze_with_client {
        tmp.freeze_with(client)?;
    }

    if let Some(signer) = &data.signer {
        tmp.sign_signer(signer.clone());
    }

    Ok(tmp)
}
