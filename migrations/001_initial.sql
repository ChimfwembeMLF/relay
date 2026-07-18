CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE systems (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    prefix TEXT NOT NULL UNIQUE,
    enabled_countries TEXT[] NOT NULL,
    webhook_url TEXT,
    api_key_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE wallets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    system_id UUID NOT NULL REFERENCES systems(id) ON DELETE CASCADE,
    country TEXT NOT NULL,
    currency TEXT NOT NULL,
    balance BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (system_id, country, currency)
);

CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    system_id UUID NOT NULL REFERENCES systems(id),
    wallet_id UUID NOT NULL REFERENCES wallets(id),
    external_id TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    request_hash TEXT NOT NULL,
    amount BIGINT NOT NULL CHECK (amount > 0),
    currency TEXT NOT NULL,
    country TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'completed', 'failed')),
    gateway TEXT NOT NULL,
    gateway_reference TEXT,
    gateway_status TEXT,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (system_id, idempotency_key)
);

CREATE TABLE webhook_delivery_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_id UUID NOT NULL REFERENCES transactions(id),
    attempt_number INT NOT NULL CHECK (attempt_number BETWEEN 1 AND 3),
    url TEXT NOT NULL,
    status_code INT,
    success BOOLEAN NOT NULL,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_transactions_system_id ON transactions(system_id);
CREATE INDEX idx_transactions_external_id ON transactions(system_id, external_id);
CREATE INDEX idx_transactions_gateway_reference ON transactions(gateway_reference) WHERE gateway_reference IS NOT NULL;
CREATE INDEX idx_wallets_system_id ON wallets(system_id);
CREATE INDEX idx_webhook_attempts_transaction_id ON webhook_delivery_attempts(transaction_id);
