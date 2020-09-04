use shargrila_chain::*;

pub const DEFAULT_END_SLOT: Slot = 100;
pub const DEFAULT_USER_NUM: usize = 10000;
pub const DEFAULT_PERCENTAGE_OF_MINIMUM: f64 = 0.;
pub const DEFAULT_PERCENTAGE_OF_WEIGHTED_RANDOM: f64 = 0.;
pub const DEFAULT_PERCENTAGE_OF_DECREASING_MINIMUM: f64 = 0.;
pub const INITIAL_SETUP_SLOTS: Slot = 10;

// User behavior
pub const AVERAGE_SHARD_SWITCHING_INTERVAL: Slot = 100;
pub const DEFAULT_GAS_PREMIUM: GasPrice = 1_000_000_000;

// Transaction generation
pub const TRANSACTION_OCCUPANCY: f64 = 2.0;
pub const AVERAGE_GAS_PER_TRANSACTION: Gas =
    (GAS_CREATE_CROSS_TRANSFER + GAS_APPLY_CROSS_TRANSFER) / 2;
pub const GLOBAL_GAS_TARGET: Gas = BLOCK_GAS_TARGET * SHARD_NUM as Gas;
pub const GLOBAL_TRANSACTION_GAS_PER_SLOT: Gas =
    (TRANSACTION_OCCUPANCY * GLOBAL_GAS_TARGET as f64) as Gas;
pub const GLOBAL_TRANSACTION_NUM: usize =
    (GLOBAL_TRANSACTION_GAS_PER_SLOT / AVERAGE_GAS_PER_TRANSACTION) as usize;

// Constants in UserGraph::new_random()
pub const MAX_FEE_CAP: GasPrice = INITIAL_BASE_FEE * 200;
pub const MAX_TARGET_USER_NUM: usize = 15;
pub const POPULAR_USER_ADDRESS: Address = 0;
pub const PERCENTAGE_OF_USERS_TRANSFERRING_TO_POPULAR_USER: f64 = 0.1;

// Path
pub const DEFAULT_OUTPUT_DIR_PATH: &str = "data";
pub const OUTPUT_BASE_FEE_CSV_FILENAME: &str = "base_fee.csv";
pub const OUTPUT_ACTIVE_USER_NUM_CSV_FILENAME: &str = "active_user_num.csv";
pub const OUTPUT_USERS_CSV_FILENAME: &str = "users.csv";
pub const OUTPUT_FUNCTION_NUM_CSV_FILENAME: &str = "function_num.csv";
pub const OUTPUT_MEMPOOL_CSV_FILENAME: &str = "mempool.csv";

// No need to change
pub const RAND_SEED: u64 = 1337;
pub const DUMMY_NONCE: Nonce = 1337;
