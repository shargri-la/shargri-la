use crate::*;

#[derive(Clone, Debug)]
/// Transaction, which consists of a set of functions.
pub struct Transaction {
    // The account whose public key corresponds to the signature and which pays the gas cost.
    pub from: Address,
    pub to: Address,

    pub shard_id: usize,
    pub nonce: Nonce,
    pub functions: Vec<Function>,
    pub gas_premium: GasPrice,
    pub fee_cap: GasPrice,
    pub gas_limit: Gas,
    pub hash: TransactionHash,
}

impl Transaction {
    pub fn new(
        from: Address,
        to: Address,
        shard_id: usize,
        functions: Vec<Function>,
        gas_premium: GasPrice,
        fee_cap: GasPrice,
        nonce: Nonce,
    ) -> Self {
        Self {
            from,
            to,
            shard_id,
            functions,
            gas_premium,
            fee_cap,
            gas_limit: Gas::MAX,
            nonce,
            hash: Transaction::generate_transaction_hash(from, shard_id, nonce),
        }
    }

    pub fn generate_transaction_hash(
        from: Address,
        shard_id: usize,
        nonce: Nonce,
    ) -> TransactionHash {
        // Assumption: (from, shard_id, nonce) is unique

        fn convert_bytes(mut x: u64) -> Vec<u8> {
            (0..8)
                .map(|_| {
                    let b = (x % 256) as u8;
                    x /= 256;
                    b
                })
                .collect()
        }

        let mut bytes = Vec::new();
        bytes.extend(convert_bytes(from as u64));
        bytes.extend(convert_bytes(shard_id as u64));
        bytes.extend(convert_bytes(nonce as u64));

        // simple hash, sdbm (but u64)
        let mut hash: u64 = 0;
        bytes.iter().for_each(|&byte| {
            let mut next_hash = byte as u64;
            next_hash = next_hash.overflowing_add(hash.overflowing_shl(6).0).0;
            next_hash = next_hash.overflowing_add(hash.overflowing_shl(16).0).0;
            next_hash = next_hash.overflowing_sub(hash).0;
            hash = next_hash;
        });
        hash
    }
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

pub type TransactionAndReceipt = (Transaction, Option<Receipt>);
