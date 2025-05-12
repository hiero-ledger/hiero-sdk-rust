// SPDX-License-Identifier: Apache-2.0

use futures_core::future::BoxFuture;
use time::Duration;

use crate::execute::execute;
use crate::query::cost::QueryCost;
use crate::query::payment_transaction::PaymentTransaction;
use crate::{
    AccountId,
    Client,
    Error,
    Hbar,
    TransactionId,
    TransactionReceiptQuery,
};

mod any;
mod cost;
mod execute;
pub(super) mod payment_transaction;
mod protobuf;

pub(crate) use any::AnyQueryData;
pub use any::{
    AnyQuery,
    AnyQueryResponse,
};
pub(crate) use execute::{
    response_header,
    QueryExecute,
};
pub(crate) use protobuf::ToQueryProtobuf;

/// A query that can be executed on the Hiero network.
#[derive(Debug, Default)]
pub struct Query<D>
where
    D: QueryExecute,
{
    pub(crate) data: D,
    pub(crate) payment: PaymentTransaction,
}

impl<D> Query<D>
where
    D: QueryExecute + Default,
{
    /// Create a new query ready for configuration and execution.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<D> Query<D>
where
    D: QueryExecute,
{
    /// Returns the account IDs of the nodes that this query may be submitted to.
    ///
    /// Defaults to the full list of nodes configured on the client; or, the node account IDs
    /// configured on the query payment transaction (if explicitly provided).
    #[must_use]
    pub fn get_node_account_ids(&self) -> Option<&[AccountId]> {
        self.payment.get_node_account_ids()
    }

    /// Sets the account IDs of the nodes that this query may be submitted to.
    ///
    /// Defaults to the full list of nodes configured on the client; or, the node account IDs
    /// configured on the query payment transaction (if explicitly provided).
    pub fn node_account_ids(&mut self, ids: impl IntoIterator<Item = AccountId>) -> &mut Self {
        self.payment.node_account_ids(ids);
        self
    }

    /// Returns the explicit payment amount for this query.
    ///
    /// The client will submit exactly this amount for the payment of this query. Hiero
    /// will not return any remainder (over the actual cost for this query).
    #[must_use]
    pub fn get_payment_amount(&self) -> Option<Hbar> {
        self.payment.get_amount()
    }

    /// Sets the explicit payment amount for this query.
    ///
    /// The client will submit exactly this amount for the payment of this query. Hiero
    /// will not return any remainder (over the actual cost for this query).
    pub fn payment_amount(&mut self, amount: Hbar) -> &mut Self {
        self.payment.amount(amount);
        self
    }

    /// Returns the maximum payment allowable for this query.
    #[must_use]
    pub fn get_max_amount(&self) -> Option<Hbar> {
        self.payment.get_max_amount()
    }

    /// Sets the maximum payment allowable for this query.
    ///
    /// When a query is executed without an explicit payment amount set,
    /// the client will first request the cost of the given query from the node it will be
    /// submitted to and attach a payment for that amount from the operator account on the client.
    ///
    /// If the returned value is greater than this value, a [`MaxQueryPaymentExceeded`](crate::Error::MaxQueryPaymentExceeded) error
    /// will be returned.
    ///
    /// Defaults to the maximum payment amount configured on the client.
    ///
    /// Sets to `None` to allow unlimited payment amounts.
    pub fn max_payment_amount(&mut self, max: impl Into<Option<Hbar>>) -> &mut Self {
        self.payment.max_amount(max);
        self
    }

    /// Returns the duration that the payment transaction is valid for, once finalized and signed.
    #[must_use]
    pub fn get_payment_transaction_valid_duration(&self) -> Option<Duration> {
        self.payment.get_transaction_valid_duration()
    }

    /// Sets the duration that the payment transaction is valid for, once finalized and signed.
    ///
    /// Defaults to 120 seconds (or two minutes).
    pub fn payment_transaction_valid_duration(&mut self, duration: Duration) -> &mut Self {
        self.payment.transaction_valid_duration(duration);
        self
    }

    /// Returns the maximum transaction fee the payer account is willing to pay
    /// for the query payment transaction.
    #[must_use]
    pub fn get_max_payment_transaction_fee(&self) -> Option<Hbar> {
        self.payment.get_max_transaction_fee()
    }

    /// Sets the maximum transaction fee the payer account is willing to pay for the query
    /// payment transaction.
    ///
    /// Defaults to 1 hbar.
    pub fn max_payment_transaction_fee(&mut self, fee: Hbar) -> &mut Self {
        self.payment.max_transaction_fee(fee);
        self
    }

    /// Returns the note / description that should be recorded in the transaction record for the payment transaction.
    #[must_use]
    pub fn get_payment_transaction_memo(&self) -> &str {
        self.payment.get_transaction_memo()
    }

    /// Sets a note / description that should be recorded in the transaction record for the payment transaction.
    ///
    /// Maximum length of 100 characters.
    pub fn payment_transaction_memo(&mut self, memo: impl AsRef<str>) -> &mut Self {
        self.payment.transaction_memo(memo);
        self
    }

    /// Returns the explicit transaction ID used to identify this query's payment transaction, if set
    /// .
    #[must_use]
    pub fn get_payment_transaction_id(&self) -> Option<TransactionId> {
        self.payment.get_transaction_id()
    }

    /// Sets an explicit transaction ID to use to identify the payment transaction
    /// on this query.
    ///
    /// Overrides payer account defined on this query or on the client.
    pub fn payment_transaction_id(&mut self, id: TransactionId) -> &mut Self {
        self.payment.transaction_id(id);
        self
    }

    /// Fetch the cost of this query.
    pub async fn get_cost(&self, client: &Client) -> crate::Result<Hbar> {
        self.get_cost_with_optional_timeout(client, None).await
    }

    pub(crate) async fn get_cost_with_optional_timeout(
        &self,
        client: &Client,
        timeout: Option<std::time::Duration>,
    ) -> crate::Result<Hbar> {
        if !self.data.is_payment_required() {
            return Ok(Hbar::ZERO);
        }

        QueryCost::new(self).execute(client, timeout).await
    }

    /// Fetch the cost of this query.
    pub async fn get_cost_with_timeout(
        &self,
        client: &Client,
        timeout: std::time::Duration,
    ) -> crate::Result<Hbar> {
        self.get_cost_with_optional_timeout(client, Some(timeout)).await
    }
}

impl<D> Query<D>
where
    D: QueryExecute,
{
    /// Execute this query against the provided client of the Hiero network.
    // todo:
    #[allow(clippy::missing_errors_doc)]
    pub async fn execute(&mut self, client: &Client) -> crate::Result<D::Response> {
        self.execute_with_optional_timeout(client, None).await
    }

    // eww long name
    pub(crate) async fn execute_with_optional_timeout(
        &mut self,
        client: &Client,
        timeout: Option<std::time::Duration>,
    ) -> crate::Result<D::Response> {
        fn recurse_receipt(
            transaction_id: &TransactionId,
            client: Client,
            timeout: Option<std::time::Duration>,
        ) -> BoxFuture<'_, ()> {
            Box::pin(async move {
                let _ = TransactionReceiptQuery::new()
                    .transaction_id(*transaction_id)
                    .execute_with_optional_timeout(&client, timeout)
                    .await;
            })
        }

        // hack: this is a TransactionRecordQuery, which means we need to run the receipt first.
        if let Some(transaction_id) = self.data.transaction_id() {
            if self.data.is_payment_required() {
                let client = client.clone();
                recurse_receipt(&transaction_id, client, timeout).await;
            }
        }

        if self.payment.get_amount().is_none() && self.data.is_payment_required() {
            // should this inherit the timeout?
            // payment is required but none was specified, query the cost
            let cost = QueryCost::new(self).execute(client, None).await?;

            if self.payment.get_max_amount().is_none() {
                // N.B. This can still be `None`.
                self.payment.max_amount(client.default_max_query_payment());
            }

            if let Some(max_amount) = self.payment.get_max_amount() {
                if cost > max_amount {
                    return Err(Error::MaxQueryPaymentExceeded {
                        query_cost: cost,
                        max_query_payment: max_amount,
                    });
                }
            }

            self.payment.amount(cost);
        }

        if self.data.is_payment_required() {
            self.payment.freeze_with(client)?;
        }

        execute(client, self, timeout).await
    }

    /// Execute this query against the provided client of the Hiero network.
    // todo:
    #[allow(clippy::missing_errors_doc)]
    pub async fn execute_with_timeout(
        &mut self,
        client: &Client,
        timeout: std::time::Duration,
    ) -> crate::Result<D::Response> {
        self.execute_with_optional_timeout(client, Some(timeout)).await
    }
}
