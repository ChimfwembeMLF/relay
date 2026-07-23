# Implementation Plan: Invoice Pay Page

**Branch**: `003-invoice-pay-page` | **Date**: 2026-07-18 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/003-invoice-pay-page/spec.md`

**Note**: Extends Feature 002 — adds public HTML routes and invoice webhooks; closes atomic seed gap.

## Summary

Add a payer-facing hosted pay page at `GET/POST /pay/{reference}` so QR deep links from Feature 002
work end-to-end. Payers view invoice details and submit mobile-money payment without API keys.
Extend webhooks for `invoice.paid` events. Wrap system registration + wallet seeding in a single
DB transaction.

## Technical Context

**Language/Version**: Rust 1.75+ (extends existing codebase)

**Primary Dependencies**: Existing stack only — no new crates required for v1 HTML (`include_str!`
templates). Optional: `tower-governor` or custom in-memory rate limiter.

**Storage**: PostgreSQL — migration `003_pay_page.sql` (global unique index on `invoices.reference`,
optional `event_type` on webhook attempts)

**Testing**: cargo test — integration tests for pay page GET states, POST collect, idempotent replay,
webhook on pay-page success, atomic registration rollback

**Target Platform**: Same as 001/002 — Linux/Docker, local Homebrew Postgres

**Project Type**: web-service extension (adds public HTML routes alongside JSON API)

**Performance Goals**: Pay page GET p95 < 500ms; POST collect same as API collect (< 2s including gateway)

**Constraints**: No API key on public routes; reference is capability token; integer minor units;
MMO providers only in v1 form

**Scale/Scope**: 2 public routes, 1 HTML template module, webhook extension, 1 transactional fix,
1 migration

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Pre-Design | Post-Design |
|-----------|------------|-------------|
| I. Internal-First Simplicity | PASS — minimal HTML, no SPA | PASS — static templates, reuses collect logic |
| II. System Isolation | PASS | PASS — invoice lookup scoped; no cross-system data on page |
| III. Idempotent Payments | PASS | PASS — hidden idempotency key per page load; replay safe |
| IV. Reliable External Relay | PASS | PASS — same deposit adapter + 3-retry |
| V. Observability | PASS | PASS — log pay page views/collects with reference + system_id |
| Security | PASS | PASS — no secrets in HTML; HMAC form token; rate limit POST; generic 404 |

**Gate result**: PASS

## Project Structure

### Documentation (this feature)

```text
specs/003-invoice-pay-page/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── openapi.yaml
│   └── pay-page-ui.md
├── checklists/
│   └── requirements.md
└── tasks.md                    # /speckit-tasks
```

### Source Code (additions to repo root)

```text
src/
├── api/
│   ├── pay_page.rs             # NEW — GET/POST /pay/{reference}
│   ├── routes.rs               # + public pay routes (no auth middleware)
│   ├── systems.rs              # transactional register+seed
│   └── invoices.rs             # extract shared collect helper
├── db/
│   └── invoices.rs             # + get_invoice_by_reference_public
├── webhook/
│   └── sender.rs               # + deliver_invoice_webhook
├── pay/
│   └── templates/              # NEW — HTML fragments
│       ├── open.html
│       ├── paid.html
│       ├── expired.html
│       └── not_found.html
migrations/
└── 003_pay_page.sql
tests/
├── pay_page_test.rs            # NEW
├── invoice_webhook_test.rs     # NEW
└── seed_on_register_test.rs    # + atomic rollback case
```

**Structure Decision**: Single binary; `pay/templates/` keeps HTML out of Rust string literals;
`pay_page.rs` handles routing and form parsing.

## Complexity Tracking

> No constitution violations requiring justification.

| Item | Notes |
|------|-------|
| Public unauthenticated route | Justified by QR payer workflow; reference entropy + rate limit mitigate abuse |
| HTML in payment relay | Minimal scope — 4 templates, no frontend build pipeline |
