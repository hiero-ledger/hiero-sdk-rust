use std::net::SocketAddr;
use std::sync::atomic::{
    AtomicBool,
    AtomicUsize,
    Ordering,
};
use std::sync::Arc;

use futures_util::future::BoxFuture;
use jsonrpsee::server::middleware::rpc::{
    RpcService,
    RpcServiceT,
};
use jsonrpsee::server::{
    RpcServiceBuilder,
    Server,
};
use jsonrpsee::types::Request;
use jsonrpsee::MethodResponse;
use methods::{
    RpcServer,
    RpcServerImpl,
};
use tokio::signal;

mod errors;
mod helpers;
pub(crate) mod methods;
mod responses;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let server_addr = run_server().await?;
    let url = format!("http://{}", server_addr);

    tracing::info!("Server is running at {}", url);

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let ctrl_c_future = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
        running_clone.store(false, Ordering::SeqCst);
    };

    tokio::select! {
        _ = ctrl_c_future => {}
        _ = tokio::signal::ctrl_c() => {}
    }

    Ok(())
}

async fn run_server() -> anyhow::Result<SocketAddr> {
    let m = RpcServiceBuilder::new().layer_fn(move |service: RpcService| TckMiddleware {
        service,
        count: Arc::new(AtomicUsize::new(0)),
    });

    let server = Server::builder().set_rpc_middleware(m).build("127.0.0.1:8544").await?;

    let addr = server.local_addr()?;
    let handle = server.start(RpcServerImpl.into_rpc());

    tokio::spawn(handle.stopped());

    Ok(addr)
}

#[derive(Clone)]
struct TckMiddleware<S> {
    service: S,
    count: Arc<AtomicUsize>,
}

impl<'a, S> RpcServiceT<'a> for TckMiddleware<S>
where
    S: RpcServiceT<'a> + Send + Sync + Clone + 'static,
{
    type Future = BoxFuture<'a, MethodResponse>;
    fn call(&self, req: Request<'a>) -> Self::Future {
        let count = self.count.clone();
        let service = self.service.clone();
        Box::pin(async move {
            let rp = service.call(req).await;
            count.fetch_add(1, Ordering::SeqCst);
            rp
        })
    }
}
