# Feature Specification: Modernize Merchant & Pay UI (Coinbase Design System)

**Feature Branch**: `004-modernize-merchant-and`

**Created**: 2026-07-23

**Status**: Draft

**Input**: User description: "the ui is very basic i need you to modernise it" using `frontend/DESIGN.md` (Coinbase design analysis) as the visual source of truth

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Cohesive design system applied (Priority: P1)

A merchant opens the Relay SPA (home, sign-in, register, dashboard shell) and immediately recognizes a modern, institutional financial UI: Coinbase Blue as the single brand voltage on primary actions, calm display typography, white canvas with soft gray bands, and hairline structure — not a sparse default form layout.

**Why this priority**: Visual language is the core request; without tokens and shell polish, other screens stay inconsistent.

**Independent Test**: Load `/`, `/login`, `/register`, and `/dashboard` (authenticated) and verify colors, type, buttons, and nav match DESIGN.md tokens.

**Acceptance Scenarios**:

1. **Given** a visitor on `/`, **When** the page loads, **Then** primary CTAs use Coinbase Blue (`#0052ff`), display text uses calm weight (~400), and the canvas is white with soft atmospheric bands — not the prior Tekrem/ERP purple-blue theme.
2. **Given** a signed-in merchant, **When** they use the retractable sidebar, **Then** the shell uses design-system surfaces (hairlines, soft elevation, primary active states) while preserving collapse/expand behavior.
3. **Given** DESIGN.md tokens, **When** an engineer inspects CSS variables / Tailwind theme, **Then** primary, ink, body, muted, hairline, canvas, and surface-* tokens map 1:1 to documented colors.

---

### User Story 2 - Merchant workspace screens feel product-grade (Priority: P1)

A merchant completes core workspace tasks (overview, invoices list/detail/create, payouts, transactions, webhooks, reports) on screens that use consistent page headers, data tables as asset-row style lists, status pills, and empty states — not bare HTML tables with sparse chrome.

**Why this priority**: These are the daily-use surfaces; modernization must land here to feel real.

**Independent Test**: Walk each authenticated route and confirm shared page chrome + table/form patterns.

**Acceptance Scenarios**:

1. **Given** the invoices list, **When** invoices exist, **Then** rows use hairline separators, status badges with semantic colors where applicable, and a clear primary “New invoice” CTA.
2. **Given** an empty list (invoices/transactions/webhooks), **When** the page loads, **Then** a composed empty state with title, short lede, and optional CTA appears — not only “No rows”.
3. **Given** forms (login, register, new invoice, payout, webhooks), **When** rendered, **Then** fields use design-system inputs/labels and primary pill CTAs.

---

### User Story 3 - Public pay page matches brand confidence (Priority: P2)

A payer opens `/pay/{reference}` and sees a focused checkout: brand mark, amount as a number-display hero, clean form, success/error states with calm institutional styling (including optional dark elevated card treatment from DESIGN.md product-ui cards).

**Why this priority**: Customer-facing; high trust impact, but narrower surface than the merchant app.

**Independent Test**: Open pay page for open/paid/expired references and verify layout + states.

**Acceptance Scenarios**:

1. **Given** an open invoice, **When** the pay page loads, **Then** amount is visually dominant, form is compact, and primary Pay CTA is Coinbase Blue pill.
2. **Given** paid/expired/not-found, **When** the page loads, **Then** status composition uses design-system typography and muted body copy without clutter.

---

### User Story 4 - Admin backoffice stays coherent (Priority: P3)

A platform admin at `/admin` sees the same design system for login and systems list/detail — no orphaned basic cards.

**Why this priority**: Lower traffic than merchant UI but must not look like a different product.

**Independent Test**: Unlock admin, list systems, open detail.

**Acceptance Scenarios**:

1. **Given** admin login, **When** signed in, **Then** tables and headers match merchant workspace patterns.

---

### Edge Cases

- Sidebar collapsed: icons remain readable; brand mark (favicon) used when wordmark does not fit.
- Mobile: drawer/sidebar patterns remain usable; primary CTAs remain thumb-reachable; no horizontal overflow.
- Reduced motion: intentional UI motion (if any) respects `prefers-reduced-motion`.
- Existing Tekrem logo assets remain unless replaced; primary *color* follows DESIGN.md (Coinbase Blue), not ERP brandBlue.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: UI MUST adopt `frontend/DESIGN.md` as the visual source of truth (colors, typography scale, radius, spacing, component recipes).
- **FR-002**: Tailwind / CSS variables MUST be remapped from the prior Tekrem/ERP theme to DESIGN.md Coinbase tokens (primary `#0052ff`, ink `#0a0b0d`, canvas white, soft surfaces, hairlines, semantic up/down).
- **FR-003**: Typography MUST use Coinbase Display / Sans equivalents via licensed or close open substitutes (document chosen web-safe/Google substitutes if Coinbase fonts are unavailable).
- **FR-004**: Primary buttons MUST use pill radius and Coinbase Blue; secondary/outline variants MUST follow DESIGN.md recipes.
- **FR-005**: Merchant shell (sidebar + top bar) MUST be restyled without removing retractable sidebar behavior or route structure.
- **FR-006**: Shared page patterns MUST include: page title + lede, primary action placement, table/list chrome, empty states, form field groups, error text using semantic-down where appropriate.
- **FR-007**: Pay page MUST be restyled to the same system with amount emphasis and status compositions.
- **FR-008**: Admin pages MUST use the same token set and shared chrome.
- **FR-009**: Modernization MUST NOT change API contracts, auth flows (username/password), or payment business logic.
- **FR-010**: `frontend/DESIGN.md` MUST remain the referenced design doc (update header notes if theme switch from ERP is documented).

### Key Entities *(include if feature involves data)*

- **Design Token Set**: Named colors, type roles, radii, spacing from DESIGN.md mapped into CSS/Tailwind.
- **UI Surface**: Named app routes/screens in scope (home, auth, merchant workspace, pay, admin).
- **Component Recipe**: Button, input, card, table row, badge, nav item, empty state mapped from DESIGN.md `components.*`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of in-scope SPA routes use the new token set (no residual ERP primary `#0D3A69` / mauve secondary as brand voltage).
- **SC-002**: A reviewer can match primary CTA, hairline, and body text colors to DESIGN.md within a 5-minute checklist.
- **SC-003**: Merchant can complete login → overview → create invoice → view list without layout breakage on desktop (≥1280px) and mobile (≤390px width).
- **SC-004**: Pay page open/success/error states remain fully usable after restyle (no missing fields or broken submit).
- **SC-005**: Frontend production build succeeds (`npm run build`) with no TypeScript errors.

## Assumptions

- DESIGN.md Coinbase analysis is authoritative for visual language; Tekrem logos may remain as product marks.
- No new backend endpoints required; this is a frontend presentation feature.
- Licensed Coinbase fonts may be unavailable — use documented close substitutes (e.g. Inter/system for Sans, a restrained display face or Inter for Display) while preserving size/weight/letter-spacing from DESIGN.md.
- Existing React + shadcn + Tailwind stack is retained; theme tokens and component classNames are updated rather than introducing a new UI framework.
- Dark editorial hero bands from DESIGN.md are optional accents (home/pay), not a global dark-mode product requirement for v1.
