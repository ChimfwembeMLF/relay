# Data Model: Wallet Seeding, Invoices & Reports

**Feature**: 002-wallet-invoices-reports | **Date**: 2026-07-18

## New & Extended Entities

### WalletSeedConfig (runtime, not persisted)

Operator configuration merged from file + env.

| Field | Type | Notes |
|-------|------|-------|
| defaults | Map country → { currency, amount } | Minor units |
| overrides | Per registration request | Optional |

### WalletSeedEvent (new table)

Audit trail for initial wallet funding.

| Field | Type | Constraints |
|-------|------|-------------|
| id | UUID | PK |
| system_id | UUID | FK → systems(id) |
| wallet_id | UUID | FK → wallets(id) |
| country | TEXT | NOT NULL |
| currency | TEXT | NOT NULL |
| amount | BIGINT | NOT NULL — seeded amount |
| source | TEXT | NOT NULL | `default`, `override`, `manual` |
| created_at | TIMESTAMPTZ | NOT NULL |

**Index**: `(system_id, created_at DESC)`

### Invoice (new table)

| Field | Type | Constraints |
|-------|------|-------------|
| id | UUID | PK |
| system_id | UUID | FK → systems(id) |
| reference | TEXT | NOT NULL, UNIQUE globally |
| description | TEXT | NULL |
| amount | BIGINT | NOT NULL, > 0 |
| currency | TEXT | NOT NULL |
| country | TEXT | NOT NULL |
| status | TEXT | NOT NULL | `open`, `paid`, `expired`, `cancelled` |
| expires_at | TIMESTAMPTZ | NOT NULL |
| paid_at | TIMESTAMPTZ | NULL |
| transaction_id | UUID | NULL FK → transactions(id) |
| qr_payload_url | TEXT | NOT NULL — encoded payment URL |
| created_at | TIMESTAMPTZ | NOT NULL |
| updated_at | TIMESTAMPTZ | NOT NULL |

**Indexes**:
- `(system_id, status, created_at DESC)`
- `(reference)` UNIQUE
- `(system_id, created_at DESC)`

**State transitions**:

```text
open ──pay success──► paid
open ──expires──────► expired
open ──cancel───────► cancelled
paid / expired / cancelled → terminal (immutable)
```

### Transaction (extended)

| New field | Type | Notes |
|-----------|------|-------|
| invoice_id | UUID | NULL FK → invoices(id) |
| direction | TEXT | `payout` (default) or `deposit` |

Existing payout rows default `direction = 'payout'`, `invoice_id = NULL`.

### Report DTOs (API only, not persisted)

**ReportSummary**: `{ total_count, total_amount, by_status: { status → { count, amount } } }`

**ReportDetailRow**: Type-specific flat map serialized to JSON/CSV columns.

## Registration Flow (updated)

```text
POST /systems
  1. Insert system
  2. For each enabled_country:
       resolve currency + amount (override > default > 0)
       INSERT wallet
       INSERT wallet_seed_event
  3. Commit
  4. Return system + api_key (wallets queryable immediately)
```

**Invariant**: Seed and system insert share one transaction.

## Invoice Collection Flow

```text
POST /invoices → open invoice + QR
POST /invoices/{id}/collect (or payment with invoice_id)
  1. Validate invoice open + not expired + amount match
  2. gateway.process_deposit()
  3. INSERT transaction (direction=deposit)
  4. CREDIT wallet balance
  5. UPDATE invoice → paid, link transaction_id
```

## Report Queries

**Transactions report**:
- Filter: `system_id`, `created_at BETWEEN from AND to`, optional `status`
- Summary: `GROUP BY status` → count, sum(amount)
- Detail: ordered rows limit 10k

**Wallets report**:
- Current balances from `wallets`
- Period delta: sum deposits − sum payouts per wallet in date range

**Invoices report**:
- Filter by `created_at` range + optional `status`
- Summary by status; detail rows optional

## Migration

`002_invoices_and_seed_events.sql`:
- CREATE `invoices`, `wallet_seed_events`
- ALTER `transactions` ADD `invoice_id`, ADD `direction` DEFAULT `'payout'`

## Validation Rules

- Invoice amount must match collect request amount exactly (no partial pay v1)
- Seed amount ≥ 0; zero allowed (wallet exists, empty)
- Report `from` must be ≤ `to`; max range 366 days v1
- Cancel only from `open` state
