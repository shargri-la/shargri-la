use crate::*;

/// Shard chain with the definition of the on-chain state transition rule.
pub struct Shard {
    pub id: usize,

    pub blocks: Vec<ShardBlock>,
    pub states: Vec<ShardState>,

    // Included in a state but only needs to be kept in a snapshot
    pub accounts: HashMap<Address, Account>,
    pub receipts: HashMap<TransactionHash, Receipt>,

    // Shard block proposer variables
    pub moving_accounts: HashMap<Address, Account>,
    pub mempool: Vec<(Transaction, Option<Receipt>)>,
    used_receipts: HashSet<TransactionHash>,
    account_nonce: HashMap<Address, Nonce>,
}

impl Shard {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            blocks: Vec::new(),
            states: vec![ShardState::new(INITIAL_BASE_FEE)],
            accounts: HashMap::new(),
            receipts: HashMap::new(),
            moving_accounts: HashMap::new(),
            used_receipts: HashSet::new(),
            mempool: Vec::new(),
            account_nonce: HashMap::new(),
        }
    }

    pub fn get_account(&self, addr: Address) -> Option<&Account> {
        self.accounts.get(&addr)
    }

    pub fn get_moving_account(&self, addr: Address) -> Option<&Account> {
        self.moving_accounts.get(&addr)
    }

    pub fn remove_account(&mut self, addr: Address) {
        self.accounts.remove(&addr);
        self.moving_accounts.remove(&addr);
    }

    fn move_account(&mut self, addr: Address) -> Data {
        let account = self
            .accounts
            .get(&addr)
            .expect("the account does not exist");
        let data = serde_json::to_string(account).unwrap();
        self.moving_accounts.insert(addr, account.clone());
        self.accounts.remove(&addr);
        data
    }

    fn insert_account(&mut self, mut account: Account) {
        account.shard_id = self.id;
        self.accounts.insert(account.addr, account);
    }

    pub fn push_transaction(&mut self, transaction: Transaction, receipt: Option<Receipt>) {
        self.mempool.push((transaction, receipt));
    }

    pub fn process_slot(&mut self) {
        // Sort in descending order by fee cap
        let base_fee = self.get_base_fee();
        self.mempool.sort_by(|a, b| {
            let b = std::cmp::min(b.0.fee_cap, b.0.gas_premium + base_fee);
            let a = std::cmp::min(a.0.fee_cap, a.0.gas_premium + base_fee);
            b.cmp(&a)
        });

        let mut block = ShardBlock::new(self.blocks.len() as Slot);

        let mut executed_num = 0;
        let mut skip_transactions = Vec::new();

        let mut receipts = HashMap::new();

        for (transaction, receipt) in self.mempool.clone().iter() {
            let estimated_gas = self.estimate_transaction_gas(transaction);

            // TODO: Run it and revert it if it doesn't work
            if block.gas_used + estimated_gas > BLOCK_GAS_LIMIT {
                break;
            }

            if transaction.fee_cap <= base_fee {
                break;
            }

            let (result, data, gas) = self.execute_transaction(transaction, receipt.clone());

            if result == TransactionExecutionResult::Skip {
                skip_transactions.push(transaction.clone());
                continue;
            } else {
                block.gas_used += gas;

                // No revert, so apply the state transitions
                block.executed_transactions.push(transaction.clone());
                let receipt = Receipt::new(
                    block.number,
                    transaction,
                    gas,
                    match result {
                        TransactionExecutionResult::Success => true,
                        _ => false,
                    },
                    data,
                );
                receipts.insert(transaction.hash, receipt.clone());
                self.receipts.insert(transaction.hash, receipt);
            }
            executed_num += 1;
        }
        // Removes transactions in excess of MEMPOOL_TRANSACTION_NUM
        self.mempool = self.mempool[executed_num..].to_vec();
        //self.mempool.extend(skip_transactions);

        self.mempool.sort_by(|a, b| {
            let b = std::cmp::min(b.0.fee_cap, b.0.gas_premium + base_fee);
            let a = std::cmp::min(a.0.fee_cap, a.0.gas_premium + base_fee);
            b.cmp(&a)
        });

        let length = std::cmp::min(self.mempool.len(), MEMPOOL_TRANSACTION_NUM);
        self.mempool = self.mempool[..length].to_vec();
        self.blocks.push(block);

        let mut state = self.generate_next_state();
        state.receipts = receipts;
        self.states.push(state);
    }

    fn generate_next_state(&mut self) -> ShardState {
        assert!(!self.blocks.is_empty());
        let prev_base_fee = self
            .states
            .last()
            .expect("the genesis state does not exist")
            .base_fee;
        let block_gas_used = self
            .blocks
            .last()
            .expect("the genesis block does not exist")
            .gas_used;
        let base_fee = ShardState::compute_updated_gasprice(prev_base_fee, block_gas_used);
        ShardState::new(base_fee)
    }

    /// Estimate the gas usage before including the transaction in the chain.
    fn estimate_transaction_gas(&self, transaction: &Transaction) -> Gas {
        transaction
            .functions
            .iter()
            .map(|function| function.gas())
            .sum()
    }

    pub fn get_base_fee(&self) -> GasPrice {
        self.states
            .last()
            .expect("the genesis state does not exist")
            .base_fee
    }

    /// Execute a transaction with validation.
    fn execute_transaction(
        &mut self,
        transaction: &Transaction,
        receipt: Option<Receipt>,
    ) -> (TransactionExecutionResult, Data, Gas) {
        // TODO: Vec<Data>
        assert_eq!(transaction.shard_id, self.id);
        let mut transaction_gas_used = 0;
        let mut data = Data::new();

        // nonce validation
        self.account_nonce.entry(transaction.from).or_insert(0);

        match transaction
            .nonce
            .cmp(self.account_nonce.get(&transaction.from).unwrap())
        {
            Ordering::Greater => (TransactionExecutionResult::Skip, data, 0),
            Ordering::Less => (TransactionExecutionResult::Fail, data, 0),
            Ordering::Equal => {
                *self.account_nonce.entry(transaction.from).or_insert(0) += 1; // TODO: nonce bug?

                let mut success = true;
                for function in &transaction.functions {
                    // If there is an illegal function, it will be terminated.
                    transaction_gas_used += function.gas();
                    let (success_func, data_func) = self.execute_function(function, &receipt);
                    if let Some(data_func) = data_func {
                        data = data_func;
                    }
                    success = success_func;
                }

                if success {
                    (
                        TransactionExecutionResult::Success,
                        data,
                        transaction_gas_used,
                    )
                } else {
                    (TransactionExecutionResult::Fail, data, transaction_gas_used)
                }
            }
        }
    }

    fn execute_function(
        &mut self,
        function: &Function,
        receipt: &Option<Receipt>,
    ) -> (bool, Option<Data>) {
        let mut data = None;
        if function.ftype == FunctionType::Transfer {
            if !self.accounts.contains_key(&function.source)
                && !self.accounts.contains_key(&function.target)
            {
                return (false, None);
            }
        } else if function.ftype == FunctionType::CreateCrossTransfer {
            if !self.accounts.contains_key(&function.target) {
                return (false, None);
            }

            if function.source != function.target {
                return (false, None);
            }
        } else if function.ftype == FunctionType::ApplyCrossTransfer {
        } else if function.ftype == FunctionType::CreateCrossTransferAll {
            if !self.accounts.contains_key(&function.target) {
                return (false, None);
            }

            data = Some(self.move_account(function.target));
        } else if function.ftype == FunctionType::ApplyCrossTransferAll {
            if receipt.is_none() {
                return (false, None);
            }

            let receipt = receipt.clone().unwrap();
            if self.used_receipts.contains(&receipt.transaction_hash) {
                return (false, None);
            }

            let data = receipt.data;
            let account: Account = match serde_json::from_str(&data) {
                Ok(res) => res,
                Err(_) => {
                    unreachable!();
                }
            };
            self.insert_account(account);
        } else {
            return (false, None);
        }
        (true, data)
    }
}
