# Research: Internal Payment Relay

**Feature**: 001-payment-relay | **Date**: 2026-07-18

## 1. Primary Payment Gateway

**Decision**: pawaPay Merchant API v2 as the default gateway adapter; trait-based adapter for
future Stripe/Adyen.

**Rationale**: Repository includes `pawaPay Merchant API V2.postman_collection.json` with
deposits and payouts endpoints. pawaPay supports mobile-money disbursements across African
markets, matching wallet-debit payment flows in the README. The API exposes idempotent
`payoutId`/`depositId` fields and explicit `DUPLICATE_IGNORED` status aligned with relay
idempotency requirements.

**Alternatives considered**:
- **Stripe only**: Better card coverage but no evidence in repo assets; deferred to adapter #2.
- **Direct multi-gateway routing**: Adds routing complexity; v1 uses `FALLBACK_GATEWAY` env default.

## 2. Web Framework & Runtime

**Decision**: axum 0.7 on tokio full runtime.

**Rationale**: README already specifies axum; mature ecosystem, native async, excellent
middleware/tower integration for auth and tracing. Single-process deployment fits internal relay.

**Alternatives considered**:
- **actix-web**: Comparable performance; less alignment with existing README scaffold.
- **Go/Fiber**: Would abandon Rust choice already documented.

## 3. Database & Migrations

**Decision**: PostgreSQL 15+ with sqlx compile-time checked queries and versioned SQL migrations.

**Rationale**: README schema uses UUID, TIMESTAMPTZ, UNIQUE constraints for idempotency and
wallet scoping. sqlx fits Rust stack; migrations satisfy constitution versioning requirement.

**Alternatives considered**:
- **SQLite**: Simpler local dev but weaker concurrent write semantics for payment ledger.
- **Diesel ORM**: Heavier abstraction; sqlx keeps SQL explicit for financial tables.

## 4. API Key Storage

**Decision**: Generate `sk_live_<random>` keys; store SHA-256 hash only; return plaintext once at
registration.

**Rationale**: Constitution requires hashed-at-rest keys. SHA-256 with constant-time compare is
sufficient for high-entropy API keys (not passwords). bcrypt adds latency on every request without
 proportional benefit for random 256-bit tokens.

**Alternatives considered**:
- **Plaintext storage**: Rejected — violates constitution.
- **JWT session tokens**: Adds rotation complexity; API keys simpler for server-to-server internal
  systems.

## 5. Idempotency Strategy

**Decision**: Global unique constraint on `idempotency_key`; on conflict return stored transaction
if request body hash matches, else 409 Conflict.

**Rationale**: FR-006 requires no duplicate charges. pawaPay uses client-supplied UUIDs
(`payoutId`) — map relay idempotency key to gateway ID. Body-hash comparison resolves edge case
where same key, different payload.

**Alternatives considered**:
- **Per-system scoped idempotency keys**: Allowed via composite unique (system_id, key) — adopted
  in schema for clearer isolation.
- **Redis dedup cache**: Extra infra; Postgres constraint is source of truth.

## 6. Wallet Model

**Decision**: Pre-funded wallet per (system_id, country, currency); debit on successful payout;
auto-create on first payment attempt.

**Rationale**: README handlers check balance before gateway call. Wallets represent allocated
float per market, not live gateway balance.

**Alternatives considered**:
- **No wallet ledger (pass-through)**: Rejected — spec FR-008/FR-009 require wallet tracking.
- **Double-entry accounting v1**: Over-engineered for internal relay; single balance column suffices.

## 7. Gateway Retry Policy

**Decision**: Up to 3 attempts with exponential backoff (100ms, 400ms, 1600ms) on transient
errors: HTTP 5xx, timeout, pawaPay `PROVIDER_TEMPORARILY_UNAVAILABLE`.

**Rationale**: Constitution minimum 3 retries. Non-retryable codes (invalid phone, amount bounds)
fail immediately.

**Alternatives considered**:
- **Background job queue**: Better for long outages; v1 inline retry keeps architecture simple.
- **Infinite retry**: Risk of duplicate gateway calls without idempotency mapping.

## 8. Webhook Delivery

**Decision**: Async fire-and-forget with 3 delivery attempts; HMAC-SHA256 signature header
`X-Relay-Signature`; persist attempts in `webhook_delivery_attempts`.

**Rationale**: SC-003 targets 95% delivery within 30s. Audit table satisfies observability
principle. Signing satisfies constitution webhook verification requirement.

**Alternatives considered**:
- **Synchronous webhook in payment handler**: Blocks response; rejected.
- **No signing v1**: Rejected — constitution requires verifiable payloads.

## 9. External ID Format

**Decision**: Accept client-supplied `external_id`; optionally validate against pattern
`{PREFIX}_{SYSTEM_SHORT}_{YYYYMMDD}_{ALPHANUM6}`; relay-generated IDs use same format when
 omitted.

**Rationale**: FR-012 specifies format for correlation. Validation is warn-only in v1 to avoid
 breaking caller-supplied IDs.

**Alternatives considered**:
- **Server-only ID generation**: Stricter but breaks callers already using own order IDs.

## 10. Ambiguous Gateway Timeout

**Decision**: Persist transaction as `pending`; poll pawaPay status endpoint (or reconcile job)
before marking failed; never double-charge on retry.

**Rationale**: Edge case from spec. Idempotent gateway ID + pending state prevents duplicate
payouts.

**Alternatives considered**:
- **Immediate failure on timeout**: Risk false negatives when gateway actually succeeded.

## 11. Testing Approach

**Decision**: Integration tests against docker-compose Postgres; mock `Gateway` trait for unit
tests; optional wiremock for pawaPay HTTP in integration suite.

**Rationale**: Constitution requires integration tests for payment flows, idempotency, wallets,
webhooks.

**Alternatives considered**:
- **E2E against live pawaPay sandbox**: Valuable for manual quickstart; too flaky for CI default.
