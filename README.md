![](Shargri-La.png)

Shargri-La is a transaction-level sharding simulator for protocol testing against users' behavior on a sharded blockchain. The goal of Shargri-La is to help researchers to design or refine sharding protocols.

Shargri-La performs a [discrete-event simulation](https://en.wikipedia.org/wiki/Discrete-event_simulation), which proceeds with slots. At each slot, Shargri-La simulates transaction creation by users, block proposals by validators, and state transitions.

## Scope of Shargri-La
Shargri-La is a transaction-level simulator, meaning that it focuses on users’ behavior on a sharded blockchain. Therefore, Shargri-La covers the following things:
- Transaction creation by users
- Application-level state transitions
- Cross-shard transactions
- Transaction selection by block proposers
- Transaction fee mechanism and gas auction

On the other hand, Shargri-La does *not* simulate the following things:
- P2P network (gossip protocols, peer discovery, etc.)
- Consensus algorithm
- Data availability mechanism
- Cryptographic processing (signatures, hashes, etc.)

## Version 0.1.0: EIP-1559 and ETH Transfers
For simplicity, we assume that all the on-chain activities are only the transfers of ETH. 

We partially adopt [Eth1x64 Variant 1 "Apostille,"](https://ethresear.ch/t/eth1x64-variant-1-apostille/7365) i.e., each shard contains the Eth1 state transition rule and supports receipt-based cross-shard communication. Also, we adopt [EIP-1559](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1559.md) as the transaction pricing mechanism.

## Usage
We verified to work them with Rust 1.45.0.

```
cargo run --release
```


### Options
```
FLAGS:
        --popular_user_exists         Whether or not there ia a popular user
        --popular_user_is_switcher    Whether or not the popular user is a switcher
    -h, --help                        Prints help information
    -V, --version                     Prints version information

OPTIONS:
        --csv <BIG_QUERY_CSV>
            A BigQuery Ethereum transactions csv file

        --end_slot <END_SLOT>                                                    The number of slots
        --output_dir_path <OUTPUT_DIR_PATH>                                      The path of the output directory
        --percentage_of_decreasing_minimum <PERCENTAGE_OF_DECREASING_MINIMUM>    
        --percentage_of_minimum <PERCENTAGE_OF_MINIMUM>                          
        --percentage_of_weighted_random <PERCENTAGE_OF_WEIGHTED_RANDOM>          
        --user_num <USER_NUM>                                                    The maximum number of users
```

### (WIP) Using BigQuery 
Use `transactions` table of the `crypto_ethereum` datasets.
```
cargo run --release -- --csv BIG_QUERY_ETHEREUM_TRANSACTION_CSV
```

### Visualizer
Simulation results can be visualized by the sample programs using Matplotlib.
We verified to work them with Python 3.8.5 (venv).

Install:
```
pip install -r requirements.txt
```

Visualize:
```
make visualize
```

Result images are saved in the `data` directory.

