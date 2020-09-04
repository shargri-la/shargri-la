use crate::*;

#[derive(Clone)]
pub struct ShardBlock {
    pub executed_transactions: Vec<Transaction>,
    pub gas_used: Gas,
    pub number: Slot,
}

impl ShardBlock {
    pub fn new(number: Slot) -> Self {
        Self {
            executed_transactions: Vec::new(),
            gas_used: 0,
            number,
        }
    }
}
