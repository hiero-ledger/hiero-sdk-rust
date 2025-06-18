// SPDX-License-Identifier: Apache-2.0
mod error;

use std::any::type_name;
use std::borrow::Cow;
use std::ops::ControlFlow;
use std::time::{
    Duration,
    Instant,
};

use backoff::{
    ExponentialBackoff,
    ExponentialBackoffBuilder,
};
use futures_core::future::BoxFuture;
use futures_util::StreamExt;
use prost::Message;
use rand::seq::SliceRandom;
use rand::thread_rng;
use tonic::metadata::AsciiMetadataValue;
use tonic::transport::Channel;
use tonic::Request;
use triomphe::Arc;

use crate::client::NetworkData;
use crate::execute::error::is_tonic_status_transient;
use crate::ping_query::PingQuery;
use crate::{
    client,
    retry,
    AccountId,
    BoxGrpcFuture,
    Client,
    Error,
    Status,
    TransactionId,
    ValidateChecksums,
};

pub(crate) trait Execute: ValidateChecksums {
    type GrpcRequest: Clone + Message;

    type GrpcResponse: Message;

    /// Additional context returned from each call to `make_request`. Upon
    /// a successful request, the associated response context is passed to
    /// `make_response`.
    type Context: Send;

    type Response;

    /// Account ID to be used for generating transaction IDs.
    ///
    /// This is only used `self.requires_transaction` and `self.transaction_id.is_none()`.
    fn operator_account_id(&self) -> Option<&AccountId>;

    /// Get the _explicit_ nodes that this request will be submitted to.
    fn node_account_ids(&self) -> Option<&[AccountId]>;

    /// Get the _explicit_ transaction ID that this request will use.
    fn transaction_id(&self) -> Option<TransactionId>;

    /// Get whether to generate transaction IDs for request creation.
    fn requires_transaction_id(&self) -> bool;

    /// Returns whether to regenerate transaction IDs for request creation.
    ///
    /// Transaction ID regeneration only can happen when `transaction_id` is None and `requires_transaction_id` is true.
    fn regenerate_transaction_id(&self) -> Option<bool> {
        None
    }

    /// Check whether to retry an pre-check status.
    fn should_retry_pre_check(&self, _status: Status) -> bool {
        false
    }

    /// Check whether we should retry an otherwise successful response.
    #[allow(unused_variables)]
    fn should_retry(&self, response: &Self::GrpcResponse) -> bool {
        false
    }

    /// Add metadata to the request.
    fn add_metadata(&self, metadata: &mut tonic::metadata::MetadataMap) {
        let user_agent = format!("hiero-sdk-rust/{}", env!("CARGO_PKG_VERSION"));
        metadata.insert("x-user-agent", user_agent.parse().unwrap());
    }

    /// Create a new request for execution.
    ///
    /// A created request is cached per node until any request returns
    /// `TransactionExpired`; in which case, the request cache is cleared.
    fn make_request(
        &self,
        transaction_id: Option<&TransactionId>,
        node_account_id: AccountId,
    ) -> crate::Result<(Self::GrpcRequest, Self::Context)>;

    /// Execute the created GRPC request against the provided GRPC channel.
    fn execute(
        &self,
        channel: Channel,
        request: Self::GrpcRequest,
    ) -> BoxGrpcFuture<Self::GrpcResponse>;

    /// Create a response from the GRPC response and the saved transaction
    /// and node account ID from the successful request.
    fn make_response(
        &self,
        response: Self::GrpcResponse,
        context: Self::Context,
        node_account_id: AccountId,
        transaction_id: Option<&TransactionId>,
    ) -> crate::Result<Self::Response>;

    /// Create an error from the given pre-check status.
    fn make_error_pre_check(
        &self,
        status: Status,
        transaction_id: Option<&TransactionId>,
        response: Self::GrpcResponse,
    ) -> crate::Error;

    /// Extract the pre-check status from the GRPC response.
    fn response_pre_check_status(response: &Self::GrpcResponse) -> crate::Result<i32>;
}

struct ExecuteContext {
    // When `Some` the `transaction_id` will be regenerated when expired.
    operator_account_id: Option<AccountId>,
    network: Arc<NetworkData>,
    backoff_config: ExponentialBackoff,
    max_attempts: usize,
    // timeout for a single grpc request.
    grpc_timeout: Option<Duration>,
}

pub(crate) async fn execute<E>(
    client: &Client,
    executable: &E,
    timeout: Option<Duration>,
) -> crate::Result<E::Response>
where
    E: Execute + Sync,
{
    if client.auto_validate_checksums() {
        let ledger_id = client.ledger_id_internal();
        let ledger_id = ledger_id
            .as_ref()
            .expect("Client had auto_validate_checksums enabled but no ledger ID");

        executable.validate_checksums(ledger_id.as_ref_ledger_id())?;
    }

    let operator_account_id = 'op: {
        if executable.transaction_id().is_some()
            || !executable
                .regenerate_transaction_id()
                .unwrap_or(client.default_regenerate_transaction_id())
        {
            break 'op None;
        }

        executable
            .operator_account_id()
            .copied()
            .or_else(|| client.load_operator().as_ref().map(|it| it.account_id))
    };

    let backoff = client.backoff();
    let mut backoff_builder = ExponentialBackoffBuilder::new();

    backoff_builder
        .with_initial_interval(backoff.initial_backoff)
        .with_max_interval(backoff.max_backoff);

    if let Some(timeout) = timeout.or(backoff.request_timeout) {
        backoff_builder.with_max_elapsed_time(Some(timeout));
    }

    execute_inner(
        &ExecuteContext {
            max_attempts: backoff.max_attempts,
            backoff_config: backoff_builder.build(),
            operator_account_id,
            network: client.net().0.load_full(),
            grpc_timeout: backoff.grpc_timeout,
        },
        executable,
    )
    .await
}

async fn execute_inner<E>(ctx: &ExecuteContext, executable: &E) -> crate::Result<E::Response>
where
    E: Execute + Sync,
{
    fn recurse_ping(ctx: &ExecuteContext, index: usize) -> BoxFuture<'_, bool> {
        Box::pin(async move {
            let ctx = ExecuteContext {
                operator_account_id: None,
                network: Arc::clone(&ctx.network),
                backoff_config: ctx.backoff_config.clone(),
                max_attempts: ctx.max_attempts,
                grpc_timeout: ctx.grpc_timeout,
            };
            let ping_query = PingQuery::new(ctx.network.node_ids()[index]);

            execute_inner(&ctx, &ping_query).await.is_ok()
        })
    }

    // the overall timeout for the backoff starts measuring from here
    let backoff = ctx.backoff_config.clone();

    // TODO: cache requests to avoid signing a new request for every node in a delayed back-off

    // if we need to generate a transaction ID for this request (and one was not provided),
    // generate one now
    let explicit_transaction_id = executable.transaction_id();
    let mut transaction_id = executable
        .requires_transaction_id()
        .then_some(explicit_transaction_id)
        .and_then(|it| it.or_else(|| ctx.operator_account_id.map(TransactionId::generate)));

    // if we were explicitly given a list of nodes to use, we iterate through each
    // of the given nodes (in a random order)
    let explicit_node_indexes = executable
        .node_account_ids()
        .map(|ids| ctx.network.node_indexes_for_ids(ids))
        .transpose()?;

    let explicit_node_indexes = explicit_node_indexes.as_deref();

    let layer = move || async move {
        loop {
            let mut last_error: Option<Error> = None;

            let random_node_indexes = random_node_indexes(&ctx.network, explicit_node_indexes)
                .ok_or(retry::Error::EmptyTransient)?;

            let random_node_indexes = {
                let random_node_indexes = &random_node_indexes;
                let client = ctx;
                let now = Instant::now();
                futures_util::stream::iter(random_node_indexes.iter().copied()).filter(
                    move |&node_index| async move {
                        // NOTE: For pings we're relying on the fact that they have an explict node index.
                        explicit_node_indexes.is_some()
                            || client.network.node_recently_pinged(node_index, now)
                            || recurse_ping(client, node_index).await
                    },
                )
            };

            let mut random_node_indexes = std::pin::pin!(random_node_indexes);

            while let Some(node_index) = random_node_indexes.next().await {
                let tmp = execute_single(ctx, executable, node_index, &mut transaction_id).await;

                log::log!(
                    match &tmp {
                        Ok(ControlFlow::Break(_)) => log::Level::Debug,
                        Ok(ControlFlow::Continue(_)) => log::Level::Warn,
                        Err(e) =>
                            if e.is_transient() {
                                log::Level::Warn
                            } else {
                                log::Level::Error
                            },
                    },
                    "Execution of {} on node at index {node_index} / node id {} {}",
                    type_name::<E>(),
                    ctx.network.channel(node_index).0,
                    match &tmp {
                        Ok(ControlFlow::Break(_)) => Cow::Borrowed("succeeded"),
                        Ok(ControlFlow::Continue(err)) =>
                            format!("will continue due to {err:?}").into(),
                        Err(err) => format!("failed due to {err:?}").into(),
                    },
                );

                match tmp? {
                    ControlFlow::Continue(err) => last_error = Some(err),
                    ControlFlow::Break(res) => return Ok(res),
                }
            }

            match last_error {
                Some(it) => return Err(retry::Error::Transient(it)),
                // this can only happen if we skipped every node due to pinging it coming up `false` (unhealthy)... The node will be marked as unhealthy, soo
                None => continue,
            }
        }
    };

    // the outer loop continues until we timeout or reach the maximum number of "attempts"
    // an attempt is counted when we have a successful response from a node that must either
    // be retried immediately (on a new node) or retried after a backoff.
    crate::retry(backoff, Some(ctx.max_attempts), layer).await
}

fn map_tonic_error(
    status: tonic::Status,
    network: &client::NetworkData,
    node_index: usize,
    request_free: bool,
) -> retry::Error {
    const MIME_HTML: &[u8] = b"text/html";

    match status.code() {
        // if the node says it isn't available, then we should just try again with a different node.
        tonic::Code::Unavailable | tonic::Code::ResourceExhausted => {
            // NOTE: this is an "unhealthy" node
            network.mark_node_unhealthy(node_index);

            // try the next node in our allowed list, immediately
            retry::Error::Transient(status.into())
        }

        // todo: find a way to make this less fragile
        // hack:
        // if this happens:
        // the node is completely borked (we're probably seeing the load balancer's response),
        // and we have no clue if the effect went through
        tonic::Code::Internal
            if status.metadata().get("content-type").map(AsciiMetadataValue::as_bytes)
                == Some(MIME_HTML) =>
        {
            network.mark_node_unhealthy(node_index);

            // hack to the hack:
            // if this is a free request let's try retrying it anyway...
            match request_free {
                true => retry::Error::Transient(status.into()),
                false => retry::Error::Permanent(status.into()),
            }
        }

        _ if is_tonic_status_transient(&status) => {
            network.mark_node_unhealthy(node_index);

            retry::Error::Transient(status.into())
        }

        // fail immediately
        _ => retry::Error::Permanent(status.into()),
    }
}

async fn execute_single<E: Execute + Sync>(
    ctx: &ExecuteContext,
    executable: &E,
    node_index: usize,
    transaction_id: &mut Option<TransactionId>,
) -> retry::Result<ControlFlow<E::Response, Error>> {
    let (node_account_id, channel) = ctx.network.channel(node_index);

    log::debug!(
        "Preparing {} on node at index {node_index} / node id {node_account_id}",
        type_name::<E>()
    );

    let (request, context) = executable
        .make_request(transaction_id.as_ref(), node_account_id)
        // Does not represent a network error or error returned by a node
        .map_err(retry::Error::Permanent)?;

    log::debug!(
        "Executing {} on node at index {node_index} / node id {node_account_id}",
        type_name::<E>()
    );

    let mut req = Request::new(request);
    executable.add_metadata(req.metadata_mut());

    let fut = executable.execute(channel, req.into_inner());

    let response = match ctx.grpc_timeout {
        Some(it) => match tokio::time::timeout(it, fut).await {
            Ok(it) => it,
            Err(_) => {
                return Ok(ControlFlow::Continue(crate::Error::GrpcStatus(
                    tonic::Status::deadline_exceeded("explicitly given grpc timeout was exceeded"),
                )))
            }
        },
        None => fut.await,
    };

    let response = response.map(tonic::Response::into_inner).map_err(|status| {
        map_tonic_error(status, &ctx.network, node_index, transaction_id.is_none())
    });

    let response = match response {
        Ok(response) => response,
        Err(retry::Error::Transient(err)) => {
            return Ok(ControlFlow::Continue(err));
        }

        Err(e) => return Err(e),
    };

    // at this point, any failure isn't from the node, it's from the request.
    ctx.network.mark_node_healthy(node_index);

    let status = E::response_pre_check_status(&response)
        .and_then(|status| {
            // not sure how to proceed, fail immediately
            Status::try_from(status).or_else(|_| Err(Error::ResponseStatusUnrecognized(status)))
        })
        .map_err(retry::Error::Permanent)?;

    match status {
        Status::Ok if executable.should_retry(&response) => Err(retry::Error::Transient(
            executable.make_error_pre_check(status, transaction_id.as_ref(), response),
        )),

        Status::Ok => executable
            .make_response(response, context, node_account_id, transaction_id.as_ref())
            .map(ControlFlow::Break)
            .map_err(retry::Error::Permanent),

        Status::Busy | Status::PlatformNotActive => {
            // NOTE: this is a "busy" node
            // try the next node in our allowed list, immediately
            Ok(ControlFlow::Continue(executable.make_error_pre_check(
                status,
                transaction_id.as_ref(),
                response,
            )))
        }

        // would do an `if_let` but, not stable ._.
        Status::TransactionExpired if ctx.operator_account_id.is_some() => {
            // the transaction that was generated has since expired
            // re-generate the transaction ID and try again, immediately

            let new = TransactionId::generate(ctx.operator_account_id.unwrap());

            *transaction_id = Some(new);

            Ok(ControlFlow::Continue(executable.make_error_pre_check(
                status,
                transaction_id.as_ref(),
                response,
            )))
        }

        _ if executable.should_retry_pre_check(status) => {
            // conditional retry on pre-check should back-off and try again
            Err(retry::Error::Transient(executable.make_error_pre_check(
                status,
                transaction_id.as_ref(),
                response,
            )))
        }

        _ => {
            // any other pre-check is an error that the user needs to fix, fail immediately
            Err(retry::Error::Permanent(executable.make_error_pre_check(
                status,
                transaction_id.as_ref(),
                response,
            )))
        }
    }
}

// todo: return an iterator.
fn random_node_indexes(
    network: &client::NetworkData,
    explicit_node_indexes: Option<&[usize]>,
) -> Option<Vec<usize>> {
    // cache the rng impl and "now" because `thread_rng` is TLS (a thread local),
    // and because using the same reference time avoids situations where a node that wasn't available becomes available.
    let mut rng = thread_rng();
    let now = Instant::now();

    if let Some(indexes) = explicit_node_indexes {
        let tmp: Vec<_> =
            indexes.iter().copied().filter(|index| network.is_node_healthy(*index, now)).collect();

        let mut indexes = if tmp.is_empty() { indexes.to_vec() } else { tmp };

        assert!(!indexes.is_empty(), "empty explicitly set nodes");

        indexes.shuffle(&mut rng);

        return Some(indexes);
    }

    {
        let mut indexes: Vec<_> = network.healthy_node_indexes(now).collect();

        if indexes.is_empty() {
            return None;
        }

        // would put this inline, but borrowck wouldn't allow that.
        let amount = (indexes.len() + 2) / 3;

        let (shuffled, _) = indexes.partial_shuffle(&mut rng, amount);

        Some(shuffled.to_vec())
    }
}
