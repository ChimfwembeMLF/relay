# Tasks: Invoice Pay Page

**Input**: Design documents from `/specs/003-invoice-pay-page/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Integration tests for pay page GET/POST, invoice webhooks, atomic registration.

## Phase 1: Setup

- [x] T001 Create migration `migrations/003_pay_page.sql` (webhook `event_type` column)
- [x] T002 [P] Add HTML templates under `src/pay/templates/` (open, paid, expired, cancelled, not_found, error)
- [x] T003 [P] Extend `.env.example` with `PAY_PAGE_RATE_LIMIT`

## Phase 2: Foundational

- [x] T004 Add `get_invoice_by_reference_public` in `src/db/invoices.rs`
- [x] T005 [P] Create `src/pay/mod.rs` with template render, form token HMAC, provider map, amount formatting
- [x] T006 Register `pay` module in `src/lib.rs`
- [x] T007 Extract shared `collect_invoice_internal` in `src/api/invoices.rs` for API + pay page reuse
- [x] T008 Extend `record_webhook_attempt` with `event_type` in `src/db/queries.rs`

## Phase 3: User Story 1 — Pay Page View (P1)

- [x] T009 [US1] Implement `GET /pay/{reference}` in `src/api/pay_page.rs`
- [x] T010 [US1] Wire public pay routes in `src/api/routes.rs` (no auth middleware)
- [x] T011 [P] [US1] Add pay page GET tests in `tests/pay_page_test.rs`

## Phase 4: User Story 2 — Pay Page Collect (P1)

- [x] T012 [US2] Implement `POST /pay/{reference}` with form parse + collect in `src/api/pay_page.rs`
- [x] T013 [US2] Add in-memory rate limit middleware for `POST /pay/*` in `src/api/pay_page.rs`
- [x] T014 [P] [US2] Add pay page POST + idempotency tests in `tests/pay_page_test.rs`

## Phase 5: User Story 3 — Invoice Webhooks (P2)

- [x] T015 [US3] Add `deliver_invoice_webhook` in `src/webhook/sender.rs`
- [x] T016 [US3] Trigger invoice webhook from `collect_invoice_internal` in `src/api/invoices.rs`
- [x] T017 [P] [US3] Add webhook test in `tests/invoice_webhook_test.rs`

## Phase 6: User Story 4 — Atomic Registration (P3)

- [x] T018 [US4] Refactor `seed_system_wallets` to accept transaction executor in `src/seed/mod.rs`
- [x] T019 [US4] Wrap system create + seed in single transaction in `src/api/systems.rs`

## Phase 7: Polish

- [x] T020 [P] Update `README.md` with pay page endpoints
- [x] T021 Run quickstart validation scenarios and fix gaps

**MVP scope**: T001–T014 (pay page view + collect)

**Total tasks**: 21
