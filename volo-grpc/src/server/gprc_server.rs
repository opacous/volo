use std::time::Duration;
use async_std::io::{Read, Write};
use async_std::stream::Stream;
use motore::layer::{Layer, Stack};
use crate::{body};
use motore::Service;
use crate::{Request, Response};
use crate::body::Body;
use crate::server::{NamedService, Router};
use crate::status::BoxBody;

pub trait GrpcServer<L, Route> {
    fn timeout(self, timeout: Duration) -> Self;
    fn create_router_from_service<S>(&mut self, svc: S) -> Router<L>
        where
            S: Service<Request<Body>, Response<BoxBody>> //TODO: The first param should be the context!
            + NamedService
            + Clone
            + Send
            + 'static,
            S::Future: Send + 'static,
            L: Clone,
    ;
     fn add_optional_service<S>(&mut self, svc: Option<S>) -> Router<L>
        where
            S: Service<Request<Body>, Response<BoxBody>>
            + NamedService
            + Clone
            + Send
            + 'static,
            S::Future: Send + 'static,
            L: Clone,
    ;

    fn add_routes(&mut self, routes: Route) -> Router<L>
        where
            L: Clone,
    ;

    fn layer<NewLayer>(self, new_layer: NewLayer) -> Server<Stack<NewLayer, L>>;

    async fn serve_with_shutdown<S, I, F, IO, IE, ResBody>(
        self,
        svc: S,
        incoming: I,
        signal: Option<F>,
    ) -> Result<(), crate::Error>
        where
            L: Layer<S>,
            L::Service: Service<Request<Body>, Response=Response<ResBody>> + Clone + Send + 'static,
            <<L as Layer<S>>::Service as Service<Request<Body>>>::Future: Send + 'static,
            <<L as Layer<S>>::Service as Service<Request<Body>>>::Error: Into<crate::Error> + Send,
            I: Stream<Item=Result<IO, IE>>,
            IO: Read + Write + Unpin + Send + 'static,
            IO::ConnectInfo: Clone + Send + Sync + 'static,
            IE: Into<crate::Error>, // TODO: Things below are still fucked
            F: Future<Output=()>,
            ResBody: http_body::Body<Data=Bytes> + Send + 'static,
            ResBody::Error: Into<crate::Error>,
    ;
}


// A default batteries included `transport` server.
///
/// This is a wrapper around [`hyper::Server`] and provides an easy builder
/// pattern style builder [`Server`]. This builder exposes easy configuration parameters
/// for providing a fully featured http2 based gRPC server. This should provide
/// a very good out of the box http2 server for use with tonic but is also a
/// reference implementation that should be a good starting point for anyone
/// wanting to create a more complex and/or specific implementation.
#[derive(Clone)]
pub struct Server<L = Identity> {
    trace_interceptor: Option<TraceInterceptor>,
    concurrency_limit: Option<usize>,
    timeout: Option<Duration>,
    #[cfg(feature = "tls")]
    tls: Option<TlsAcceptor>,
    init_stream_window_size: Option<u32>,
    init_connection_window_size: Option<u32>,
    max_concurrent_streams: Option<u32>,
    tcp_keepalive: Option<Duration>,
    tcp_nodelay: bool,
    http2_keepalive_interval: Option<Duration>,
    http2_keepalive_timeout: Option<Duration>,
    http2_adaptive_window: Option<bool>,
    http2_max_pending_accept_reset_streams: Option<usize>,
    max_frame_size: Option<u32>,
    accept_http1: bool,
    service_builder: ServiceBuilder<L>,
}

