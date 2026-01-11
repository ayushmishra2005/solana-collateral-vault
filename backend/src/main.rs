use backend::*;
use std::sync::Arc;
use sqlx::postgres::PgPoolOptions;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use backend::vault_manager::VaultManager;
use backend::database::Database;
use backend::vault_monitor::VaultMonitor;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/collateral_vault".to_string());
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .map_err(|e| Error::Database(e))?;

    let pool_clone = pool.clone();

    // Solana client setup
    let _rpc_url = std::env::var("RPC_URL")
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    
    let program_id = Pubkey::from_str(
        &std::env::var("PROGRAM_ID")
            .unwrap_or_else(|_| "8vjbjPhoD2rav71J8mgbVxcYdbbqST78y2bzMPRqoGr9".to_string())
    ).map_err(|e| Error::InvalidAccount(format!("Invalid program ID: {}", e)))?;

    let mint = Pubkey::from_str(
        &std::env::var("USDT_MINT")
            .unwrap_or_else(|_| "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string())
    ).map_err(|e| Error::InvalidAccount(format!("Invalid mint: {}", e)))?;

    let client = Arc::new(
        anchor_client::Client::new(
            anchor_client::Cluster::Devnet,
            std::sync::Arc::new(solana_sdk::signature::Keypair::new()),
        )
    );

    let vault_manager = Arc::new(VaultManager::new(
        client,
        program_id,
        Database::new(pool.clone()),
        mint,
    ));
    
    let vault_monitor = Arc::new(VaultMonitor::new(Database::new(pool.clone())));

    // Create API router
    let app = api::create_router(
        vault_manager,
        vault_monitor,
        Arc::new(Database::new(pool_clone)),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await
        .map_err(|e| Error::SolanaClient(format!("Failed to bind: {}", e)))?;

    
    axum::serve(listener, app).await
        .map_err(|e| Error::SolanaClient(format!("Server error: {}", e)))?;

    Ok(())
}

