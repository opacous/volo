#![doc(
    html_logo_url = "https://github.com/cloudwego/volo/raw/main/.github/assets/logo.png?sanitize=true"
)]
#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]
#![feature(impl_trait_in_assoc_type)]
#![feature(associated_type_bounds)]
#![feature(iter_collect_into)]

pub mod body;
pub mod client;
pub mod codec;
#[doc(hidden)]
pub mod codegen;
pub mod context;
pub mod layer;
pub mod message;
pub mod metadata;
pub mod request;
pub mod response;
pub mod server;
pub mod status;
pub mod transport;
mod sleep;
mod surf_helper;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
pub type BoxStream<'l, T> = std::pin::Pin<Box<dyn futures::Stream<Item = T> + Send + Sync + 'l>>;

// get_headers_from_surf_req gets all of headers of surf in a convenient way
pub fn get_headers_from_surf_req(req : &surf::Request) -> http::header::HeaderMap {
    // get all of the headers out in a &HeaderMap
    let mut current_header_map: http::header::HeaderMap = http::header::HeaderMap::new();

    req
        .iter()
        .for_each(|(name, value)| {
            value.iter().for_each(|individual_value| {
                current_header_map
                    .insert(
                        name.as_str().parse::<http::HeaderName>().unwrap(),
                        individual_value.as_str().parse::<http::HeaderValue>().unwrap()
                    );
            });
        });

    current_header_map
}

pub fn get_headers_from_surf_resp(resp : &surf::Response) -> http::header::HeaderMap {
    // get all of the headers out in a &HeaderMap
    let mut current_header_map: http::header::HeaderMap = http::header::HeaderMap::new();

    resp
        .iter()
        .for_each(|(name, value)| {
            value.iter().for_each(|individual_value| {
                current_header_map
                    .insert(
                        name.as_str().parse::<http::HeaderName>().unwrap(),
                        individual_value.as_str().parse::<http::HeaderValue>().unwrap()
                    );
            });
        });

    current_header_map
}


pub use client::Client;
pub use codec::decode::RecvStream;
pub use message::{RecvEntryMessage, SendEntryMessage};
pub use request::{IntoRequest, IntoStreamingRequest, Request};
pub use response::Response;
pub use status::{Code, Status};
