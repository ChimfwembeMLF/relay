# Tasks: Wallet Seeding, Invoices & Reports

**Input**: Design documents from `/specs/002-wallet-invoices-reports/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Included per constitution — integration tests for seed-on-register, invoice lifecycle, reports, CSV parity.

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: User story label (US1–US5)
- Include exact file paths in descriptions

## Path Conventions

Extends existing Rust project at repository root (`src/`, `migrations/`, `tests/`).

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Migration, dependencies, and operator seed config file

- [x] T001 Add `qrcode`, `image`, and `csv` crates to `Cargo.toml`
- [x] T002 Create migration `migrations/002_invoices_and_seed_events.sql` (invoices, wallet_seed_events, alter transactions)
- [x] T003 [P] Create `config/wallet_seed_defaults.json` with ZM and US default entries
- [x] T004 [P] Extend `.env.example` with `INVOICE_PAY_BASE_URL`, `WALLET_SEED_DEFAULTS_PATH`, `WALLET_SEED_DEFAULTS_JSON`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core modules and gateway deposit support — blocks all user stories

**⚠️ CRITICAL**: No user story work until this phase is complete

- [x] T005 Extend `src/config.rs` to load wallet seed defaults and invoice pay base URL
- [x] T006 [P] Add Invoice, WalletSeedEvent, WalletSeedOverride, report DTO structs in `src/models.rs`
- [x] T007 [P] Create `src/seed/config.rs` with country→currency→amount resolution logic
- [x] T008 [P] Create `src/seed/mod.rs` exporting seed orchestration entry point
- [x] T009 [P] Create `src/qr/mod.rs` with PNG base64 QR generation from URL string
- [x] T010 Add `process_deposit()` to `PaymentGateway` trait in `src/gateway/traits.rs`
- [x] T011 Implement pawaPay deposit adapter (`POST /v2/deposits`) in `src/gateway/pawapay.rs`
- [x] T012 [P] Implement `process_deposit()` on `MockGateway` in `src/gateway/mock.rs`
- [x] T013 Register `seed` and `qr` modules in `src/lib.rs`

**Checkpoint**: Foundation ready — user story implementation can begin

---

## Phase 3: User Story 1 — Auto-Seed Wallets on Registration (Priority: P1) 🎯 MVP

**Goal**: New systems receive funded wallets for every enabled country without manual SQL

**Independent Test**: Register system with `["ZM","US"]`; GET wallets shows correct balances immediately

### Implementation for User Story 1

- [x] T014 [P] [US1] Add `seed_wallets_for_system` and `record_wallet_seed_event` queries in `src/db/queries.rs`
- [x] T015 [US1] Implement `seed::seed_system_wallets()` transactional orchestration in `src/seed/mod.rs`
- [x] T016 [US1] Hook wallet seeding into `create_system` handler in `src/api/systems.rs` (same DB transaction)
- [x] T017 [US1] Add `wallets_seeded` count to `CreateSystemResponse` in `src/models.rs` and handler response
- [x] T018 [P] [US1] Add integration test in `tests/seed_on_register_test.rs`

**Checkpoint**: User Story 1 complete — zero manual SQL onboarding (SC-006)

---

## Phase 4: User Story 4 — Operator Default Seed Configuration (Priority: P2)

**Goal**: Operator-configurable defaults with optional per-registration overrides

**Independent Test**: Change default JSON; register system; override one country; verify precedence

### Implementation for User Story 4

- [x] T019 [US4] Add optional `wallet_seeds` field to `CreateSystemRequest` in `src/models.rs`
- [x] T020 [US4] Merge override > default > zero fallback in `src/seed/config.rs`
- [x] T021 [US4] Validate override countries ⊆ `enabled_countries` in `src/api/systems.rs`
- [x] T022 [P] [US4] Add override precedence test cases to `tests/seed_on_register_test.rs`

**Checkpoint**: Seed configuration flexible without code changes

---

## Phase 5: User Story 2 — Generate Invoice with QR Code (Priority: P1)

**Goal**: Create pay-in invoices with QR codes and deposit-based collection

**Independent Test**: POST invoice → receive QR; collect → invoice paid + wallet credited

### Implementation for User Story 2

- [x] T023 [P] [US2] Add invoice CRUD, expiry lazy-update, and status transition queries in `src/db/queries.rs`
- [x] T024 [P] [US2] Add `create_deposit_with_credit` atomic wallet credit + transaction insert in `src/db/queries.rs`
- [x] T025 [US2] Implement `POST /invoices`, `GET /invoices`, `GET /invoices/{reference}` in `src/api/invoices.rs`
- [x] T026 [US2] Implement `POST /invoices/{id}/collect` with deposit gateway call in `src/api/invoices.rs`
- [x] T027 [US2] Implement `POST /invoices/{id}/cancel` in `src/api/invoices.rs`
- [x] T028 [US2] Wire invoice routes in `src/api/routes.rs`
- [x] T029 [US2] Generate QR via `src/qr/mod.rs` on invoice create; store `qr_payload_url`
- [x] T030 [P] [US2] Add invoice lifecycle integration test in `tests/invoices_test.rs` (create, collect, expired, idempotent replay)

**Checkpoint**: Invoices + QR collection flow works end-to-end (SC-002)

---

## Phase 6: User Story 3 — Transaction & Wallet Reports (Priority: P2)

**Goal**: JSON summary reports for transactions, wallets, and invoices by date range

**Independent Test**: Insert fixture data; GET each report type; verify summary totals and system isolation

### Implementation for User Story 3

- [x] T031 [P] [US3] Add transaction summary/detail report queries in `src/db/queries.rs`
- [x] T032 [P] [US3] Add wallet balance + period delta report queries in `src/db/queries.rs`
- [x] T033 [P] [US3] Add invoice summary/detail report queries in `src/db/queries.rs`
- [x] T034 [US3] Implement `GET /reports/transactions`, `/reports/wallets`, `/reports/invoices` in `src/api/reports.rs`
- [x] T035 [US3] Wire report routes in `src/api/routes.rs`
- [x] T036 [P] [US3] Add report integration tests in `tests/reports_test.rs`

**Checkpoint**: JSON reports available for finance reconciliation (SC-004)

---

## Phase 7: User Story 5 — Export Reports for Finance (Priority: P3)

**Goal**: CSV export matching JSON detail columns

**Independent Test**: `format=csv` returns downloadable file with correct headers and row parity

### Implementation for User Story 5

- [x] T037 [US5] Add CSV serialization helper in `src/api/reports.rs` using `csv` crate
- [x] T038 [US5] Support `format=csv` query param with `Content-Disposition` attachment on all report endpoints
- [x] T039 [US5] Enforce 10k row limit with clear error/pagination note in `src/api/reports.rs`
- [x] T040 [P] [US5] Add CSV column parity test in `tests/reports_test.rs`

**Checkpoint**: CSV exports match API detail data (SC-005)

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Logging, docs, quickstart validation

- [x] T041 [P] Add structured tracing for seed, invoice, and report handlers in `src/api/systems.rs`, `src/api/invoices.rs`, `src/api/reports.rs`
- [x] T042 Extend `specs/002-wallet-invoices-reports/contracts/openapi.yaml` if implementation diverges during build
- [x] T043 Run validation scenarios from `specs/002-wallet-invoices-reports/quickstart.md` and fix gaps
- [x] T044 [P] Update root `README.md` implementation section with invoices, seed config, and report endpoints

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — **blocks all user stories**
- **US1 (Phase 3)**: Depends on Phase 2
- **US4 (Phase 4)**: Depends on US1 seed hook (extends same flow)
- **US2 (Phase 5)**: Depends on Phase 2 (deposit gateway); independent of reports
- **US3 (Phase 6)**: Depends on Phase 2 + invoice/transaction data (best after US2 for invoice reports)
- **US5 (Phase 7)**: Depends on US3 report handlers
- **Polish (Phase 8)**: Depends on desired stories complete

### User Story Dependencies

| Story | Priority | Depends on | Notes |
|-------|----------|------------|-------|
| US1 | P1 | Foundational | MVP — auto seed |
| US4 | P2 | US1 | Config/overrides on same registration path |
| US2 | P1 | Foundational + deposit gateway | Invoices + QR |
| US3 | P2 | Foundational; invoice report needs US2 data | JSON reports |
| US5 | P3 | US3 | CSV export layer |

### Parallel Opportunities

- Phase 1: T003, T004 parallel
- Phase 2: T006, T007, T008, T009, T012 parallel
- Phase 5: T023, T024, T030 parallel tracks
- Phase 6: T031, T032, T033, T036 parallel
- Phase 8: T041, T044 parallel

---

## Parallel Example: User Story 2

```bash
# Parallel tracks after T025 starts:
Task T023: invoice queries in src/db/queries.rs
Task T024: deposit+credit queries in src/db/queries.rs

# Parallel after handlers:
Task T030: tests/invoices_test.rs
```

---

## Implementation Strategy

### MVP First (US1 + US2)

1. Complete Phases 1–2 (setup + foundation)
2. Complete Phase 3 (US1 auto-seed)
3. Complete Phase 5 (US2 invoices + QR)
4. **STOP and VALIDATE** using quickstart steps 1–4
5. Demo pay-in invoice collection

### Incremental Delivery

1. Setup + Foundational
2. US1 + US4 → auto onboarding with config
3. US2 → invoices and QR collection
4. US3 → JSON reports
5. US5 → CSV exports
6. Polish

### Suggested MVP Scope

**T001–T030** (Phases 1–3 and 5) delivers the user's core ask: seeded wallets + invoices with QR codes.

Reports (T031–T040) can follow as a second increment.

---

## Notes

- Total tasks: **44**
- MVP scope: **T001–T030** (seed + invoices); full feature: **T001–T044**
- Builds on implemented `001-payment-relay` codebase — do not rewrite existing endpoints
- pawaPay deposits: reference `pawaPay Merchant API V2.postman_collection.json` deposits section
- Existing payout flow remains unchanged; new `direction=deposit` rows for collections
