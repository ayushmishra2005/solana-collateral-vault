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
PROGRAM_ID=Fwfy9VwuzRqhQoCh4pk9JJ3dpBdTipUMPLVByCLWp6hf
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

## Testing

### Test the Anchor Program

```bash
# Run all tests
anchor test

# Run with local validator
anchor test --skip-local-validator

# Run specific test file
anchor test tests/collateral-vault.ts
```

### Test the Backend API

```bash
# Start the backend server first
cd backend
cargo run

# In another terminal, test endpoints
curl http://localhost:3000/vault/tvl

# Initialize a vault (replace with your pubkey)
curl -X POST http://localhost:3000/vault/initialize \
  -H "Content-Type: application/json" \
  -d '{"user": "YOUR_PUBKEY_HERE"}'

# Get vault balance
curl http://localhost:3000/vault/balance/YOUR_PUBKEY_HERE
```

### Manual Testing Flow

1. **Initialize Vault**
   ```bash
   curl -X POST http://localhost:3000/vault/initialize \
     -H "Content-Type: application/json" \
     -d '{"user": "YOUR_PUBKEY"}'
   ```

2. **Deposit Collateral**
   ```bash
   curl -X POST http://localhost:3000/vault/deposit \
     -H "Content-Type: application/json" \
     -d '{"user": "YOUR_PUBKEY", "amount": 1000000}'
   ```

3. **Check Balance**
   ```bash
   curl http://localhost:3000/vault/balance/YOUR_PUBKEY
   ```

4. **Withdraw Collateral**
   ```bash
   curl -X POST http://localhost:3000/vault/withdraw \
     -H "Content-Type: application/json" \
     -d '{"user": "YOUR_PUBKEY", "amount": 500000}'
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

## API Endpoints

- `POST /vault/initialize` - Create new vault
- `POST /vault/deposit` - Deposit collateral
- `POST /vault/withdraw` - Withdraw collateral
- `GET /vault/balance/:user` - Get vault balance
- `GET /vault/transactions/:user` - Get transaction history
- `GET /vault/tvl` - Get total value locked

See [API.md](./API.md) for detailed API documentation.

## Documentation

- [ARCHITECTURE.md](./ARCHITECTURE.md) - System architecture and design
- [SMART_CONTRACT.md](./SMART_CONTRACT.md) - Smart contract specifications
- [SPL_TOKEN_INTEGRATION.md](./SPL_TOKEN_INTEGRATION.md) - SPL Token integration guide
- [BACKEND_SERVICE.md](./BACKEND_SERVICE.md) - Backend service documentation
- [SECURITY.md](./SECURITY.md) - Security analysis and best practices
- [DEPLOYMENT.md](./DEPLOYMENT.md) - Production deployment guide
- [API.md](./API.md) - REST API documentation

## Troubleshooting

### Program Build Fails

```bash
# Update Anchor
avm install latest
avm use latest

# Clean and rebuild
anchor clean
anchor build
```

### Database Connection Issues

```bash
# Check PostgreSQL is running
sudo systemctl status postgresql

# Verify connection
psql -U postgres -d collateral_vault
```

### Transaction Failures

- Ensure you have sufficient SOL for transaction fees
- Check RPC endpoint is accessible
- Verify account addresses are correct
- Check compute unit limits

### Backend Won't Start

- Verify `.env` file exists with correct values
- Check database is running and accessible
- Ensure port 3000 is not in use
- Check logs for specific error messages

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

This project is part of a technical assessment for GoQuant.

## Support

For issues or questions:
- Check the documentation files
- Review logs for error messages
- Open an issue on the repository

