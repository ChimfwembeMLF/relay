# Research: Invoice Pay Page

**Feature**: 003-invoice-pay-page | **Date**: 2026-07-18

## 1. Public Pay Page Auth Model

**Decision**: No API key on `/pay/{reference}`. The invoice `reference` acts as an unguessable
capability token (`INV_{prefix}_{8-char-uuid}` ≈ 128-bit entropy in suffix).

**Rationale**: Payers are external; merchant API keys must not appear in browsers. Reference
uniqueness is enforced at DB level.

**Alternatives considered**:
- **Signed JWT in QR URL**: More complex QR payload; rejected for v1.
- **Short numeric PIN**: Easier to brute-force; rejected.

## 2. HTML Rendering Approach

**Decision**: Server-rendered HTML via Rust `include_str!` templates with minimal placeholder
substitution (`{{amount}}`, `{{reference}}`, etc.). No Askama/Tera dependency for v1.

**Rationale**: Constitution internal-first simplicity; one static template per page state (open,
paid, expired, error). Keeps dependency tree unchanged.

**Alternatives considered**:
- **Askama templates**: Better ergonomics but new compile-time dependency; defer if templates grow.
- **JSON + client SPA**: Violates simplicity; rejected.

## 3. Pay Page Collect Flow

**Decision**: `POST /pay/{reference}` handler internally reuses existing deposit+credit logic
(`create_deposit_with_credit`, `mark_invoice_paid`). Idempotency key = UUID generated per page
load, stored in hidden form field.

**Rationale**: DRY with API collect path; single ledger semantics. Per-page UUID prevents
double-submit on refresh while allowing intentional retry with new page load.

**Alternatives considered**:
- **Public API collect without auth**: Would expose `/invoices/{id}/collect`; rejected — ID is
  less opaque than reference and requires API key today.

## 4. Invoice Lookup by Reference (Global)

**Decision**: Add `get_invoice_by_reference_public(pool, reference)` query without `system_id`
filter; references are globally unique (add UNIQUE constraint if not already present).

**Rationale**: Pay page has no API key to determine system scope; reference must be globally unique.

## 5. Invoice Paid Webhooks

**Decision**: New webhook event type `invoice.paid` with payload mirroring payout webhook structure
plus `invoice_reference`, `invoice_id`. Reuse `webhook::sender` with extended payload builder.

**Rationale**: FR-007/FR-008; same delivery/retry policy as payments (3 attempts, logged in
`webhook_delivery_attempts`).

## 6. Atomic Registration + Seed

**Decision**: Wrap `create_system` + all `seed_system_wallets` inserts in one `sqlx` transaction
using `pool.begin()` in `create_system` handler.

**Rationale**: FR-009; aligns with 002 research intent that was not fully implemented.

## 7. Rate Limiting

**Decision**: In-memory sliding window per IP on `POST /pay/{reference}` only (10 req/min default,
configurable via `PAY_PAGE_RATE_LIMIT` env). Use existing tower middleware pattern.

**Rationale**: FR-010; prevents brute-force reference guessing combined with POST spam. In-memory
sufficient for single-instance internal relay v1.

**Alternatives considered**:
- **Redis rate limit**: Over-engineered for v1 internal deployment.

## 8. CSRF Protection

**Decision**: Single-use CSRF token stored in invoice metadata is overkill; use SameSite=Lax cookie
+ random form token bound to reference in server session cache (in-memory HashMap with TTL 1h).

**Rationale**: Public form POST needs basic CSRF protection without full session store.

**Simpler v1 fallback**: For MVP, omit CSRF if pay page is GET+POST same origin only with
Referer check — **Decision**: include lightweight form token in hidden field validated on POST
(token = HMAC(reference + expiry + server secret), no server-side session store).

**Final**: HMAC form token `{reference}:{expires_at}` signed with `WEBHOOK_SIGNING_SECRET` —
stateless CSRF token, no session store.
