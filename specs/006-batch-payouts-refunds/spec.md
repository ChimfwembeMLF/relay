# Feature Specification: Batch Payouts, Refunds & Full-Country Registration

**Feature Branch**: `006-batch-payouts-refunds`

**Created**: 2026-07-23

**Status**: Draft

**Input**: User description: "I need batch payouts, also add refunds, and enable all countries on registration"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Registration enables all supported countries (Priority: P1)

A new merchant registers with business and login fields only (still no country picker). The system enables **every country in the product catalog** and seeds a wallet for each country/currency pair (including DRC CDF and USD), so Overview and forms immediately offer the full market set.

**Why this priority**: Reverses the Zambia-only default from 005; unlocks multi-market invoices and payouts without admin follow-up.

**Independent Test**: Register a system; enabled countries match the full catalog; wallets exist for each catalog currency (e.g. ZM/ZMW, KE/KES, CD/CDF, CD/USD); register UI still has no country field.

**Acceptance Scenarios**:

1. **Given** a visitor on register, **When** the form renders, **Then** there is still no country/wallet picker.
2. **Given** successful registration, **When** the system and wallets are inspected, **Then** all catalog countries are enabled and each required currency wallet exists with the configured seed balance (default zero unless overridden).
3. **Given** a client still sends a partial `enabled_countries` list, **When** public register runs, **Then** the server ignores it and enables the full catalog set.

---

### User Story 2 - Batch payouts (Priority: P1)

A merchant (dashboard or API) submits **many payouts in one batch**—each line has amount, country, currency, recipient phone, and mobile network. The system processes each line independently, returns per-line success/failure, and never double-pays when the same batch or line is safely retried.

**Why this priority**: Core new capability for payroll-style and mass disbursement workflows.

**Independent Test**: Submit a batch of at least three valid payout lines (plus one invalid); receive mixed results; wallet debits only for successful lines; retry with the same batch idempotency does not duplicate successful lines.

**Acceptance Scenarios**:

1. **Given** a merchant with sufficient wallet balance, **When** they submit a batch of valid payout lines, **Then** each successful line becomes a completed payout transaction and the corresponding wallet is debited.
2. **Given** a batch with some invalid lines (bad phone, unsupported provider, insufficient funds for that line), **When** processing finishes, **Then** valid lines can succeed while invalid ones fail with clear per-line errors (partial success).
3. **Given** a dashboard batch payout screen, **When** the merchant adds multiple rows (or pastes a simple table/CSV of lines), **Then** they can review and submit without sending one request per recipient.
4. **Given** the same batch idempotency key is reused with the same payload, **When** the request is retried, **Then** the original batch result is returned without creating duplicate payouts.

---

### User Story 3 - Refunds on paid collections (Priority: P1)

A merchant refunds a **paid invoice** (full or partial). Relay debits the merchant’s matching wallet and sends funds back to the customer’s mobile money destination (from the original collection when available, otherwise merchant-supplied). Refunds are idempotent and cannot exceed the remaining refundable amount on that invoice.

**Why this priority**: Required for customer support and chargeback-style corrections after successful pay-page or deposit collections.

**Independent Test**: Pay an invoice; issue a partial refund; invoice remaining refundable decreases; wallet debit matches refund; second identical refund request is idempotent; refund above remaining amount is rejected.

**Acceptance Scenarios**:

1. **Given** a paid invoice with known payer phone/provider, **When** the merchant requests a full refund, **Then** the customer receives a mobile-money payout for the invoice amount and the merchant wallet is debited.
2. **Given** a paid invoice, **When** the merchant requests a partial refund, **Then** only that amount is sent and further refunds are limited to the remainder.
3. **Given** an open, cancelled, expired, or already fully refunded invoice, **When** a refund is attempted, **Then** it is rejected with a clear reason.
4. **Given** dashboard invoice detail, **When** the invoice is paid and refundable, **Then** the merchant can start a refund (amount + destination confirmation) without leaving the product UI.

---

### User Story 4 - API and operational visibility (Priority: P2)

Integrators can create batches and refunds via the same auth model as single payouts (API key or session). Merchants can see batch and refund outcomes in transactions / invoice detail with statuses suitable for support.

**Why this priority**: SDKs and ops need parity with the UI; secondary to core money movement.

**Independent Test**: Create batch and refund with API key; list/retrieve results; confirm webhooks (when configured) fire for resulting payout transactions consistent with existing payout events.

**Acceptance Scenarios**:

1. **Given** API credentials, **When** a batch or refund is created, **Then** the response includes stable identifiers and per-item/refund status.
2. **Given** webhook URL configured, **When** a batch line or refund payout completes, **Then** the merchant system receives the existing payout-style notification for that transaction.

---

### Edge Cases

- Batch where total requested amount exceeds wallet balance: fail lines as funds run out (or fail the whole batch only if pre-check cannot allocate)—prefer **per-line** insufficient-funds failures after sequential processing order.
- Duplicate phones in one batch: allowed; each line is a separate payout.
- Refund when original payer MSISDN/provider unknown: merchant must supply destination; otherwise reject.
- Concurrent refunds on the same invoice: must not exceed remaining refundable amount (last conflicting request fails safely).
- Catalog country added later: new registrations get the expanded set; existing systems are unchanged by this feature unless separately migrated (out of scope).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Public registration MUST enable all countries present in the product country/MNO catalog and seed wallets for every catalog currency for those countries (including dual-currency DRC).
- **FR-002**: Public registration MUST continue to omit country/wallet selection from the UI and MUST ignore client-supplied enabled-country lists that would shrink the full catalog set.
- **FR-003**: Merchants MUST be able to submit a batch of payouts (multiple recipients) in one operation via dashboard and API.
- **FR-004**: Each batch line MUST carry the same commercial fields as a single payout (amount, country, currency, phone, provider/network) and MUST be validated against the catalog and enabled countries.
- **FR-005**: Batch processing MUST support partial success with per-line outcomes; successful lines MUST debit the correct wallet; failed lines MUST NOT debit.
- **FR-006**: Batches MUST be idempotent under a batch-level idempotency key (same key + same body → same result; same key + different body → conflict).
- **FR-007**: Merchants MUST be able to refund a paid invoice in full or in part, limited by remaining refundable amount.
- **FR-008**: A refund MUST debit the merchant wallet for the invoice’s country/currency and MUST send funds to the customer via mobile money (reuse original collection destination when available).
- **FR-009**: Refunds MUST be idempotent and MUST reject refunds on non-refundable invoice states or amounts above the remaining refundable balance.
- **FR-010**: Dashboard MUST provide a batch payout entry experience (multi-row and/or pasteable line list) and a refund action on eligible invoice detail.
- **FR-011**: Batch line payouts and refund payouts MUST appear in the merchant’s transaction history with clear linkage to batch id and/or invoice reference where applicable.
- **FR-012**: Existing single-payout behavior MUST remain available and unchanged for one-off sends.

### Key Entities

- **Batch payout**: A merchant-initiated group of payout lines sharing one batch identity and idempotency key; contains many **batch lines**.
- **Batch line**: One recipient payout attempt within a batch (amount, country, currency, destination, status, linked transaction when created).
- **Refund**: A money-return against a paid invoice (amount, destination, status, linked payout transaction, remaining-refundable tracking on the invoice).
- **Invoice (extended)**: Gains refundable/refunded totals (or equivalent) so remaining refund capacity is enforceable.
- **System (registration)**: `enabled_countries` becomes the full catalog set for new public registrations.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: After registration, merchants see wallets for 100% of catalog countries/currencies without any country selection step.
- **SC-002**: A merchant can disburse to at least 20 recipients in one batch submission in under 5 minutes of operator time (excluding gateway latency).
- **SC-003**: At least 95% of first-attempt batch submissions with valid lines produce correct per-line success/failure without duplicate successful payouts on safe retry.
- **SC-004**: Support can complete a full or partial refund of a paid invoice in under 2 minutes from invoice detail, with wallet and customer destination updated consistently.
- **SC-005**: Attempts to over-refund or refund non-paid invoices are blocked 100% of the time in acceptance testing.

## Assumptions

- “All countries” means the existing PawaPay product catalog from feature 005 (not arbitrary ISO countries).
- Seed balances for newly enabled countries remain **zero** unless `wallet_seeds` overrides are supplied (same seed mechanism as today).
- Existing merchants keep their current `enabled_countries`; this feature does not auto-migrate them to the full catalog.
- Batch input in v1 is multi-row UI + API array; file upload UX is optional nice-to-have if paste/CSV text is enough.
- Refunds are **invoice-based** (paid collections), not arbitrary wallet withdrawals labeled “refund.”
- Refund money movement reuses the existing payout/gateway path (no separate gateway “refund API” required for v1).
- Partial batch success is preferred over all-or-nothing.
- Max batch size will be set in planning to a practical limit (order of tens to low hundreds of lines) to protect the relay.
- Webhooks for batch lines and refunds reuse existing payout transaction webhook semantics.
