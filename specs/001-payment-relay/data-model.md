# Data Model: Internal Payment Relay

**Feature**: 001-payment-relay | **Date**: 2026-07-18

## Entity Relationship Overview

```text
System 1──* Wallet
System 1──* Transaction
Wallet 1──* Transaction
Transaction 0──* WebhookDeliveryAttempt
```

## System

Registered internal consumer of the relay.

| Field | Type | Constraints | Notes |
|-------|------|-------------|-------|
| id | UUID | PK, default gen_random_uuid() | Public system identifier |
| name | TEXT | NOT NULL | Human-readable label |
| prefix | TEXT | NOT NULL, UNIQUE | 2–8 uppercase alphanumeric; used in external IDs |
| enabled_countries | TEXT[] | NOT NULL | ISO 3166-1 alpha-2 codes |
| webhook_url | TEXT | NULL | HTTPS URL for terminal payment events |
| api_key_hash | TEXT | NOT NULL | SHA-256 hex of API key |
| created_at | TIMESTAMPTZ | NOT NULL, default NOW() | |
| updated_at | TIMESTAMPTZ | NOT NULL, default NOW() | |

**Validation rules**:
- `prefix` must match `^[A-Z0-9]{2,8}$`
- `enabled_countries` must be non-empty
- `webhook_url` if set must be HTTPS

## Wallet

Per-system balance container for a country/currency pair.

| Field | Type | Constraints | Notes |
|-------|------|-------------|-------|
| id | UUID | PK | |
| system_id | UUID | FK → systems(id) ON DELETE CASCADE | |
| country | TEXT | NOT NULL | ISO country code |
| currency | TEXT | NOT NULL | ISO 4217 code |
| balance | BIGINT | NOT NULL, default 0 | Minor units (cents); may be negative only if explicitly allowed — v1 rejects debit below zero |
| created_at | TIMESTAMPTZ | NOT NULL | |
| updated_at | TIMESTAMPTZ | NOT NULL | |

**Unique**: `(system_id, country, currency)`

**Validation rules**:
- `balance` updated atomically with transaction insert in single DB transaction
- Auto-created on first payment for unseen (system_id, country, currency)

## Transaction

Payment attempt and lifecycle record.

| Field | Type | Constraints | Notes |
|-------|------|-------------|-------|
| id | UUID | PK | Relay payment ID |
| system_id | UUID | FK → systems(id) | |
| wallet_id | UUID | FK → wallets(id) | |
| external_id | TEXT | NOT NULL | Caller correlation ID |
| idempotency_key | TEXT | NOT NULL | |
| request_hash | TEXT | NOT NULL | SHA-256 of canonical request body |
| amount | BIGINT | NOT NULL | Minor units; must be > 0 |
| currency | TEXT | NOT NULL | |
| country | TEXT | NOT NULL | |
| status | TEXT | NOT NULL | `pending`, `completed`, `failed` |
| gateway | TEXT | NOT NULL | e.g. `pawapay` |
| gateway_reference | TEXT | NULL | pawaPay payoutId/depositId |
| gateway_status | TEXT | NULL | Raw gateway status string |
| error | TEXT | NULL | Failure message/code summary |
| created_at | TIMESTAMPTZ | NOT NULL | |
| updated_at | TIMESTAMPTZ | NOT NULL | |

**Unique**: `(system_id, idempotency_key)`

**Indexes**:
- `(system_id, external_id)`
- `(gateway_reference)` where not null
- `(system_id, created_at DESC)`

**State transitions**:

```text
                    ┌──────────┐
         create     │ pending  │
        ──────────► │          │
                    └────┬─────┘
                         │
           gateway OK    │    gateway fail (terminal)
              ┌──────────┼──────────┐
              ▼                     ▼
        ┌──────────┐         ┌──────────┐
        │ completed│         │  failed  │
        └──────────┘         └──────────┘
```

- `pending` → `completed`: gateway returns success; wallet debited (if not already)
- `pending` → `failed`: terminal gateway error after retries
- Terminal states are immutable except audit corrections (out of v1 scope)

## WebhookDeliveryAttempt

Audit log for outbound webhooks.

| Field | Type | Constraints | Notes |
|-------|------|-------------|-------|
| id | UUID | PK | |
| transaction_id | UUID | FK → transactions(id) | |
| attempt_number | INT | NOT NULL | 1–3 |
| url | TEXT | NOT NULL | Snapshot of target URL |
| status_code | INT | NULL | HTTP response code |
| success | BOOLEAN | NOT NULL | |
| error | TEXT | NULL | |
| created_at | TIMESTAMPTZ | NOT NULL | |

## PaymentMethod (embedded, not persisted)

Tokenized payment instruction passed in API request only.

| Field | Type | Notes |
|-------|------|-------|
| type | enum string | `mmo`, `card`, `bank` — v1 primary: `mmo` for pawaPay |
| details | JSON object | Gateway-specific; e.g. provider, phoneNumber for MMO |

**Rule**: No raw PAN or CVV fields accepted; reject requests containing them.

## Domain Invariants

1. A transaction's `country` MUST be in parent system's `enabled_countries`.
2. Duplicate `idempotency_key` for same system MUST return existing row if `request_hash` matches.
3. Duplicate key with different `request_hash` MUST return HTTP 409.
4. Wallet debit and transaction insert MUST be one atomic database transaction.
5. Webhook dispatched only on transition to `completed` or `failed`.
6. Amounts MUST be positive integers in minor currency units.

## Migration Notes

Single migration `001_initial.sql` creates all tables, constraints, and indexes. Future migrations
add columns only with backward-compatible defaults per constitution.
