# Tasks: Modernize Merchant & Pay UI (Coinbase Design System)

**Input**: Design documents from `/specs/004-modernize-merchant-and/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/ui-design-system.md, quickstart.md

**Tests**: Not requested (manual DESIGN.md checklist + `npm run build` only). No TDD tasks.

**Organization**: Tasks grouped by user story. **Fidelity rule**: map `frontend/DESIGN.md` literally — no invented colors, gradients, glow, or layouts. **Sidebar is in scope.**

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete work)
- **[Story]**: User story label (`[US1]`…`[US4]`)
- Include exact file paths in every task

## Path Conventions

- Frontend SPA: `frontend/src/`, `frontend/tailwind.config.js`, `frontend/index.html`, `frontend/DESIGN.md`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm SoT and fidelity before token work

- [X] T001 Confirm `frontend/DESIGN.md` header documents sole SoT, font substitutes, and full-SPA scope including sidebar
- [X] T002 [P] Align token/shell checklist in `specs/004-modernize-merchant-and/contracts/ui-design-system.md` with DESIGN.md `colors:` and `top-nav-light` / `nav-link`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Literal DESIGN.md tokens + shared primitives — MUST complete before story screens

**⚠️ CRITICAL**: No user story page work until this phase is complete. Do not invent theme values.

- [X] T003 Remap CSS variables in `frontend/src/index.css` to DESIGN.md colors (`primary` `#0052ff`, `primary-active`, `primary-disabled`, ink, body, muted, canvas, surface-soft/strong, hairline, semantic-up/down, surface-dark / surface-dark-elevated)
- [X] T004 Update `frontend/tailwind.config.js` theme colors, `fontFamily` (display/sans/mono substitutes), and `borderRadius` (`md` 12px, `xl` 24px, `pill` 100px) from DESIGN.md `rounded:`
- [X] T005 [P] Wire Inter + IBM Plex Mono (or documented substitutes) in `frontend/index.html`; remove Figtree/Gilmer ERP pairing if present
- [X] T006 Map DESIGN.md typography scale (size/weight/letter-spacing for display/title/body/nav-link/button/number-display) into Tailwind theme or utility classes in `frontend/tailwind.config.js` / `frontend/src/index.css`
- [X] T007 Update `frontend/src/components/ui/button.tsx` to DESIGN.md `button-primary` / `button-primary-active` / `button-primary-disabled` / `button-secondary-light` / `button-pill-cta` (pill radius, heights 44/56)
- [X] T008 [P] Update `frontend/src/components/ui/input.tsx` to DESIGN.md `text-input` (~48px, `rounded.md` 12px, hairline, primary focus)
- [X] T009 [P] Update `frontend/src/components/ui/card.tsx` to DESIGN.md `product-ui-card-light` (`rounded.xl` 24px, hairline, no heavy shadow)
- [X] T010 [P] Update `frontend/src/components/ui/badge.tsx` to DESIGN.md `badge-pill` with text-forward semantic-up/down (no loud solid chips)
- [X] T011 [P] Update `frontend/src/components/ui/table.tsx` to DESIGN.md `asset-row` padding and hairline separators
- [X] T012 [P] Update `frontend/src/components/ui/label.tsx` and `frontend/src/components/ui/select.tsx` to body/title token typography from DESIGN.md
- [X] T013 Remove any ERP `brandBlue` / `brandMauve` / `#0D3A69` / `#E394AF` brand usage from `frontend/tailwind.config.js` and `frontend/src/index.css`
- [X] T014 Add or verify `frontend/src/components/EmptyState.tsx` (title, lede, optional CTA) using DESIGN.md title/body tokens only
- [X] T015 Grep `frontend/src` for invented gradients/glow/ERP leftovers and strip anything not grounded in DESIGN.md recipes

**Checkpoint**: Tokens + primitives match DESIGN.md — story surfaces can be restyled

---

## Phase 3: User Story 1 - Cohesive design system applied (Priority: P1) 🎯 MVP

**Goal**: Home, auth, and **full shell (sidebar + top bar)** match DESIGN.md; collapse/drawer behavior preserved.

**Independent Test**: Load `/`, `/login`, `/register`, `/dashboard`; collapse sidebar; confirm `#0052ff` CTAs, canvas/hairline shell, calm display weight — no ERP chrome on sidebar.

### Implementation for User Story 1

- [X] T016 [US1] Restyle retractable sidebar in `frontend/src/components/AppLayout.tsx` per contracts shell table (`colors.canvas`, hairline dividers, `typography.nav-link`, body/muted inactive, primary active) without changing collapse/persistence/mobile drawer behavior
- [X] T017 [US1] Restyle sticky top bar in `frontend/src/components/AppLayout.tsx` to DESIGN.md `top-nav-light` (~64px, canvas, hairline, nav-link)
- [X] T018 [P] [US1] Restyle `frontend/src/pages/HomePage.tsx` using only DESIGN.md `hero-band-dark` or `hero-band-light` + `button-primary` / outline recipes (no invented gradients)
- [X] T019 [P] [US1] Restyle `frontend/src/pages/LoginPage.tsx` with `product-ui-card-light` + `text-input` + `button-primary`
- [X] T020 [P] [US1] Restyle `frontend/src/pages/RegisterPage.tsx` (form + post-register API key panel) with same DESIGN.md recipes
- [X] T021 [US1] Restyle `frontend/src/pages/DashboardPage.tsx` (title/lede, wallet/invoice tables as asset-rows, API panel, EmptyState)
- [X] T022 [US1] Verify `frontend/src/components/BrandLogo.tsx` / BrandMark sizing in expanded and collapsed sidebar on light canvas (keep Tekrem assets; color system remains DESIGN.md)

**Checkpoint**: US1 MVP — shell including sidebar + home/auth/overview match DESIGN.md

---

## Phase 4: User Story 2 - Merchant workspace screens feel product-grade (Priority: P1)

**Goal**: Daily merchant routes use DESIGN.md headers, asset-rows, badge-pill status, empty states, and form recipes.

**Independent Test**: Walk Overview→Invoices→Payouts→Transactions→Webhooks→Reports; empty lists use EmptyState; forms use pill primary CTAs.

### Implementation for User Story 2

- [X] T023 [P] [US2] Restyle invoices list + filters + EmptyState in `frontend/src/pages/InvoicesPage.tsx`
- [X] T024 [P] [US2] Restyle new invoice form in `frontend/src/pages/NewInvoicePage.tsx`
- [X] T025 [P] [US2] Restyle invoice detail (pay link, QR, actions) in `frontend/src/pages/InvoiceDetailPage.tsx`
- [X] T026 [P] [US2] Restyle payout form in `frontend/src/pages/PaymentsPage.tsx`
- [X] T027 [P] [US2] Restyle transactions table + EmptyState in `frontend/src/pages/TransactionsPage.tsx`
- [X] T028 [P] [US2] Restyle webhooks form + table + EmptyState in `frontend/src/pages/WebhooksPage.tsx`
- [X] T029 [P] [US2] Restyle reports filters + summary panels in `frontend/src/pages/ReportsPage.tsx`
- [X] T030 [US2] Apply semantic-up/down text colors for statuses via `frontend/src/components/ui/badge.tsx` usage on workspace pages (DESIGN.md: text color, not loud fills)

**Checkpoint**: US2 — merchant workspace matches DESIGN.md recipes

---

## Phase 5: User Story 3 - Public pay page matches brand confidence (Priority: P2)

**Goal**: `/pay/{reference}` uses DESIGN.md checkout emphasis; dark cards only via named recipes.

**Independent Test**: Open/paid/expired/not-found usable; amount dominant; Pay CTA pill primary.

### Implementation for User Story 3

- [X] T031 [US3] Restyle pay shell (header/footer) in `frontend/src/pages/PayPage.tsx` using `footer-light` / `legal-band` tokens from DESIGN.md
- [X] T032 [US3] Emphasize amount with DESIGN.md `number-display` (mono substitute) and compact `text-input` fields in `frontend/src/pages/PayPage.tsx`
- [X] T033 [US3] Restyle loading/success/error/not-found/not-payable states in `frontend/src/pages/PayPage.tsx` using `product-ui-card-light` and optional `product-ui-card-dark` only
- [X] T034 [US3] Ensure `prefers-reduced-motion` handling in `frontend/src/index.css` (and PayPage if any motion) per edge case

**Checkpoint**: US3 — pay page matches DESIGN.md

---

## Phase 6: User Story 4 - Admin backoffice stays coherent (Priority: P3)

**Goal**: Admin login and systems list/detail share DESIGN.md tokens with merchant workspace.

**Independent Test**: Sign in at `/admin`, list systems, open detail — same chrome as merchant.

### Implementation for User Story 4

- [X] T035 [P] [US4] Restyle admin login + systems list in `frontend/src/pages/AdminPage.tsx` to DESIGN.md card/table/button recipes
- [X] T036 [P] [US4] Restyle admin system detail (wallets + webhooks tables) in `frontend/src/pages/AdminSystemPage.tsx`

**Checkpoint**: US4 — admin coherent with merchant DESIGN.md UI

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Fidelity audit and validation

- [X] T037 Grep `frontend/` (exclude `node_modules`) for residual ERP `#0D3A69` / `#E394AF` / `brandBlue` / `brandMauve` and remove brand usage
- [X] T038 Grep `frontend/src` for non-DESIGN.md decorative patterns (glow, multi-layer shadows, invented gradient heroes) and remove or replace with named recipes
- [X] T039 [P] Spot-check mobile (≤390px) sidebar drawer + primary CTAs on home/login/invoices per `specs/004-modernize-merchant-and/quickstart.md`
- [X] T040 [P] Spot-check desktop (≥1280px) sidebar collapse + shell hairlines/active nav per shell contract
- [X] T041 Run `npm run build` in `frontend/` and fix TypeScript/CSS errors
- [X] T042 Execute visual checklist in `specs/004-modernize-merchant-and/quickstart.md` (including sidebar scenario) and note pass/fail
- [X] T043 [P] Confirm `cargo test --test pay_page_test` still green (no API changes expected)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Start immediately
- **Foundational (Phase 2)**: Depends on Setup — **BLOCKS** all user stories
- **US1 (Phase 3)**: After Foundational — MVP (**includes sidebar**)
- **US2 (Phase 4)**: After Foundational (ideally after US1 shell for consistency)
- **US3 (Phase 5)**: After Foundational; independent of US2
- **US4 (Phase 6)**: After Foundational; independent of US2/US3
- **Polish (Phase 7)**: After desired stories complete

### User Story Dependencies

- **US1 (P1)**: No dependency on other stories — MVP; **sidebar required**
- **US2 (P1)**: Uses EmptyState + primitives; independent of US1 pages except shared tokens
- **US3 (P2)**: Independent pay page file
- **US4 (P3)**: Independent admin pages

### Parallel Opportunities

- Phase 2: T005, T008–T012 parallel after T003/T004
- Phase 3: T018–T020 parallel after T016/T017 start
- Phase 4: T023–T029 parallel across page files
- Phase 6: T035–T036 parallel
- Phase 7: T039–T040, T043 parallel with build once code frozen

---

## Parallel Example: User Story 1 (shell first)

```bash
# Shell must land; then auth/home in parallel:
Task: "Restyle sidebar in frontend/src/components/AppLayout.tsx"
Task: "Restyle top bar in frontend/src/components/AppLayout.tsx"
# Then:
Task: "Restyle HomePage.tsx from DESIGN.md hero-band recipes"
Task: "Restyle LoginPage.tsx"
Task: "Restyle RegisterPage.tsx"
```

---

## Parallel Example: User Story 2

```bash
Task: "Restyle InvoicesPage.tsx"
Task: "Restyle NewInvoicePage.tsx"
Task: "Restyle PaymentsPage.tsx"
Task: "Restyle TransactionsPage.tsx"
Task: "Restyle WebhooksPage.tsx"
Task: "Restyle ReportsPage.tsx"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1 Setup
2. Phase 2 Foundational (literal tokens + primitives)
3. Phase 3 US1 (**sidebar + top bar** + home/auth/overview)
4. **STOP and VALIDATE** quickstart token smoke + sidebar scenario
5. Demo MVP

### Incremental Delivery

1. Setup + Foundational → DESIGN.md foundation
2. US1 → shell including sidebar
3. US2 → merchant workspace
4. US3 → pay page
5. US4 → admin
6. Polish → fidelity grep + build + quickstart

### Parallel Team Strategy

1. Shared: Phase 1–2
2. Then: Dev A US1 shell, Dev B US2 pages, Dev C US3+US4

---

## Notes

- Do **not** change API/auth business logic (`FR-009`)
- Preserve retractable sidebar **behavior** (`FR-005`); restyle chrome only
- Do **not** invent visuals outside DESIGN.md (plan fidelity rules)
- Tekrem logo assets may remain; brand **color** is Coinbase Blue `#0052ff`
- No automated visual regression tasks unless added later

## Task Summary

| Phase | Tasks | Count |
|-------|-------|-------|
| Setup | T001–T002 | 2 |
| Foundational | T003–T015 | 13 |
| US1 (incl. sidebar) | T016–T022 | 7 |
| US2 | T023–T030 | 8 |
| US3 | T031–T034 | 4 |
| US4 | T035–T036 | 2 |
| Polish | T037–T043 | 7 |
| **Total** | | **43** |

**Suggested MVP**: Phases 1–3 (T001–T022) — DESIGN.md tokens + **sidebar/shell** + home/auth/overview.
