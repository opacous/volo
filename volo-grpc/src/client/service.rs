use std::future::Future;
use std::task::Poll;
use http_body::Body;
use motore::Service;
use volo::context::Context;

pub trait GrpcService<ReqBody> {
    /// Responses body given by the service.
    type ResponseBody: Body;
    /// Errors produced by the service.
    type Error: Into<crate::Error>;
    /// The future response value.
    type Future: Future<Output = Result<http::Response<Self::ResponseBody>, Self::Error>>;

    /// Returns `Ready` when the service is able to process requests.
    ///
    /// Reference [`Service::poll_ready`].
    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>>;

    /// Process the request and return the response asynchronously.
    ///
    /// Reference [`Service::call`].
    fn call(&mut self, request: http::Request<ReqBody>) -> Self::Future;
}