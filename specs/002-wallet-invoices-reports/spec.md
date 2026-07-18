# Feature Specification: Wallet Seeding, Invoices & Reports

**Feature Branch**: `002-wallet-invoices-reports`

**Created**: 2026-07-18

**Status**: Draft

**Input**: User description: "All new systems/users should have wallets seeded automatically. Need invoices with QR codes and reporting."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Auto-Seed Wallets on System Registration (Priority: P1)

When an operator registers a new internal system, the relay automatically creates and funds
starter wallets for every enabled country so the system can process payments immediately without
manual SQL seeding.

**Why this priority**: Manual wallet seeding blocks onboarding and caused the current operational
gap for new systems.

**Independent Test**: Register a new system with multiple enabled countries; verify wallets exist
with configured starting balances for each country/currency pair before any payment is attempted.

**Acceptance Scenarios**:

1. **Given** default seed configuration (e.g., 100,000 minor units per country),
   **When** a system is registered with enabled countries `["ZM", "US"]`,
   **Then** wallets are created for each country with the correct default currency and starting balance.
2. **Given** a registered system,
   **When** an operator queries wallets immediately after registration,
   **Then** all expected country wallets appear without manual intervention.
3. **Given** seed configuration sets zero balance for a country,
   **When** registration completes,
   **Then** the wallet exists with zero balance (not omitted).

---

### User Story 2 - Generate Invoice with QR Code (Priority: P1)

An internal system creates an invoice for a payer amount; the relay returns invoice details plus
a QR code the payer can scan to initiate or reference payment (mobile money / checkout flow).

**Why this priority**: QR-based invoices are a core collection workflow requested for field and
counter sales.

**Independent Test**: Create invoice via API; receive invoice ID, amount, expiry, and QR payload
(image or encoded string); scan/decode yields payment reference linkable to relay payment.

**Acceptance Scenarios**:

1. **Given** a registered system and valid invoice request (amount, currency, country, description),
   **When** the system creates an invoice,
   **Then** the relay returns invoice metadata and a QR code representing the payment reference.
2. **Given** an open invoice,
   **When** a payment is completed with matching invoice reference,
   **Then** the invoice status updates to paid and links to the transaction.
3. **Given** an invoice past its expiry time,
   **When** payment is attempted,
   **Then** the relay rejects payment with a clear expired-invoice error.
4. **Given** a paid invoice,
   **When** duplicate payment is attempted against the same invoice,
   **Then** the relay returns the original payment result (idempotent).

---

### User Story 3 - Transaction & Wallet Reports (Priority: P2)

Operators and internal systems generate reports summarizing payments, wallet balances, and invoice
status over a date range for reconciliation and management.

**Why this priority**: Reporting supports finance ops after collection workflows (invoices) and
wallet float management (seeding) are in place.

**Independent Test**: Request a report for a date range; receive aggregated totals, counts by
status, and line-item detail export for the authenticated system only.

**Acceptance Scenarios**:

1. **Given** a system with transaction history,
   **When** the operator requests a transaction report for a date range,
   **Then** the relay returns totals (count, volume by status) and optional detailed rows.
2. **Given** multiple wallets across countries,
   **When** a wallet balance report is requested,
   **Then** current balances and period net change per wallet are returned.
3. **Given** invoices created in a period,
   **When** an invoice report is requested,
   **Then** counts and amounts by status (open, paid, expired, cancelled) are returned.
4. **Given** a report request for system A's credentials,
   **When** executed,
   **Then** no data from system B is included.

---

### User Story 4 - Operator Default Seed Configuration (Priority: P2)

Platform operators configure default starting balances per country/currency applied to all newly
registered systems (with optional per-system overrides).

**Why this priority**: Seed amounts vary by market; configuration avoids code changes for balance
adjustments.

**Independent Test**: Update default seed config; register new system; verify new balances match
updated config.

**Acceptance Scenarios**:

1. **Given** updated default seed map (country → currency → amount),
   **When** a new system registers,
   **Then** wallets use the updated amounts.
2. **Given** per-system override in registration request,
   **When** provided,
   **Then** overrides take precedence over global defaults for specified countries.

---

### User Story 5 - Export Reports for Finance (Priority: P3)

Finance users export report data in a standard tabular format suitable for spreadsheets and
 downstream accounting tools.

**Why this priority**: Enables month-end close without building a separate BI tool in v1.

**Independent Test**: Request CSV export for transaction report; file contains headers and rows
matching API detail view.

**Acceptance Scenarios**:

1. **Given** a transaction report with detail rows,
   **When** export format CSV is requested,
   **Then** a downloadable CSV is returned with consistent column headers.
2. **Given** large result sets,
   **When** export exceeds row limit,
   **Then** the relay paginates or streams with clear limits documented in response.

---

### Edge Cases

- What happens when enabled country has no default currency mapping in seed config?
- How are partial payments against an invoice handled (if allowed)?
- What if registration succeeds but wallet seed partially fails (some countries seeded, others not)?
- How are cancelled invoices distinguished from expired?
- Report requests spanning empty date ranges?
- QR code generation when invoice amount is below gateway minimum?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST auto-create wallets for each enabled country upon successful registration.
- **FR-002**: System MUST apply configurable default starting balance per country/currency when seeding wallets.
- **FR-003**: System MUST support optional per-registration wallet seed overrides.
- **FR-004**: System MUST expose invoice creation with amount, currency, country, description, and expiry.
- **FR-005**: System MUST generate a QR code encoding invoice payment reference (URL or structured payload).
- **FR-006**: System MUST track invoice lifecycle states: open, paid, expired, cancelled.
- **FR-007**: System MUST link paid invoices to the completing transaction.
- **FR-008**: System MUST reject payments against expired or cancelled invoices.
- **FR-009**: System MUST provide transaction summary reports filterable by date range and status.
- **FR-010**: System MUST provide wallet balance reports with period activity summary.
- **FR-011**: System MUST provide invoice summary reports filterable by date range and status.
- **FR-012**: System MUST scope all reports and invoices to the authenticated system.
- **FR-013**: System MUST support CSV export for report detail views.
- **FR-014**: System MUST record wallet seed events in an auditable log for reconciliation.
- **FR-015**: System MUST maintain backward compatibility with existing payment API (invoice reference
  optional on payment requests).

### Key Entities

- **WalletSeedConfig**: Default and override map of country/currency → starting balance (minor units).
- **WalletSeedEvent**: Audit record of initial funding on registration (system, wallet, amount, timestamp).
- **Invoice**: Billable request with reference, amount, status, expiry, QR payload, linked transaction.
- **Report**: Parameterized query result (summary + optional detail rows) for transactions, wallets, or invoices.
- **ReportExport**: Generated CSV (or future formats) from report detail.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: New systems receive all enabled-country wallets within 5 seconds of registration (100% in test scenarios).
- **SC-002**: Operators can create an invoice and obtain a scannable QR code in under 30 seconds.
- **SC-003**: 95% of invoice lookups by reference return status in under 1 second.
- **SC-004**: Transaction reports for a 30-day window generate in under 3 seconds for up to 10,000 rows.
- **SC-005**: CSV exports match API detail data with zero column mismatches in validation tests.
- **SC-006**: Zero manual SQL steps required to onboard a new system for standard markets.

## Assumptions

- Default currency per country follows a static operator-maintained map (e.g., ZM→ZMW, US→USD).
- QR codes encode HTTPS URLs or EMV-style payloads readable by target mobile-money apps; v1 prioritizes URL-based deep links with invoice reference.
- Reports are API-first (JSON + CSV); no graphical dashboard in v1.
- Invoice partial payment is out of scope unless amended; full amount only.
- Global seed configuration is managed by operators via config file or admin API (not public self-service).
- Existing relay authentication (API keys) applies to invoice and report endpoints.

## Dependencies

- Builds on feature `001-payment-relay` (systems, wallets, transactions, auth).
- pawaPay (or configured gateway) minimum amounts apply when invoice payment is executed through existing payment flow.
