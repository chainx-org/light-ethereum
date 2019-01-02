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

//! Lazily-decoded owning views of RLP-encoded blockchain objects.
//! These views are meant to contain _trusted_ data -- without encoding
//! errors or inconsistencies.
//!
//! In general these views are useful when only a few fields of an object
//! are relevant. In these cases it's more efficient to decode the object piecemeal.
//! When the entirety of the object is needed, it's better to upgrade it to a fully
//! decoded object where parts like the hash can be saved.

use ethereum_types::{H256, Bloom, U256, Address};
use hash::keccak;
use crate::header::{BlockNumber, Header as FullHeader};
use heapsize::HeapSizeOf;
use rlp::{self, Rlp};
use crate::views::HeaderView;

/// Owning header view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header(Vec<u8>);

impl HeapSizeOf for Header {
	fn heap_size_of_children(&self) -> usize { self.0.heap_size_of_children() }
}

impl Header {
	/// Create a new owning header view.
	/// Expects the data to be an RLP-encoded header -- any other case will likely lead to
	/// panics further down the line.
	pub fn new(encoded: Vec<u8>) -> Self { Header(encoded) }

	/// Upgrade this encoded view to a fully owned `Header` object.
	pub fn decode(&self) -> Result<FullHeader, rlp::DecoderError> {
		rlp::decode(&self.0)
	}

	/// Get a borrowed header view onto the data.
	#[inline]
	pub fn view(&self) -> HeaderView { view!(HeaderView, &self.0) }

	/// Get the rlp of the header.
	#[inline]
	pub fn rlp(&self) -> Rlp { Rlp::new(&self.0) }

	/// Consume the view and return the raw bytes.
	pub fn into_inner(self) -> Vec<u8> { self.0 }
}

// forwarders to borrowed view.
impl Header {
	/// Returns the header hash.
	pub fn hash(&self) -> H256 { keccak(&self.0) }

	/// Returns the parent hash.
	pub fn parent_hash(&self) -> H256 { self.view().parent_hash() }

	/// Returns the uncles hash.
	pub fn uncles_hash(&self) -> H256 { self.view().uncles_hash() }

	/// Returns the author.
	pub fn author(&self) -> Address { self.view().author() }

	/// Returns the state root.
	pub fn state_root(&self) -> H256 { self.view().state_root() }

	/// Returns the transaction trie root.
	pub fn transactions_root(&self) -> H256 { self.view().transactions_root() }

	/// Returns the receipts trie root
	pub fn receipts_root(&self) -> H256 { self.view().receipts_root() }

	/// Returns the block log bloom
	pub fn log_bloom(&self) -> Bloom { self.view().log_bloom() }

	/// Difficulty of this block
	pub fn difficulty(&self) -> U256 { self.view().difficulty() }

	/// Number of this block.
	pub fn number(&self) -> BlockNumber { self.view().number() }

	/// Time this block was produced.
	pub fn timestamp(&self) -> u64 { self.view().timestamp() }

	/// Gas limit of this block.
	pub fn gas_limit(&self) -> U256 { self.view().gas_limit() }

	/// Total gas used in this block.
	pub fn gas_used(&self) -> U256 { self.view().gas_used() }

	/// Block extra data.
	pub fn extra_data(&self) -> Vec<u8> { self.view().extra_data() }

	/// Engine-specific seal fields.
	pub fn seal(&self) -> Vec<Vec<u8>> { self.view().seal() }
}
