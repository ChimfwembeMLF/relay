# Quickstart: Invoice Pay Page

**Feature**: 003-invoice-pay-page | **Date**: 2026-07-18

Prerequisites: Features 001 and 002 running; migration `003_*` applied; React frontend built.

## 0. Build & run

```bash
cargo run   # build.rs runs npm install + npm run build for frontend/
```

The server serves the SPA at `/pay/{reference}` and static assets at `/pay/assets/*`. Requires Node.js/`npm`. Skip with `SKIP_FRONTEND_BUILD=1` if `frontend/dist` already exists.

For hot reload during UI work:

```bash
# Terminal 1
SKIP_FRONTEND_BUILD=1 cargo run

# Terminal 2
cd frontend && npm run dev   # http://localhost:5173/pay/{reference}
```

## 1. Configure environment

Ensure `.env` has:

```env
INVOICE_PAY_BASE_URL=http://localhost:8080
FALLBACK_GATEWAY=mock
```

Restart relay after changes. Use `FALLBACK_GATEWAY=mock` for local dev without a pawaPay sandbox token.

## 2. Create an invoice (merchant API)

```bash
# Register a system (once) or use an existing API key from feature 002
curl -s -X POST http://localhost:8080/systems \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Demo Merchant",
    "prefix": "DEMO01",
    "enabled_countries": ["ZM"],
    "webhook_url": "https://example.com/webhook"
  }' | jq .

export API_KEY="<api_key from response>"

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
```

**Expected**: React pay page showing amount `ZMW 50.00`, description, and a phone/provider form.

Load invoice data via JSON API:

```bash
curl -s "http://localhost:8080/api/pay/$REFERENCE" | jq .
```

## 4. Pay from the page

Submit the form in the browser with phone `260763456789`, or via API:

```bash
# Fetch fresh form_token and idempotency_key from GET /api/pay/$REFERENCE
curl -s -X POST "http://localhost:8080/api/pay/$REFERENCE" \
  -H "Content-Type: application/json" \
  -d '{
    "phone": "260763456789",
    "provider": "MTN_MOMO_ZMB",
    "idempotency_key": "<from GET response>",
    "form_token": "<from GET response>"
  }' | jq .
```

**Expected**: Success message in UI; invoice status `paid`; wallet balance increased.

## 5. Verify webhook (optional)

If the system has `webhook_url` configured, check receiver logs for `invoice.paid` after step 4.

## 6. Validation checklist

| Criterion | How to verify |
|-----------|----------------|
| SC-001 | Pay page loads in < 2s |
| SC-002 | Full flow without `POST /invoices/{id}/collect` API call |
| SC-003 | Webhook received when URL configured |
| SC-004 | Double POST with same idempotency key shows same result |
| SC-005 | `/pay/INV_FAKE_00000000` shows invalid-link message in React UI |

## 7. Automated tests

```bash
cargo test pay_page
```

## Troubleshooting

| Issue | Fix |
|-------|-----|
| Blank page / "Pay frontend not built" | Ensure Node/`npm` are installed, then `cargo build` (or unset `SKIP_FRONTEND_BUILD`) |
| 404 on pay page | Confirm reference matches invoice; check migration 003 applied |
| API returns 403 on submit | Regenerate page data for fresh `form_token` / `idempotency_key` |
| Deposit fails | Set `FALLBACK_GATEWAY=mock` in `.env` for local dev, or use valid pawaPay sandbox token |
| Old HTML instead of React | Restart server after rebuilding frontend |

## Next Step

Feature implemented — run automated tests:

```bash
cargo test pay_page
```
