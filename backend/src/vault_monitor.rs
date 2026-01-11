use crate::database::Database;
use crate::error::Result;
use tokio::time::{interval, Duration};

pub struct VaultMonitor {
    database: Database,
}

impl VaultMonitor {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn monitor_vaults(&self) -> Result<()> {
        let mut interval = interval(Duration::from_secs(300));

        loop {
            interval.tick().await;
        }
    }

    pub async fn get_tvl(&self) -> Result<u64> {
        self.database.get_total_tvl().await
    }
}

