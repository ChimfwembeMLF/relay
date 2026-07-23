# Data Model: 006 Batch Payouts, Refunds & Full-Country Registration

## System (registration change)

| Field | Change |
|-------|--------|
| `enabled_countries` | On public register: set to full catalog ISO-2 list (not client-provided; not Zambia-only) |

Wallets: one row per `(system_id, country, currency)` for every catalog currency (including `CD`/`CDF` and `CD`/`USD`).

## Invoice (extended)

| Field | Type | Notes |
|-------|------|-------|
| existing… | | unchanged (`status`: open/paid/expired/cancelled) |
| `refunded_amount` | `BIGINT NOT NULL DEFAULT 0` | Sum of successful refund payouts |
| `payer_phone` | `TEXT NULL` | Set on successful collect |
| `payer_provider` | `TEXT NULL` | Correspondent code on successful collect |

**Derived**:
- `remaining_refundable = amount - refunded_amount` (when `status = paid`)
- `fully_refunded` when `status = paid` AND `remaining_refundable = 0`

**Rules**:
- Refunds only when `status = paid` and `remaining_refundable > 0`
- Each successful refund increments `refunded_amount` atomically (must not exceed `amount`)
- Status remains `paid` after partial or full refund

## PayoutBatch

| Field | Type | Notes |
|-------|------|-------|
| `id` | UUID PK | |
| `system_id` | UUID FK | |
| `idempotency_key` | TEXT | Unique with system_id |
| `request_hash` | TEXT | Canonical body hash |
| `status` | TEXT | `completed` \| `partial` \| `failed` (aggregate) |
| `line_count` | INT | |
| `success_count` | INT | |
| `failure_count` | INT | |
| `created_at` | TIMESTAMPTZ | |

## PayoutBatchLine

| Field | Type | Notes |
|-------|------|-------|
| `id` | UUID PK | |
| `batch_id` | UUID FK | |
| `line_index` | INT | 0-based order |
| `external_id` | TEXT | Optional client; else generated |
| `amount` | BIGINT | Minor units |
| `currency` | TEXT | |
| `country` | TEXT | ISO-2 |
| `phone` | TEXT | |
| `provider` | TEXT | Correspondent |
| `status` | TEXT | `completed` \| `failed` \| `skipped` |
| `error` | TEXT NULL | |
| `transaction_id` | UUID NULL | FK transactions |
| `line_idempotency_key` | TEXT | e.g. `{batch_key}:{line_index}` |

## Refund

| Field | Type | Notes |
|-------|------|-------|
| `id` | UUID PK | |
| `system_id` | UUID FK | |
| `invoice_id` | UUID FK | |
| `amount` | BIGINT | |
| `currency` / `country` | TEXT | Copied from invoice |
| `phone` / `provider` | TEXT | Destination used |
| `idempotency_key` | TEXT | Unique with system_id |
| `request_hash` | TEXT | |
| `status` | TEXT | `completed` \| `failed` |
| `transaction_id` | UUID NULL | Linked payout tx |
| `error` | TEXT NULL | |
| `created_at` | TIMESTAMPTZ | |

## Transaction (extended)

| Field | Change |
|-------|--------|
| `batch_id` | UUID NULL — set for batch line payouts |
| `refund_id` | UUID NULL — set for refund payouts |
| `direction` | Remains `payout` for batch lines and refunds (deposits unchanged) |

## Relationships

```text
System 1──* PayoutBatch 1──* PayoutBatchLine *──? Transaction
System 1──* Invoice 1──* Refund *──? Transaction
Invoice.refunded_amount ← sum(successful Refund.amount)
```

## Validation rules

- Batch: 1–100 lines; each line catalog-valid; country in system `enabled_countries`
- Refund: `0 < amount ≤ remaining_refundable`; wallet balance ≥ amount; destination phone+provider required
- Idempotency conflicts → 409
