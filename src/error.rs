// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! General error types for use in ethcore.

use std::{fmt, error};
use std::time::SystemTime;
use ethereum_types::{H256, U256, Address, Bloom};
use unexpected::{Mismatch, OutOfBounds};
//use ethtrie::TrieError;
use crate::header::BlockNumber;
use ethkey::Error as EthkeyError;
use rlp;

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
/// Errors concerning block processing.
pub enum BlockError {
	/// Block has too many uncles.
	TooManyUncles(OutOfBounds<usize>),
	/// Extra data is of an invalid length.
	ExtraDataOutOfBounds(OutOfBounds<usize>),
	/// Seal is incorrect format.
	InvalidSealArity(Mismatch<usize>),
	/// Block has too much gas used.
	TooMuchGasUsed(OutOfBounds<U256>),
	/// Uncles hash in header is invalid.
	InvalidUnclesHash(Mismatch<H256>),
	/// An uncle is from a generation too old.
	UncleTooOld(OutOfBounds<BlockNumber>),
	/// An uncle is from the same generation as the block.
	UncleIsBrother(OutOfBounds<BlockNumber>),
	/// An uncle is already in the chain.
	UncleInChain(H256),
	/// An uncle is included twice.
	DuplicateUncle(H256),
	/// An uncle has a parent not in the chain.
	UncleParentNotInChain(H256),
	/// State root header field is invalid.
	InvalidStateRoot(Mismatch<H256>),
	/// Gas used header field is invalid.
	InvalidGasUsed(Mismatch<U256>),
	/// Transactions root header field is invalid.
	InvalidTransactionsRoot(Mismatch<H256>),
	/// Difficulty is out of range; this can be used as an looser error prior to getting a definitive
	/// value for difficulty. This error needs only provide bounds of which it is out.
	DifficultyOutOfBounds(OutOfBounds<U256>),
	/// Difficulty header field is invalid; this is a strong error used after getting a definitive
	/// value for difficulty (which is provided).
	InvalidDifficulty(Mismatch<U256>),
	/// Seal element of type H256 (max_hash for Ethash, but could be something else for
	/// other seal engines) is out of bounds.
	MismatchedH256SealElement(Mismatch<H256>),
	/// Proof-of-work aspect of seal, which we assume is a 256-bit value, is invalid.
	InvalidProofOfWork(OutOfBounds<U256>),
	/// Some low-level aspect of the seal is incorrect.
	InvalidSeal,
	/// Gas limit header field is invalid.
	InvalidGasLimit(OutOfBounds<U256>),
	/// Receipts trie root header field is invalid.
	InvalidReceiptsRoot(Mismatch<H256>),
	/// Timestamp header field is invalid.
	InvalidTimestamp(OutOfBounds<SystemTime>),
	/// Timestamp header field is too far in future.
	TemporarilyInvalid(OutOfBounds<SystemTime>),
	/// Log bloom header field is invalid.
	InvalidLogBloom(Mismatch<Bloom>),
	/// Number field of header is invalid.
	InvalidNumber(Mismatch<BlockNumber>),
	/// Block number isn't sensible.
	RidiculousNumber(OutOfBounds<BlockNumber>),
	/// Too many transactions from a particular address.
	TooManyTransactions(Address),
	/// Parent given is unknown.
	UnknownParent(H256),
	/// Uncle parent given is unknown.
	UnknownUncleParent(H256),
	/// No transition to epoch number.
	UnknownEpochTransition(u64),
}

impl fmt::Display for BlockError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use self::BlockError::*;

		let msg = match *self {
			TooManyUncles(ref oob) => format!("Block has too many uncles. {}", oob),
			ExtraDataOutOfBounds(ref oob) => format!("Extra block data too long. {}", oob),
			InvalidSealArity(ref mis) => format!("Block seal in incorrect format: {}", mis),
			TooMuchGasUsed(ref oob) => format!("Block has too much gas used. {}", oob),
			InvalidUnclesHash(ref mis) => format!("Block has invalid uncles hash: {}", mis),
			UncleTooOld(ref oob) => format!("Uncle block is too old. {}", oob),
			UncleIsBrother(ref oob) => format!("Uncle from same generation as block. {}", oob),
			UncleInChain(ref hash) => format!("Uncle {} already in chain", hash),
			DuplicateUncle(ref hash) => format!("Uncle {} already in the header", hash),
			UncleParentNotInChain(ref hash) => format!("Uncle {} has a parent not in the chain", hash),
			InvalidStateRoot(ref mis) => format!("Invalid state root in header: {}", mis),
			InvalidGasUsed(ref mis) => format!("Invalid gas used in header: {}", mis),
			InvalidTransactionsRoot(ref mis) => format!("Invalid transactions root in header: {}", mis),
			DifficultyOutOfBounds(ref oob) => format!("Invalid block difficulty: {}", oob),
			InvalidDifficulty(ref mis) => format!("Invalid block difficulty: {}", mis),
			MismatchedH256SealElement(ref mis) => format!("Seal element out of bounds: {}", mis),
			InvalidProofOfWork(ref oob) => format!("Block has invalid PoW: {}", oob),
			InvalidSeal => "Block has invalid seal.".into(),
			InvalidGasLimit(ref oob) => format!("Invalid gas limit: {}", oob),
			InvalidReceiptsRoot(ref mis) => format!("Invalid receipts trie root in header: {}", mis),
			InvalidTimestamp(ref oob) => {
				let oob = oob.map(|st| st.elapsed().unwrap_or_default().as_secs());
				format!("Invalid timestamp in header: {}", oob)
			},
			TemporarilyInvalid(ref oob) => {
				let oob = oob.map(|st| st.elapsed().unwrap_or_default().as_secs());
				format!("Future timestamp in header: {}", oob)
			},
			InvalidLogBloom(ref oob) => format!("Invalid log bloom in header: {}", oob),
			InvalidNumber(ref mis) => format!("Invalid number in header: {}", mis),
			RidiculousNumber(ref oob) => format!("Implausible block number. {}", oob),
			UnknownParent(ref hash) => format!("Unknown parent: {}", hash),
			UnknownUncleParent(ref hash) => format!("Unknown uncle parent: {}", hash),
			UnknownEpochTransition(ref num) => format!("Unknown transition to epoch number: {}", num),
			TooManyTransactions(ref address) => format!("Too many transactions from: {}", address),
		};

		f.write_fmt(format_args!("Block error ({})", msg))
	}
}

impl error::Error for BlockError {
	fn description(&self) -> &str {
		"Block error"
	}
}

error_chain! {
	types {
		Error, ErrorKind, ErrorResultExt, EthcoreResult;
	}

	foreign_links {
		//Trie(TrieError) #[doc = "Error concerning TrieDBs."];
		Block(BlockError) #[doc = "Error concerning block processing."];
		Ethkey(EthkeyError) #[doc = "Ethkey error."];
		Decoder(rlp::DecoderError) #[doc = "RLP decoding errors"];
	}

	errors {
		#[doc = "PoW hash is invalid or out of date."]
		PowHashInvalid {
			description("PoW hash is invalid or out of date.")
			display("PoW hash is invalid or out of date.")
		}

		#[doc = "The value of the nonce or mishash is invalid."]
		PowInvalid {
			description("The value of the nonce or mishash is invalid.")
			display("The value of the nonce or mishash is invalid.")
		}
	}
}
