# Tasks: Internal Payment Relay

**Input**: Design documents from `/specs/001-payment-relay/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Included per constitution requirement — integration tests for payment flows, idempotency, wallets, and webhooks.

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: User story label (US1–US5)
- Include exact file paths in descriptions

## Path Conventions

Single Rust project at repository root: `src/`, `tests/`, `migrations/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and repository scaffold

- [x] T001 Create `Cargo.toml` with axum, tokio, sqlx, reqwest, serde, uuid, chrono, tracing, sha2, tower-http dependencies per plan.md
- [x] T002 [P] Create `docker-compose.yml` with PostgreSQL 15 service for local development
- [x] T003 [P] Create `.env.example` with DATABASE_URL, PORT, PAWAPAY_API_TOKEN, PAWAPAY_BASE_URL, WEBHOOK_SIGNING_SECRET, FALLBACK_GATEWAY
- [x] T004 [P] Create `.gitignore` for Rust target/, .env, and IDE artifacts
- [x] T005 Create module tree: `src/main.rs`, `src/config.rs`, `src/error.rs`, `src/models.rs`, `src/auth.rs`, `src/api/mod.rs`, `src/db/mod.rs`, `src/gateway/mod.rs`, `src/webhook/mod.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST complete before user story work

**⚠️ CRITICAL**: No user story work until this phase is complete

- [x] T006 Create `migrations/001_initial.sql` with systems, wallets, transactions, webhook_delivery_attempts tables per data-model.md
- [x] T007 Implement environment loading in `src/config.rs`
- [x] T008 Implement `AppError` enum and HTTP response mapping in `src/error.rs`
- [x] T009 [P] Define System, Wallet, Transaction, WebhookDeliveryAttempt structs in `src/models.rs`
- [x] T010 Implement Postgres pool init and shared query helpers in `src/db/mod.rs` and `src/db/queries.rs`
- [x] T011 Implement API key generation, SHA-256 hashing, and auth middleware in `src/auth.rs`
- [x] T012 Implement axum router skeleton and server bootstrap with tracing in `src/main.rs` and `src/api/routes.rs`
- [x] T013 [P] Define `PaymentGateway` trait in `src/gateway/traits.rs`
- [x] T014 [P] Export gateway modules in `src/gateway/mod.rs`

**Checkpoint**: Foundation ready — user story implementation can begin

---

## Phase 3: User Story 1 — Register an Internal System (Priority: P1) 🎯 MVP

**Goal**: Operators register internal systems and receive API credentials

**Independent Test**: POST `/systems` returns id, prefix, api_key; GET `/systems/:id` works; invalid API key returns 401 on protected routes

### Implementation for User Story 1

- [x] T015 [P] [US1] Add `create_system`, `get_system_by_id`, prefix uniqueness checks in `src/db/queries.rs`
- [x] T016 [US1] Implement `POST /systems` and `GET /systems/:id` handlers in `src/api/systems.rs`
- [x] T017 [US1] Register system routes in `src/api/routes.rs`
- [x] T018 [P] [US1] Add integration test for registration and auth rejection in `tests/integration/systems_test.rs`

**Checkpoint**: User Story 1 fully functional — systems can register and authenticate

---

## Phase 4: User Story 2 — Process a Payment (Priority: P1)

**Goal**: Authenticated systems submit idempotent payments routed through pawaPay with persisted status

**Independent Test**: POST `/payments` with valid API key returns status + gateway_reference; duplicate idempotency key returns same result; country/balance validation enforced

### Implementation for User Story 2

- [x] T019 [P] [US2] Add wallet get-or-create, transaction CRUD, idempotency lookup, atomic debit in `src/db/queries.rs`
- [x] T020 [P] [US2] Implement pawaPay v2 payout adapter with 3-retry policy in `src/gateway/pawapay.rs`
- [x] T021 [US2] Implement `POST /payments` handler with country check, balance check, idempotency, gateway call in `src/api/payments.rs`
- [x] T022 [US2] Implement `GET /payments/:id` handler scoped to authenticated system in `src/api/payments.rs`
- [x] T023 [US2] Wire payment routes in `src/api/routes.rs`
- [x] T024 [P] [US2] Add payment happy-path integration test with mocked gateway in `tests/integration/payments_test.rs`
- [x] T025 [P] [US2] Add idempotency replay and 409 conflict integration test in `tests/integration/idempotency_test.rs`

**Checkpoint**: User Stories 1 + 2 deliver MVP — register, pay, query payment by id

---

## Phase 5: User Story 4 — Query Wallets and Balances (Priority: P2)

**Goal**: Systems list country/currency wallets and balances scoped to their system_id

**Independent Test**: GET `/wallets/:system_id` returns only owning system's wallets with correct balances after payments

### Implementation for User Story 4

- [x] T026 [US4] Add `list_wallets_by_system` query in `src/db/queries.rs`
- [x] T027 [US4] Implement `GET /wallets/:system_id` handler with API key system scoping in `src/api/wallets.rs`
- [x] T028 [US4] Wire wallet routes in `src/api/routes.rs`
- [x] T029 [P] [US4] Add wallet listing integration test in `tests/integration/wallets_test.rs`

**Checkpoint**: Wallet visibility works independently of webhook delivery

---

## Phase 6: User Story 3 — Receive Payment Webhooks (Priority: P2)

**Goal**: Terminal payment states trigger signed webhook delivery with retry audit trail

**Independent Test**: Completed/failed payment POSTs webhook payload matching `contracts/webhook-payload.json` with valid `X-Relay-Signature`

### Implementation for User Story 3

- [x] T030 [P] [US3] Implement HMAC-SHA256 signing and 3-attempt delivery in `src/webhook/sender.rs`
- [x] T031 [US3] Add `record_webhook_attempt` query in `src/db/queries.rs`
- [x] T032 [US3] Spawn async webhook dispatch from terminal transitions in `src/api/payments.rs`
- [x] T033 [P] [US3] Add webhook delivery integration test with mock HTTP server in `tests/integration/webhooks_test.rs`

**Checkpoint**: Downstream systems receive auditable payment notifications

---

## Phase 7: User Story 5 — Trace External IDs (Priority: P3)

**Goal**: Support staff and systems locate transactions by external_id with format validation

**Independent Test**: GET `/transactions/:system_id?external_id=...` returns matching transaction for owning system only

### Implementation for User Story 5

- [x] T034 [P] [US5] Add external_id format validator in `src/models.rs`
- [x] T035 [US5] Add `list_transactions_by_system` with optional external_id filter in `src/db/queries.rs`
- [x] T036 [US5] Implement `GET /transactions/:system_id` handler in `src/api/payments.rs`
- [x] T037 [US5] Wire transaction list route in `src/api/routes.rs`

**Checkpoint**: All five user stories independently functional

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Shared test infrastructure, documentation, and end-to-end validation

- [x] T038 [P] Create shared test fixtures and DB helpers in `tests/common/mod.rs`
- [x] T039 [P] Add structured tracing fields (system_id, external_id, idempotency_key) across handlers in `src/api/payments.rs` and `src/api/systems.rs`
- [x] T040 Run full validation scenarios from `specs/001-payment-relay/quickstart.md` and document any gaps
- [x] T041 [P] Align root `README.md` setup/run instructions with implemented project structure

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — **blocks all user stories**
- **User Stories (Phases 3–7)**: Depend on Phase 2 completion
- **Polish (Phase 8)**: Depends on desired user stories being complete

### User Story Dependencies

| Story | Priority | Depends on | Notes |
|-------|----------|------------|-------|
| US1 | P1 | Foundational | MVP entry point |
| US2 | P1 | US1 (registered system + auth) | Core payment flow |
| US4 | P2 | US2 (wallets populated by payments) | Read-only endpoint |
| US3 | P2 | US2 (terminal payment states) | Async side effect |
| US5 | P3 | US2 (transactions exist) | Query/filter only |

### Within Each User Story

- DB queries before handlers
- Handlers before route wiring
- Core implementation before integration tests
- Validate checkpoint before next priority

### Parallel Opportunities

- **Phase 1**: T002, T003, T004 in parallel after T001
- **Phase 2**: T009, T013, T014 in parallel
- **Phase 4**: T019, T020, T024, T025 in parallel (different files)
- **Phase 6**: T030, T033 in parallel
- **Phase 8**: T038, T039, T041 in parallel

---

## Parallel Example: User Story 2

```bash
# Parallel implementation tracks:
Task T019: wallet/transaction queries in src/db/queries.rs
Task T020: pawaPay adapter in src/gateway/pawapay.rs

# Parallel tests after handlers land:
Task T024: tests/integration/payments_test.rs
Task T025: tests/integration/idempotency_test.rs
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: US1 — system registration
4. Complete Phase 4: US2 — payment processing
5. **STOP and VALIDATE** using quickstart steps 1–5
6. Demo/deploy internal relay MVP

### Incremental Delivery

1. Setup + Foundational → foundation ready
2. US1 → register systems (MVP part 1)
3. US2 → process payments (MVP part 2)
4. US4 → wallet visibility
5. US3 → webhook notifications
6. US5 → external ID tracing
7. Polish → quickstart validation + docs

### Parallel Team Strategy

After Phase 2:

- **Developer A**: US1 + US2 (critical path)
- **Developer B**: US4 wallet endpoints (after US2 DB queries exist)
- **Developer C**: US3 webhook sender (after US2 terminal states exist)

---

## Notes

- Total tasks: **41**
- MVP scope: **T001–T025** (Phases 1–4)
- All tasks use checklist format with IDs and file paths
- pawaPay Postman collection at `pawaPay Merchant API V2.postman_collection.json` is reference for adapter field mapping
- Wallet funding API is out of v1 scope — seed balances via SQL per quickstart.md until added
