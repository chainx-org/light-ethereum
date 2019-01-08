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

use std::path::Path;
use std::cmp;
use std::collections::BTreeMap;
use std::sync::Arc;
use hash::{KECCAK_EMPTY_LIST_RLP};
use ethash::{self, quick_get_difficulty, slow_hash_block_number, EthashManager, OptimizeFor};
use ethereum_types::{H256, H64, U256};
use unexpected::{OutOfBounds, Mismatch};
use crate::error::{BlockError, Error};
use crate::header::{Header, BlockNumber, ExtendedHeader};
use ethjson;
use rlp::Rlp;

/// Number of blocks in an ethash snapshot.
// make dependent on difficulty incrment divisor?
const SNAPSHOT_BLOCKS: u64 = 5000;
/// Maximum number of blocks allowed in an ethash snapshot.
const MAX_SNAPSHOT_BLOCKS: u64 = 30000;

/// Ethash specific seal
#[derive(Debug, PartialEq)]
pub struct Seal {
	/// Ethash seal mix_hash
	pub mix_hash: H256,
	/// Ethash seal nonce
	pub nonce: H64,
}

impl Seal {
	/// Tries to parse rlp as ethash seal.
	pub fn parse_seal<T: AsRef<[u8]>>(seal: &[T]) -> Result<Self, Error> {
		if seal.len() != 2 {
			return Err(BlockError::InvalidSealArity(
				Mismatch {
					expected: 2,
					found: seal.len()
				}
			).into());
		}

		let mix_hash = Rlp::new(seal[0].as_ref()).as_val::<H256>()?;
		let nonce = Rlp::new(seal[1].as_ref()).as_val::<H64>()?;
		let seal = Seal {
			mix_hash,
			nonce,
		};

		Ok(seal)
	}
}

/// Ethash params.
#[derive(Debug, PartialEq)]
pub struct EthashParams {
	/// Minimum difficulty.
	pub minimum_difficulty: U256,
	/// Difficulty bound divisor.
	pub difficulty_bound_divisor: U256,
	/// Difficulty increment divisor.
	pub difficulty_increment_divisor: u64,
	/// Metropolis difficulty increment divisor.
	pub metropolis_difficulty_increment_divisor: u64,
	/// Block duration.
	pub duration_limit: u64,
	/// Homestead transition block number.
	pub homestead_transition: u64,
	/// Transition block for a change of difficulty params (currently just bound_divisor).
	pub difficulty_hardfork_transition: u64,
	/// Difficulty param after the difficulty transition.
	pub difficulty_hardfork_bound_divisor: U256,
	/// Block on which there is no additional difficulty from the exponential bomb.
	pub bomb_defuse_transition: u64,
	/// Number of first block where EIP-100 rules begin.
	pub eip100b_transition: u64,
	/// Number of first block where ECIP-1010 begins.
	pub ecip1010_pause_transition: u64,
	/// Number of first block where ECIP-1010 ends.
	pub ecip1010_continue_transition: u64,
	/// Total block number for one ECIP-1017 era.
	pub ecip1017_era_rounds: u64,
	/// Block reward in base units.
	pub block_reward: BTreeMap<BlockNumber, U256>,
	/// EXPIP-2 block height
	pub expip2_transition: u64,
	/// EXPIP-2 duration limit
	pub expip2_duration_limit: u64,
	/// Block reward contract transition block.
	pub block_reward_contract_transition: u64,
	/// Block reward contract.
	//pub block_reward_contract: Option<BlockRewardContract>,
	/// Difficulty bomb delays.
	pub difficulty_bomb_delays: BTreeMap<BlockNumber, BlockNumber>,
}

impl From<ethjson::spec::EthashParams> for EthashParams {
	fn from(p: ethjson::spec::EthashParams) -> Self {
		EthashParams {
			minimum_difficulty: p.minimum_difficulty.into(),
			difficulty_bound_divisor: p.difficulty_bound_divisor.into(),
			difficulty_increment_divisor: p.difficulty_increment_divisor.map_or(10, Into::into),
			metropolis_difficulty_increment_divisor: p.metropolis_difficulty_increment_divisor.map_or(9, Into::into),
			duration_limit: p.duration_limit.map_or(0, Into::into),
			homestead_transition: p.homestead_transition.map_or(0, Into::into),
			difficulty_hardfork_transition: p.difficulty_hardfork_transition.map_or(u64::max_value(), Into::into),
			difficulty_hardfork_bound_divisor: p.difficulty_hardfork_bound_divisor.map_or(p.difficulty_bound_divisor.into(), Into::into),
			bomb_defuse_transition: p.bomb_defuse_transition.map_or(u64::max_value(), Into::into),
			eip100b_transition: p.eip100b_transition.map_or(u64::max_value(), Into::into),
			ecip1010_pause_transition: p.ecip1010_pause_transition.map_or(u64::max_value(), Into::into),
			ecip1010_continue_transition: p.ecip1010_continue_transition.map_or(u64::max_value(), Into::into),
			ecip1017_era_rounds: p.ecip1017_era_rounds.map_or(u64::max_value(), Into::into),
			block_reward: p.block_reward.map_or_else(
				|| {
					let mut ret = BTreeMap::new();
					ret.insert(0, U256::zero());
					ret
				},
				|reward| {
					match reward {
						ethjson::spec::BlockReward::Single(reward) => {
							let mut ret = BTreeMap::new();
							ret.insert(0, reward.into());
							ret
						},
						ethjson::spec::BlockReward::Multi(multi) => {
							multi.into_iter()
								.map(|(block, reward)| (block.into(), reward.into()))
								.collect()
						},
					}
				}),
			expip2_transition: p.expip2_transition.map_or(u64::max_value(), Into::into),
			expip2_duration_limit: p.expip2_duration_limit.map_or(30, Into::into),
			block_reward_contract_transition: p.block_reward_contract_transition.map_or(0, Into::into),
			/*block_reward_contract: match (p.block_reward_contract_code, p.block_reward_contract_address) {
				(Some(code), _) => Some(BlockRewardContract::new_from_code(Arc::new(code.into()))),
				(_, Some(address)) => Some(BlockRewardContract::new_from_address(address.into())),
				(None, None) => None,
			},*/
			difficulty_bomb_delays: p.difficulty_bomb_delays.unwrap_or_default().into_iter()
				.map(|(block, delay)| (block.into(), delay.into()))
				.collect()
		}
	}
}

/// Engine using Ethash proof-of-work consensus algorithm, suitable for Ethereum
/// mainnet chains in the Olympic, Frontier and Homestead eras.
pub struct Ethash {
	ethash_params: EthashParams,
	pow: EthashManager,
}

impl Ethash {
	/// Create a new instance of Ethash engine
	pub fn new<T: Into<Option<OptimizeFor>>>(
		cache_dir: &Path,
		ethash_params: EthashParams,
		optimize_for: T,
	) -> Arc<Self> {
		Arc::new(Ethash {
			ethash_params,
			pow: EthashManager::new(cache_dir.as_ref(), optimize_for.into()),
		})
	}

	fn verify_block_basic(&self, header: &Header) -> Result<(), Error> {
		// check the seal fields.
		let seal = Seal::parse_seal(header.seal())?;

		// TODO: consider removing these lines.
		let min_difficulty = self.ethash_params.minimum_difficulty;
		if header.difficulty() < &min_difficulty {
			return Err(From::from(BlockError::DifficultyOutOfBounds(OutOfBounds { min: Some(min_difficulty), max: None, found: header.difficulty().clone() })))
		}

		let difficulty = ethash::boundary_to_difficulty(&H256(quick_get_difficulty(
			&header.bare_hash().0,
			seal.nonce.low_u64(),
			&seal.mix_hash.0
		)));

		if &difficulty < header.difficulty() {
			return Err(From::from(BlockError::InvalidProofOfWork(OutOfBounds { min: Some(header.difficulty().clone()), max: None, found: difficulty })));
		}

		Ok(())
	}

	fn verify_block_unordered(&self, header: &Header) -> Result<(), Error> {
		let seal = Seal::parse_seal(header.seal())?;

		let result = self.pow.compute_light(header.number() as u64, &header.bare_hash().0, seal.nonce.low_u64());
		let mix = H256(result.mix_hash);
		let difficulty = ethash::boundary_to_difficulty(&H256(result.value));
		/*trace!(target: "miner", "num: {num}, seed: {seed}, h: {h}, non: {non}, mix: {mix}, res: {res}",
			   num = header.number() as u64,
			   seed = H256(slow_hash_block_number(header.number() as u64)),
			   h = header.bare_hash(),
			   non = seal.nonce.low_u64(),
			   mix = H256(result.mix_hash),
			   res = H256(result.value));
        */
		if mix != seal.mix_hash {
			return Err(From::from(BlockError::MismatchedH256SealElement(Mismatch { expected: mix, found: seal.mix_hash })));
		}
		if &difficulty < header.difficulty() {
			return Err(From::from(BlockError::InvalidProofOfWork(OutOfBounds { min: Some(header.difficulty().clone()), max: None, found: difficulty })));
		}
		Ok(())
	}

	fn verify_block_family(&self, header: &Header, parent: &Header) -> Result<(), Error> {
		// we should not calculate difficulty for genesis blocks
		if header.number() == 0 {
			return Err(From::from(BlockError::RidiculousNumber(OutOfBounds { min: Some(1), max: None, found: header.number() })));
		}

		// Check difficulty is correct given the two timestamps.
		let expected_difficulty = self.calculate_difficulty(header, parent);
		if header.difficulty() != &expected_difficulty {
			return Err(From::from(BlockError::InvalidDifficulty(Mismatch { expected: expected_difficulty, found: header.difficulty().clone() })))
		}

		Ok(())
	}
}

impl Ethash {
	fn calculate_difficulty(&self, header: &Header, parent: &Header) -> U256 {
		const EXP_DIFF_PERIOD: u64 = 100_000;
		if header.number() == 0 {
			panic!("Can't calculate genesis block difficulty");
		}

		let parent_has_uncles = parent.uncles_hash() != &KECCAK_EMPTY_LIST_RLP;

		let min_difficulty = self.ethash_params.minimum_difficulty;

		let difficulty_hardfork = header.number() >= self.ethash_params.difficulty_hardfork_transition;
		let difficulty_bound_divisor = if difficulty_hardfork {
			self.ethash_params.difficulty_hardfork_bound_divisor
		} else {
			self.ethash_params.difficulty_bound_divisor
		};

		let expip2_hardfork = header.number() >= self.ethash_params.expip2_transition;
		let duration_limit = if expip2_hardfork {
			self.ethash_params.expip2_duration_limit
		} else {
			self.ethash_params.duration_limit
		};

		let frontier_limit = self.ethash_params.homestead_transition;

		let mut target = if header.number() < frontier_limit {
			if header.timestamp() >= parent.timestamp() + duration_limit {
				*parent.difficulty() - (*parent.difficulty() / difficulty_bound_divisor)
			} else {
				*parent.difficulty() + (*parent.difficulty() / difficulty_bound_divisor)
			}
		} else {
			//trace!(target: "ethash", "Calculating difficulty parent.difficulty={}, header.timestamp={}, parent.timestamp={}", parent.difficulty(), header.timestamp(), parent.timestamp());
			//block_diff = parent_diff + parent_diff // 2048 * max(1 - (block_timestamp - parent_timestamp) // 10, -99)
			let (increment_divisor, threshold) = if header.number() < self.ethash_params.eip100b_transition {
				(self.ethash_params.difficulty_increment_divisor, 1)
			} else if parent_has_uncles {
				(self.ethash_params.metropolis_difficulty_increment_divisor, 2)
			} else {
				(self.ethash_params.metropolis_difficulty_increment_divisor, 1)
			};

			let diff_inc = (header.timestamp() - parent.timestamp()) / increment_divisor;
			if diff_inc <= threshold {
				*parent.difficulty() + *parent.difficulty() / difficulty_bound_divisor * U256::from(threshold - diff_inc)
			} else {
				let multiplier: U256 = cmp::min(diff_inc - threshold, 99).into();
				parent.difficulty().saturating_sub(
					*parent.difficulty() / difficulty_bound_divisor * multiplier
				)
			}
		};
		target = cmp::max(min_difficulty, target);
		if header.number() < self.ethash_params.bomb_defuse_transition {
			if header.number() < self.ethash_params.ecip1010_pause_transition {
				let mut number = header.number();
				let original_number = number;
				for (block, delay) in &self.ethash_params.difficulty_bomb_delays {
					if original_number >= *block {
						number = number.saturating_sub(*delay);
					}
				}
				let period = (number / EXP_DIFF_PERIOD) as usize;
				if period > 1 {
					target = cmp::max(min_difficulty, target + (U256::from(1) << (period - 2)));
				}
			} else if header.number() < self.ethash_params.ecip1010_continue_transition {
				let fixed_difficulty = ((self.ethash_params.ecip1010_pause_transition / EXP_DIFF_PERIOD) - 2) as usize;
				target = cmp::max(min_difficulty, target + (U256::from(1) << fixed_difficulty));
			} else {
				let period = ((parent.number() + 1) / EXP_DIFF_PERIOD) as usize;
				let delay = ((self.ethash_params.ecip1010_continue_transition - self.ethash_params.ecip1010_pause_transition) / EXP_DIFF_PERIOD) as usize;
				target = cmp::max(min_difficulty, target + (U256::from(1) << (period - delay - 2)));
			}
		}
		target
	}
}

fn ecip1017_eras_block_reward(era_rounds: u64, mut reward: U256, block_number:u64) -> (u64, U256) {
	let eras = if block_number != 0 && block_number % era_rounds == 0 {
		block_number / era_rounds - 1
	} else {
		block_number / era_rounds
	};
	let mut divi = U256::from(1);
	for _ in 0..eras {
		reward = reward * U256::from(4);
		divi = divi * U256::from(5);
	}
	reward = reward / divi;
	(eras, reward)
}
