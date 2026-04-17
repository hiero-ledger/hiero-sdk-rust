// SPDX-License-Identifier: Apache-2.0

use tonic::service::interceptor::InterceptedService;

/// Interceptor that adds the `x-user-agent` header to every outgoing gRPC request.
#[derive(Clone)]
pub(crate) struct SdkInterceptor;

impl tonic::service::Interceptor for SdkInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        request.metadata_mut().insert(
            "x-user-agent",
            format!("hiero-sdk-rust/{}", env!("CARGO_PKG_VERSION")).parse().unwrap(),
        );
        Ok(request)
    }
}

/// A tonic channel with the SDK interceptor applied.
pub(crate) type Channel = InterceptedService<tonic::transport::Channel, SdkInterceptor>;
