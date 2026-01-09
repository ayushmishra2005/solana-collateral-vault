# SPL Token Integration Guide

## Overview

The Collateral Vault system uses SPL Token for managing USDT collateral. All token operations are performed via Cross-Program Invocations (CPIs) to the SPL Token program.

## Token Account Structure

### Associated Token Accounts

Each vault uses an Associated Token Account (ATA) for USDT storage:

```rust
vault_token_account = get_associated_token_address(
    wallet: vault_pda,
    mint: usdt_mint
)
```

**Benefits:**
- Deterministic address derivation
- Automatic account creation
- Standard Solana pattern

### Token Account Authority

The vault PDA is the authority for the vault's token account:

```rust
vault_pda = find_program_address(
    seeds: [b"vault", user_pubkey],
    program_id: collateral_vault_program
)
```

This allows the program to sign for token transfers on behalf of the vault.

## Token Transfer Flows

### Deposit Flow

**User → Vault Transfer**

```rust
token::transfer(
    CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    ),
    amount,
)?;
```

**Steps:**
1. User signs the transaction
2. CPI to SPL Token program
3. Token program transfers USDT from user ATA to vault ATA
4. Vault state updated atomically

### Withdrawal Flow

**Vault → User Transfer**

```rust
let seeds = &[
    b"vault",
    vault.owner.as_ref(),
    &[vault.bump],
];
let signer = &[&seeds[..]];

token::transfer(
    CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.vault_authority_pda.to_account_info(),
        },
        signer,
    ),
    amount,
)?;
```

**Steps:**
1. Vault PDA signs using seeds
2. CPI to SPL Token program with PDA signer
3. Token program transfers USDT from vault ATA to user ATA
4. Vault state updated atomically

### Transfer Between Vaults

**Vault A → Vault B Transfer**

```rust
let seeds = &[
    b"vault",
    from_vault.owner.as_ref(),
    &[from_vault.bump],
];
let signer = &[&seeds[..]];

token::transfer(
    CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.from_vault_token_account.to_account_info(),
            to: ctx.accounts.to_vault_token_account.to_account_info(),
            authority: ctx.accounts.from_vault_authority.to_account_info(),
        },
        signer,
    ),
    amount,
)?;
```

## CPI to Token Program

### CPI Context

All token operations use `CpiContext` to invoke the SPL Token program:

```rust
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

// Without signer (user-initiated)
CpiContext::new(
    token_program.to_account_info(),
    Transfer { ... }
)

// With PDA signer (program-initiated)
CpiContext::new_with_signer(
    token_program.to_account_info(),
    Transfer { ... },
    signer_seeds
)
```

### Account Validation

Anchor automatically validates:
- Token account ownership
- Mint consistency
- Authority permissions
- Account mutability

## Error Handling

### Common Token Errors

1. **Insufficient Funds**
   - Checked before transfer: `require!(available_balance >= amount)`
   - Token program will also validate

2. **Invalid Authority**
   - Anchor constraints ensure correct authority
   - PDA seeds must match for signing

3. **Account Not Found**
   - Associated token accounts must exist
   - Use `init` constraint for creation

### Error Propagation

```rust
token::transfer(...)?;  // Propagates TokenError
```

Token program errors are automatically converted to Anchor errors.

## Account Initialization

### Creating Associated Token Account

```rust
#[account(
    init,
    payer = user,
    associated_token::mint = mint,
    associated_token::authority = vault_authority_pda
)]
pub vault_token_account: Account<'info, TokenAccount>,
```

**Requirements:**
- Payer must have SOL for rent
- Mint account must exist
- Authority must be valid PDA

## Token Account Queries

### Checking Balance

```rust
let token_account = ctx.accounts.vault_token_account;
let balance = token_account.amount;
```

### Mint Information

```rust
let mint = ctx.accounts.mint;
let decimals = mint.decimals;
let supply = mint.supply;
```

## Best Practices

### 1. Always Validate Before Transfer

```rust
require!(
    vault.available_balance >= amount,
    ErrorCode::InsufficientAvailableBalance
);
```

### 2. Use Checked Arithmetic

```rust
vault.total_balance = vault.total_balance
    .checked_add(amount)
    .ok_or(ErrorCode::Overflow)?;
```

### 3. Atomic State Updates

Update vault state in the same transaction as token transfer to ensure consistency.

### 4. Event Emission

Emit events after successful transfers for off-chain indexing:

```rust
emit!(DepositEvent {
    user: ctx.accounts.user.key(),
    vault: vault.key(),
    amount,
    new_balance: vault.total_balance,
    timestamp: Clock::get()?.unix_timestamp,
});
```

## Integration with Position Manager

When position manager needs to lock collateral:

```rust
// In position manager program
let cpi_accounts = collateral_vault::cpi::accounts::LockCollateral {
    vault: vault_account,
    vault_authority: vault_authority_account,
    caller_program: program_id,
};

let cpi_program = ctx.accounts.collateral_vault_program.to_account_info();
let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

collateral_vault::cpi::lock_collateral(cpi_ctx, amount)?;
```

## Testing Token Operations

### Mock Token Accounts

In tests, create token accounts:

```typescript
const mint = await createMint(provider.connection, payer);
const userTokenAccount = await createAssociatedTokenAccount(
    provider.connection,
    payer,
    mint,
    user.publicKey
);
```

### Airdrop Tokens

```typescript
await mintTo(
    provider.connection,
    payer,
    mint,
    userTokenAccount,
    mintAuthority,
    1000000 // 1 USDT
);
```

## Security Considerations

1. **Authority Checks**: Always verify authority before transfers
2. **Balance Validation**: Check on-chain balance matches off-chain state
3. **Reentrancy**: Anchor's account model prevents reentrancy
4. **Integer Safety**: Use checked arithmetic for all calculations

