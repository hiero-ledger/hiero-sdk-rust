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
use methods::account::AccountRpcServer;
use methods::contract::ContractRpcServer;
use methods::ethereum::EthereumRpcServer;
use methods::file::FileRpcServer;
use methods::schedule::ScheduleRpcServer;
use methods::token::TokenRpcServer;
use methods::topic::TopicRpcServer;
use methods::utility::UtilityRpcServer;
use server::RpcServerImpl;
use tokio::signal;

mod common;
mod errors;
mod helpers;
pub(crate) mod methods;
mod responses;
mod server;

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

    let mut rpc_module = UtilityRpcServer::into_rpc(RpcServerImpl);
    rpc_module.merge(AccountRpcServer::into_rpc(RpcServerImpl))?;
    rpc_module.merge(ContractRpcServer::into_rpc(RpcServerImpl))?;
    rpc_module.merge(EthereumRpcServer::into_rpc(RpcServerImpl))?;
    rpc_module.merge(FileRpcServer::into_rpc(RpcServerImpl))?;
    rpc_module.merge(TokenRpcServer::into_rpc(RpcServerImpl))?;
    rpc_module.merge(TopicRpcServer::into_rpc(RpcServerImpl))?;
    rpc_module.merge(ScheduleRpcServer::into_rpc(RpcServerImpl))?;

    let handle = server.start(rpc_module);

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
