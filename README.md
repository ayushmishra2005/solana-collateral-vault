# Collateral Vault Management System

A decentralized collateral vault management system for perpetual futures exchanges on Solana. This system securely manages user collateral (USDT) in program-controlled vaults, enabling non-custodial trading operations.

## Features

- **PDA-based Vaults**: Each user has an isolated vault account derived from their public key
- **Secure Collateral Management**: Deposit, withdraw, lock, and unlock collateral with proper access controls
- **Cross-Program Invocations**: Position management programs can lock/unlock collateral via CPI
- **Real-time Monitoring**: Backend service tracks balances, TVL, and transaction history
- **REST API**: HTTP endpoints for vault operations
- **WebSocket Support**: Real-time event streaming for balance updates
- **PostgreSQL Integration**: Transaction history and balance snapshots

## Project Structure

```
.
├── programs/
│   └── collateral-vault/          # Anchor program (Solana smart contract)
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
├── backend/                       # Rust backend service
│   ├── src/
│   │   ├── main.rs
│   │   ├── api.rs                 # REST API endpoints
│   │   ├── vault_manager.rs       # Vault operations
│   │   ├── balance_tracker.rs     # Balance monitoring
│   │   └── ...
│   └── migrations/
│       └── 001_initial_schema.sql # Database schema
├── tests/                         # Anchor program tests
│   └── collateral-vault.ts
├── ARCHITECTURE.md                # System architecture
├── SMART_CONTRACT.md              # Smart contract documentation
├── BACKEND_SERVICE.md             # Backend service docs
├── SECURITY.md                    # Security analysis
├── DEPLOYMENT.md                  # Deployment guide
└── API.md                         # API documentation
```

## Prerequisites

- **Rust** 1.75+ ([Install Rust](https://rustup.rs/))
- **Anchor** 0.29+ ([Install Anchor](https://www.anchor-lang.com/docs/installation))
- **Solana CLI** 1.18+ ([Install Solana](https://docs.solana.com/cli/install-solana-cli-tools))
- **PostgreSQL** 14+ ([Install PostgreSQL](https://www.postgresql.org/download/))
- **Node.js** 18+ and **Yarn** ([Install Node.js](https://nodejs.org/))

## Installation

### 1. Clone and Setup

```bash
cd solana-collateral-vault
```

### 2. Install Dependencies

```bash
# Install Node dependencies
yarn install

# Build Anchor program
anchor build
```

### 3. Setup Database

```bash
# Create database
createdb collateral_vault

# Run migrations
psql collateral_vault < backend/migrations/001_initial_schema.sql
```

### 4. Configure Environment

Create a `.env` file in the root directory:

```bash
DATABASE_URL=postgresql://localhost/collateral_vault
RPC_URL=https://api.devnet.solana.com
PROGRAM_ID=8vjbjPhoD2rav71J8mgbVxcYdbbqST78y2bzMPRqoGr9
USDT_MINT=Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB
PORT=3000
```

## Running the Project

### 1. Start Local Validator (Optional - for local testing)

```bash
solana-test-validator
```

In another terminal, configure for localnet:

```bash
solana config set --url localhost
```

### 2. Deploy the Program

```bash
# Build and deploy
anchor build
anchor deploy

# Or deploy to devnet
anchor deploy --provider.cluster devnet
```

### 3. Initialize Vault Authority

After deployment, you need to initialize the vault authority. This can be done via the test suite or a separate script:

```bash
anchor test --skip-local-validator
```

### 4. Start Backend Service

```bash
cd backend
cargo run --release
```

The server will start on `http://localhost:3000`

## Complete Testing Guide (From Scratch)

### Step 1: Start Local Solana Validator

Open Terminal 1:
```bash
solana-test-validator
```

Keep this running. In a new terminal, configure Solana for localnet:
```bash
solana config set --url localhost
```

### Step 2: Create Test Token

In the same terminal:
```bash
spl-token create-token --decimals 6
```

**Important**: Note the mint address output (e.g., `EM8thaMRrQ6cYK7zzVjzw9wwcL4RJcFn7QesJhr8UkY3`). You'll need this for the `USDT_MINT` environment variable.

### Step 3: Setup Database

```bash
# Create database
createdb collateral_vault

# Run migrations
psql collateral_vault < backend/migrations/001_initial_schema.sql
```

### Step 4: Build and Deploy Program

```bash
# Build the Anchor program
anchor build

# Deploy to local validator
anchor deploy
```

### Step 5: Get Your Wallet Address

```bash
solana address
```

**Important**: For local testing, you must use the same wallet that's running the backend server (the default Solana wallet at `~/.config/solana/id.json`). This is because the backend signs transactions with this wallet. In production, users would sign their own transactions.

### Step 6: Mint Tokens to Your Wallet

```bash
# Create associated token account (replace MINT_ADDRESS with your mint from Step 2)
spl-token create-account <MINT_ADDRESS>

# Mint tokens (10,000 tokens with 6 decimals = 10000000000 in smallest units)
spl-token mint <MINT_ADDRESS> 10000
```

### Step 7: Start Backend Server

Open Terminal 2:
```bash
cd backend

# Set environment variables (replace MINT_ADDRESS with your mint from Step 2)
export DATABASE_URL=postgresql://localhost/collateral_vault
export RPC_URL=http://localhost:8899
export PROGRAM_ID=8vjbjPhoD2rav71J8mgbVxcYdbbqST78y2bzMPRqoGr9
export USDT_MINT=<MINT_ADDRESS>

# Start the server
cargo run
```

The server will start on `http://localhost:3000`. Keep this running.

### Step 8: Test All APIs

Open Terminal 3. Replace `YOUR_PUBKEY` with your wallet address from Step 5:

```bash
# Set your pubkey
YOUR_PUBKEY="<your_pubkey_from_step_5>"

# 1. Initialize Vault
curl -X POST http://localhost:3000/vault/initialize \
  -H "Content-Type: application/json" \
  -d "{\"user\": \"$YOUR_PUBKEY\"}"

# 2. Get Vault Balance
curl http://localhost:3000/vault/balance/$YOUR_PUBKEY

# 3. Deposit Tokens (1000 tokens = 1000000000 with 6 decimals)
curl -X POST http://localhost:3000/vault/deposit \
  -H "Content-Type: application/json" \
  -d "{\"user\": \"$YOUR_PUBKEY\", \"amount\": 1000000000}"

# 4. Get Balance After Deposit
curl http://localhost:3000/vault/balance/$YOUR_PUBKEY

# 5. Withdraw Tokens (500 tokens = 500000000 with 6 decimals)
curl -X POST http://localhost:3000/vault/withdraw \
  -H "Content-Type: application/json" \
  -d "{\"user\": \"$YOUR_PUBKEY\", \"amount\": 500000000}"

# 6. Get Balance After Withdraw
curl http://localhost:3000/vault/balance/$YOUR_PUBKEY

# 7. Get Transaction History
curl http://localhost:3000/vault/transactions/$YOUR_PUBKEY

# 8. Get Total Value Locked (TVL)
curl http://localhost:3000/vault/tvl
```

**Alternative**: You can use the provided test script:
```bash
# Set your pubkey as environment variable
export ADMIN_PUBKEY="<your_pubkey>"

# Run all API tests
./test-all-apis.sh
```

### Expected Results

- **Initialize**: Returns `{"success": true, "signature": "..."}`
- **Balance**: Returns vault info with balances (initially 0, then updated after deposits/withdrawals)
- **Deposit/Withdraw**: Returns transaction signature
- **Transactions**: Returns array of all transactions for the vault
- **TVL**: Returns total value locked across all vaults

### Test the Anchor Program

```bash
# Run all tests
anchor test

# Run with local validator (skip starting validator)
anchor test --skip-local-validator

# Run specific test file
anchor test tests/collateral-vault.ts
```

## Development

### Build Commands

```bash
# Build Anchor program
anchor build

# Build backend
cd backend
cargo build

# Build for release
cargo build --release
```

### Code Formatting

```bash
# Format Rust code
cargo fmt

# Format TypeScript
yarn format
```

### Linting

```bash
# Lint Rust code
cargo clippy

# Lint TypeScript
yarn lint
```

## Key Components

### Anchor Program

The Solana smart contract located in `programs/collateral-vault/src/lib.rs`:

- `initialize_vault` - Create new vault
- `deposit` - Deposit USDT into vault
- `withdraw` - Withdraw USDT from vault
- `lock_collateral` - Lock collateral for positions (CPI)
- `unlock_collateral` - Unlock collateral (CPI)
- `transfer_collateral` - Transfer between vaults (CPI)

### Backend Service

Rust service providing:

- REST API endpoints for vault operations
- WebSocket streams for real-time updates
- Database integration for transaction history
- Balance tracking and reconciliation
- TVL monitoring

**Backend File Structure:**

- `main.rs` - Entry point, sets up database connection, Solana client, and starts the HTTP server
- `api.rs` - Defines all REST API endpoints (initialize, deposit, withdraw, balance, transactions, TVL)
- `vault_manager.rs` - Core vault operations: interacts with Solana blockchain, builds transactions, fetches on-chain data
- `transaction_builder.rs` - Builds Solana instructions for initialize, deposit, and withdraw operations
- `database.rs` - PostgreSQL operations: stores transactions, calculates TVL from transaction history
- `vault_monitor.rs` - Monitors vaults and calculates TVL (used by TVL endpoint)
- `models.rs` - Data structures for API requests/responses and database records
- `error.rs` - Custom error types for the backend
- `balance_tracker.rs` - Placeholder for future balance tracking and reconciliation features
- `cpi_manager.rs` - Placeholder for future Cross-Program Invocation (CPI) management features
- `websocket.rs` - Placeholder for future WebSocket real-time event streaming

**Required Files for Testing:**
- All files in `backend/src/` are needed to run the application
- `backend/migrations/001_initial_schema.sql` is required for database setup

## API Endpoints

- `POST /vault/initialize` - Create new vault
- `POST /vault/deposit` - Deposit collateral
- `POST /vault/withdraw` - Withdraw collateral
- `GET /vault/balance/:user` - Get vault balance
- `GET /vault/transactions/:user` - Get transaction history
- `GET /vault/tvl` - Get total value locked

See [API.md](./API.md) for detailed API documentation.

## Important Notes

### Local Testing Limitation

For local testing, the user pubkey in API requests must match the admin wallet (the wallet used to run the backend server, located at `~/.config/solana/id.json`). This is because the backend signs transactions with this wallet. In production, users would sign their own transactions using their wallets.

### Environment Variables

The backend requires these environment variables:
- `DATABASE_URL` - PostgreSQL connection string (default: `postgresql://localhost/collateral_vault`)
- `RPC_URL` - Solana RPC endpoint (default: `http://localhost:8899` for local, or `https://api.devnet.solana.com` for devnet)
- `PROGRAM_ID` - Your deployed program ID (default: `8vjbjPhoD2rav71J8mgbVxcYdbbqST78y2bzMPRqoGr9`)
- `USDT_MINT` - SPL token mint address for collateral (required - no default for local testing)

### No Hardcoded Values

The system does not use hardcoded wallet addresses or pubkeys. All values are:
- Loaded from environment variables
- Read from the default Solana wallet (`~/.config/solana/id.json`)
- Provided by the user via API requests

## Documentation

- [ARCHITECTURE.md](./ARCHITECTURE.md) - System architecture and design
- [SMART_CONTRACT.md](./SMART_CONTRACT.md) - Smart contract specifications
- [SPL_TOKEN_INTEGRATION.md](./SPL_TOKEN_INTEGRATION.md) - SPL Token integration guide
- [BACKEND_SERVICE.md](./BACKEND_SERVICE.md) - Backend service documentation
- [SECURITY.md](./SECURITY.md) - Security analysis and best practices
- [DEPLOYMENT.md](./DEPLOYMENT.md) - Production deployment guide
- [API.md](./API.md) - REST API documentation

