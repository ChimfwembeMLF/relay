# Implementation Plan: Wallet Seeding, Invoices & Reports

**Branch**: `002-wallet-invoices-reports` | **Date**: 2026-07-18 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/002-wallet-invoices-reports/spec.md`

**Note**: Extends the existing `001-payment-relay` Rust service in-place (same binary, new migration).

## Summary

Extend payment-relay with: (1) automatic wallet seeding on system registration driven by operator
config and optional per-request overrides; (2) pay-in invoices with QR-encoded payment URLs and
lifecycle tracking linked to pawaPay **deposits**; (3) JSON + CSV reports for transactions, wallets,
and invoices. All endpoints remain API-key scoped per system.

## Technical Context

**Language/Version**: Rust 1.75+ (extends existing codebase)

**Primary Dependencies**: Existing stack plus `qrcode` (QR PNG/SVG), `image` (QR raster), `csv`
(CSV export), `rust_decimal` not needed (keep i64 minor units)

**Storage**: PostgreSQL 15+ — new tables: `invoices`, `wallet_seed_events`; migration `002_*`;
extend `systems` registration payload (no breaking change to response shape)

**Testing**: cargo test + integration tests for seed-on-register, invoice lifecycle, report queries,
CSV export parity; mock deposit gateway trait extension

**Target Platform**: Same as 001 — Linux/Docker, local Homebrew Postgres

**Project Type**: web-service extension (REST only, no dashboard UI)

**Performance Goals**: Registration + seed < 5s for up to 20 countries; invoice create < 500ms;
reports < 3s for 10k transaction rows (SC-004)

**Constraints**: Integer minor units; invoice full-payment only; QR v1 = HTTPS URL deep link;
reports max 10k detail rows per request with pagination

**Scale/Scope**: 3 new route groups (invoices, reports, seed config read), 1 migration, extend
`create_system` + optional `invoice_id` on payments/deposits path

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Pre-Design | Post-Design |
|-----------|------------|-------------|
| I. Internal-First Simplicity | PASS — API-only, no SaaS UI | PASS — reports JSON/CSV only; seed via config |
| II. System Isolation | PASS | PASS — all queries scoped by `system_id` + API key |
| III. Idempotent Payments | PASS | PASS — invoice pay uses idempotency key; paid invoice replay returns linked tx |
| IV. Reliable External Relay | PASS | PASS — deposits use same 3-retry adapter pattern as payouts |
| V. Observability | PASS | PASS — wallet_seed_events audit + structured invoice/report logs |
| Security | PASS | PASS — QR URLs use unguessable refs; no secrets in QR payload |

**Gate result**: PASS

**Complexity note**: Introduces **deposit (pay-in)** path alongside existing **payout (pay-out)** path.
Justified because invoices are collection workflows; constitution simplicity preserved by reusing
gateway trait with second method rather than a new service.

## Project Structure

### Documentation (this feature)

```text
specs/002-wallet-invoices-reports/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── openapi.yaml
│   └── invoice-qr-payload.md
└── tasks.md                    # /speckit-tasks
```

### Source Code (additions to repo root)

```text
src/
├── config.rs                   # + wallet seed defaults, invoice base URL
├── models.rs                   # + Invoice, WalletSeedEvent, report DTOs
├── db/queries.rs               # + seed, invoice, report queries
├── api/
│   ├── systems.rs              # hook wallet seed on register
│   ├── invoices.rs             # NEW
│   ├── reports.rs              # NEW
│   └── payments.rs             # + optional invoice_id, deposit path
├── gateway/
│   ├── traits.rs               # + process_deposit()
│   └── pawapay.rs              # + POST /v2/deposits
├── seed/
│   ├── mod.rs                  # NEW — seed orchestration
│   └── config.rs               # NEW — country→currency→amount map
└── qr/
    └── mod.rs                  # NEW — QR PNG base64 generation
migrations/
└── 002_invoices_and_seed_events.sql
config/
└── wallet_seed_defaults.json   # operator default balances
tests/
├── seed_on_register_test.rs
├── invoices_test.rs
└── reports_test.rs
```

**Structure Decision**: Extend single binary; `seed/` and `qr/` modules keep registration and invoice
concerns isolated without new crates.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Second gateway operation (deposit) | Invoices are pay-in collection | Reusing payout API would debit wallet instead of collecting funds |
| CSV export endpoint | Finance export requirement (FR-013) | JSON-only fails SC-005 and user request |
