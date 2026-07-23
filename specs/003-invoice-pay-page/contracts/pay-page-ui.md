# Pay Page UI Contract

**Feature**: 003-invoice-pay-page | **Version**: 2 (React SPA)

Design tokens and component specs: `frontend/DESIGN.md`.

## Architecture

| Route | Purpose |
|-------|---------|
| `GET /pay/{reference}` | Serves React SPA shell (`frontend/dist/index.html`) |
| `GET /api/pay/{reference}` | JSON invoice data for the client |
| `POST /api/pay/{reference}` | JSON payment submission |

Static assets: `/pay/assets/*` from `frontend/dist/assets/`.

## View States

| State | When shown |
|-------|------------|
| Loading | SPA bootstrapping, fetching `/api/pay/{reference}` |
| Ready (open) | Invoice status `open`, not expired, `payable: true` |
| Success | Payment completed or invoice already `paid` |
| Expired | Status `expired` or past `expires_at` |
| Cancelled | Status `cancelled` |
| Not found | Unknown reference (API 404) |
| Error | Load failure or payment gateway error |

## Open Page Required Elements

- Invoice reference (footer meta)
- Amount formatted with currency (e.g., `ZMW 50.00`)
- Description (if present)
- Expiry datetime (locale-friendly format)
- Form fields:
  - Mobile number (`phone`)
  - Provider (select from API `providers` list)
- Primary CTA: "Pay now"
- Hidden server-side: `form_token`, `idempotency_key` (from GET API, not shown in DOM)

## Success Page Required Elements

- Success message
- Amount paid (large display)
- Reference

## Security Requirements

- No API keys, system IDs, or webhook URLs in UI or JSON responses
- Not-found message identical for all invalid references
- Meta viewport tag for mobile
- `form_token` verified server-side on POST

## Styling

Follow `frontend/DESIGN.md`:

- `reservation-card` for main checkout card
- `text-input` for phone and provider fields
- `button-primary` (Rausch) for submit
- `top-nav` + `legal-band` page chrome
