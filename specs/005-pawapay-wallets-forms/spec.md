# Feature Specification: PawaPay Wallets, Country Catalog & Forms

**Feature Branch**: `005-pawapay-wallets-forms`

**Created**: 2026-07-23

**Status**: Draft

**Input**: User description: Supported PawaPay countries/wallets (Benin, Cameroon, Côte d'Ivoire, DRC CDF+USD, Gabon, Kenya, Congo-Brazzaville, Rwanda, Senegal, Sierra Leone, Uganda, Zambia); remove country/wallet selection from register and default Zambia (hidden); show balances per enabled country with MNOs; proper country/provider dropdowns with flags and validation; auto-set currency from country; users must not need raw backend correspondent codes; restyle reports to Coinbase DESIGN.md / shadcn.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Register defaults to Zambia without country UI (Priority: P1)

A new merchant registers with username/password and business fields only. The system enables **Zambia (ZM)** by default and seeds the Zambia wallet — no country or wallet picker appears on the register form.

**Why this priority**: Removes a confusing free-text country field and aligns onboarding with the primary market.

**Independent Test**: Register a system; confirm API/system has `enabled_countries` containing Zambia only; ZMW wallet exists; register form has no country field.

**Acceptance Scenarios**:

1. **Given** a visitor on `/register`, **When** the form renders, **Then** there is no country/wallet multi-select or comma-separated ISO field.
2. **Given** a successful registration, **When** wallets are listed, **Then** at least one Zambia/ZMW wallet exists and enabled countries default to Zambia.

---

### User Story 2 - Overview shows balances and MNOs per country (Priority: P1)

A signed-in merchant opens Overview and sees each enabled-country wallet balance with the human-readable MNOs available in that country (from the PawaPay catalog), so they can cross-check balances against operators without knowing correspondent codes.

**Why this priority**: Core operational visibility for multi-country wallets.

**Independent Test**: With a Zambia-enabled system, Overview shows ZMW balance and Airtel / MTN / Zamtel labels (not raw codes as the only label).

**Acceptance Scenarios**:

1. **Given** wallets for enabled countries, **When** Overview loads, **Then** each row/card shows country name, currency, balance, and MNO display names.
2. **Given** DRC with CDF and USD wallets (if enabled), **When** Overview loads, **Then** both currency wallets appear distinctly.

---

### User Story 3 - Invoice and payout forms use catalog dropdowns (Priority: P1)

On New Invoice and Send Payout, the merchant picks a **country** from a flagged dropdown of enabled countries; **currency** is derived and not freely typed; **provider** is a dropdown of human MNO labels for that country. The SPA maps selections to backend ISO country + currency + correspondent codes.

**Why this priority**: Prevents invalid payloads and removes need for merchants to memorize PawaPay codes.

**Independent Test**: Create invoice for Zambia; submit payout selecting MTN; network/API uses `ZM`/`ZMW`/`MTN_MOMO_ZMB` (or project’s stored ISO convention) without showing those codes as the primary UI labels.

**Acceptance Scenarios**:

1. **Given** New Invoice, **When** country changes, **Then** currency updates to the catalog currency for that country (DRC offers CDF vs USD where applicable).
2. **Given** Payout form, **When** country is selected, **Then** provider options are only that country’s MNOs with friendly labels.
3. **Given** invalid/empty required fields, **When** submit is attempted, **Then** validation errors appear and no request is sent.

---

### User Story 4 - Shared PawaPay country/MNO catalog (Priority: P1)

The product maintains a single catalog of supported countries and MNOs matching PawaPay docs for the markets the user listed, used by register defaults, Overview, invoice/payout/pay-page provider lists, and validation.

**Why this priority**: One source of truth prevents drift between UI and gateway.

**Independent Test**: Catalog includes user’s markets with correct currencies and MNO labels/codes; Zambia default resolves from catalog.

**Acceptance Scenarios**:

1. **Given** the catalog, **When** Zambia is resolved, **Then** MNOs include Airtel, MTN, Zamtel with PawaPay correspondent codes.
2. **Given** a country not in catalog, **When** used in forms, **Then** it is rejected or unavailable in dropdowns.

---

### User Story 5 - Reports UI matches DESIGN.md (Priority: P2)

Reports filters and summary panels use the Coinbase DESIGN.md / shadcn patterns (page title, pill CTAs, asset-row tables, calm cards) — not a sparse ugly form layout.

**Why this priority**: Visual consistency after 004; secondary to catalog/forms correctness.

**Independent Test**: Open `/reports`; UI matches DESIGN.md tokens and component recipes.

**Acceptance Scenarios**:

1. **Given** `/reports`, **When** the page loads, **Then** filter chrome and results use design-system cards/tables/buttons.
2. **Given** a report run, **When** results render, **Then** amounts use number/mono styling and semantic colors where appropriate.

---

### Edge Cases

- DRC dual currency (CDF + USD): currency selector appears only when country has multiple currencies.
- Country codes: UI may show ISO-2 flags/labels while backend continues existing ISO-2 (`ZM`) unless a migration to ISO-3 is explicitly included — catalog must map correctly to PawaPay correspondents.
- Systems already registered with other countries remain valid; register change is default-only for new registrations.
- Pay page provider list must stay consistent with the same catalog.
- Phone validation should match selected country dialing expectations where feasible (at least non-empty + basic format).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Maintain a PawaPay-aligned country/MNO catalog covering at least: Benin, Cameroon, Côte d'Ivoire, DRC (CDF+USD), Gabon, Kenya, Republic of the Congo, Rwanda, Senegal, Sierra Leone, Uganda, Zambia — with display name, ISO code(s), currency(ies), MNO label, and correspondent code.
- **FR-002**: Register MUST NOT expose country/wallet selection; MUST default `enabled_countries` to Zambia and seed Zambia wallet(s).
- **FR-003**: Overview MUST show wallet balances per country/currency and list enabled MNOs for that country from the catalog.
- **FR-004**: Invoice and payout forms MUST use country dropdowns (flags + names); currency MUST be derived from country (or explicit dual-currency choice for DRC); provider MUST be a friendly MNO dropdown mapped to correspondent codes for the API.
- **FR-005**: Users MUST NOT be required to type raw correspondent codes or free-form ISO lists for normal flows.
- **FR-006**: Forms MUST validate required fields and reject inconsistent country/currency/provider combinations.
- **FR-007**: Reports page MUST be restyled to `frontend/DESIGN.md` + existing shadcn primitives.
- **FR-008**: Pay-page provider options MUST use the same catalog for invoice country.
- **FR-009**: No change to payment idempotency or ledger rules; catalog/UI mapping only plus register default behavior.

### Key Entities

- **CountryCatalogEntry**: country identity, currencies, MNOs.
- **MnoProvider**: display label + correspondent code + country scope.
- **Wallet** (existing): balance per system/country/currency.
- **ReportSurface**: reports UI composition (no new domain entity).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: New registrations never require country input and always enable Zambia.
- **SC-002**: Overview lists balance + MNO labels for every wallet of the system.
- **SC-003**: Invoice/payout can be completed using only dropdown selections (no raw provider codes typed).
- **SC-004**: Invalid country/currency/provider combos are blocked client-side and/or server-side.
- **SC-005**: Reports page matches DESIGN.md visual checklist (primary pill CTA, hairlines, title/lede).
- **SC-006**: `npm run build` and relevant Rust tests pass.

## Assumptions

- User’s listed markets/MNOs are the v1 catalog scope (not necessarily every PawaPay market worldwide).
- PawaPay docs (https://docs.pawapay.io/using_the_api) are authoritative for correspondent codes and currencies.
- Existing Relay wallet country storage remains ISO-2 (`ZM`) unless plan research chooses otherwise; catalog maps ISO-2 ↔ PawaPay ISO-3 as needed.
- Enabling additional countries beyond Zambia for a system may remain admin/API-only in v1 unless a settings UI is added later.
- DESIGN.md / shadcn stack from feature 004 remains SoT for Reports visuals.
