-- Create transaction type enum
CREATE TYPE transaction_type AS ENUM ('deposit', 'withdrawal', 'lock', 'unlock', 'transfer');

-- Transactions table
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vault VARCHAR(44) NOT NULL,
    transaction_type transaction_type NOT NULL,
    amount BIGINT NOT NULL,
    signature VARCHAR(88),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_transactions_vault ON transactions(vault);
CREATE INDEX idx_transactions_timestamp ON transactions(timestamp DESC);
CREATE INDEX idx_transactions_type ON transactions(transaction_type);

-- Balance snapshots table
CREATE TABLE balance_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vault VARCHAR(44) NOT NULL,
    total_balance BIGINT NOT NULL,
    locked_balance BIGINT NOT NULL,
    available_balance BIGINT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_balance_snapshots_vault ON balance_snapshots(vault);
CREATE INDEX idx_balance_snapshots_timestamp ON balance_snapshots(timestamp DESC);

-- Vaults table (for tracking vault metadata)
CREATE TABLE vaults (
    owner VARCHAR(44) PRIMARY KEY,
    vault VARCHAR(44) NOT NULL UNIQUE,
    token_account VARCHAR(44) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_vaults_vault ON vaults(vault);

-- Reconciliation logs table
CREATE TABLE reconciliation_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vault VARCHAR(44) NOT NULL,
    on_chain_balance BIGINT NOT NULL,
    database_balance BIGINT NOT NULL,
    discrepancy BIGINT NOT NULL,
    resolved BOOLEAN DEFAULT FALSE,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_reconciliation_logs_vault ON reconciliation_logs(vault);
CREATE INDEX idx_reconciliation_logs_resolved ON reconciliation_logs(resolved);

-- Audit trail table
CREATE TABLE audit_trail (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vault VARCHAR(44),
    action VARCHAR(50) NOT NULL,
    details JSONB,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_trail_vault ON audit_trail(vault);
CREATE INDEX idx_audit_trail_timestamp ON audit_trail(timestamp DESC);

