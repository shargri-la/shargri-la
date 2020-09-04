use crate::*;

#[derive(Clone)]
pub struct ShardState {
    pub base_fee: GasPrice,
    pub receipts: HashMap<TransactionHash, Receipt>,
}

impl ShardState {
    pub fn new(base_fee: GasPrice) -> Self {
        Self {
            base_fee,
            receipts: HashMap::new(),
        }
    }

    pub fn compute_updated_gasprice(prev_base_fee: GasPrice, block_gas_used: Gas) -> GasPrice {
        if block_gas_used > BLOCK_GAS_TARGET {
            let delta = prev_base_fee * (block_gas_used - BLOCK_GAS_TARGET)
                / BLOCK_GAS_TARGET as GasPrice
                / BASE_FEE_MAX_CHANGE_DENOMINATOR as GasPrice;
            std::cmp::min(prev_base_fee + delta, MAX_GASPRICE)
        } else {
            let delta = prev_base_fee * (BLOCK_GAS_TARGET - block_gas_used)
                / BLOCK_GAS_TARGET as GasPrice
                / BASE_FEE_MAX_CHANGE_DENOMINATOR as GasPrice;
            std::cmp::max(prev_base_fee, MIN_GASPRICE + delta) - delta
        }
    }
}
