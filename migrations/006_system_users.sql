-- Dashboard login users (one or more per tenant system)
CREATE TABLE system_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    system_id UUID NOT NULL REFERENCES systems(id) ON DELETE CASCADE,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_system_users_system_id ON system_users(system_id);

CREATE TABLE system_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    system_user_id UUID NOT NULL REFERENCES system_users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_system_sessions_token_hash ON system_sessions(token_hash);
CREATE INDEX idx_system_sessions_expires_at ON system_sessions(expires_at);
