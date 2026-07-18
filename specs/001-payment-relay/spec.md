# Feature Specification: Internal Payment Relay

**Feature Branch**: `001-payment-relay`

**Created**: 2026-07-18

**Status**: Draft

**Input**: User description: "Lightweight internal payment relay for owned systems — register systems, route payments to external gateways, track per-system wallets, and notify via webhooks."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Register an Internal System (Priority: P1)

An platform operator registers a new internal system (e.g., E-Commerce, Subscriptions) so it
can authenticate and process payments through the relay.

**Why this priority**: No system can use the relay without registration and credentials.

**Independent Test**: Register a system via the registration API and receive a unique system ID,
prefix, and API key; verify the system appears in lookups and rejects invalid credentials.

**Acceptance Scenarios**:

1. **Given** valid registration input (name, prefix, enabled countries, optional webhook URL),
   **When** the operator submits registration, **Then** the relay returns a system ID, prefix,
   and one-time API key.
2. **Given** a registered system, **When** a request uses an invalid API key, **Then** the
   relay rejects the request with an authentication error.
3. **Given** duplicate prefix or invalid country list, **When** registration is attempted,
   **Then** the relay returns a validation error without creating partial records.

---

### User Story 2 - Process a Payment (Priority: P1)

An internal system submits a payment for a specific amount, currency, and country; the relay
forwards it to an external gateway and returns a definitive status.

**Why this priority**: Payment processing is the core value of the relay.

**Independent Test**: Submit a payment with a valid API key and idempotency key; receive
completed, failed, or pending status with an external reference suitable for reconciliation.

**Acceptance Scenarios**:

1. **Given** a registered system and valid payment request, **When** the gateway succeeds,
   **Then** the relay returns completed status with a gateway reference and records the
   transaction.
2. **Given** a transient gateway failure, **When** the relay retries up to three times,
   **Then** the final outcome reflects success or failure after retries are exhausted.
3. **Given** the same idempotency key resubmitted, **When** the original payment already
   exists, **Then** the relay returns the original result without duplicate processing.
4. **Given** an amount in minor units and supported currency/country for the system,
   **When** payment is processed, **Then** wallet balance reflects the outcome where applicable.

---

### User Story 3 - Receive Payment Webhooks (Priority: P2)

When a payment reaches a terminal state, the originating internal system receives a webhook at
its configured URL.

**Why this priority**: Downstream systems need asynchronous notification to update orders,
subscriptions, or ledgers without polling.

**Independent Test**: Complete a payment for a system with a webhook URL; verify the webhook
payload includes payment ID, external ID, status, and enough detail to reconcile.

**Acceptance Scenarios**:

1. **Given** a system with a webhook URL and a completed payment, **When** processing finishes,
   **Then** the relay delivers a webhook with status and identifiers.
2. **Given** webhook delivery failure, **When** retries are attempted, **Then** delivery attempts
   are logged and eventual success or failure is auditable.

---

### User Story 4 - Query Wallets and Balances (Priority: P2)

An internal system checks wallet balances per country/currency to reconcile available funds.

**Why this priority**: Operators and systems need visibility into balances held per market.

**Independent Test**: After payments, query wallets for a system and see country-specific
balances in the correct currency.

**Acceptance Scenarios**:

1. **Given** a system's first payment in a country/currency, **When** the wallet did not exist,
   **Then** the relay auto-creates the wallet and returns accurate balance.
2. **Given** multiple wallets across countries, **When** the system queries wallets,
   **Then** only that system's wallets are returned.

---

### User Story 5 - Trace External IDs (Priority: P3)

Payments use a standardized external ID format so internal systems and support can correlate
records across systems and the relay.

**Why this priority**: Supports operations, debugging, and cross-system reconciliation.

**Independent Test**: Generate or accept external IDs following the prefix-based format and
retrieve transactions by external ID.

**Acceptance Scenarios**:

1. **Given** a system prefix, **When** an external ID is generated or validated,
   **Then** it follows `{PREFIX}_{SYSTEM_SHORT}_{DATE}_{RANDOM}` semantics.
2. **Given** a known external ID, **When** queried, **Then** the matching transaction is returned
   for the owning system only.

---

### Edge Cases

- What happens when a payment is submitted for a country not enabled for the system?
- How does the system handle gateway timeout mid-flight (ambiguous state)?
- What happens when webhook URL is unreachable after all retries?
- How are zero-amount or negative-amount payments rejected?
- What happens when idempotency key matches but request body differs?
- How does the relay behave when the database is temporarily unavailable?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST allow registration of internal systems with name, prefix, enabled
  countries, optional webhook URL, and auto-generated API key.
- **FR-002**: System MUST authenticate API requests using per-system API keys over HTTPS.
- **FR-003**: System MUST accept payment requests with system ID, external ID, amount (minor
  units), currency, country, payment method reference, and idempotency key.
- **FR-004**: System MUST forward payments to configured external payment gateways.
- **FR-005**: System MUST retry failed gateway calls up to three times for transient errors.
- **FR-006**: System MUST enforce idempotency so duplicate keys never create duplicate charges.
- **FR-007**: System MUST persist transaction status (pending, completed, failed) and gateway
  reference.
- **FR-008**: System MUST auto-create country/currency wallets on first use per system.
- **FR-009**: System MUST maintain wallet balances updated by payment outcomes.
- **FR-010**: System MUST send webhooks to the system's configured URL on terminal payment states.
- **FR-011**: System MUST expose wallet balance queries scoped to the authenticated system.
- **FR-012**: System MUST support external ID format using system prefix and correlation fields.
- **FR-013**: System MUST reject payments for countries outside a system's enabled list.
- **FR-014**: System MUST NOT expose multi-tenant SaaS features (public signup, billing plans,
  customer dashboards) in scope of this feature.

### Key Entities

- **System**: Registered internal consumer with ID, name, prefix, enabled countries, webhook URL,
  and API credentials.
- **Wallet**: Per-system, per-country (and currency) balance container with transaction history
  linkage.
- **Transaction**: Payment attempt with external ID, idempotency key, amount, status, gateway
  reference, and timestamps.
- **Payment Method**: Tokenized reference to payer instrument (no raw card storage by relay).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An operator can register a system and process a first successful payment in under
  10 minutes using documented steps.
- **SC-002**: 100% of duplicate submissions with the same idempotency key return the original
  outcome without duplicate ledger entries (verified in test scenarios).
- **SC-003**: At least 95% of webhooks for completed payments are delivered within 30 seconds
  under normal operating conditions.
- **SC-004**: Support staff can locate any payment by external ID or gateway reference in a
  single lookup.
- **SC-005**: Internal systems can query wallet balances that match transaction history for
  reconciliation audits.

## Assumptions

- Consumers are trusted internal systems operated by the same organization (not public third
  parties).
- External payment gateways (e.g., Stripe, Adyen, pawaPay) are configured separately; the relay
  routes to an operator-selected default or configured gateway.
- Payment methods arrive as gateway tokens, not raw PAN data.
- Single-region deployment is acceptable for initial release; high-availability is a future
  concern unless amended.
- English-only operator documentation is sufficient for v1.
