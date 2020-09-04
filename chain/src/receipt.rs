use crate::*;

#[derive(Clone, Debug)]
pub struct Receipt {
    slot_number: Slot,
    from: Address,
    to: Address,
    gas_used: Gas,
    pub status: bool,
    pub transaction_hash: TransactionHash,
    pub data: Data,
}

impl Receipt {
    pub fn new(
        slot_number: Slot,
        transaction: &Transaction,
        gas_used: Gas,
        status: bool,
        data: Data,
    ) -> Self {
        Self {
            slot_number,
            from: transaction.from,
            to: transaction.to,
            gas_used,
            status,
            transaction_hash: transaction.hash,
            data,
        }
    }
}
