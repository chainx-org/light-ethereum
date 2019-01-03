// Copyright 2018 Chainpool
extern crate common_types as types;
extern crate jsonrpc_core;
extern crate jsonrpc_minihttp_server;
extern crate keccak_hash as hash;
extern crate parity_bytes as bytes;
extern crate parity_machine;
#[macro_use]
extern crate jsonrpc_macros;
#[cfg(feature = "serialize")]
extern crate ethereum_types;
extern crate rustc_hex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate parity_codec as codec;
#[macro_use]
extern crate parity_codec_derive;

pub mod header;
#[macro_use]
pub mod views;
pub mod encoded;
mod rpc_bytes;

use jsonrpc_core::{IoHandler, Result};
use jsonrpc_minihttp_server::ServerBuilder;

build_rpc_trait! {
    pub trait Rpc {
        /// Adds two numbers and returns a result
        #[rpc(name = "ping")]
        fn ping(&self) -> Result<u64>;
    }
}

pub struct RpcImpl;
impl Rpc for RpcImpl {
    fn ping(&self) -> Result<u64> {
        Ok(911)
    }
}

fn main() {
    let mut io = IoHandler::new();
    io.extend_with(RpcImpl.to_delegate());

    let server = ServerBuilder::new(io)
        .threads(3)
        .start_http(&"127.0.0.1:3030".parse().unwrap())
        .unwrap();

    server.wait().unwrap();
}
