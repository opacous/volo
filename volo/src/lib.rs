#![feature(impl_trait_in_assoc_type)]
#![doc(
    html_logo_url = "https://github.com/cloudwego/volo/raw/main/.github/assets/logo.png?sanitize=true"
)]
#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]

pub use async_trait::async_trait;
pub use motore::{layer, layer::Layer, service, Service};

pub mod context;
pub mod discovery;
pub mod loadbalance;
pub mod net;
pub mod util;
pub use hack::Unwrap;
#[cfg(target_family = "unix")]
pub mod hotrestart;

pub mod client;
mod hack;
mod macros;

pub use faststr::FastStr;
pub use metainfo::METAINFO;