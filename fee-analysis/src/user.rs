use crate::*;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum StrategyType {
    NonSwitcher,
    WeightedRandom,
    Minimum,
    DecreasingMinimum,
}

impl StrategyType {
    pub fn is_switcher(&self) -> bool {
        !(self == &StrategyType::NonSwitcher)
    }
}

/// User.
/// Account management and strategy.
/// Users can have multiple accounts, but this implementation only allow one.
pub struct User {
    pub account_addr: Address,
    /// Transactions awaiting confirmation.
    pub unconfirmed_transactions_in_shard: Vec<Vec<(Slot, TransactionAndReceipt)>>,
    /// When executing transactions are empty, a user will send these transactions.
    pub unsent_transactions_in_shard: Vec<VecDeque<(Transaction, TransactionHash)>>,
    pub nonce_in_shard: Vec<Nonce>,
    pub user_type: StrategyType,
}

impl User {
    pub fn new(account_addr: Address, user_type: StrategyType) -> Self {
        Self {
            account_addr,
            unconfirmed_transactions_in_shard: vec![Vec::new(); SHARD_NUM],
            unsent_transactions_in_shard: vec![VecDeque::new(); SHARD_NUM],
            nonce_in_shard: vec![0; SHARD_NUM],
            user_type,
        }
    }

    /// Estimate the shard to which the fee is best suited.
    pub fn pick_low_fee_shard_id_and_movement_fee_cap(
        &self,
        environment: &Environment,
    ) -> (usize, Option<GasPrice>) {
        // How long the account has been on the same shard
        let (_, account) = environment.blockchain.get_account(self.account_addr);
        let account = account.unwrap();

        // Expected transaction fees (per slot) when you are on a certain shard
        let mut expected_fees_per_slot = Vec::new();
        for shard_id_f in 0..environment.blockchain.shards.len() {
            let mut expected_fee: Wei = 0;
            for (&to, edge) in environment
                .user_graph
                .edges
                .get(self.account_addr)
                .unwrap()
                .iter()
            {
                let (_, account_t) = environment.blockchain.get_account(to);
                if account_t.is_none() {
                    continue;
                }
                let account_t = account_t.unwrap();
                let shard_id_t = account_t.shard_id;
                let gas_price_f;
                let gas_price_t;
                if environment.blockchain.slot < INITIAL_SETUP_SLOTS {
                    gas_price_f = edge.fee_cap;
                    gas_price_t = edge.fee_cap;
                } else {
                    gas_price_f = environment.blockchain.shards[shard_id_f].get_base_fee();
                    gas_price_t = environment.blockchain.shards[shard_id_t].get_base_fee();
                }

                let fee = if shard_id_f == shard_id_t {
                    (edge.transfer_probability_in_slot * (GAS_TRANSFER * gas_price_f) as f64) as Wei
                } else {
                    (edge.transfer_probability_in_slot
                        * (GAS_APPLY_CROSS_TRANSFER * gas_price_f
                            + GAS_CREATE_CROSS_TRANSFER * gas_price_t)
                            as f64) as Wei
                };

                expected_fee += fee;
            }

            expected_fee *= AVERAGE_SHARD_SWITCHING_INTERVAL as Wei;
            if account.shard_id != shard_id_f {
                let gas_price_f = environment.blockchain.shards[account.shard_id].get_base_fee();
                let gas_price_t = environment.blockchain.shards[shard_id_f].get_base_fee();
                expected_fee += GAS_CREATE_CROSS_TRANSFER_ALL * gas_price_f
                    + GAS_APPLY_CROSS_TRANSFER_ALL * gas_price_t;
            }

            expected_fees_per_slot.push(expected_fee);
        }

        let mut reduction_and_shard_ids = Vec::new();
        for (shard_id, &expected_fee) in expected_fees_per_slot.iter().enumerate() {
            if expected_fees_per_slot[account.shard_id] < expected_fee {
                continue;
            }
            let reduction = expected_fees_per_slot[account.shard_id] - expected_fee;
            reduction_and_shard_ids.push((reduction, shard_id));
        }

        let mut sorted_reduction_and_shard_ids = reduction_and_shard_ids;
        sorted_reduction_and_shard_ids.sort();

        if self.user_type == StrategyType::NonSwitcher {
            return (account.shard_id, None);
        }

        let total_reduction: u128 = sorted_reduction_and_shard_ids.iter().map(|x| x.0).sum();
        if total_reduction > 0 {
            if self.user_type == StrategyType::WeightedRandom {
                let threshold = RND.lock().unwrap().next_u32() as f64 / u32::MAX as f64;
                let mut cumulative_reduction = 0;
                for &(reduction, shard_id) in sorted_reduction_and_shard_ids.iter() {
                    cumulative_reduction += reduction;
                    if cumulative_reduction as f64 / total_reduction as f64 > threshold {
                        if shard_id == account.shard_id {
                            return (shard_id, None);
                        } else {
                            let fee_cap = reduction
                                / (GAS_APPLY_CROSS_TRANSFER_ALL + GAS_CREATE_CROSS_TRANSFER_ALL);
                            return (shard_id, Some(fee_cap));
                        }
                    }
                }
            } else if self.user_type == StrategyType::Minimum {
                let &(reduction, shard_id) = sorted_reduction_and_shard_ids.last().unwrap();
                if shard_id == account.shard_id {
                    return (shard_id, None);
                } else {
                    let fee_cap =
                        reduction / (GAS_APPLY_CROSS_TRANSFER_ALL + GAS_CREATE_CROSS_TRANSFER_ALL);
                    return (shard_id, Some(fee_cap));
                }
            } else if self.user_type == StrategyType::DecreasingMinimum {
                // TODO: refine
                for &(reduction, shard_id) in sorted_reduction_and_shard_ids.iter().rev() {
                    if shard_id == account.shard_id {
                        return (shard_id, None);
                    } else {
                        let shard = &environment.blockchain.shards[shard_id];
                        let states_length = shard.states.len();
                        if states_length < 2 {
                            break;
                        }
                        if shard.states[states_length - 2].base_fee
                            < shard.states[states_length - 1].base_fee
                        {
                            continue;
                        }
                        let fee_cap = reduction
                            / (GAS_APPLY_CROSS_TRANSFER_ALL + GAS_CREATE_CROSS_TRANSFER_ALL);
                        return (shard_id, Some(fee_cap));
                    }
                }
            }
        }
        (account.shard_id, None)
    }
}
