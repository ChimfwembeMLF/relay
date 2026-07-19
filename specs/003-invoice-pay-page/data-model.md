# Data Model: Invoice Pay Page

**Feature**: 003-invoice-pay-page | **Date**: 2026-07-18

## Existing Entities (unchanged schema, extended usage)

### Invoice

| Field | Usage in pay page |
|-------|-------------------|
| `reference` | URL path param; globally unique lookup key |
| `amount`, `currency`, `country` | Display on pay page |
| `description` | Display (optional) |
| `status` | Controls which HTML template renders |
| `expires_at` | Display + validation on POST |
| `transaction_id` | Shown on paid confirmation |

**State transitions** (unchanged): `open` → `paid` | `expired` | `cancelled`

### Transaction

Created on successful pay-page deposit with `direction=deposit`, `invoice_id` set.

### WebhookDeliveryAttempt

Reuse existing table; `event_type` column added or payload distinguishes `payment.completed` vs
`invoice.paid`.

## Schema Changes

### Migration `003_pay_page.sql`

```sql
-- Ensure reference is globally unique (may already be unique per system)
CREATE UNIQUE INDEX IF NOT EXISTS idx_invoices_reference_global ON invoices (reference);

-- Optional: distinguish webhook event types
ALTER TABLE webhook_delivery_attempts
  ADD COLUMN IF NOT EXISTS event_type TEXT NOT NULL DEFAULT 'payment.completed';
```

## Validation Rules

| Rule | Where enforced |
|------|----------------|
| Reference format `INV_*` or UUID | Pay page lookup |
| POST only when `status = open` and not expired | Pay handler |
| Phone number E.164-ish for country | Form validation |
| Idempotency key required on POST | Hidden field, max 128 chars |
| Form token HMAC valid | POST handler |

## Relationships

```text
System 1──* Invoice
Invoice 0──1 Transaction (when paid)
Transaction *──1 Wallet
Pay Page ──lookup──> Invoice ──collect──> Transaction
Invoice paid ──webhook──> System.webhook_url
```

## No New Entities

Pay page is presentation + routing layer over existing invoice/deposit model. No `PaySession`
table in v1 — idempotency via standard transaction idempotency key.
