# Quickstart: Invoice Pay Page

**Feature**: 003-invoice-pay-page | **Date**: 2026-07-18

Prerequisites: Features 001 and 002 running; migration `003_*` applied.

## 1. Configure pay base URL

Ensure `.env` has:

```env
INVOICE_PAY_BASE_URL=http://localhost:8080
```

Restart relay after changes.

## 2. Create an invoice (merchant API)

```bash
# Use existing system from 002 quickstart
curl -s -X POST http://localhost:8080/invoices \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $API_KEY" \
  -d '{
    "amount": 5000,
    "currency": "ZMW",
    "country": "ZM",
    "description": "Counter sale #99"
  }' | jq .
```

Save `reference` and open `qr_url` in a browser (or scan QR).

## 3. View pay page

```bash
open "http://localhost:8080/pay/$REFERENCE"
# or
curl -s "http://localhost:8080/pay/$REFERENCE" | head -20
```

**Expected**: HTML page showing amount `5000`, currency `ZMW`, description, and payment form.

## 4. Pay from the page

Submit the form in browser with phone `260763456789`, or:

```bash
# Extract form token and idempotency key from HTML, then:
curl -s -X POST "http://localhost:8080/pay/$REFERENCE" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "phone=260763456789&provider=MTN_MOMO_ZMB&idempotency_key=pay-page-001&form_token=$TOKEN"
```

**Expected**: Success HTML; invoice status `paid`; wallet balance increased.

## 5. Verify webhook (optional)

If system has `webhook_url` configured, check receiver logs for `invoice.paid` event after step 4.

## 6. Validation checklist

| Criterion | How to verify |
|-----------|----------------|
| SC-001 | Pay page loads in < 2s |
| SC-002 | Full flow without `POST /invoices/{id}/collect` API call |
| SC-003 | Webhook received when URL configured |
| SC-004 | Double POST with same idempotency key shows same result |
| SC-005 | `/pay/INV_FAKE_00000000` returns generic not-found |

## 7. Automated tests

```bash
DATABASE_URL=postgres://thecodefather@localhost:5432/payment_relay cargo test pay_page
```

## Troubleshooting

| Issue | Fix |
|-------|-----|
| 404 on pay page | Confirm reference matches invoice; check migration 003 applied |
| Form rejected | Regenerate page for fresh form token / idempotency key |
| Deposit fails | Use mock gateway in tests; verify pawaPay credentials in prod |

## Next Step

Run **`/speckit-tasks`** to generate implementation tasks.
