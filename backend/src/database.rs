use sqlx::PgPool;
use chrono::Utc;
use crate::models::{TransactionRecord, TransactionType, BalanceSnapshot};
use crate::error::Result;
use std::str::FromStr;

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_transaction(
        &self,
        vault: &str,
        transaction_type: TransactionType,
        amount: u64,
        signature: Option<&str>,
    ) -> Result<TransactionRecord> {
        let id = uuid::Uuid::new_v4();
        let timestamp = Utc::now();
        
        sqlx::query(
            r#"
            INSERT INTO transactions (id, vault, transaction_type, amount, signature, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(id)
        .bind(vault)
        .bind(transaction_type as TransactionType)
        .bind(amount as i64)
        .bind(signature)
        .bind(timestamp)
        .execute(&self.pool)
        .await?;

        Ok(TransactionRecord {
            id: Some(id),
            vault: vault.to_string(),
            transaction_type,
            amount,
            signature: signature.map(|s| s.to_string()),
            timestamp,
        })
    }

    pub async fn get_transactions(
        &self,
        vault: &str,
        limit: i64,
    ) -> Result<Vec<TransactionRecord>> {
        let rows = sqlx::query_as::<_, (uuid::Uuid, String, TransactionType, i64, Option<String>, chrono::DateTime<Utc>)>(
            r#"
            SELECT id, vault, transaction_type, amount, signature, timestamp
            FROM transactions
            WHERE vault = $1
            ORDER BY timestamp DESC
            LIMIT $2
            "#,
        )
        .bind(vault)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id, vault, transaction_type, amount, signature, timestamp)| TransactionRecord {
            id: Some(id),
            vault,
            transaction_type,
            amount: amount as u64,
            signature,
            timestamp,
        }).collect())
    }

    pub async fn create_balance_snapshot(
        &self,
        vault: &str,
        total_balance: u64,
        locked_balance: u64,
        available_balance: u64,
    ) -> Result<BalanceSnapshot> {
        let id = uuid::Uuid::new_v4();
        let timestamp = Utc::now();
        
        sqlx::query(
            r#"
            INSERT INTO balance_snapshots (id, vault, total_balance, locked_balance, available_balance, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(id)
        .bind(vault)
        .bind(total_balance as i64)
        .bind(locked_balance as i64)
        .bind(available_balance as i64)
        .bind(timestamp)
        .execute(&self.pool)
        .await?;

        Ok(BalanceSnapshot {
            id: Some(id),
            vault: vault.to_string(),
            total_balance,
            locked_balance,
            available_balance,
            timestamp,
        })
    }

    pub async fn get_latest_balance(
        &self,
        vault: &str,
    ) -> Result<Option<BalanceSnapshot>> {
        let row = sqlx::query_as::<_, (uuid::Uuid, String, i64, i64, i64, chrono::DateTime<Utc>)>(
            r#"
            SELECT id, vault, total_balance, locked_balance, available_balance, timestamp
            FROM balance_snapshots
            WHERE vault = $1
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
        )
        .bind(vault)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(id, vault, total_balance, locked_balance, available_balance, timestamp)| BalanceSnapshot {
            id: Some(id),
            vault,
            total_balance: total_balance as u64,
            locked_balance: locked_balance as u64,
            available_balance: available_balance as u64,
            timestamp,
        }))
    }

    pub async fn get_total_tvl(&self) -> Result<u64> {
        let rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT transaction_type::text, amount
            FROM transactions
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut total: i64 = 0;
        for (tx_type_str, amount) in rows {
            match tx_type_str.as_str() {
                "deposit" => total += amount,
                "withdrawal" => total -= amount,
                _ => {}
            }
        }

        Ok(total.max(0) as u64)
    }

    pub async fn get_all_vaults(&self) -> Result<Vec<String>> {
        let rows = sqlx::query_as::<_, (String,)>(
            r#"
            SELECT DISTINCT vault
            FROM transactions
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(vault,)| vault).collect())
    }
}

