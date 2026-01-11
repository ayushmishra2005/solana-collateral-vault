use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use crate::models::*;
use crate::vault_manager::VaultManager;
use crate::vault_monitor::VaultMonitor;
use crate::database::Database;
use std::sync::Arc;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

pub fn create_router(
    vault_manager: Arc<VaultManager>,
    vault_monitor: Arc<VaultMonitor>,
    database: Arc<Database>,
) -> Router {
    Router::new()
        .route("/vault/initialize", post(initialize_vault))
        .route("/vault/deposit", post(deposit))
        .route("/vault/withdraw", post(withdraw))
        .route("/vault/balance/:user", get(get_balance))
        .route("/vault/transactions/:user", get(get_transactions))
        .route("/vault/tvl", get(get_tvl))
        .with_state(AppState {
            vault_manager,
            vault_monitor,
            database,
        })
}

#[derive(Clone)]
struct AppState {
    vault_manager: Arc<VaultManager>,
    vault_monitor: Arc<VaultMonitor>,
    database: Arc<Database>,
}

async fn initialize_vault(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(req): Json<InitializeVaultRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.vault_manager.initialize_vault(&req.user).await {
        Ok(signature) => Ok(Json(serde_json::json!({
            "success": true,
            "signature": signature
        }))),
        Err(e) => {
            let status = if e.to_string().contains("already exists") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            Err((
                status,
                Json(serde_json::json!({
                    "success": false,
                    "error": e.to_string()
                }))
            ))
        }
    }
}

async fn deposit(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(req): Json<DepositRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.vault_manager.deposit(&req.user, req.amount).await {
        Ok(signature) => Ok(Json(serde_json::json!({
            "success": true,
            "signature": signature
        }))),
        Err(e) => {
            let status = if e.to_string().contains("Vault not found") {
                StatusCode::NOT_FOUND
            } else if e.to_string().contains("requires user's wallet") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            Err((
                status,
                Json(serde_json::json!({
                    "success": false,
                    "error": e.to_string()
                }))
            ))
        }
    }
}

async fn withdraw(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(req): Json<WithdrawRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.vault_manager.withdraw(&req.user, req.amount).await {
        Ok(signature) => Ok(Json(serde_json::json!({
            "success": true,
            "signature": signature
        }))),
        Err(e) => {
            let status = if e.to_string().contains("Vault not found") {
                StatusCode::NOT_FOUND
            } else if e.to_string().contains("Insufficient") {
                StatusCode::BAD_REQUEST
            } else if e.to_string().contains("requires user's wallet") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            Err((
                status,
                Json(serde_json::json!({
                    "success": false,
                    "error": e.to_string()
                }))
            ))
        }
    }
}

async fn get_balance(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(user): Path<String>,
) -> Result<Json<VaultInfo>, StatusCode> {
    match state.vault_manager.get_vault_info(&user).await {
        Ok(info) => Ok(Json(info)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn get_transactions(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(user): Path<String>,
) -> Result<Json<Vec<TransactionRecord>>, StatusCode> {
    let user_pubkey = Pubkey::from_str(&user)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let (vault_pda, _) = Pubkey::find_program_address(
        &[b"vault", user_pubkey.as_ref()],
        &state.vault_manager.program_id(),
    );
    
    match state.database.get_transactions(&vault_pda.to_string(), 100).await {
        Ok(transactions) => Ok(Json(transactions)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_tvl(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<TVLResponse> {
    let tvl = state.vault_monitor.get_tvl().await.unwrap_or(0);
    Json(TVLResponse {
        total_value_locked: tvl,
        total_vaults: 0,
        timestamp: chrono::Utc::now().timestamp(),
    })
}

