use std::{io, marker::PhantomData};

use futures::Future;
use http::{header::{CONTENT_TYPE, TE}, HeaderValue, uri};
use motore::Service;
use tower::{util::ServiceExt, Service as TowerService};
use volo::{net::Address, Unwrap};

use super::connect::Connector;
use crate::{
    client::Http2Config,
    codec::{
        compression::{CompressionEncoding, ACCEPT_ENCODING_HEADER, ENCODING_HEADER},
        decode::Kind,
    },
    context::{ClientContext, Config},
    Code, Request, Response, Status,
};

use surf::{Client, Body, Url};
use uri::{Uri, Builder};

/// A simple wrapper of [`surf::Client`] that implements [`Service`]
/// to make outgoing requests.
pub struct ClientTransport<U> {
    http_client: Client,
    _marker: PhantomData<fn(U)>,
}

impl<U> Clone for ClientTransport<U> {
    fn clone(&self) -> Self {
        Self {
            http_client: self.http_client.clone(),
            _marker: self._marker,
        }
    }
}

impl<U> ClientTransport<U> {
    /// Creates a new [`ClientTransport`] by setting the underlying connection
    /// with the given config.
    pub fn new(http2_config: &Http2Config, rpc_config: &Config) -> Self {
        let config = volo::net::dial::Config::new(
            rpc_config.connect_timeout,
            rpc_config.read_timeout,
            rpc_config.write_timeout,
        );
        let http = Client::new();
            // .http2_only(!http2_config.accept_http1)
            // .http2_initial_stream_window_size(http2_config.init_stream_window_size)
            // .http2_initial_connection_window_size(http2_config.init_connection_window_size)
            // .http2_max_frame_size(http2_config.max_frame_size)
            // .http2_adaptive_window(http2_config.adaptive_window)
            // .http2_keep_alive_interval(http2_config.http2_keepalive_interval)
            // .http2_keep_alive_timeout(http2_config.http2_keepalive_timeout)
            // .http2_keep_alive_while_idle(http2_config.http2_keepalive_while_idle)
            // .http2_max_concurrent_reset_streams(http2_config.max_concurrent_reset_streams)
            // .http2_max_send_buf_size(http2_config.max_send_buf_size)
            // .retry_canceled_requests(http2_config.retry_canceled_requests)
            // .build(Connector::new(Some(config)));

        ClientTransport {
            http_client: http,
            _marker: PhantomData,
        }
    }
}

impl<T, U> Service<ClientContext, Request<T>> for ClientTransport<U>
    where
        T: crate::message::SendEntryMessage + Send + 'static,
        U: crate::message::RecvEntryMessage + 'static,
{
    type Response = Response<U>;

    type Error = Status;

    type Future<'cx> = impl Future<Output=Result<Self::Response, Self::Error>> + 'cx;

    fn call<'cx, 's>(
        &'s self,
        cx: &'cx mut ClientContext,
        volo_req: Request<T>,
    ) -> Self::Future<'cx>
        where
            's: 'cx,
    {
        let mut http_client = self.http_client.clone();
        async move {
            // SAFETY: parameters controlled by volo-grpc are guaranteed to be valid.
            // get the call address from the context
            let target = cx
                .rpc_info
                .callee()
                .volo_unwrap()
                .address()
                .ok_or_else(|| {
                    io::Error::new(std::io::ErrorKind::InvalidData, "address is required")
                })?;

            let (metadata, extensions, message) = volo_req.into_parts();
            let path = cx.rpc_info.method().volo_unwrap();
            let rpc_config = cx.rpc_info.config().volo_unwrap();
            let accept_compressions = &rpc_config.accept_compressions;

            // select the compression algorithm with the highest priority by user's config
            let send_compression = rpc_config
                .send_compressions
                .as_ref()
                .map(|config| config[0]);

            let body = message.into_body(send_compression);

            // building the request with the compressed body
            let mut req_in_construction =
                surf::post(build_uri(target, path).into())
                .body(surf::Body::from_reader(body))
                .header(TE.into(), HeaderValue::from_static("trailers").into())
                .header(CONTENT_TYPE.into(), HeaderValue::from_static("application/grpc").into());

            // *req_in_construction.version_mut() = http::Version::HTTP_2;
            // *req_in_construction.headers_mut() = metadata.into_headers();
            *req_in_construction.extensions_mut() = extensions;

            // insert compression headers
            if let Some(send_compression) = send_compression {
                req_in_construction.
                    header(ENCODING_HEADER.into(), send_compression.into_header_value().into());
            }
            if let Some(accept_compressions) = accept_compressions {
                if !accept_compressions.is_empty() {
                    if let Some(header_value) = accept_compressions[0]
                        .into_accept_encoding_header_value(accept_compressions)
                    {
                        req_in_construction.
                            header(ACCEPT_ENCODING_HEADER.into(), header_value.into());
                    }
                }
            }

            // actually building the request
            let req = req_in_construction.build();

            // call the service through surf client
            let resp = http_client
                .ready()
                .await
                .map_err(|err| Status::from_error(err.into()))?
                .call(req)
                .await
                .map_err(|err| Status::from_error(err.into()))?;

            let status_code = resp.status();
            let headers = resp.headers();

            if let Some(status) = Status::from_header_map(headers) {
                if status.code() != Code::Ok {
                    return Err(status);
                }
            }

            let accept_compression = CompressionEncoding::from_encoding_header(
                headers,
                &rpc_config.accept_compressions,
            )?;

            let (parts, body) = resp.into_parts();

            let body = U::from_body(
                Some(path),
                body,
                Kind::Response(status_code),
                accept_compression,
            )?;
            let resp = http::Response::from_parts(parts, body);
            Ok(Response::from_http(resp))
        }
    }
}

fn build_uri(addr: Address, path: &str) -> Uri {
    match addr {
        Address::Ip(ip) => Builder::new()
            .scheme(http::uri::Scheme::HTTP)
            .authority(ip.to_string())
            .path_and_query(path)
            .build()
            .expect("fail to build ip uri"),
        #[cfg(target_family = "unix")]
        Address::Unix(unix) => Url::builder()
            .scheme("http+unix")
            .authority(hex::encode(unix.to_string_lossy().as_bytes()))
            .path_and_query(path)
            .build()
            .expect("fail to build unix uri"),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_build_uri_ip() {
        let addr = "127.0.0.1:8000".parse::<std::net::SocketAddr>().unwrap();
        let path = "/path?query=1";
        let uri = "http://127.0.0.1:8000/path?query=1"
            .parse::<http::Uri>()
            .unwrap();
        assert_eq!(super::build_uri(volo::net::Address::from(addr), path), uri);
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn test_build_uri_unix() {
        use std::borrow::Cow;

        let addr = "/tmp/rpc.sock".parse::<std::path::PathBuf>().unwrap();
        let path = "/path?query=1";
        let uri = "http+unix://2f746d702f7270632e736f636b/path?query=1"
            .parse::<http::Uri>()
            .unwrap();
        assert_eq!(
            super::build_uri(volo::net::Address::from(Cow::from(addr)), path),
            uri
        );
    }
}
