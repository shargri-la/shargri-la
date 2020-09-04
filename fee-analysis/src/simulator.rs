use crate::*;

/// Sharded blockchain simulator.
pub struct Simulator {
    environment: Environment,
    duration: Slot,
}

impl Simulator {
    pub fn new(duration: Slot) -> Self {
        Self {
            environment: Environment::new(),
            duration,
        }
    }

    /// Runs from slot 0 to slot (duration - 1).
    pub fn run(&mut self, arg_matches: &clap::ArgMatches) {
        self.environment.setup(&arg_matches);

        let output_dir_path = Path::new(
            arg_matches
                .value_of("OUTPUT_DIR_PATH")
                .unwrap_or(DEFAULT_OUTPUT_DIR_PATH),
        );

        if let Err(e) = std::fs::create_dir(output_dir_path) {
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                eprintln!("Error: {:?}", e)
            }
        }

        (0..self.duration).for_each(|_| {
            println!();
            println!("slot: {}", self.environment.blockchain.slot);
            self.environment.process();
        });

        if let Err(e) = self.output_csv_base_fee(output_dir_path) {
            eprintln!("Error: {:?}", e)
        }
        if let Err(e) = self.output_csv_active_user_num(output_dir_path) {
            eprintln!("Error: {:?}", e)
        }
        if let Err(e) = self.output_csv_users(output_dir_path) {
            eprintln!("Error: {:?}", e)
        }
        if let Err(e) = self.output_csv_function_num(output_dir_path) {
            eprintln!("Error: {:?}", e)
        }
        if let Err(e) = self.output_csv_mempool(output_dir_path) {
            eprintln!("Error: {:?}", e)
        }
    }

    fn output_csv_base_fee(&self, output_dir_path: &Path) -> Result<(), Box<dyn Error>> {
        let file_path = output_dir_path.join(OUTPUT_BASE_FEE_CSV_FILENAME);
        let file = File::create(file_path)?;
        let mut wtr = csv::Writer::from_writer(file);
        (0..self.duration).for_each(|slot| {
            let record = (0..SHARD_NUM)
                .map(|shard_id| {
                    self.environment.blockchain.shards[shard_id].states[slot as usize]
                        .base_fee
                        .to_string()
                })
                .collect::<Vec<_>>();

            if let Err(e) = wtr.write_record(record) {
                eprintln!("Error: {:?}", e)
            };
        });

        wtr.flush()?;
        Ok(())
    }

    fn output_csv_active_user_num(&self, output_dir_path: &Path) -> Result<(), Box<dyn Error>> {
        let file_path = output_dir_path.join(OUTPUT_ACTIVE_USER_NUM_CSV_FILENAME);
        let file = File::create(file_path)?;
        let mut wtr = csv::Writer::from_writer(file);

        (0..self.duration).for_each(|slot| {
            let user_num = self.environment.user_num_mem[slot as usize]
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>();

            if let Err(e) = wtr.write_record(user_num) {
                eprintln!("Error: {:?}", e)
            };
        });

        wtr.flush()?;
        Ok(())
    }

    fn output_csv_users(&self, output_dir_path: &Path) -> Result<(), Box<dyn Error>> {
        let file_path = output_dir_path.join(OUTPUT_USERS_CSV_FILENAME);
        let file = File::create(file_path)?;
        let mut wtr = csv::Writer::from_writer(file);

        let mut total_fee = vec![0; self.environment.users.len()];
        let mut transaction_num = vec![0; self.environment.users.len()];

        (0..self.duration).for_each(|slot| {
            self.environment.blockchain.shards.iter().for_each(|shard| {
                let base_fee = shard.states[slot as usize].base_fee;
                shard.blocks[slot as usize]
                    .executed_transactions
                    .iter()
                    .for_each(|transaction| {
                        total_fee[transaction.from] +=
                            base_fee * transaction.functions.first().unwrap().gas();
                        transaction_num[transaction.from] += 1;
                    });
            });
        });

        self.environment.users.iter().for_each(|user| {
            let user = vec![
                user.account_addr.to_string(),
                (user.user_type as usize).to_string(),
                total_fee[user.account_addr].to_string(),
                transaction_num[user.account_addr].to_string()
            ];
            if let Err(e) = wtr.write_record(user) {
                eprintln!("Error: {:?}", e)
            };
        });

        wtr.flush()?;
        Ok(())
    }

    fn output_csv_function_num(&self, output_dir_path: &Path) -> Result<(), Box<dyn Error>> {
        let file_path = output_dir_path.join(OUTPUT_FUNCTION_NUM_CSV_FILENAME);
        let file = File::create(file_path)?;
        let mut wtr = csv::Writer::from_writer(file);

        (0..self.duration).for_each(|slot| {
            let mut function_cnt = vec![0; 10];

            self.environment.blockchain.shards.iter().for_each(|shard| {
                shard.blocks[slot as usize]
                    .executed_transactions
                    .iter()
                    .for_each(|transaction| {
                        let user = &self.environment.users[transaction.from];
                        transaction.functions.iter().for_each(|function| {
                            if user.user_type.is_switcher() {
                                function_cnt[function.ftype.clone() as usize + 5] += 1;
                            } else {
                                function_cnt[function.ftype.clone() as usize] += 1;
                            }
                        });
                    });
            });
            let record = function_cnt
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>();
            if let Err(e) = wtr.write_record(record) {
                eprintln!("Error: {:?}", e)
            };
        });

        wtr.flush()?;
        Ok(())
    }

    fn output_csv_mempool(&self, output_dir_path: &Path) -> Result<(), Box<dyn Error>> {
        let file_path = output_dir_path.join(OUTPUT_MEMPOOL_CSV_FILENAME);
        let file = File::create(file_path)?;
        let mut wtr = csv::Writer::from_writer(file);

        (0..self.duration).for_each(|slot| {
            let mempool_tx_num = self.environment.mempool_tx_mem[slot as usize]
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>();

            if let Err(e) = wtr.write_record(mempool_tx_num) {
                eprintln!("Error: {:?}", e)
            };
        });

        wtr.flush()?;
        Ok(())
    }
}
