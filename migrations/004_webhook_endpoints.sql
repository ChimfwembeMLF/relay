-- Per-tenant webhook endpoints (merchants can add many)
CREATE TABLE webhook_endpoints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    system_id UUID NOT NULL REFERENCES systems(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    label TEXT,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (system_id, url)
);

CREATE INDEX idx_webhook_endpoints_system_id ON webhook_endpoints(system_id);
CREATE INDEX idx_webhook_endpoints_enabled ON webhook_endpoints(system_id) WHERE enabled = TRUE;

-- Migrate legacy single webhook_url into endpoints
INSERT INTO webhook_endpoints (system_id, url, label, enabled)
SELECT id, webhook_url, 'Primary', TRUE
FROM systems
WHERE webhook_url IS NOT NULL AND webhook_url <> ''
ON CONFLICT (system_id, url) DO NOTHING;
