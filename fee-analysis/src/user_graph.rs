use crate::*;

/// UserGraph edge.
#[derive(Clone, Debug)]
pub struct UserGraphEdge {
    pub from: Address,
    pub to: Address,
    pub fee_cap: GasPrice,
    pub transfer_probability_in_slot: f64,
}

impl UserGraphEdge {
    pub fn new(
        from: Address,
        to: Address,
        fee_cap: GasPrice,
        transfer_probability_in_slot: f64,
    ) -> Self {
        Self {
            from,
            to,
            fee_cap,
            transfer_probability_in_slot,
        }
    }
}

/// UserGraph node.
pub struct UserGraphNode {
    pub account_addr: usize,

    // For statistics
    in_degree: usize,
    out_degree: usize,
}
pub struct UserGraph {
    pub nodes: Vec<UserGraphNode>,
    pub edges: Vec<HashMap<Address, UserGraphEdge>>,
}

impl UserGraph {
    pub fn new() -> Self {
        UserGraph {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Get edge (from, to).
    pub fn get_edge(&self, from: Address, to: Address) -> &UserGraphEdge {
        match self.edges[from].get(&to) {
            Some(edge) => edge,
            None => {
                unreachable!("failed to get the edge ({},{})", from, to);
            }
        }
    }

    /// Generate an user graph randomly.
    pub fn new_random(
        user_num: usize,
        popular_user_exists: bool,
        popular_user_is_switcher: bool,
    ) -> Self {
        let mut user_graph = Self::new();
        let max_target_user_num = std::cmp::min(MAX_TARGET_USER_NUM, user_num);

        // Construct ShardedBlockchain and UserGraphNode
        user_graph.nodes = (0..user_num)
            .map(|addr| UserGraphNode {
                account_addr: addr,
                in_degree: 0,
                out_degree: 0,
            })
            .collect();
        user_graph.edges.resize(user_num, HashMap::new());

        // Construct UserGraphEdge
        let mut number_of_transactions: u64 = 0;
        for from in 0..user_num {
            let target_user_num =
                RND.lock().unwrap().next_u32() as usize % (max_target_user_num + 1);

            let transfer_to_popular_user = if popular_user_exists {
                (popular_user_is_switcher && from == POPULAR_USER_ADDRESS)
                    || RND.lock().unwrap().next_u32()
                        < (PERCENTAGE_OF_USERS_TRANSFERRING_TO_POPULAR_USER * u32::MAX as f64)
                            as u32
            } else {
                false
            };

            for i in 0..target_user_num {
                let to = if i == 0 && transfer_to_popular_user {
                    POPULAR_USER_ADDRESS
                } else {
                    RND.lock().unwrap().next_u32() as usize % user_num
                };
                if from == to {
                    continue;
                }
                let yet_normalized_transfer_probability_in_slot =
                    RND.lock().unwrap().next_u32() % 100;
                let fee_cap = RND.lock().unwrap().next_u64() as GasPrice % MAX_FEE_CAP;
                user_graph.edges[from].insert(
                    to,
                    UserGraphEdge::new(
                        from,
                        to,
                        fee_cap,
                        yet_normalized_transfer_probability_in_slot as f64,
                    ),
                );
                user_graph.nodes[from].out_degree += 1;
                user_graph.nodes[to].in_degree += 1;
                number_of_transactions += yet_normalized_transfer_probability_in_slot as u64;
            }
        }
        user_graph.edges.iter_mut().for_each(|edges| {
            edges.values_mut().for_each(|edge| {
                edge.transfer_probability_in_slot = edge.transfer_probability_in_slot
                    / number_of_transactions as f64
                    * GLOBAL_TRANSACTION_NUM as f64;
            });
        });
        user_graph
    }

    /// User generation from historical transaction logs (by BigQuery Ethereum).
    pub fn new_from_eth1_data(user_num: usize, eth1_data_file_name: &str) -> Self {
        let mut user_graph = Self::new();
        // How many blocks are targeted
        let mut block_number_set = HashSet::new();

        let mut eth1_addr_to_shargrila_addr = HashMap::<String, Address>::new();
        let mut shargrila_addr_to_eth1_addr = HashMap::<Address, String>::new();

        // Load files
        let file_path = OsString::from(eth1_data_file_name);
        let file = File::open(file_path).expect("failed to open the file");
        let mut rdr = csv::Reader::from_reader(file);

        // The variable for determining UserGraphEdge parameters
        #[derive(Debug)]
        struct Edge {
            from: Address,
            to: Address,
            cnt: usize,
            gas_price: GasPrice,
        }
        let mut number_of_transactions = 0;
        let mut edges = HashMap::new();

        for result in rdr.deserialize() {
            let record: TransactionRecord = result.expect("failed to deserialize");
            if record.receipt_status == 0 {
                continue;
            }

            let current_user_num = eth1_addr_to_shargrila_addr.len();
            if current_user_num >= user_num {
                break;
            }

            user_graph.update_map_between_eth1_addr_and_shargrila_addr(
                record.from_address.clone(),
                &mut eth1_addr_to_shargrila_addr,
                &mut shargrila_addr_to_eth1_addr,
            );
            user_graph.update_map_between_eth1_addr_and_shargrila_addr(
                record.to_address.clone(),
                &mut eth1_addr_to_shargrila_addr,
                &mut shargrila_addr_to_eth1_addr,
            );

            block_number_set.insert(record.block_number);

            number_of_transactions += 1;

            let eth1_addr_from = record.from_address.clone();
            let eth1_addr_to = record.to_address.clone();
            let shargrila_addr_from = *eth1_addr_to_shargrila_addr
                .get(&eth1_addr_from)
                .expect("failed to convert eth1 addr to shargri-la's addr");
            let shargrila_addr_to = *eth1_addr_to_shargrila_addr
                .get(&eth1_addr_to)
                .expect("failed to convert eth1 addr to shargri-la's addr");

            let mut edge = edges
                .entry((shargrila_addr_from, shargrila_addr_to))
                .or_insert(Edge {
                    from: shargrila_addr_from,
                    to: shargrila_addr_to,
                    cnt: 0,
                    gas_price: 0,
                });
            edge.gas_price += record.gas_price as GasPrice;
            edge.cnt += 1;
        }

        let current_user_num = eth1_addr_to_shargrila_addr.len();

        // Construct UserGraphNode
        user_graph.nodes = (0..current_user_num)
            .map(|addr| UserGraphNode {
                account_addr: addr,
                in_degree: 0,
                out_degree: 0,
            })
            .collect();

        // Construct UserGraphEdge
        user_graph.edges.resize(current_user_num, HashMap::new());
        edges.iter_mut().for_each(|(_, edge)| {
            edge.gas_price /= edge.cnt as GasPrice;
            let transaction_ratio = edge.cnt as f64 / number_of_transactions as f64;
            let transfer_probability_in_slot = transaction_ratio * GLOBAL_TRANSACTION_NUM as f64;
            user_graph.edges[edge.from].insert(
                edge.to,
                UserGraphEdge::new(
                    edge.from,
                    edge.to,
                    edge.gas_price,
                    transfer_probability_in_slot,
                ),
            );
            user_graph.nodes[edge.from].out_degree += 1;
            user_graph.nodes[edge.to].in_degree += 1;
        });
        user_graph.print_statistics(
            number_of_transactions,
            block_number_set.len(),
            shargrila_addr_to_eth1_addr,
        );

        user_graph
    }

    fn update_map_between_eth1_addr_and_shargrila_addr(
        &mut self,
        eth1_addr: String,
        eth1_addr_to_shargrila_addr: &mut HashMap<String, usize>,
        shargrila_addr_to_eth1_addr: &mut HashMap<usize, String>,
    ) {
        if !eth1_addr_to_shargrila_addr.contains_key(&eth1_addr) {
            let shargrila_addr = eth1_addr_to_shargrila_addr.len();
            eth1_addr_to_shargrila_addr.insert(eth1_addr.clone(), shargrila_addr);
            shargrila_addr_to_eth1_addr.insert(shargrila_addr, eth1_addr);
        }
    }

    fn print_statistics(
        &self,
        number_of_transactions: usize,
        block_number_set_len: usize,
        shargrila_addr_to_eth1_addr: HashMap<usize, String>,
    ) {
        println!("STATISTICS");
        println!("The number of accounts: {}", self.nodes.len());
        println!("The number of transactions: {}", number_of_transactions);
        println!("The number of blocks: {}", block_number_set_len);

        // out degree ranking
        let mut out_degree_and_addr = Vec::new();
        self.nodes.iter().for_each(|node| {
            out_degree_and_addr.push((node.out_degree, node.account_addr));
        });
        out_degree_and_addr.sort();
        out_degree_and_addr.reverse();
        println!("Ranking of out degrees:");
        out_degree_and_addr
            .iter()
            .take(5)
            .for_each(|(degree, addr)| {
                println!(
                    "Shargri-La's addr {:5}, ETH1 addr {}, out degree {:5}, shard: {:3}",
                    addr,
                    shargrila_addr_to_eth1_addr.get(addr).unwrap(),
                    degree,
                    addr % SHARD_NUM
                );
            });

        // in degree ranking
        let mut in_degree_and_addr = Vec::new();
        self.nodes.iter().for_each(|node| {
            in_degree_and_addr.push((node.in_degree, node.account_addr));
        });
        in_degree_and_addr.sort();
        in_degree_and_addr.reverse();
        println!("Ranking of in degrees:");
        in_degree_and_addr
            .iter()
            .take(5)
            .for_each(|(degree, addr)| {
                println!(
                    "Shargri-La's addr {:5}, ETH1 addr {},  in degree {:5}, shard: {:3}",
                    addr,
                    shargrila_addr_to_eth1_addr.get(addr).unwrap(),
                    degree,
                    addr % SHARD_NUM
                );
            });
    }
}
