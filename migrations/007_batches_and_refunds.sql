-- Batch payouts + invoice refunds (006)

ALTER TABLE invoices
    ADD COLUMN IF NOT EXISTS refunded_amount BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS payer_phone TEXT,
    ADD COLUMN IF NOT EXISTS payer_provider TEXT;

CREATE TABLE IF NOT EXISTS payout_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    system_id UUID NOT NULL REFERENCES systems(id) ON DELETE CASCADE,
    idempotency_key TEXT NOT NULL,
    request_hash TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('completed', 'partial', 'failed')),
    line_count INT NOT NULL,
    success_count INT NOT NULL DEFAULT 0,
    failure_count INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (system_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_payout_batches_system ON payout_batches(system_id, created_at DESC);

CREATE TABLE IF NOT EXISTS payout_batch_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    batch_id UUID NOT NULL REFERENCES payout_batches(id) ON DELETE CASCADE,
    line_index INT NOT NULL,
    external_id TEXT NOT NULL,
    amount BIGINT NOT NULL CHECK (amount > 0),
    currency TEXT NOT NULL,
    country TEXT NOT NULL,
    phone TEXT NOT NULL,
    provider TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('completed', 'failed', 'skipped')),
    error TEXT,
    transaction_id UUID REFERENCES transactions(id),
    line_idempotency_key TEXT NOT NULL,
    UNIQUE (batch_id, line_index)
);

CREATE INDEX IF NOT EXISTS idx_payout_batch_lines_batch ON payout_batch_lines(batch_id);

CREATE TABLE IF NOT EXISTS refunds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    system_id UUID NOT NULL REFERENCES systems(id) ON DELETE CASCADE,
    invoice_id UUID NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    amount BIGINT NOT NULL CHECK (amount > 0),
    currency TEXT NOT NULL,
    country TEXT NOT NULL,
    phone TEXT NOT NULL,
    provider TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    request_hash TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('completed', 'failed')),
    transaction_id UUID REFERENCES transactions(id),
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (system_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_refunds_invoice ON refunds(invoice_id, created_at DESC);

ALTER TABLE transactions
    ADD COLUMN IF NOT EXISTS batch_id UUID REFERENCES payout_batches(id),
    ADD COLUMN IF NOT EXISTS refund_id UUID REFERENCES refunds(id);
