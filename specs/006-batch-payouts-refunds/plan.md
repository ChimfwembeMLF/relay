# Implementation Plan: Batch Payouts, Refunds & Full-Country Registration

**Branch**: `006-batch-payouts-refunds` | **Date**: 2026-07-23 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/006-batch-payouts-refunds/spec.md`

## Summary

Enable **all catalog countries** on public registration (still no country picker). Add **batch payouts** (multi-line API + dashboard multi-row/paste) with partial success and batch-level idempotency. Add **invoice refunds** (full/partial) that debit the merchant wallet and payout to the customer via the existing gateway path. Persist payer MSISDN/provider on collect so refunds can reuse the original destination. Invoice status stays **`paid`** with `refunded_amount` tracking (fully refunded when remaining is zero).

## Technical Context

**Language/Version**: Rust (edition 2021, axum) + TypeScript 5.7 / React 19 SPA

**Primary Dependencies**: Existing payment-relay; Vite SPA; shadcn/ui; `frontend/DESIGN.md`; PawaPay gateway (payout + deposit already integrated)

**Storage**: PostgreSQL — new `payout_batches` / `payout_batch_lines` tables; invoice columns for `refunded_amount`, payer `phone`/`provider`; optional `batch_id` / `refund_id` on transactions; expand `config/wallet_seed_defaults.json` (or catalog-driven seed) for all catalog currencies

**Testing**: `cargo test` (seed register, batch, refund, pay_page); frontend `npm run build`; quickstart manual checklist

**Target Platform**: Relay API + SPA from `frontend/dist`

**Project Type**: Web application (Rust API + React SPA)

**Performance Goals**: Synchronous batch processing up to **100 lines** per request; sequential per-line gateway calls; operator can submit ≥20 lines in under 5 minutes (excl. gateway latency)

**Constraints**: Constitution idempotency (batch key + refund key); system isolation; amounts in minor units; no new SaaS surfaces; existing single `/payments` unchanged; existing systems not auto-migrated to full catalog

**Scale/Scope**: ~12 countries, dual-currency CD; 1 batch API + UI; 1 refund API + invoice detail UI; registration seed expansion

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Internal-First Simplicity | PASS | Merchant/API tools only; no multi-tenant SaaS billing |
| II. System Isolation | PASS | Batches/refunds scoped by authenticated system; wallets per system |
| III. Idempotent Payment Processing | PASS | Batch idempotency key + per-refund idempotency; reuse payout debit path |
| IV. Reliable External Relay | PASS | Refunds/batch lines use existing gateway payout + retries |
| V. Observability & Auditability | PASS | Batch/refund IDs linked on transactions; structured logs |
| Security & Compliance | PASS | Same auth (API key / session); no card data; amounts integer minor units |
| Spec-driven workflow | PASS | Artifacts under `specs/006-batch-payouts-refunds/` |

**Gate result**: PASS — proceed to Phase 0/1.

### Post-design Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I–V + Security | PASS | Contracts define batch/refund APIs; ledger remains debit-on-success payout |
| Spec-driven | PASS | research / data-model / contracts / quickstart produced |

**Gate result (post Phase 1)**: PASS.

## Project Structure

### Documentation (this feature)

```text
specs/006-batch-payouts-refunds/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── batch-payouts.md
│   └── invoice-refunds.md
└── tasks.md              # (/speckit-tasks — not created here)
```

### Source Code (repository root)

```text
migrations/007_batches_and_refunds.sql
config/wallet_seed_defaults.json          # All catalog country/currency seeds (amount 0)
src/catalog/mod.rs                        # all_iso2() / register_enabled_countries()
src/seed/config.rs                        # Multi-currency seed resolution (CD CDF+USD)
src/api/systems.rs                        # Force full catalog on register
src/api/batches.rs                        # NEW batch create/get
src/api/refunds.rs                        # NEW refund create (or invoices::refund)
src/api/invoices.rs                       # Persist payer on collect; expose refundable fields
src/api/routes.rs / mod.rs
src/db/batches.rs                         # NEW
src/db/invoices.rs                        # refunded_amount, payer fields, refund helpers
src/db/queries.rs                         # Optional batch_id/refund_id on NewTransaction
src/models.rs
openapi/openapi.yaml
tests/seed_on_register_test.rs
tests/batch_payout_test.rs                # NEW
tests/invoice_refund_test.rs              # NEW
frontend/src/api.ts
frontend/src/pages/PaymentsPage.tsx       # Batch multi-row (+ keep single or tab)
frontend/src/pages/InvoiceDetailPage.tsx  # Refund action
frontend/src/main.tsx / AppLayout         # Nav if new route
```

**Structure Decision**: Extend existing Rust monolith + SPA; no new service. Batch and refund modules alongside `payments` / `invoices`.

## Complexity Tracking

> No constitution violations requiring justification.
