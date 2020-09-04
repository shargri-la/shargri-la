use crate::*;

// Ref: https://github.com/ethereum/eth2.0-specs/blob/v0.12.2/specs/phase0/beacon-chain.md
pub const SLOTS_PER_EPOCH: Slot = 32;
pub const SHARD_NUM: usize = 64;

// EIP-1559 Parameters
// Ref: https://github.com/ethereum/EIPs/blob/e320c9c341f30d77e41fbb389742d9a0b5b5a1e6/EIPS/eip-1559.md
pub const BLOCK_GAS_TARGET: Gas = 10_000_000;
pub const BLOCK_GAS_LIMIT: Gas = BLOCK_GAS_TARGET * 2;
pub const INITIAL_BASE_FEE: GasPrice = 1_000_000_000;
pub const BASE_FEE_MAX_CHANGE_DENOMINATOR: usize = 8;
pub const MAX_GASPRICE: GasPrice = 16_384_000_000_000;
pub const MIN_GASPRICE: GasPrice = 0;

pub const MEMPOOL_TRANSACTION_NUM: usize = 10_000;

// Gas
// TODO: to exactly guessed values
pub const GAS_TRANSFER: Gas = 21_000;
pub const GAS_CREATE_CROSS_TRANSFER: Gas = 31_785;
pub const GAS_APPLY_CROSS_TRANSFER: Gas = 52_820;
pub const GAS_CREATE_CROSS_TRANSFER_ALL: Gas = GAS_CREATE_CROSS_TRANSFER;
pub const GAS_APPLY_CROSS_TRANSFER_ALL: Gas = GAS_APPLY_CROSS_TRANSFER;
