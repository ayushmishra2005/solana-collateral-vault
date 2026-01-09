# Deployment Guide

## Prerequisites

### Required Software

- Rust 1.75+ (with async/await support)
- Anchor 0.29+
- Solana CLI tools
- PostgreSQL 14+
- Node.js 18+ and Yarn
- Git

### System Requirements

- 4GB+ RAM
- 50GB+ disk space
- Network access to Solana RPC endpoint

## Environment Setup

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Install Solana CLI

```bash
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
```

### 3. Install Anchor

```bash
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest
```

### 4. Install PostgreSQL

**macOS:**
```bash
brew install postgresql@14
brew services start postgresql@14
```

**Linux:**
```bash
sudo apt-get install postgresql-14
sudo systemctl start postgresql
```

### 5. Install Node.js and Yarn

```bash
# Install Node.js via nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
nvm use 18

# Install Yarn
npm install -g yarn
```

## Database Setup

### 1. Create Database

```bash
createdb collateral_vault
```

### 2. Run Migrations

```bash
cd backend
psql collateral_vault < migrations/001_initial_schema.sql
```

### 3. Verify Schema

```bash
psql collateral_vault -c "\dt"
```

## Anchor Program Deployment

### 1. Build Program

```bash
anchor build
```

This will:
- Compile the Rust program
- Generate IDL
- Create deploy keypair

### 2. Configure Network

Update `Anchor.toml` for your target network:

```toml
[provider]
cluster = "devnet"  # or "mainnet-beta"
wallet = "~/.config/solana/id.json"
```

### 3. Deploy to Devnet

```bash
# Get SOL for deployment
solana airdrop 2

# Deploy program
anchor deploy
```

### 4. Initialize Vault Authority

After deployment, initialize the vault authority:

```typescript
// In tests or separate script
const tx = await program.methods
  .initializeVaultAuthority([authorizedProgramId])
  .accounts({
    admin: adminKeypair.publicKey,
    vaultAuthority: vaultAuthorityPda,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

### 5. Verify Deployment

```bash
solana program show <PROGRAM_ID>
```

## Backend Service Deployment

### 1. Configure Environment

Create `.env` file:

```bash
DATABASE_URL=postgresql://localhost/collateral_vault
RPC_URL=https://api.devnet.solana.com
PROGRAM_ID=<YOUR_PROGRAM_ID>
USDT_MINT=Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB
PORT=3000
```

### 2. Build Backend

```bash
cd backend
cargo build --release
```

### 3. Run Backend

```bash
cargo run --release
```

Or as a service:

```bash
# Create systemd service file
sudo nano /etc/systemd/system/collateral-vault.service
```

```ini
[Unit]
Description=Collateral Vault Backend Service
After=network.target postgresql.service

[Service]
Type=simple
User=your-user
WorkingDirectory=/path/to/solana-collateral-vault/backend
Environment="DATABASE_URL=postgresql://localhost/collateral_vault"
Environment="RPC_URL=https://api.devnet.solana.com"
Environment="PROGRAM_ID=<YOUR_PROGRAM_ID>"
ExecStart=/path/to/solana-collateral-vault/backend/target/release/server
Restart=always

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable collateral-vault
sudo systemctl start collateral-vault
```

## Production Deployment

### 1. Mainnet Deployment

**Warning**: Deploy to mainnet only after thorough testing on devnet.

```bash
# Update Anchor.toml
[provider]
cluster = "mainnet-beta"

# Deploy
anchor deploy --provider.cluster mainnet-beta
```

### 2. Security Checklist

- [ ] Program audited
- [ ] All tests passing
- [ ] Security review completed
- [ ] Backup procedures in place
- [ ] Monitoring configured
- [ ] Incident response plan ready

### 3. Monitoring Setup

**Recommended Tools:**
- Prometheus for metrics
- Grafana for dashboards
- Sentry for error tracking
- PagerDuty for alerts

### 4. Backup Strategy

**Database:**
```bash
# Daily backups
pg_dump collateral_vault > backup_$(date +%Y%m%d).sql
```

**On-Chain:**
- All state is on-chain (immutable)
- Index events for off-chain queries

## Docker Deployment (Optional)

### Dockerfile

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/server /usr/local/bin/server
CMD ["server"]
```

### docker-compose.yml

```yaml
version: '3.8'
services:
  backend:
    build: .
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgresql://postgres:password@db/collateral_vault
      - RPC_URL=https://api.devnet.solana.com
    depends_on:
      - db
  
  db:
    image: postgres:14
    environment:
      - POSTGRES_DB=collateral_vault
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

## Verification

### 1. Health Check

```bash
curl http://localhost:3000/vault/tvl
```

### 2. Test Vault Operations

```bash
# Initialize vault
curl -X POST http://localhost:3000/vault/initialize \
  -H "Content-Type: application/json" \
  -d '{"user": "YOUR_USER_PUBKEY"}'

# Check balance
curl http://localhost:3000/vault/balance/YOUR_USER_PUBKEY
```

### 3. Monitor Logs

```bash
# Backend logs
journalctl -u collateral-vault -f

# Database logs
tail -f /var/log/postgresql/postgresql-14-main.log
```

## Troubleshooting

### Program Deployment Fails

**Issue**: Insufficient SOL
```bash
solana balance
solana airdrop 2
```

**Issue**: Program too large
- Optimize code
- Use program data accounts if needed

### Backend Connection Issues

**Issue**: Database connection failed
```bash
# Check PostgreSQL is running
sudo systemctl status postgresql

# Check connection
psql -U postgres -d collateral_vault
```

**Issue**: RPC connection failed
- Check RPC endpoint is accessible
- Verify network connectivity
- Try different RPC provider

### Transaction Failures

**Issue**: Insufficient compute units
- Increase compute budget in transaction builder
- Optimize program instructions

**Issue**: Account not found
- Verify account addresses
- Check account initialization

## Maintenance

### Regular Tasks

1. **Database Maintenance**
   - Vacuum database weekly
   - Archive old transactions monthly
   - Monitor disk space

2. **Program Updates**
   - Test on devnet first
   - Deploy during low-traffic periods
   - Monitor for issues

3. **Security Updates**
   - Keep dependencies updated
   - Monitor security advisories
   - Apply patches promptly

### Backup and Recovery

**Database Backup:**
```bash
# Daily backup
pg_dump collateral_vault | gzip > backup_$(date +%Y%m%d).sql.gz

# Restore
gunzip < backup_20240115.sql.gz | psql collateral_vault
```

**Configuration Backup:**
- Backup `.env` files
- Backup Anchor keypairs (securely)
- Document all configuration

## Scaling

### Horizontal Scaling

- Multiple backend instances behind load balancer
- Database read replicas
- Separate RPC endpoints per instance

### Vertical Scaling

- Increase database resources
- Upgrade server hardware
- Optimize queries and indexes

## Support

For issues or questions:
- Check logs first
- Review documentation
- Open GitHub issue
- Contact support team

