# Tasks: PawaPay Wallets, Country Catalog & Forms

**Input**: Design documents from `/specs/005-pawapay-wallets-forms/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/country-mno-catalog.md, quickstart.md

**Tests**: Not requested in spec (manual quickstart + `npm run build` / targeted Rust tests in polish). No TDD tasks.

**Organization**: Catalog (US4) is implemented before register/overview/forms because it blocks them. Reports (US5) is last.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete work)
- **[Story]**: `[US1]`…`[US5]` for story phases only
- Include exact file paths in every task

## Path Conventions

- Backend: `src/`, `config/`
- Frontend: `frontend/src/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Align seed defaults and module placeholders with the plan

- [X] T001 Update `config/wallet_seed_defaults.json` for Zambia (`ZM`/`ZMW`); remove or stop relying on non-PawaPay `US` seed for product path
- [X] T002 [P] Add catalog module skeleton `src/catalog/mod.rs` and declare `mod catalog` in `src/lib.rs` (or `src/main.rs` crate root as used by the project)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared helpers both API and SPA will use — MUST complete before story UI/API wiring beyond catalog data

**⚠️ CRITICAL**: No register/overview/form story work until catalog data + `providers_for_country` replacement are ready (US4 phase below completes the data; this phase only scaffolds shared validation helpers if needed)

- [X] T003 Define catalog types and lookup helpers (by iso2, mnos, currencies) in `src/catalog/mod.rs`
- [X] T004 [P] Create `frontend/src/lib/catalog.ts` with matching TypeScript types and empty/stub export shape ready for country rows

**Checkpoint**: Module/types ready — populate catalog in US4 next

---

## Phase 3: User Story 4 - Shared PawaPay country/MNO catalog (Priority: P1) 🎯 Foundation for other stories

**Goal**: Single catalog for listed markets with ISO-2/ISO-3, currencies, friendly MNO labels, and correspondent codes.

**Independent Test**: Resolve Zambia → Airtel/MTN/Zamtel correspondents; unknown country returns empty/error; pay providers use catalog.

### Implementation for User Story 4

- [X] T005 [US4] Populate full country/MNO rows in `src/catalog/mod.rs` per `specs/005-pawapay-wallets-forms/research.md` (BJ, CM, CI, CD, GA, KE, CG, RW, SN, SL, UG, ZM)
- [X] T006 [US4] Replace `providers_for_country` in `src/pay/mod.rs` to delegate to catalog (remove sparse ZM/US-only list)
- [X] T007 [P] [US4] Mirror full catalog data (flags, names, currencies, MNO label→correspondent) in `frontend/src/lib/catalog.ts`
- [X] T008 [US4] Add catalog validation helpers (country/currency/provider consistency) in `src/catalog/mod.rs` for API use

**Checkpoint**: Catalog is SoT for providers and form mapping

---

## Phase 4: User Story 1 - Register defaults to Zambia without country UI (Priority: P1) 🎯 MVP

**Goal**: Register has no country field; server forces Zambia and seeds ZMW wallet.

**Independent Test**: Register UI has no country input; new system `enabled_countries` is `["ZM"]` with ZMW wallet.

### Implementation for User Story 1

- [X] T009 [US1] Force `enabled_countries = ["ZM"]` on public register in `src/api/systems.rs` (ignore client overrides)
- [X] T010 [US1] Remove countries/wallet fields and related state from `frontend/src/pages/RegisterPage.tsx`; stop sending `enabled_countries` from the form (or send nothing and rely on server)
- [X] T011 [US1] Confirm seed path uses `config/wallet_seed_defaults.json` for ZM in `src/seed/mod.rs` / register flow (adjust only if needed)

**Checkpoint**: New merchants onboard Zambia-only without country UI

---

## Phase 5: User Story 2 - Overview shows balances and MNOs per country (Priority: P1)

**Goal**: Overview lists each wallet’s country name, currency, balance, and catalog MNO labels.

**Independent Test**: Dashboard shows Zambia balance plus Airtel/MTN/Zamtel labels (not codes-only).

### Implementation for User Story 2

- [X] T012 [US2] Enrich wallet display in `frontend/src/pages/DashboardPage.tsx` using `frontend/src/lib/catalog.ts` (country display name + MNO labels per wallet.country)
- [X] T013 [P] [US2] Style Overview wallet section with DESIGN.md/shadcn patterns already in the app (title, table/cards, calm badges) in `frontend/src/pages/DashboardPage.tsx`

**Checkpoint**: Merchant can cross-check balances against enabled MNOs

---

## Phase 6: User Story 3 - Invoice and payout forms use catalog dropdowns (Priority: P1)

**Goal**: Flagged country select, auto currency (DRC dual select), friendly provider select; submit ISO-2 + currency + correspondent codes.

**Independent Test**: Create ZM invoice and payout selecting “MTN”; request body uses `MTN_MOMO_ZMB` without user typing it.

### Implementation for User Story 3

- [X] T014 [US3] Add `frontend/src/components/CountrySelect.tsx` (flag + name; options filtered to system enabled countries)
- [X] T015 [P] [US3] Add `frontend/src/components/ProviderSelect.tsx` (MNO labels; value = correspondent code)
- [X] T016 [US3] Rework `frontend/src/pages/NewInvoicePage.tsx` to use CountrySelect, derive/fix currency (DRC CDF/USD), validate before submit
- [X] T017 [US3] Rework `frontend/src/pages/PaymentsPage.tsx` to use CountrySelect + ProviderSelect + derived currency; remove free-typed provider/currency defaults like raw `MTN_MOMO_ZMB`
- [X] T018 [US3] Optionally validate country/currency/provider on create invoice / process payment in `src/api/` handlers using `src/catalog/mod.rs` helpers
- [X] T019 [P] [US3] Ensure pay-page provider list remains catalog-driven via `src/api/pay_page.rs` + `src/pay/mod.rs` (no UI code change required if API already uses new providers)

**Checkpoint**: Forms never require raw correspondent codes

---

## Phase 7: User Story 5 - Reports UI matches DESIGN.md (Priority: P2)

**Goal**: Reports filters and results use DESIGN.md / shadcn page chrome.

**Independent Test**: `/reports` shows title/lede, pill Run CTA, card/table asset-rows — not sparse plain form.

### Implementation for User Story 5

- [X] T020 [US5] Restyle filter band and actions in `frontend/src/pages/ReportsPage.tsx` (pill primary, labels, spacing per DESIGN.md)
- [X] T021 [US5] Restyle transaction/invoice/wallet result panels in `frontend/src/pages/ReportsPage.tsx` (cards, mono amounts, semantic up/down where meaningful)

**Checkpoint**: Reports visually coherent with merchant app

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Validation and cleanup

- [X] T022 Grep frontend forms for free-text country/provider patterns and remove leftovers in `frontend/src/pages/`
- [X] T023 [P] Run `npm run build` in `frontend/` and fix TypeScript errors
- [X] T024 [P] Run `cargo test --test pay_page_test` and fix regressions from catalog provider changes
- [X] T025 Execute validation checklist in `specs/005-pawapay-wallets-forms/quickstart.md` and note pass/fail

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup → Foundational → US4 (catalog)** blocks US1–US3 and pay providers
- **US1** after US4 (seed/register defaults)
- **US2** after US4 (needs catalog labels); independent of US1 UI
- **US3** after US4; benefits from US1 Zambia-enabled systems
- **US5** after Foundational; independent of US1–US3 functionally
- **Polish** after desired stories

### User Story Dependencies

- **US4**: First implementable story (catalog) — no dependency on other stories
- **US1**: Depends on catalog/seed Zambia defaults
- **US2**: Depends on catalog
- **US3**: Depends on catalog (+ CountrySelect/ProviderSelect)
- **US5**: Independent visually

### Parallel Opportunities

- T001/T002; T003/T004
- After T005: T006 sequential with pay; T007 parallel on TS
- T014/T015 parallel then T016/T017
- T020/T021 can be sequential in same file (not parallel)
- T023/T024 parallel in polish

---

## Parallel Example: Catalog

```bash
Task: "Populate src/catalog/mod.rs from research.md"
Task: "Mirror catalog in frontend/src/lib/catalog.ts"
```

## Parallel Example: Form controls

```bash
Task: "Add CountrySelect.tsx"
Task: "Add ProviderSelect.tsx"
# then wire NewInvoicePage + PaymentsPage
```

---

## Implementation Strategy

### MVP First

1. Setup + Foundational + **US4 catalog**
2. **US1 register Zambia default**
3. Validate register → Overview wallet exists
4. Demo MVP

### Incremental Delivery

1. Catalog → register → overview MNOs → forms → reports → polish

### Parallel Team Strategy

- Dev A: Rust catalog + register force + pay providers
- Dev B: TS catalog + Country/Provider selects + forms
- Dev C: Dashboard MNOs + Reports restyle

---

## Notes

- Keep wallet country ISO-2; map ISO-3 only at gateway if needed
- Do not invent per-MNO balances
- DESIGN.md for Reports only (forms use existing shadcn)
- Server must force Zambia on register (FR-002)

## Task Summary

| Phase | Tasks | Count |
|-------|-------|-------|
| Setup | T001–T002 | 2 |
| Foundational | T003–T004 | 2 |
| US4 Catalog | T005–T008 | 4 |
| US1 Register | T009–T011 | 3 |
| US2 Overview | T012–T013 | 2 |
| US3 Forms | T014–T019 | 6 |
| US5 Reports | T020–T021 | 2 |
| Polish | T022–T025 | 4 |
| **Total** | | **25** |

**Suggested MVP**: T001–T011 (catalog + Zambia register default).
