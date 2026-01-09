# Smart Contract Documentation

## Program ID

```
Fwfy9VwuzRqhQoCh4pk9JJ3dpBdTipUMPLVByCLWp6hf
```

## Account Structures

### CollateralVault

Main account structure storing vault state.

```rust
#[account]
pub struct CollateralVault {
    pub owner: Pubkey,              // 32 bytes
    pub token_account: Pubkey,      // 32 bytes
    pub total_balance: u64,         // 8 bytes
    pub locked_balance: u64,        // 8 bytes
    pub available_balance: u64,     // 8 bytes
    pub total_deposited: u64,       // 8 bytes
    pub total_withdrawn: u64,       // 8 bytes
    pub created_at: i64,            // 8 bytes
    pub bump: u8,                   // 1 byte
}
// Total: 8 (discriminator) + 105 = 113 bytes
```

**PDA Seeds:** `[b"vault", user_pubkey]`

### VaultAuthority

Stores authorized programs that can lock/unlock collateral.

```rust
#[account]
pub struct VaultAuthority {
    pub authorized_programs: Vec<Pubkey>,  // 4 + (32 * N) bytes
    pub bump: u8,                          // 1 byte
}
```

**PDA Seeds:** `[b"vault_authority"]`

**Max Authorized Programs:** 10 (configurable)

## Instructions

### initialize_vault

Creates a new vault for a user.

**Accounts:**
- `user` (mut, signer) - User creating the vault
- `vault` (init, mut) - PDA vault account
- `vault_token_account` (init, mut) - Associated token account
- `mint` - USDT mint account
- `vault_authority_pda` - PDA authority for token account
- `vault_authority` - Vault authority account
- `token_program` - SPL Token program
- `associated_token_program` - Associated Token program
- `system_program` - System program

**Constraints:**
- Vault must not already exist
- User must pay for account initialization

**Events:**
```rust
VaultInitialized {
    user: Pubkey,
    vault: Pubkey,
    timestamp: i64,
}
```

### deposit

Deposits USDT into the vault.

**Accounts:**
- `user` (mut, signer) - User depositing
- `vault` (mut) - User's vault account
- `user_token_account` (mut) - User's USDT token account
- `vault_token_account` (mut) - Vault's USDT token account
- `mint` - USDT mint
- `vault_authority_pda` - PDA authority
- `token_program` - SPL Token program

**Parameters:**
- `amount: u64` - Amount to deposit (must be > 0)

**Constraints:**
- Amount must be greater than 0
- User must have sufficient balance
- Vault must be initialized

**Events:**
```rust
DepositEvent {
    user: Pubkey,
    vault: Pubkey,
    amount: u64,
    new_balance: u64,
    timestamp: i64,
}
```

### withdraw

Withdraws USDT from the vault.

**Accounts:**
- `user` (mut, signer) - Vault owner
- `vault` (mut) - User's vault account
- `user_token_account` (mut) - User's USDT token account
- `vault_token_account` (mut) - Vault's USDT token account
- `mint` - USDT mint
- `vault_authority_pda` - PDA authority (signer)
- `vault_authority` - Vault authority account
- `token_program` - SPL Token program

**Parameters:**
- `amount: u64` - Amount to withdraw (must be > 0)

**Constraints:**
- User must be vault owner
- Available balance must be >= amount
- Amount must be greater than 0

**Events:**
```rust
WithdrawEvent {
    user: Pubkey,
    vault: Pubkey,
    amount: u64,
    new_balance: u64,
    timestamp: i64,
}
```

### lock_collateral

Locks collateral for a position (CPI callable).

**Accounts:**
- `vault` (mut) - Vault account
- `vault_authority` - Vault authority account
- `caller_program` - Program making the CPI call

**Parameters:**
- `amount: u64` - Amount to lock (must be > 0)

**Constraints:**
- Caller program must be in authorized_programs
- Available balance must be >= amount
- Amount must be greater than 0

**Events:**
```rust
LockEvent {
    user: Pubkey,
    vault: Pubkey,
    amount: u64,
    locked_balance: u64,
    timestamp: i64,
}
```

### unlock_collateral

Unlocks collateral when a position closes (CPI callable).

**Accounts:**
- `vault` (mut) - Vault account
- `vault_authority` - Vault authority account
- `caller_program` - Program making the CPI call

**Parameters:**
- `amount: u64` - Amount to unlock (must be > 0)

**Constraints:**
- Caller program must be in authorized_programs
- Locked balance must be >= amount
- Amount must be greater than 0

**Events:**
```rust
UnlockEvent {
    user: Pubkey,
    vault: Pubkey,
    amount: u64,
    locked_balance: u64,
    timestamp: i64,
}
```

### transfer_collateral

Transfers collateral between vaults (for settlements/liquidations).

**Accounts:**
- `from_vault` (mut) - Source vault
- `to_vault` (mut) - Destination vault
- `from_vault_token_account` (mut) - Source token account
- `to_vault_token_account` (mut) - Destination token account
- `from_vault_authority` - Source vault PDA authority (signer)
- `to_vault_authority` - Destination vault PDA authority
- `vault_authority` - Vault authority account
- `caller_program` - Program making the CPI call
- `token_program` - SPL Token program

**Parameters:**
- `amount: u64` - Amount to transfer (must be > 0)

**Constraints:**
- Caller program must be in authorized_programs
- From vault available balance must be >= amount
- Amount must be greater than 0

**Events:**
```rust
TransferEvent {
    from_user: Pubkey,
    to_user: Pubkey,
    from_vault: Pubkey,
    to_vault: Pubkey,
    amount: u64,
    timestamp: i64,
}
```

### initialize_vault_authority

Initializes the vault authority account (admin only).

**Accounts:**
- `admin` (mut, signer) - Admin initializing
- `vault_authority` (init) - Vault authority account
- `system_program` - System program

**Parameters:**
- `authorized_programs: Vec<Pubkey>` - List of authorized program IDs

## Error Codes

```rust
#[error_code]
pub enum ErrorCode {
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Insufficient available balance")]
    InsufficientAvailableBalance,
    #[msg("Insufficient locked balance")]
    InsufficientLockedBalance,
    #[msg("Unauthorized owner")]
    UnauthorizedOwner,
    #[msg("Unauthorized program")]
    UnauthorizedProgram,
    #[msg("Integer overflow")]
    Overflow,
    #[msg("Integer underflow")]
    Underflow,
}
```

## PDA Seeds Reference

| Account | Seeds | Bump |
|---------|-------|------|
| User Vault | `[b"vault", user_pubkey]` | Stored in account |
| Vault Authority | `[b"vault_authority"]` | Stored in account |
| Vault Token Account | Associated Token Account | N/A |

## Authority Validation

1. **Withdraw**: Checks `vault.owner == user.key()`
2. **Lock/Unlock**: Checks `vault_authority.authorized_programs.contains(caller_program)`
3. **Transfer**: Checks authorized program + sufficient balance

## Rent Exemption

All accounts are initialized as rent-exempt:
- Vault accounts: ~113 bytes (rent-exempt minimum)
- Vault Authority: Variable based on authorized programs count

