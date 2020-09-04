use crate::*;

/// Account.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    pub addr: Address,
    pub shard_id: usize,
    pub balance: Wei,
}

impl Account {
    pub fn new(addr: Address, shard_id: usize) -> Self {
        Self {
            addr,
            shard_id,
            balance: Wei::MAX / 10,
        }
    }
}

impl Hash for Account {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.addr.hash(state);
    }
}
