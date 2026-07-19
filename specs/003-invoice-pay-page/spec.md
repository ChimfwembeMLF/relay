# Feature Specification: Invoice Pay Page

**Feature Branch**: `003-invoice-pay-page`

**Created**: 2026-07-18

**Status**: Draft

**Input**: User description: "Hosted invoice pay page for QR deep links with payer-facing collect flow"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Payer Views Invoice from QR Link (Priority: P1)

A payer scans the QR code on a receipt or counter display. The relay serves a mobile-friendly
pay page showing the invoice amount, currency, description, expiry, and current status.

**Why this priority**: Feature 002 generates QR URLs pointing to `/pay/{reference}` but no page
exists — the QR workflow is incomplete without this.

**Independent Test**: Create an invoice via API; open `GET /pay/{reference}` in a browser; verify
amount, description, and `open` status render without an API key.

**Acceptance Scenarios**:

1. **Given** an open invoice with reference `INV_RTL_a1b2c3d4`,
   **When** a payer opens `/pay/INV_RTL_a1b2c3d4`,
   **Then** the page displays amount, currency, country, description, and expiry.
2. **Given** a paid invoice,
   **When** the pay URL is opened,
   **Then** the page shows a paid confirmation (no payment form).
3. **Given** an expired or cancelled invoice,
   **When** the pay URL is opened,
   **Then** the page shows a clear non-payable state with no active form.
4. **Given** an unknown reference,
   **When** `/pay/{reference}` is requested,
   **Then** the relay returns a generic not-found page (no invoice metadata leaked).

---

### User Story 2 - Payer Pays from Hosted Page (Priority: P1)

A payer enters their mobile-money phone number on the pay page and submits. The relay initiates
the deposit (same as `POST /invoices/{id}/collect`) and shows success or failure on-page.

**Why this priority**: Completes the end-to-end QR collection flow without requiring the merchant
system to call collect on behalf of the payer.

**Independent Test**: Open pay page for open invoice; submit valid phone; verify invoice becomes
paid, wallet credited, and confirmation shown.

**Acceptance Scenarios**:

1. **Given** an open invoice on the pay page,
   **When** the payer submits a valid mobile-money phone number,
   **Then** the deposit is initiated, invoice becomes `paid`, and a success page is shown.
2. **Given** a duplicate submit (same payer session / idempotency token),
   **When** the form is posted again,
   **Then** the original payment result is shown without double-charging.
3. **Given** gateway deposit failure,
   **When** the payer submits,
   **Then** the page shows a retryable error without marking the invoice paid.
4. **Given** an expired invoice,
   **When** the payer submits payment,
   **Then** the relay rejects with an expired message (no deposit attempted).

---

### User Story 3 - Invoice Paid Webhook (Priority: P2)

When an invoice is paid via the pay page (or API collect), the owning system's configured
webhook URL receives a signed notification with invoice and transaction details.

**Why this priority**: Merchants need async notification when payers complete QR payments without
polling the API.

**Independent Test**: Register system with webhook URL; pay invoice via pay page; verify webhook
delivery with correct payload and signature.

**Acceptance Scenarios**:

1. **Given** a system with `webhook_url` configured,
   **When** an invoice is paid via the pay page,
   **Then** a signed webhook is POSTed with invoice reference, amount, and transaction ID.
2. **Given** webhook delivery fails,
   **When** retries exhaust,
   **Then** the payment remains completed and failure is logged (same policy as payout webhooks).

---

### User Story 4 - Atomic System Registration + Seed (Priority: P3)

System registration and wallet seeding run in a single database transaction so a failed seed
does not leave a registered system without wallets.

**Why this priority**: Closes a gap from Feature 002 where seed runs after system insert.

**Independent Test**: Simulate seed failure; verify no system row or partial wallets remain.

**Acceptance Scenarios**:

1. **Given** valid registration with enabled countries,
   **When** all seeds succeed,
   **Then** system and wallets commit together.
2. **Given** a seed failure for all countries,
   **When** registration is attempted,
   **Then** the entire registration rolls back.

---

### Edge Cases

- Pay page accessed after invoice just expired mid-session: show expired state on next GET or POST.
- Concurrent pay page submit and API collect: idempotency prevents double credit; one wins, other replays.
- Reference enumeration: generic 404 for invalid references; rate-limit POST attempts per IP.
- Production requires HTTPS on `INVOICE_PAY_BASE_URL`; HTTP allowed locally only.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST serve `GET /pay/{reference}` as a mobile-friendly HTML page without API key auth.
- **FR-002**: System MUST resolve invoices by `reference` globally (reference is unique across systems).
- **FR-003**: Pay page MUST display amount, currency, country, description, expiry, and status for valid references.
- **FR-004**: System MUST serve `POST /pay/{reference}` accepting mobile-money phone number and provider, initiating deposit collection for open invoices.
- **FR-005**: Pay page collect MUST be idempotent (hidden idempotency token per page load or deterministic key).
- **FR-006**: System MUST NOT expose API keys, internal system IDs, or other systems' data on the public pay page.
- **FR-007**: System MUST deliver signed webhooks on invoice paid events when `webhook_url` is configured.
- **FR-008**: Webhook payload MUST include invoice reference, amount, currency, status, and linked transaction ID.
- **FR-009**: System MUST wrap system registration and wallet seeding in a single database transaction (FR from gap fix).
- **FR-010**: System MUST apply basic rate limiting on `POST /pay/{reference}` to reduce abuse.

### Key Entities

- **Invoice** (existing): Pay page reads by `reference`; status drives UI state.
- **Transaction** (existing): Created on successful deposit; linked to invoice on pay.
- **WebhookDelivery** (existing pattern): Extended to invoice-paid event type.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Payer can view invoice details from QR link in under 2 seconds (p95).
- **SC-002**: End-to-end QR pay (scan → submit → paid) completable without merchant API calls.
- **SC-003**: 100% of successful pay-page payments trigger webhook when URL configured.
- **SC-004**: Zero double-credits on duplicate pay page submits (idempotency verified in tests).
- **SC-005**: Invalid references return indistinguishable not-found responses.

## Assumptions

- v1 pay page supports mobile-money (MMO) only, matching existing deposit adapter fields.
- HTML is server-rendered minimal CSS (no SPA, no React build step).
- Reference string is the security capability (unguessable); no separate payer auth.
- Provider list is derived from invoice country (static map, same as payout providers).
- Webhook signing reuses existing HMAC mechanism from payout webhooks.
