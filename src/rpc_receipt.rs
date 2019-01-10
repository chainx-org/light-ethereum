// Copyright 2018 Chainpool

use ethereum_types::{U64, Address, Bloom, H256, U256};
use crate::rpc_log::Log;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
    /// Transaction Hash
    pub transaction_hash: Option<H256>,
    /// Transaction index
    pub transaction_index: Option<U256>,
    /// Block hash
    pub block_hash: Option<H256>,
    /// Sender
    pub from: Option<Address>,
    /// Recipient
    pub to: Option<Address>,
    /// Block number
    pub block_number: Option<U256>,
    /// Cumulative gas used
    pub cumulative_gas_used: U256,
    /// Gas used
    pub gas_used: Option<U256>,
    /// Contract address
    pub contract_address: Option<Address>,
    /// Logs
    pub logs: Vec<Log>,
    /// State Root
    #[serde(rename = "root")]
    pub state_root: Option<H256>,
    /// Logs bloom
    pub logs_bloom: Bloom,
    /// Status code
    #[serde(rename = "status")]
    pub status_code: Option<U64>,
}
