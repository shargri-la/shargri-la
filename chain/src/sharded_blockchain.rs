use crate::*;

/// Sharded blockchain.
pub struct ShardedBlockchain {
    pub slot: Slot,
    pub epoch: Epoch,
    pub shards: Vec<Shard>,

    pub addr_to_shard_id: HashMap<Address, usize>,

    // For statistics
    pub account_num: usize,
}

impl ShardedBlockchain {
    // Constants
    const GENESIS_SLOT: Slot = 0;
    const GENESIS_EPOCH: Epoch = 0;

    pub fn new() -> Self {
        Self {
            slot: ShardedBlockchain::GENESIS_SLOT,
            epoch: ShardedBlockchain::GENESIS_EPOCH,
            shards: (0..SHARD_NUM).map(Shard::new).collect(),
            addr_to_shard_id: HashMap::new(),
            account_num: 0,
        }
    }

    /// Process to the given slot.
    pub fn process_slots(&mut self, slot: Slot) {
        assert!(self.slot <= slot);
        while self.slot < slot {
            self.process_slot();
            if (self.slot + 1) % SLOTS_PER_EPOCH == 0 {
                self.process_epoch();
            }
            self.slot += 1;
        }
    }

    /// Process of a slot.
    pub fn process_slot(&mut self) {
        self.shards
            .iter_mut()
            .for_each(|shard| shard.process_slot());
    }

    /// Process of a epoch.
    pub fn process_epoch(&mut self) {
        self.epoch += 1;
    }

    /// Update value in addr_to_shard_id with key = addr.
    pub fn update_addr_to_shard_id(&mut self, addr: Address) {
        let &shard_id = self
            .addr_to_shard_id
            .get(&addr)
            .expect("the account does not exist");
        let account = self.shards[shard_id].get_account(addr);
        let moving_account = self.shards[shard_id].get_moving_account(addr);
        if account.is_some() || moving_account.is_some() {
            return;
        }
        for shard in self.shards.iter() {
            let account = shard.get_account(addr);
            let moving_account = shard.get_moving_account(addr);
            if account.is_some() || moving_account.is_some() {
                self.addr_to_shard_id.insert(addr, shard.id);
            }
        }
    }

    /// Get an account with address = addr.
    pub fn get_account(&self, addr: Address) -> (bool, Option<&Account>) {
        let &shard_id = self
            .addr_to_shard_id
            .get(&addr)
            .expect("the account does not exist");
        let account = self.shards[shard_id].get_account(addr);
        let moving_account = self.shards[shard_id].get_moving_account(addr);
        if account.is_some() {
            (true, account)
        } else if moving_account.is_some() {
            (false, moving_account)
        } else {
            unreachable!("failed to get account");
            //(false, None)
        }
    }
}

impl Default for ShardedBlockchain {
    fn default() -> Self {
        Self::new()
    }
}
