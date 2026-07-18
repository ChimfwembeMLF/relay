CREATE TABLE wallet_seed_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    system_id UUID NOT NULL REFERENCES systems(id) ON DELETE CASCADE,
    wallet_id UUID NOT NULL REFERENCES wallets(id) ON DELETE CASCADE,
    country TEXT NOT NULL,
    currency TEXT NOT NULL,
    amount BIGINT NOT NULL,
    source TEXT NOT NULL CHECK (source IN ('default', 'override', 'manual')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_wallet_seed_events_system_id ON wallet_seed_events(system_id, created_at DESC);

CREATE TABLE invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    system_id UUID NOT NULL REFERENCES systems(id) ON DELETE CASCADE,
    reference TEXT NOT NULL UNIQUE,
    description TEXT,
    amount BIGINT NOT NULL CHECK (amount > 0),
    currency TEXT NOT NULL,
    country TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('open', 'paid', 'expired', 'cancelled')),
    expires_at TIMESTAMPTZ NOT NULL,
    paid_at TIMESTAMPTZ,
    transaction_id UUID REFERENCES transactions(id),
    qr_payload_url TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_invoices_system_status ON invoices(system_id, status, created_at DESC);
CREATE INDEX idx_invoices_system_created ON invoices(system_id, created_at DESC);

ALTER TABLE transactions ADD COLUMN invoice_id UUID REFERENCES invoices(id);
ALTER TABLE transactions ADD COLUMN direction TEXT NOT NULL DEFAULT 'payout';

CREATE INDEX idx_transactions_invoice_id ON transactions(invoice_id) WHERE invoice_id IS NOT NULL;
CREATE INDEX idx_transactions_direction ON transactions(system_id, direction, created_at DESC);
