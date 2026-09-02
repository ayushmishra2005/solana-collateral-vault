# Collateral Vault Management System

A Solana collateral vault for perpetual trading workflows.

Program ID: `8vjbjPhoD2rav71J8mgbVxcYdbbqST78y2bzMPRqoGr9`

## What it does

Each user has a vault PDA and an associated SPL token account.

### Implemented

- Deposit collateral
- Withdraw available collateral
- Lock collateral for positions
- Unlock collateral
- Transfer collateral between vaults
- Allowlist of programs that may lock, unlock, or transfer through CPI
- Rust backend with a REST API
- PostgreSQL storage for transaction history

### Not implemented / planned

- WebSocket streaming (`backend/src/websocket.rs` is unused)
- Balance reconciliation (`backend/src/balance_tracker.rs` is a stub)
- Backend CPI helpers (`backend/src/cpi_manager.rs` returns placeholder signatures)
- Background vault monitoring (`VaultMonitor::monitor_vaults` is an empty loop)
- API authentication
- Rate limiting
- Changing the authorized program list after init
- Crediting or recovering tokens sent directly to a vault token account

TVL is computed from recorded deposit and withdrawal rows. `GET /vault/tvl` currently reports `total_vaults` as `0`.

## Architecture

- **Anchor program** (`programs/collateral-vault`): on-chain vault state and token moves
- **Vault PDA**: `["vault", user_pubkey]`
- **SPL token account**: ATA owned by the vault PDA
- **VaultAuthority**: singleton PDA `["vault_authority"]` with `admin` and `authorized_programs`
- **Authorized calling programs**: listed at VaultAuthority init
- **CPI authority PDA**: `["cpi_authority"]`, derived for the calling program
- **Rust backend** (`backend/`): builds and submits initialize / deposit / withdraw transactions
- **PostgreSQL**: transactions, vault metadata, and unused snapshot / audit tables

Collateral stays in the vault ATA. Lock and unlock only change vault accounting.

## Security model

### Vault ownership

Only the vault owner can withdraw available collateral.

### CPI authorization

`lock_collateral`, `unlock_collateral`, and `transfer_collateral` are meant to be called by another program.

The vault checks all of these:

- `caller_program` is executable
- `caller_program` is in `authorized_programs`
- `cpi_authority` is a signer
- `cpi_authority` is the PDA `["cpi_authority"]` for `caller_program`

Only that program can make the PDA a signer with `invoke_signed`. Passing an authorized program id is not enough, because anyone can include that account in a transaction.

### Authority initialization

`VaultAuthority` is a singleton. `initialize_vault_authority` can run once.

The signer must be this program's BPF upgrade authority. The instruction checks the program account and its ProgramData account:

```
program.programdata_address() == program_data.key()
program_data.upgrade_authority_address == Some(upgrade_authority.key())
```

An arbitrary first caller cannot initialize it. The upgrade authority is stored as `vault_authority.admin`. `admin` is a stored pubkey only. There is no instruction that uses it after init, and the allowlist cannot be changed.

Local tests set `[test] upgradeable = true` in `Anchor.toml` so the provider wallet is the upgrade authority.

### Authorized program limit

`MAX_AUTHORIZED_PROGRAMS` is `10`. The same constant is used for account size and for init checks. Duplicate program ids are rejected.

### Accounting

Internal accounting:

```
total_balance == available_balance + locked_balance
```

Solvency:

```
vault_token_account.amount >= total_balance
```

Deposit, withdraw, lock, unlock, and transfer normally keep:

```
vault_token_account.amount == total_balance
```

Anyone can send SPL tokens directly to a vault token account. That surplus does not increase `total_balance` or `available_balance`, and it cannot be withdrawn through the vault. There is no sync instruction.

`total_balance` is accounted collateral, not the raw token-account amount.

### Locked collateral

Locked collateral cannot be withdrawn. Withdraw uses `available_balance` only.

### Checked arithmetic

Balance updates use `checked_add` and `checked_sub`.

Token transfers out of a vault are signed by the vault PDA:

```
seeds = ["vault", owner, bump]
```

## Known limitations

- An authorized program can lock, unlock, or transfer any vault.
- Those calls do not require the vault owner's signature.
- Transfer does not require the destination owner.
- The authorized program list cannot be changed after initialization.
- An immutable deploy with no upgrade authority cannot initialize VaultAuthority.
- Unsolicited SPL token surplus is not credited and has no recovery instruction.
- `programs/test-caller` is for tests only. Do not allowlist it in production.

## Program instructions

- `initialize_vault_authority` — create the singleton VaultAuthority. Signer must be the program upgrade authority.
- `initialize_vault` — create a user vault and its ATA.
- `deposit` — move tokens from the user ATA into the vault ATA and credit available balance.
- `withdraw` — owner-only. Move available tokens from the vault ATA to the user ATA.
- `lock_collateral` — authorized CPI. Move amount from available to locked.
- `unlock_collateral` — authorized CPI. Move amount from locked to available.
- `transfer_collateral` — authorized CPI. Move available tokens and accounting from one vault to another.

Amounts must be greater than zero.

## Testing

Program tests live in `tests/`. They cover:

- Authority bootstrap
- Unauthorized initialization
- Duplicate and oversized authorized program lists
- Authenticated CPI
- Direct caller impersonation
- Lock / unlock / transfer authorization
- Locked collateral withdrawal protection
- Zero amounts
- Insufficient balances
- Internal accounting invariant
- SPL token reconciliation on normal flows
- Unsolicited direct token transfers

`programs/test-caller` exists only to test authenticated CPI calls. It signs `["cpi_authority"]` and forwards lock, unlock, and transfer.

## Build and toolchain

Pinned versions:

- Anchor `0.32.1` (`Anchor.toml`)
- `anchor-lang` / `anchor-spl` `0.32.1`
- `@coral-xyz/anchor` `^0.32.1`
- Rust toolchain: nightly (`rust-toolchain.toml`)

Useful commands:

```bash
yarn install

cargo fmt --check
cargo clippy -p collateral-vault -p test-caller -- -D warnings
cargo build -p collateral-vault -p test-caller

anchor build
anchor test --skip-build
```

If `anchor` on PATH is not 0.32.1, use `~/.avm/bin/anchor-0.32.1`.

Local tests need `[test] upgradeable = true`. Without it, VaultAuthority init fails because ProgramData has no upgrade authority.

## Setup

Needs Rust, Solana CLI, Anchor 0.32.1, Node.js 18+, Yarn, and PostgreSQL if you run the backend.

```bash
yarn install
anchor build
```

Database:

```bash
createdb collateral_vault
psql collateral_vault < backend/migrations/001_initial_schema.sql
```

Environment (backend):

```bash
DATABASE_URL=postgresql://localhost/collateral_vault
RPC_URL=http://localhost:8899
PROGRAM_ID=8vjbjPhoD2rav71J8mgbVxcYdbbqST78y2bzMPRqoGr9
USDT_MINT=<your mint>
WALLET_PATH=~/.config/solana/id.json
```

`USDT_MINT` should be your test mint on localnet. The backend binds `0.0.0.0:3000`.

Deploy:

```bash
anchor build
anchor deploy
```

After deploy, call `initialize_vault_authority` with the program upgrade authority. Pass the program account and its ProgramData account. The list of authorized programs is fixed at that point.

## Backend / API

Implemented routes:

| Method | Path | Notes |
| --- | --- | --- |
| POST | `/vault/initialize` | `{ "user": "<pubkey>" }` |
| POST | `/vault/deposit` | `{ "user": "<pubkey>", "amount": <u64> }` |
| POST | `/vault/withdraw` | `{ "user": "<pubkey>", "amount": <u64> }` |
| GET | `/vault/balance/:user` | On-chain vault state |
| GET | `/vault/transactions/:user` | Last 100 rows from PostgreSQL |
| GET | `/vault/tvl` | Sum of recorded deposits minus withdrawals. `total_vaults` is always `0`. |

The backend signs with the wallet in `WALLET_PATH` or `~/.config/solana/id.json`. For local use, `user` must be that wallet. The API does not accept user-signed transactions.

```bash
cd backend
cargo run
```

`./test-all-apis.sh` hits the routes above. Set `ADMIN_PUBKEY` to the backend wallet.

## Local program test flow

```bash
solana-test-validator
solana config set --url localhost
anchor build
anchor test --skip-local-validator
```

Or let Anchor start a validator:

```bash
anchor test
```
