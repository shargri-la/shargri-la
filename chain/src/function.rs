use crate::*;

/// Simplified rules of the on-chain virtual machine.
/// This follows the token contract design of Eth1x64 Variant 1 "Apostille".
/// Ref: https://github.com/ewasm/eth1x64/blob/cfa0317f29cbf5a8ef5f67612944cbb9ba38d5b4/variant1_token_examples.md
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FunctionType {
    /// Local balance transfer.
    Transfer,
    /// Initiate transfer of a balance to a different shard.
    CreateCrossTransfer,
    /// Process an incoming transfer.
    ApplyCrossTransfer,
    CreateCrossTransferAll,
    ApplyCrossTransferAll,
}

/// Unit of operation in a transaction.
#[derive(Clone, Debug)]
pub struct Function {
    // NOTE: In reality, we need "target shard" field for X-shard functions
    pub source: Address,
    pub target: Address,
    pub ftype: FunctionType,
    pub calldata: String, // TODO
}

impl Function {
    /// Calculate the gas of a function.
    pub fn gas(&self) -> Gas {
        match self.ftype {
            FunctionType::Transfer => GAS_TRANSFER,
            FunctionType::CreateCrossTransfer => GAS_CREATE_CROSS_TRANSFER,
            FunctionType::ApplyCrossTransfer => GAS_APPLY_CROSS_TRANSFER,
            FunctionType::CreateCrossTransferAll => GAS_CREATE_CROSS_TRANSFER_ALL,
            FunctionType::ApplyCrossTransferAll => GAS_APPLY_CROSS_TRANSFER_ALL,
        }
    }
}
