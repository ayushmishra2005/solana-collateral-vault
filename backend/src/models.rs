use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultInfo {
    pub owner: String,
    pub vault: String,
    pub token_account: String,
    pub total_balance: u64,
    pub locked_balance: u64,
    pub available_balance: u64,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRecord {
    pub id: Option<uuid::Uuid>,
    pub vault: String,
    pub transaction_type: TransactionType,
    pub amount: u64,
    pub signature: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, Copy)]
#[sqlx(type_name = "transaction_type", rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Lock,
    Unlock,
    Transfer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSnapshot {
    pub id: Option<uuid::Uuid>,
    pub vault: String,
    pub total_balance: u64,
    pub locked_balance: u64,
    pub available_balance: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositRequest {
    pub user: String,
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawRequest {
    pub user: String,
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeVaultRequest {
    pub user: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TVLResponse {
    pub total_value_locked: u64,
    pub total_vaults: u64,
    pub timestamp: i64,
}

