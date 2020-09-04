use crate::*;

/// Sharded blockchain and users.
pub struct Environment {
    pub blockchain: ShardedBlockchain,
    pub user_graph: UserGraph,
    pub users: Vec<User>,
    pub user_num_mem: Vec<Vec<usize>>,
    pub mempool_tx_mem: Vec<Vec<usize>>,
    user_num: usize,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            blockchain: ShardedBlockchain::new(),
            user_graph: UserGraph::new(),
            users: Vec::new(),
            user_num_mem: Vec::new(),
            mempool_tx_mem: Vec::new(),
            user_num: DEFAULT_USER_NUM,
        }
    }

    pub fn setup(&mut self, arg_matches: &clap::ArgMatches) {
        println!("Setup...");

        let percentage_of_minimum =
            if let Some(percentage_of_minimum) = arg_matches.value_of("PERCENTAGE_OF_MINIMUM") {
                percentage_of_minimum
                    .parse()
                    .expect("PERCENTAGE_OF_MINIMUM must be a positive real number")
            } else {
                DEFAULT_PERCENTAGE_OF_MINIMUM
            };

        let percentage_of_weighted_random = if let Some(percentage_of_weighted_random) =
            arg_matches.value_of("PERCENTAGE_OF_WEIGHTED_RANDOM")
        {
            percentage_of_weighted_random
                .parse()
                .expect("PERCENTAGE_OF_WEIGHTED_RANDOM must be a positive real number")
        } else {
            DEFAULT_PERCENTAGE_OF_WEIGHTED_RANDOM
        };

        let percentage_of_decreasing_minimum = if let Some(percentage_of_decreasing_minimum) =
            arg_matches.value_of("PERCENTAGE_OF_DECREASING_MINIMUM")
        {
            percentage_of_decreasing_minimum
                .parse()
                .expect("PERCENTAGE_OF_DECREASING_MINIMUM must be a positive real number")
        } else {
            DEFAULT_PERCENTAGE_OF_DECREASING_MINIMUM
        };

        let popular_user_exists = arg_matches.is_present("POPULAR_USER_EXISTS");
        let popular_user_is_switcher = arg_matches.is_present("POPULAR_USER_IS_SWITCHER");

        if let Some(user_num) = arg_matches.value_of("USER_NUM") {
            self.user_num = user_num
                .parse()
                .expect("user_num must be a positive integer");
        }

        if let Some(past_log_file_name) = arg_matches.value_of("BIG_QUERY_CSV") {
            self.user_graph = UserGraph::new_from_eth1_data(self.user_num, past_log_file_name);
        } else {
            self.user_graph =
                UserGraph::new_random(self.user_num, popular_user_exists, popular_user_is_switcher);
        }

        (0..self.user_graph.nodes.len()).for_each(|addr| {
            self.deploy_account(addr);

            #[allow(clippy::collapsible_if)]
            if popular_user_exists && addr == POPULAR_USER_ADDRESS {
                if popular_user_is_switcher {
                    self.users.push(User::new(addr, StrategyType::Minimum));
                } else {
                    self.users.push(User::new(addr, StrategyType::NonSwitcher));
                }
            } else {
                if addr <= (percentage_of_minimum * self.user_graph.nodes.len() as f64) as usize {
                    self.users.push(User::new(addr, StrategyType::Minimum));
                } else if addr
                    <= ((percentage_of_minimum + percentage_of_weighted_random)
                        * self.user_graph.nodes.len() as f64) as usize
                {
                    self.users
                        .push(User::new(addr, StrategyType::WeightedRandom));
                } else if addr
                    <= ((percentage_of_minimum
                        + percentage_of_weighted_random
                        + percentage_of_decreasing_minimum)
                        * self.user_graph.nodes.len() as f64) as usize
                {
                    self.users
                        .push(User::new(addr, StrategyType::DecreasingMinimum));
                } else {
                    self.users.push(User::new(addr, StrategyType::NonSwitcher));
                }
            }
        });

        println!("Setup is complete.\n");
    }

    /// Deploy new account.
    fn deploy_account(&mut self, addr: usize) {
        let shard_id = addr % self.blockchain.shards.len();
        let account = Account::new(addr, shard_id);
        self.blockchain.account_num += 1;
        self.blockchain.addr_to_shard_id.insert(addr, shard_id);

        self.blockchain.shards[shard_id]
            .accounts
            .entry(addr)
            .or_insert(account);
    }

    /// Next step.
    pub fn process(&mut self) {
        let transactions = self.generate_transactions_per_slot();
        self.broadcast_transactions_per_slot(transactions);
        self.blockchain.process_slots(self.blockchain.slot + 1);

        let account_num = self
            .blockchain
            .shards
            .iter()
            .map(|shard| shard.accounts.len())
            .collect();
        self.user_num_mem.push(account_num);

        let mempool_tx_num = self
            .blockchain
            .shards
            .iter()
            .map(|shard| shard.mempool.len())
            .collect();
        self.mempool_tx_mem.push(mempool_tx_num);

        self.print_statistics();
    }

    fn print_statistics(&self) {
        const DEBUG_SHARD_NUM: usize = 10;
        let debug_shard_num = std::cmp::min(DEBUG_SHARD_NUM, SHARD_NUM);

        print!("{:>20}", "mempool:");
        (0..debug_shard_num).for_each(|i| {
            print!(" {:9}", self.blockchain.shards[i].mempool.len());
        });
        println!("  ...");

        print!("{:>20}", "gas used:");
        (0..debug_shard_num).for_each(|i| {
            print!(
                " {:9}",
                self.blockchain.shards[i]
                    .blocks
                    .last()
                    .expect("the genesis block does not exist")
                    .gas_used
            );
        });
        println!("  ...");

        print!("{:>20}", "active users:");
        (0..debug_shard_num).for_each(|i| {
            print!(" {:9}", self.blockchain.shards[i].accounts.len());
        });
        println!("  ...");

        print!("{:>20}", "switching users:");
        (0..debug_shard_num).for_each(|i| {
            print!(" {:9}", self.blockchain.shards[i].moving_accounts.len());
        });
        println!("  ...");

        print!("{:>20}", "base fee (Gwei):");
        (0..debug_shard_num).for_each(|i| {
            print!(
                " {:9.4}",
                self.blockchain.shards[i].get_base_fee() as f64 / 1_000_000_000.0
            );
        });
        println!("  ...");
    }

    fn get_user_next_shard_ids_and_reduction(&mut self) -> Vec<(usize, Option<GasPrice>)> {
        self.users
            .iter()
            .map(|user| user.pick_low_fee_shard_id_and_movement_fee_cap(&self))
            .collect()
    }

    fn get_shard_id_from_addr(&self, shargrila_addr: Address) -> usize {
        *self
            .blockchain
            .addr_to_shard_id
            .get(&shargrila_addr)
            .expect("failed to convert addr to shard_id")
    }

    fn determine_fee_cap(&self, from: Address, to: Address) -> GasPrice {
        self.user_graph.get_edge(from, to).fee_cap
    }

    /// Broadcast transactions.
    fn broadcast_transactions_per_slot(&mut self, transactions: Vec<TransactionAndReceipt>) {
        for (transaction, receipt) in transactions {
            self.blockchain.shards[transaction.shard_id].push_transaction(transaction, receipt);
        }
    }

    /// Generate transactions for one slot.
    fn generate_transactions_per_slot(&mut self) -> Vec<TransactionAndReceipt> {
        let mut transactions: Vec<TransactionAndReceipt> = Vec::new();

        let precomputed_next_shard_ids_and_reduction = self.get_user_next_shard_ids_and_reduction();

        let user_graph_edges = self.user_graph.edges.clone();
        for (from, edges) in user_graph_edges.iter().enumerate() {
            // Whether or not transactions was executed
            let mut executed_transaction_hashes = HashSet::new();
            let mut moved_account_addr_and_new_shard_id = Vec::new();
            for unconfirmed_transactions in
                self.users[from].unconfirmed_transactions_in_shard.iter()
            {
                for (_, (transaction, _)) in unconfirmed_transactions.iter() {
                    self.blockchain.shards[transaction.shard_id]
                        .blocks
                        .last()
                        .expect("the genesis block does not exist")
                        // last block
                        .executed_transactions
                        .iter()
                        .for_each(|executed_transaction| {
                            if transaction.hash == executed_transaction.hash {
                                executed_transaction_hashes.insert(transaction.hash);
                            }
                            if executed_transaction.functions[0].ftype
                                == FunctionType::ApplyCrossTransferAll
                            {
                                moved_account_addr_and_new_shard_id.push((
                                    executed_transaction.from,
                                    executed_transaction.shard_id,
                                ));
                            }
                        });
                }
            }
            for (addr, shard_id) in moved_account_addr_and_new_shard_id.iter() {
                for shard in self.blockchain.shards.iter_mut() {
                    if shard.id == *shard_id {
                        continue;
                    }
                    shard.remove_account(*addr);
                }
                self.blockchain.update_addr_to_shard_id(*addr);
            }

            // Eliminate confirmed transactions
            (0..SHARD_NUM).for_each(|shard_id| {
                self.users[from].unconfirmed_transactions_in_shard[shard_id] = self.users[from]
                    .unconfirmed_transactions_in_shard[shard_id]
                    .iter()
                    .cloned()
                    .filter(|(_, (transaction, _))| {
                        !executed_transaction_hashes.contains(&transaction.hash)
                    })
                    .collect();
            });

            // If a waiting transaction can be sent, send it.
            transactions.append(&mut self.get_pending_transactions_per_slot(from));

            // Movement
            transactions.append(&mut self.generate_movement_transactions_per_slot(
                from,
                &precomputed_next_shard_ids_and_reduction,
            ));

            // Transfer
            transactions.append(&mut self.generate_transfer_transactions_per_slot(from, edges));
        }

        println!("The number of transactions: {}", transactions.len());

        transactions
    }

    fn get_pending_transactions_per_slot(&mut self, from: Address) -> Vec<TransactionAndReceipt> {
        let mut transactions = Vec::new();
        for shard_id in 0..SHARD_NUM {
            if self.users[from].unconfirmed_transactions_in_shard[shard_id].is_empty()
                && !self.users[from].unsent_transactions_in_shard[shard_id].is_empty()
            {
                let (mut transaction, prev_transaction_hash) = self.users[from]
                    .unsent_transactions_in_shard[shard_id]
                    .pop_front()
                    .unwrap();

                if transaction.fee_cap <= self.blockchain.shards[shard_id].get_base_fee() {
                    self.users[from].unsent_transactions_in_shard[shard_id]
                        .push_front((transaction, prev_transaction_hash));
                    continue;
                }
                transaction.nonce = self.users[from].nonce_in_shard[shard_id];

                let receipt = {
                    let (_, account) = self.blockchain.get_account(from);
                    if account.is_none() {
                        continue;
                    }
                    let account = account.unwrap();
                    self.blockchain.shards[account.shard_id]
                        .receipts
                        .get(&prev_transaction_hash)
                };
                if receipt.is_none() || !receipt.unwrap().status {
                    continue;
                }
                let receipt = Some(receipt.unwrap().clone());

                transactions.push((transaction.clone(), receipt.clone()));
                self.users[from].unconfirmed_transactions_in_shard[shard_id]
                    .push((self.blockchain.slot, (transaction, receipt)));
                self.users[from].nonce_in_shard[shard_id] += 1;
            }
        }
        transactions
    }

    fn generate_movement_transactions_per_slot(
        &mut self,
        from: Address,
        next_shard_ids: &[(usize, Option<GasPrice>)],
    ) -> Vec<TransactionAndReceipt> {
        let mut transactions = Vec::new();
        if RND.lock().unwrap().next_u32() <= u32::MAX / AVERAGE_SHARD_SWITCHING_INTERVAL as u32
            && self.blockchain.slot > INITIAL_SETUP_SLOTS
        {
            let shard_f = self.get_shard_id_from_addr(from);
            let (shard_t, fee_cap) = next_shard_ids[from];
            // TODO
            if fee_cap.is_none() {
                return Vec::new();
            }
            let fee_cap = fee_cap.unwrap();

            if shard_f != shard_t
                && self.users[from].unconfirmed_transactions_in_shard[shard_f].is_empty()
                && self.users[from].unconfirmed_transactions_in_shard[shard_t].is_empty()
            {
                if fee_cap <= self.blockchain.shards[shard_f].get_base_fee()
                    || fee_cap <= self.blockchain.shards[shard_t].get_base_fee()
                {
                    return Vec::new();
                }
                let nonce = self.users[from].nonce_in_shard[shard_f];
                let transaction = Transaction::new(
                    from,
                    from,
                    shard_f,
                    vec![Function {
                        source: from,
                        target: from,
                        ftype: FunctionType::CreateCrossTransferAll,
                        calldata: "".to_string(),
                    }],
                    DEFAULT_GAS_PREMIUM,
                    fee_cap,
                    nonce,
                );
                transactions.push((transaction.clone(), None));
                self.users[from].unconfirmed_transactions_in_shard[shard_f]
                    .push((self.blockchain.slot, (transaction.clone(), None)));
                self.users[from].nonce_in_shard[shard_f] += 1;

                self.users[from].unsent_transactions_in_shard[shard_t].push_back((
                    Transaction::new(
                        from,
                        from,
                        shard_t,
                        vec![Function {
                            source: from,
                            target: from,
                            ftype: FunctionType::ApplyCrossTransferAll,
                            calldata: shard_f.to_string(),
                        }],
                        DEFAULT_GAS_PREMIUM,
                        fee_cap,
                        DUMMY_NONCE, // update when sending
                    ),
                    transaction.hash,
                ));
            }
        }
        transactions
    }

    fn generate_transfer_transactions_per_slot(
        &mut self,
        from: Address,
        edges: &HashMap<Address, UserGraphEdge>,
    ) -> Vec<TransactionAndReceipt> {
        let mut transactions = Vec::new();
        let mut shuffle_to = edges.iter().map(|(&to, _)| to).collect::<Vec<_>>();
        let mut rng = thread_rng();
        shuffle_to.shuffle(&mut rng);

        for to in shuffle_to {
            let edge = edges.get(&to).unwrap();

            let p = RND.lock().unwrap().next_u32() as f64 / u32::MAX as f64;
            let skip = p > edge.transfer_probability_in_slot as f64;
            if skip {
                continue;
            }

            let shard_f = self.get_shard_id_from_addr(from);
            let shard_t = self.get_shard_id_from_addr(to);

            if !self.users[from].unconfirmed_transactions_in_shard[shard_f].is_empty()
                || !self.users[from].unconfirmed_transactions_in_shard[shard_t].is_empty()
            {
                continue;
            }

            if shard_f == shard_t {
                // Intra-shard transfer

                let fee_cap = self.determine_fee_cap(from, to);
                if fee_cap <= self.blockchain.shards[shard_f].get_base_fee() {
                    continue;
                }
                let nonce = self.users[from].nonce_in_shard[shard_f];
                let transaction = Transaction::new(
                    from,
                    to,
                    shard_f,
                    vec![Function {
                        source: from,
                        target: to,
                        ftype: FunctionType::Transfer,
                        calldata: "".to_string(),
                    }],
                    DEFAULT_GAS_PREMIUM,
                    fee_cap,
                    nonce,
                );
                transactions.push((transaction.clone(), None));
                self.users[from].unconfirmed_transactions_in_shard[shard_f]
                    .push((self.blockchain.slot, (transaction, None)));
                self.users[from].nonce_in_shard[shard_f] += 1;
            } else {
                // Cross-shard transfer
                let fee_cap = self.determine_fee_cap(from, to) * GAS_TRANSFER
                    / (GAS_CREATE_CROSS_TRANSFER + GAS_APPLY_CROSS_TRANSFER);
                if fee_cap <= self.blockchain.shards[shard_f].get_base_fee()
                    || fee_cap <= self.blockchain.shards[shard_t].get_base_fee()
                {
                    continue;
                }
                let nonce = self.users[from].nonce_in_shard[shard_f];
                let transaction = Transaction::new(
                    from,
                    to,
                    shard_f,
                    vec![Function {
                        source: from,
                        target: from,
                        ftype: FunctionType::CreateCrossTransfer,
                        calldata: "".to_string(),
                    }],
                    DEFAULT_GAS_PREMIUM,
                    fee_cap,
                    nonce,
                );
                transactions.push((transaction.clone(), None));
                self.users[from].unconfirmed_transactions_in_shard[shard_f]
                    .push((self.blockchain.slot, (transaction.clone(), None)));
                self.users[from].nonce_in_shard[shard_f] += 1;

                self.users[from].unsent_transactions_in_shard[shard_t].push_back((
                    Transaction::new(
                        from,
                        to,
                        shard_t,
                        vec![Function {
                            source: from,
                            target: to,
                            ftype: FunctionType::ApplyCrossTransfer,
                            calldata: "".to_string(),
                        }],
                        DEFAULT_GAS_PREMIUM,
                        fee_cap,
                        DUMMY_NONCE, // update when sending
                    ),
                    transaction.hash,
                ));
            }
        }

        transactions
    }
}
