# Pay Page HTML Contract

**Feature**: 003-invoice-pay-page | **Version**: 1

## Templates

| Template | When rendered |
|----------|---------------|
| `open.html` | Invoice status `open` and not expired |
| `paid.html` | Invoice status `paid` |
| `expired.html` | Status `expired` or past `expires_at` |
| `cancelled.html` | Status `cancelled` |
| `not_found.html` | Unknown reference |
| `error.html` | Gateway/validation failure on POST |

## Open Page Required Elements

- Invoice reference (human-readable)
- Amount formatted with currency (e.g., `ZMW 50.00` from minor units)
- Description (if present)
- Expiry datetime (local-friendly format)
- Form fields:
  - `phone` (text, required)
  - `provider` (select, required — options from country map)
  - `idempotency_key` (hidden, UUID v4)
  - `form_token` (hidden, HMAC)
- Submit button: "Pay now"

## Paid Page Required Elements

- Success message
- Amount paid
- Reference
- Optional: transaction ID for support

## Security Requirements

- No API keys, system IDs, or webhook URLs in HTML
- `not_found.html` identical for all invalid references
- Meta viewport tag for mobile: `<meta name="viewport" content="width=device-width, initial-scale=1">`
- Form POST to same `/pay/{reference}` path

## Styling

- Minimal inline CSS or single `<style>` block
- Readable on 320px width
- No external CDN dependencies (offline-friendly)

## Form Token Generation

```text
form_token = HMAC-SHA256(WEBHOOK_SIGNING_SECRET, "{reference}:{invoice_expires_at_rfc3339}")
```

Validated on POST before processing payment.
