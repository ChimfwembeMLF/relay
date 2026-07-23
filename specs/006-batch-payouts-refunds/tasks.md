# Tasks: Batch Payouts, Refunds & Full-Country Registration

**Input**: Design documents from `/specs/006-batch-payouts-refunds/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/batch-payouts.md, contracts/invoice-refunds.md, quickstart.md

**Tests**: Quickstart calls for `tests/batch_payout_test.rs`, `tests/invoice_refund_test.rs`, and updates to `tests/seed_on_register_test.rs`. Included in story/polish phases (not strict TDD-first).

**Organization**: Foundational migration + models first; US1 (full-country register) MVP; then US2 batch; US3 refunds; US4 API visibility/OpenAPI; polish.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete work)
- **[Story]**: `[US1]`…`[US4]` for story phases only
- Include exact file paths in every task

## Path Conventions

- Backend: `src/`, `migrations/`, `config/`, `tests/`
- Frontend: `frontend/src/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Seed defaults and catalog helpers for full-country registration

- [X] T001 Expand `config/wallet_seed_defaults.json` to all catalog countries/currencies (amount `0`; include `CD` for both CDF and USD per research R2 — structure as needed for seed resolver)
- [X] T002 [P] Add `register_enabled_countries()` / `all_iso2()` helpers in `src/catalog/mod.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Schema + models shared by batch and refunds — MUST complete before US2/US3

**⚠️ CRITICAL**: No batch/refund API work until migration and model fields exist

- [X] T003 Create `migrations/007_batches_and_refunds.sql` (`payout_batches`, `payout_batch_lines`, `refunds`; invoice `refunded_amount`, `payer_phone`, `payer_provider`; transactions `batch_id`, `refund_id`)
- [X] T004 Extend invoice/transaction models and request/response types in `src/models.rs` per `specs/006-batch-payouts-refunds/data-model.md`
- [X] T005 [P] Add batch/refund structs and create/get helpers skeleton in `src/db/batches.rs` (wire `mod` in `src/db/mod.rs`)
- [X] T006 Extend `src/db/invoices.rs` and `src/db/queries.rs` for payer fields, `refunded_amount` updates, and optional `batch_id`/`refund_id` on `NewTransaction`

**Checkpoint**: Schema and DB helpers ready — user stories can proceed

---

## Phase 3: User Story 1 - Registration enables all supported countries (Priority: P1) 🎯 MVP

**Goal**: Public register enables full catalog and seeds every catalog currency wallet; UI still has no country picker.

**Independent Test**: Register → `enabled_countries` = all catalog ISO-2; wallets include ZM/ZMW and CD/CDF + CD/USD; register form has no country field.

### Implementation for User Story 1

- [X] T007 [US1] Extend `src/seed/config.rs` (and `src/seed/mod.rs` if needed) so one country can resolve **multiple** currency wallets from defaults/catalog (DRC CDF+USD)
- [X] T008 [US1] Force full-catalog `enabled_countries` on public register in `src/api/systems.rs` (ignore client list; use catalog helper from T002)
- [X] T009 [P] [US1] Update register success copy in `frontend/src/pages/RegisterPage.tsx` (all catalog countries enabled — not “Zambia only”)
- [X] T010 [US1] Update `tests/seed_on_register_test.rs` for full-catalog wallet count / ignore client `enabled_countries`

**Checkpoint**: New merchants get full catalog wallets without country UI

---

## Phase 4: User Story 2 - Batch payouts (Priority: P1)

**Goal**: Sync batch create with partial success, max 100 lines, batch idempotency; dashboard multi-row/paste UI.

**Independent Test**: POST batch with mixed valid/invalid lines → per-line results; wallet debit only on success; idempotent replay; UI can submit ≥3 lines.

### Implementation for User Story 2

- [X] T011 [US2] Implement batch create/get persistence in `src/db/batches.rs` (idempotency by `system_id` + key + request hash)
- [X] T012 [US2] Implement `POST /batches` and `GET /batches/{id}` in `src/api/batches.rs` per `specs/006-batch-payouts-refunds/contracts/batch-payouts.md` (sequential line processing, reuse payout/gateway debit path from `src/api/payments.rs` patterns)
- [X] T013 [US2] Register batch routes in `src/api/routes.rs` and `src/api/mod.rs`
- [X] T014 [P] [US2] Add batch client helpers in `frontend/src/api.ts`
- [X] T015 [US2] Add batch mode (multi-row + optional CSV/TSV paste) to `frontend/src/pages/PaymentsPage.tsx` (keep single payout available)
- [X] T016 [US2] Add `tests/batch_payout_test.rs` (partial success + idempotency)

**Checkpoint**: Batch API + Payouts UI work independently of refunds

---

## Phase 5: User Story 3 - Refunds on paid collections (Priority: P1)

**Goal**: Full/partial invoice refunds; status stays `paid`; persist payer on collect; debit wallet + gateway payout.

**Independent Test**: Pay invoice → partial refund → `refunded_amount` increases, status `paid`; over-refund rejected; idempotent replay.

### Implementation for User Story 3

- [X] T017 [US3] Persist `payer_phone` / `payer_provider` on successful collect in `src/api/invoices.rs` / `src/db/invoices.rs` (and pay-page collect path)
- [X] T018 [US3] Expose `refunded_amount`, `remaining_refundable`, `fully_refunded`, payer fields on invoice JSON in `src/models.rs` / invoice handlers
- [X] T019 [US3] Implement `POST /invoices/{id}/refund` in `src/api/refunds.rs` (or `src/api/invoices.rs`) per `specs/006-batch-payouts-refunds/contracts/invoice-refunds.md`
- [X] T020 [US3] Wire refund route in `src/api/routes.rs` / `src/api/mod.rs`
- [X] T021 [P] [US3] Add refund client helper in `frontend/src/api.ts`
- [X] T022 [US3] Add refund UI on `frontend/src/pages/InvoiceDetailPage.tsx` (amount + destination confirmation)
- [X] T023 [US3] Add `tests/invoice_refund_test.rs` (partial, over-refund, idempotency, insufficient balance)

**Checkpoint**: Refunds work end-to-end for paid invoices

---

## Phase 6: User Story 4 - API and operational visibility (Priority: P2)

**Goal**: OpenAPI + transaction linkage so integrators/support see batch/refund outcomes.

**Independent Test**: OpenAPI documents batch/refund; transactions list/show carry `batch_id`/`refund_id` when set; webhooks still fire for payout txs.

### Implementation for User Story 4

- [X] T024 [US4] Document `POST/GET /batches` and `POST /invoices/{id}/refund` (and invoice refund fields) in `openapi/openapi.yaml`
- [X] T025 [P] [US4] Ensure transaction list/detail responses include `batch_id` / `refund_id` when present in `src/models.rs` / `src/api/payments.rs`
- [X] T026 [US4] Confirm webhook sender still emits for refund/batch-line payout transactions (adjust `src/webhook/sender.rs` only if linkage fields required)

**Checkpoint**: External API docs and ops visibility complete

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Build, regression, quickstart validation

- [X] T027 [P] Update `specs/003-invoice-pay-page/contracts/openapi.yaml` or shared OpenAPI copy if the project keeps a duplicate in sync with `openapi/openapi.yaml`
- [X] T028 Run `npm run build` in `frontend/` and fix TypeScript errors
- [X] T029 Run `cargo test --test seed_on_register_test --test batch_payout_test --test invoice_refund_test` and fix failures
- [X] T030 Execute validation checklist in `specs/006-batch-payouts-refunds/quickstart.md` and note pass/fail in that file

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Start immediately
- **Foundational (Phase 2)**: After Setup — **blocks US2/US3** (US1 needs T001–T002 + seed; can overlap once T001–T002 done)
- **US1**: After T001–T002 (+ seed changes); does not need batch/refund tables
- **US2 / US3**: After Foundational (T003–T006); can proceed in parallel after foundation
- **US4**: After US2 + US3 APIs exist
- **Polish**: After desired stories complete

### User Story Dependencies

- **US1 (P1)**: Catalog helpers + seed expansion — MVP for registration
- **US2 (P1)**: Needs foundational schema; independent of US3
- **US3 (P1)**: Needs foundational schema; independent of US2
- **US4 (P2)**: Needs US2 + US3 endpoints

### Parallel Opportunities

- T001 ∥ T002
- T005 ∥ parts of T004 after T003
- After foundation: US2 and US3 in parallel (different files)
- T014 ∥ T011–T013 (API client while backend lands carefully)
- T021 ∥ T019–T020
- T027 ∥ T028

---

## Parallel Example: User Story 2

```bash
# After T011–T013 backend routes:
Task: "Add batch client helpers in frontend/src/api.ts"
# Then UI:
Task: "Add batch mode to frontend/src/pages/PaymentsPage.tsx"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1 Setup (T001–T002)
2. US1 seed + register (T007–T010)
3. **STOP and VALIDATE**: Register → full catalog wallets

### Incremental Delivery

1. Setup + Foundational → schema ready
2. US1 → full-country register (MVP)
3. US2 → batch payouts
4. US3 → refunds
5. US4 → OpenAPI / visibility
6. Polish → tests + quickstart

### Parallel Team Strategy

- After Foundational: Dev A = US2 batch, Dev B = US3 refunds, then US4 together

---

## Notes

- Max batch size **100**; sync sequential processing; partial success
- Invoice status stays **`paid`**; track `refunded_amount`
- Do not auto-migrate existing systems’ `enabled_countries`
- Single `POST /payments` remains unchanged
- Commit after each logical group; mark tasks `[X]` during `/speckit-implement`
