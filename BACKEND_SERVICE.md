# Backend Service Documentation

## Overview

The Rust backend service provides off-chain infrastructure for the Collateral Vault system, including REST APIs, WebSocket streams, database management, and transaction building.

## Architecture

### Module Structure

```
backend/src/
├── main.rs              # Server entry point
├── lib.rs               # Library exports
├── api.rs               # REST API endpoints
├── websocket.rs         # WebSocket streams
├── vault_manager.rs     # Vault operations
├── balance_tracker.rs   # Balance monitoring
├── transaction_builder.rs # Transaction construction
├── cpi_manager.rs       # CPI handling
├── vault_monitor.rs     # Vault monitoring
├── database.rs          # Database operations
├── models.rs            # Data models
└── error.rs             # Error types
```

## Core Components

### VaultManager

Manages vault lifecycle and operations.

**Location:** `backend/src/vault_manager.rs`

**Methods:**
- `initialize_vault(user: &str) -> Result<String>` - Create new vault
- `deposit(user: &str, amount: u64) -> Result<String>` - Deposit collateral
- `withdraw(user: &str, amount: u64) -> Result<String>` - Withdraw collateral
- `get_vault_info(user: &str) -> Result<VaultInfo>` - Get vault state

**Responsibilities:**
- Build Anchor transactions
- Submit transactions to Solana
- Record transactions in database
- Handle transaction errors

### BalanceTracker

Monitors vault balances in real-time.

**Location:** `backend/src/balance_tracker.rs`

**Methods:**
- `track_vault(vault: &str) -> Result<()>` - Track specific vault
- `reconcile_balance(vault: &str) -> Result<()>` - Reconcile on-chain vs DB
- `start_monitoring()` - Start background monitoring

**Responsibilities:**
- Poll Solana for vault state
- Create balance snapshots
- Detect discrepancies
- Alert on anomalies

### TransactionBuilder

Constructs Solana transactions with proper compute budgets.

**Location:** `backend/src/transaction_builder.rs`

**Methods:**
- `build_deposit_transaction(instructions) -> Result<Transaction>`
- `build_withdraw_transaction(instructions) -> Result<Transaction>`

**Features:**
- Compute unit limits
- Priority fees
- Recent blockhash management
- Transaction signing

### CPIManager

Handles cross-program invocations.

**Location:** `backend/src/cpi_manager.rs`

**Methods:**
- `lock_collateral(user: &Pubkey, amount: u64) -> Result<String>`
- `unlock_collateral(user: &Pubkey, amount: u64) -> Result<String>`
- `is_authorized(program: &Pubkey) -> bool`

**Responsibilities:**
- Build CPI transactions
- Verify authorized programs
- Handle CPI errors
- Maintain consistency

### VaultMonitor

Monitors all vaults for security and analytics.

**Location:** `backend/src/vault_monitor.rs`

**Methods:**
- `monitor_vaults() -> Result<()>` - Background monitoring
- `get_tvl() -> Result<u64>` - Total value locked

**Responsibilities:**
- Track TVL
- Detect unauthorized access
- Generate alerts
- Analytics aggregation

### Database

PostgreSQL operations for transaction history and state.

**Location:** `backend/src/database.rs`

**Methods:**
- `create_transaction(...) -> Result<TransactionRecord>`
- `get_transactions(vault, limit) -> Result<Vec<TransactionRecord>>`
- `create_balance_snapshot(...) -> Result<BalanceSnapshot>`
- `get_latest_balance(vault) -> Result<Option<BalanceSnapshot>>`
- `get_total_tvl() -> Result<u64>`

## REST API

### Endpoints

#### POST /vault/initialize

Creates a new vault for a user.

**Request:**
```json
{
  "user": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU"
}
```

**Response:**
```json
{
  "success": true,
  "signature": "5j7s8K9...",
}
```

#### POST /vault/deposit

Deposits collateral into vault.

**Request:**
```json
{
  "user": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
  "amount": 1000000
}
```

**Response:**
```json
{
  "success": true,
  "signature": "5j7s8K9...",
}
```

#### POST /vault/withdraw

Withdraws collateral from vault.

**Request:**
```json
{
  "user": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
  "amount": 500000
}
```

**Response:**
```json
{
  "success": true,
  "signature": "5j7s8K9...",
}
```

#### GET /vault/balance/:user

Gets vault balance information.

**Response:**
```json
{
  "owner": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
  "vault": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
  "token_account": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
  "total_balance": 1000000,
  "locked_balance": 200000,
  "available_balance": 800000,
  "total_deposited": 2000000,
  "total_withdrawn": 1000000,
  "created_at": 1699123456
}
```

#### GET /vault/transactions/:user

Gets transaction history for a vault.

**Query Parameters:**
- `limit` (optional): Number of transactions (default: 100)

**Response:**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "vault": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
    "transaction_type": "deposit",
    "amount": 1000000,
    "signature": "5j7s8K9...",
    "timestamp": "2024-01-15T10:30:00Z"
  }
]
```

#### GET /vault/tvl

Gets total value locked across all vaults.

**Response:**
```json
{
  "total_value_locked": 50000000,
  "total_vaults": 1250,
  "timestamp": 1699123456
}
```

## WebSocket Streams

### Connection

```
ws://localhost:3000/ws
```

### Events

#### Balance Update

```json
{
  "type": "balance_update",
  "vault": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
  "total_balance": 1000000,
  "locked_balance": 200000,
  "available_balance": 800000,
  "timestamp": 1699123456
}
```

#### Deposit Notification

```json
{
  "type": "deposit",
  "user": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
  "amount": 1000000,
  "signature": "5j7s8K9...",
  "timestamp": 1699123456
}
```

#### Withdrawal Notification

```json
{
  "type": "withdrawal",
  "user": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
  "amount": 500000,
  "signature": "5j7s8K9...",
  "timestamp": 1699123456
}
```

#### TVL Update

```json
{
  "type": "tvl_update",
  "total_value_locked": 50000000,
  "timestamp": 1699123456
}
```

## Database Schema

See `backend/migrations/001_initial_schema.sql` for full schema.

### Tables

1. **transactions** - Transaction history
2. **balance_snapshots** - Hourly/daily balance snapshots
3. **vaults** - Vault metadata
4. **reconciliation_logs** - On-chain vs off-chain reconciliation
5. **audit_trail** - Security audit logs

## Configuration

### Environment Variables

```bash
DATABASE_URL=postgresql://localhost/collateral_vault
RPC_URL=https://api.devnet.solana.com
PROGRAM_ID=8vjbjPhoD2rav71J8mgbVxcYdbbqST78y2bzMPRqoGr9
USDT_MINT=Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB
PORT=3000
```

## Transaction Building

### Compute Budget

All transactions include compute budget instructions:

```rust
ComputeBudgetInstruction::set_compute_unit_limit(200_000)
```

### Priority Fees

Priority fees can be added for faster confirmation:

```rust
ComputeBudgetInstruction::set_compute_unit_price(priority_fee)
```

### Recent Blockhash

Transactions use recent blockhash for validity:

```rust
let recent_blockhash = rpc_client.get_latest_blockhash()?;
transaction.sign(&[keypair], recent_blockhash);
```

## Error Handling

### Error Types

```rust
pub enum Error {
    SolanaClient(String),
    AnchorClient(String),
    Database(sqlx::Error),
    Serialization(serde_json::Error),
    InvalidAccount(String),
    InsufficientBalance,
    VaultNotFound,
    TransactionFailed(String),
    Unauthorized,
}
```

### Error Responses

All API errors return appropriate HTTP status codes:
- `400` - Bad Request
- `404` - Not Found
- `500` - Internal Server Error

## Performance

### Caching

- Vault balances cached for 5 seconds
- TVL cached for 1 minute
- Transaction history paginated

### Database Indexing

- Indexed on `vault` for fast lookups
- Indexed on `timestamp` for time-based queries
- Composite indexes for common queries

## Security

### Input Validation

- All pubkeys validated before use
- Amounts checked for overflow
- User authentication (to be implemented)

### Rate Limiting

- Per-user rate limits on deposits/withdrawals
- Global rate limits on API endpoints

### Audit Logging

All operations logged to `audit_trail` table for security analysis.

