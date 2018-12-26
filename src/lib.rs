// Copyright 2018 Chainpool
extern crate keccak_hash as hash;
extern crate parity_bytes as bytes;
extern crate common_types as types;
extern crate parity_machine;
extern crate jsonrpc_core;
extern crate jsonrpc_minihttp_server;
#[macro_use]
extern crate jsonrpc_macros;

mod header;

use jsonrpc_core::{IoHandler, Result};
use jsonrpc_minihttp_server::{ServerBuilder};

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
