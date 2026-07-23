# Implementation Plan: PawaPay Wallets, Country Catalog & Forms

**Branch**: `005-pawapay-wallets-forms` | **Date**: 2026-07-23 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/005-pawapay-wallets-forms/spec.md`

## Summary

Introduce a single PawaPay-aligned **country/MNO catalog** for the markets Relay will support (user-listed). Default new registrations to **Zambia only** (no country field on register). Surface wallet **balances + friendly MNO labels** on Overview. Replace free-text country/currency/provider on invoice & payout forms with **flagged country dropdowns**, **auto currency**, and **human provider dropdowns** that map to backend correspondent codes. Restyle **Reports** to `frontend/DESIGN.md` / shadcn. Align pay-page providers with the same catalog.

## Technical Context

**Language/Version**: Rust (axum) + TypeScript 5.7 / React 19 SPA

**Primary Dependencies**: Existing payment-relay; Vite SPA; shadcn/ui; `frontend/DESIGN.md`; PawaPay gateway already integrated

**Storage**: Existing wallets / systems tables; catalog may be static config (JSON/Rust const + TS mirror) — no new tables required for v1 unless seed defaults expansion needs config file only

**Testing**: Rust register/wallet/pay tests; frontend `npm run build`; manual form/overview/reports checklist in quickstart

**Target Platform**: Relay API + SPA served from `frontend/dist`

**Project Type**: Web application (Rust API + React SPA)

**Performance Goals**: Catalog lookups O(1)/in-memory; no extra gateway calls for MNO list in v1

**Constraints**: Preserve idempotent payments; keep ISO-2 wallet `country` convention unless explicitly migrated; users never type raw correspondent codes; DESIGN.md fidelity for Reports; register hides country UI

**Scale/Scope**: ~12 countries, ~25 MNO rows; Register, Dashboard, NewInvoice, Payments, PayPage providers, Reports, seed defaults config

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Internal-First Simplicity | PASS | Catalog + form UX for existing merchant SPA; no SaaS billing |
| II. System Isolation | PASS | Wallets remain per-system; catalog is shared reference data only |
| III. Idempotent Payment Processing | PASS | No change to idempotency keys / ledger |
| IV. Reliable External Relay | PASS | Same PawaPay gateway; better correspondent selection accuracy |
| V. Observability & Auditability | PASS | Existing tx/wallet history unchanged |
| Security & Compliance | PASS | No new secrets; amounts still minor units |
| Spec-driven workflow | PASS | Artifacts under `specs/005-pawapay-wallets-forms/` |

**Gate result**: PASS — proceed to Phase 0/1.

### Post-design Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I–V + Security | PASS | Contracts are catalog + UI form contracts; no payment rule change |
| Spec-driven | PASS | research / data-model / contracts / quickstart produced |

**Gate result (post Phase 1)**: PASS.

## Project Structure

### Documentation (this feature)

```text
specs/005-pawapay-wallets-forms/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── country-mno-catalog.md
└── tasks.md              # (/speckit-tasks — not created here)
```

### Source Code (repository root)

```text
config/wallet_seed_defaults.json   # Expand Zambia (+ optional other defaults)
src/pay/mod.rs                     # Replace sparse providers_for_country with catalog
src/catalog/                       # NEW optional module OR extend pay/ with catalog.rs
src/api/systems.rs                 # Register default enabled_countries = ["ZM"]
frontend/src/
├── lib/catalog.ts                 # TS mirror of catalog (flags, labels, codes)
├── components/CountrySelect.tsx   # Flagged country dropdown
├── components/ProviderSelect.tsx  # Friendly MNO dropdown
├── pages/RegisterPage.tsx         # Remove countries field
├── pages/DashboardPage.tsx        # Balances + MNO chips/labels
├── pages/NewInvoicePage.tsx       # Country → currency → create
├── pages/PaymentsPage.tsx         # Country → currency → provider
├── pages/ReportsPage.tsx          # DESIGN.md restyle
└── pages/PayPage.tsx              # Providers from catalog (via API)
```

**Structure Decision**: Shared catalog in Rust (API/pay) + TypeScript mirror for SPA forms; no new microservice.

## Complexity Tracking

> No constitution violations requiring justification.
