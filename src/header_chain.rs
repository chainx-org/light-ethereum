// Copyright 2019 Chainpool

use crate::encoded;
use ethereum_types::{H256, U256};

pub struct BlockDescriptor {
    pub hash: H256,
    pub number: u64,
    pub total_difficulty: U256,
}

struct Candidate {
    hash: H256,
    parent_hash: H256,
    total_difficulty: U256,
}

pub struct HeaderChain {
    genesis_header: encoded::Header,
    best_block: BlockDescriptor,
}
