# Research: Wallet Seeding, Invoices & Reports

**Feature**: 002-wallet-invoices-reports | **Date**: 2026-07-18

## 1. Wallet Auto-Seeding on Registration

**Decision**: Seed wallets synchronously inside the `create_system` DB transaction; write
`wallet_seed_events` audit rows; load defaults from `config/wallet_seed_defaults.json` with env
override `WALLET_SEED_DEFAULTS_JSON`.

**Rationale**: FR-001/FR-014 require immediate wallets + audit trail. Synchronous seeding meets
SC-001 (< 5s) for ≤20 countries. Transaction ensures registration rolls back if seed fails entirely.

**Partial failure policy**: If one country lacks currency mapping, skip with logged warning and
record failed seed event; registration still succeeds (documented in edge cases). Operator fixes
config and can POST admin re-seed endpoint (future) or manual top-up.

**Alternatives considered**:
- **Async background job**: Faster HTTP response but violates SC-001 immediate wallet availability.
- **SQL-only manual seed**: Current pain point; rejected.

## 2. Country → Currency Mapping

**Decision**: Static map in `src/seed/config.rs` with JSON file overlay:

```json
{ "ZM": { "currency": "ZMW", "amount": 100000 }, "US": { "currency": "USD", "amount": 10000 } }
```

**Rationale**: Spec assumption; avoids runtime FX logic. Per-registration overrides in
`CreateSystemRequest.wallet_seeds: Option<Vec<WalletSeedOverride>>`.

**Alternatives considered**:
- **External geo-IP service**: Over-engineered for internal systems with explicit enabled countries.

## 3. Invoice Model & Pay-In Flow

**Decision**: Invoices are **pay-in (collection)**. QR encodes:

`{INVOICE_PAY_BASE_URL}/pay/{invoice_reference}`

where `invoice_reference` is a UUID v4 or `INV_{prefix}_{short}` string.

Payment completion uses pawaPay **deposits** (`POST /v2/deposits` per Postman collection), credits
wallet on success, marks invoice `paid`, links `transaction_id`.

**Rationale**: Spec assumes collection QR workflow. Existing payout path debits wallet — wrong
direction for invoice collection.

**Alternatives considered**:
- **Payout with negative amount**: Invalid ledger semantics.
- **External hosted checkout only**: Deferred; v1 QR URL can point to relay-hosted minimal pay page later.

## 4. QR Code Generation

**Decision**: Rust `qrcode` crate → PNG → base64 data URL in API response; also return raw `qr_url`
string for clients that render themselves.

**Rationale**: FR-005 requires scannable QR in API response. PNG base64 works in mobile admin tools
and Postman without extra frontend.

**Alternatives considered**:
- **SVG only**: Less universal in PDF invoices; PNG chosen as primary.
- **Client-side QR generation**: Pushes burden to every consumer; rejected for v1.

## 5. Invoice Lifecycle

**Decision**: States: `open` → `paid` | `expired` | `cancelled`. Cron-less expiry: check
`expires_at` on read and payment attempt; lazy transition to `expired` persisted on first access
after expiry.

**Rationale**: Avoids background scheduler complexity for v1 internal relay.

**Idempotency**: Paying a paid invoice returns linked transaction (FR idempotency alignment).

## 6. Reports Architecture

**Decision**: Three report endpoints under `/reports/{type}` with query params `from`, `to`, `format=json|csv`:

| Type | Summary | Detail |
|------|---------|--------|
| `transactions` | count/volume by status | optional rows |
| `wallets` | current balance + period delta | per wallet row |
| `invoices` | count/amount by status | optional rows |

SQL uses indexed `created_at` filters + `system_id` equality.

**Rationale**: FR-009–FR-013; meets SC-004/SC-005 with single codebase pattern.

**Alternatives considered**:
- **Materialized views**: Premature for v1 volume.
- **Embedded BI (Metabase)**: Violates internal-first simplicity for initial delivery.

## 7. CSV Export

**Decision**: `csv` crate; `Content-Type: text/csv` + `Content-Disposition: attachment` when
`format=csv`. Max 10,000 rows; `413` or paginated `next_cursor` if exceeded.

**Rationale**: FR-013, SC-005.

## 8. Payment API Backward Compatibility

**Decision**: Add optional `invoice_id` to existing payment/deposit request; all existing clients
unchanged. Separate `POST /invoices/{id}/collect` endpoint wraps deposit + invoice mark-paid for
explicit invoice flows.

**Rationale**: FR-015.

## 9. Gateway Trait Extension

**Decision**: Add `process_deposit()` to `PaymentGateway` trait; implement in `pawapay.rs` using
`/v2/deposits` with same retry policy as payouts.

**Rationale**: Constitution IV retry rules apply equally to collection.

## 10. Testing Strategy

**Decision**: Integration tests with `MockGateway` implementing deposit success; seed test registers
system with 2 countries and asserts wallet count/balances; report tests insert fixtures and assert
summary totals + CSV column headers.

**Rationale**: Constitution integration test requirement extended to new flows.
