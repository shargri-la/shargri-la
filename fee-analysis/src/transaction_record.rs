use crate::*;

#[allow(dead_code)]
type Timestamp = String;
#[allow(dead_code)]
type Hash = String;

/// CSV data records in BigQuery Ethereum.
#[derive(Debug, Deserialize, Clone)]
pub struct TransactionRecord {
    //hash: Hash,
    pub nonce: u64,
    pub transaction_index: u64,
    pub from_address: String,
    pub to_address: String,
    pub value: Wei,
    pub gas: u64,
    pub gas_price: u64,
    //pub input: String,
    pub receipt_cumulative_gas_used: u64,
    pub receipt_gas_used: u64,
    pub receipt_contract_address: String,
    pub receipt_root: String,
    pub receipt_status: u64,
    pub block_timestamp: Timestamp,
    pub block_number: u64,
    //pub block_hash: Hash,
}
