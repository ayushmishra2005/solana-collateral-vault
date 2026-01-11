use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Solana client error: {0}")]
    SolanaClient(String),
    
    #[error("Anchor client error: {0}")]
    AnchorClient(String),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Invalid account: {0}")]
    InvalidAccount(String),
    
    #[error("Insufficient balance")]
    InsufficientBalance,
    
    #[error("Vault not found")]
    VaultNotFound,
    
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    #[error("Unauthorized")]
    Unauthorized,
}

pub type Result<T> = std::result::Result<T, Error>;

