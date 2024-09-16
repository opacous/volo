use {
    crate::{get_headers_from_surf_resp, Response},
    http::{HeaderMap, HeaderValue, Version},
    http_types::{Extensions, Version::Http0_9},
};

/// Parts takes a [surf::Response] and extracts all the non-body juicy bits
/// non-destructively. Very similar to a [hyper::Parts].
pub struct Parts {
    pub status: http::StatusCode,
    pub version: http::Version,
    pub headers: HeaderMap<HeaderValue>,
    pub extensions: Extensions,
}

impl Parts {
    fn new(resp: &surf::Response) -> Self {
        Self {
            status: http::StatusCode::from_u16(resp.status() as u16).unwrap(),
            version: match resp.version().unwrap() {
                Version::Http0_9 => Version::HTTP_09,
                Version::Http1_0 => Version::HTTP_10,
                Version::Http1_1 => Version::HTTP_11,
                Version::Http2_0 => Version::HTTP_2,
                Version::Http3_0 => Version::HTTP_3,
            },
            headers: get_headers_from_surf_resp(&resp),
            extensions: Default::default(),
        }
    }
}
