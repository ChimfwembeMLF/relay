# Research: 006 Batch Payouts, Refunds & Full-Country Registration

**Date**: 2026-07-23

## R1 — Registration enables full catalog (not Zambia-only)

**Decision**: Public `POST /systems` sets `enabled_countries` to **all catalog ISO-2 codes** via `catalog::register_enabled_countries()` (or equivalent). Client-supplied lists are ignored. Seed wallets for **every** `(country, currency)` in the catalog (DRC → CDF and USD). Default seed amounts remain `0` unless `wallet_seeds` overrides.

**Rationale**: Spec US1 reverses 005 Zambia-only default; catalog is the single source of truth.

**Alternatives considered**:
- Keep Zambia default + admin enable — rejected (user asked for all countries on registration).
- Enable countries without seeding all wallets — rejected (Overview/forms need wallets).

## R2 — Multi-currency seed defaults

**Decision**: Expand `config/wallet_seed_defaults.json` to list every catalog currency keyed by ISO-2, with DRC as either dual keys or structured multi-currency entries matching whatever `seed/config.rs` already supports — **extend seed resolver** so one country can yield multiple wallets when the catalog lists multiple currencies.

**Rationale**: Current seed is 1:1 country→single currency; CD requires CDF+USD.

**Alternatives considered**:
- Seed only default currency per country — rejected (breaks DRC USD invoices/payouts).
- Hardcode CD special-case only — weaker than catalog-driven list.

## R3 — Batch processing model

**Decision**: **Synchronous** HTTP request: validate all lines, then process **sequentially**; return full batch result with per-line status. Max **100** lines per batch. Partial success allowed. Insufficient funds fail the current line and continue (or stop subsequent lines for that wallet once balance insufficient — sequential debit naturally enforces this).

**Rationale**: Matches existing single-payout request/response mental model; no job queue in Relay today; SC-002 (≥20 lines) fits sync with gateway mock/sandbox.

**Alternatives considered**:
- Async job + poll — more infra; defer.
- All-or-nothing transaction — rejected by spec (partial success).

## R4 — Batch idempotency

**Decision**: Batch-level `idempotency_key` unique per `system_id`. Store request hash of canonical batch body. Same key + same hash → return stored batch. Same key + different hash → `409 Conflict`. Each successful line creates a normal payout `transactions` row with its own `idempotency_key` derived as `{batch_key}:{line_index}` (or UUID stored on line) so retries never double-debit.

**Rationale**: Constitution III; aligns with existing payment idempotency.

**Alternatives considered**: Rely only on per-line keys from client — harder UX for dashboard paste.

## R5 — Invoice status after refund

**Decision** (clarify recommended default, applied for plan): Keep invoice `status = paid`. Add `refunded_amount` (minor units). Remaining refundable = `amount - refunded_amount`. When remaining is `0`, treat as fully refunded for API/UI (`refundable = false` / `fully_refunded` flag) **without** replacing `paid` as the primary status enum value (optional display badge only).

**Rationale**: Preserves collection fact; partial refunds are amount-driven; minimal enum churn.

**Alternatives considered**:
- `partially_refunded` / `refunded` status enum — more migrations and UI branches.
- Always set status `refunded` on any refund — ambiguous for partials.

## R6 — Persist payer destination for refunds

**Decision**: On successful collect/pay, persist `payer_phone` and `payer_provider` (correspondent) on the **invoice**. Refund defaults to those values; merchant may override if missing or for support exceptions. Insufficient wallet balance → reject refund (`402` / `insufficient_balance`).

**Rationale**: Phone/provider are not stored today; required for FR-008.

**Alternatives considered**:
- Parse from gateway only — not available offline.
- Require merchant always re-enter destination — worse UX when pay-page data exists.

## R7 — Refund money movement

**Decision**: Refund = create a **payout** (debit wallet + gateway `process_payment`) linked via `refund_id` / `invoice_id`. Direction remains `payout` (or add `direction = 'refund'` if cheap — prefer **`payout` + `refund_id` set** to avoid report breakage). Idempotency key required on refund request.

**Rationale**: No PawaPay-specific refund API in current gateway trait; reuse reliable payout path.

**Alternatives considered**:
- Wallet adjustment without gateway — violates “send funds back to customer.”
- New gateway method — unnecessary for v1.

## R8 — Batch UI shape

**Decision**: Extend **Payouts** page with a “Batch” mode: add/remove rows + optional CSV/TSV paste (phone, amount, country, provider label or correspondent). Single payout form remains available (tab or toggle).

**Rationale**: FR-010; avoids new nav clutter unless needed.

**Alternatives considered**: Separate `/batches` route only — fine if toggle feels crowded; plan allows either.

## R9 — Existing systems

**Decision**: No automatic migration of existing `enabled_countries` to full catalog. Admin/manual enable remains out of this feature’s auto path (admin may still set countries via existing admin tools if present).

**Rationale**: Spec assumption; avoids surprise wallet fan-out on legacy tenants.
