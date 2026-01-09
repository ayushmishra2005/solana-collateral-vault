# System Architecture

## Overview

The Collateral Vault Management System is a decentralized custody layer for a perpetual futures exchange on Solana. It securely manages user collateral (USDT) in program-controlled vaults, enabling non-custodial trading operations.

## Architecture Components

### 1. Solana Smart Contract (Anchor Program)

The core on-chain component that manages vault accounts and collateral operations.

**Location:** `programs/collateral-vault/src/lib.rs`

**Key Features:**
- PDA-based vault accounts for each user
- Associated token accounts for USDT storage
- Cross-program invocations (CPIs) for position management
- Atomic state updates
- Event emissions for off-chain indexing

### 2. Rust Backend Service

Off-chain service that interfaces with the Solana program and provides APIs.

**Location:** `backend/src/`

**Components:**
- Vault Manager: Handles vault lifecycle and operations
- Balance Tracker: Monitors vault balances in real-time
- Transaction Builder: Constructs Solana transactions
- CPI Manager: Manages cross-program invocations
- Vault Monitor: Tracks TVL and security
- REST API: HTTP endpoints for client interactions
- WebSocket: Real-time event streaming

### 3. PostgreSQL Database

Stores transaction history, balance snapshots, and audit logs.

**Location:** `backend/migrations/001_initial_schema.sql`

## Vault Structure

### PDA Derivation

Vaults are derived using Program Derived Addresses (PDAs):

```
vault_pda = find_program_address(
    seeds: [b"vault", user_pubkey],
    program_id: collateral_vault_program
)
```

### Account Structure

```
CollateralVault {
    owner: Pubkey,              // User who owns the vault
    token_account: Pubkey,      // Associated token account for USDT
    total_balance: u64,         // Total USDT in vault
    locked_balance: u64,        // USDT locked in positions
    available_balance: u64,     // Available for withdrawal (total - locked)
    total_deposited: u64,       // Lifetime deposits
    total_withdrawn: u64,       // Lifetime withdrawals
    created_at: i64,            // Unix timestamp
    bump: u8                     // PDA bump seed
}
```

## Data Flow

### Deposit Flow

```
User Wallet → SPL Token Transfer (CPI) → Vault Token Account
                ↓
         Update Vault State
                ↓
         Emit Deposit Event
                ↓
         Backend Indexes Event
                ↓
         Update Database
```

### Withdrawal Flow

```
User Request → Verify Available Balance
                ↓
         SPL Token Transfer (CPI) → User Wallet
                ↓
         Update Vault State
                ↓
         Emit Withdraw Event
```

### Lock/Unlock Flow (CPI from Position Manager)

```
Position Manager → CPI to Vault Program
                        ↓
                 Verify Authorized Program
                        ↓
                 Update Locked Balance
                        ↓
                 Emit Lock/Unlock Event
```

## Security Model

### Access Control

1. **Vault Ownership**: Only vault owner can withdraw
2. **Authorized Programs**: Only programs in `VaultAuthority` can lock/unlock
3. **PDA Signing**: Vault operations use PDA as signer for token transfers
4. **Balance Validation**: All operations check sufficient balance before execution

### Threat Mitigation

- **Integer Overflow/Underflow**: All arithmetic uses `checked_add`/`checked_sub`
- **Reentrancy**: Anchor's account model prevents reentrancy attacks
- **Double Spending**: Atomic state updates prevent race conditions
- **Unauthorized Access**: Constraint checks on all account structs

## CPI Flow Diagram

```
┌─────────────────┐
│ Position Manager│
│    Program      │
└────────┬────────┘
         │ CPI: lock_collateral()
         ▼
┌─────────────────┐
│  Vault Program  │
│  - Verify Auth  │
│  - Lock Balance │
│  - Emit Event   │
└─────────────────┘
         │
         ▼
┌─────────────────┐
│  Backend Indexer│
│  - Update DB    │
│  - Notify WS    │
└─────────────────┘
```

## Performance Considerations

- **PDA Lookups**: O(1) vault access via PDA derivation
- **Account Size**: Fixed-size accounts for predictable rent
- **Event Indexing**: Off-chain indexing for query performance
- **Batch Operations**: Backend can batch multiple operations
- **Compute Budget**: Transactions include compute unit limits

## Scalability

- **Isolated Vaults**: Each user has independent vault (no shared state)
- **Parallel Processing**: Multiple vaults can be processed concurrently
- **Database Sharding**: Can shard by vault owner for scale
- **Caching**: Backend caches frequently accessed vault data

