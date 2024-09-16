#![feature(impl_trait_in_assoc_type)]

use std::net::SocketAddr;

use volo_grpc::server::{Server, ServiceBuilder};

pub struct S;

#[volo::main]
async fn main() {
    let addr: SocketAddr = "[::]:8080".parse().unwrap();
    let addr = volo::net::Address::from(addr);

    Server::new()
        .add_service(ServiceBuilder::new(examples_now::gen::volo_gen::hello::GreeterServer::new(S)).build())
        .run(addr, hyper)
        .await
        .unwrap();
}
