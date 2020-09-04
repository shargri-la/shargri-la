#[derive(PartialEq)]
pub enum TransactionExecutionResult {
    Success,
    Fail,
    Skip, // Nonce error
}
