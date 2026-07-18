# Invoice QR Payload Format

**Feature**: 002-wallet-invoices-reports | **Version**: 1

## Overview

v1 QR codes encode an HTTPS URL (deep link) pointing to the relay pay endpoint. Scanning opens
the pay flow or provides a reference for mobile-money deposit initiation.

## URL Structure

```text
{INVOICE_PAY_BASE_URL}/pay/{reference}
```

| Component | Example | Notes |
|-----------|---------|-------|
| `INVOICE_PAY_BASE_URL` | `https://pay.relay.internal` | Env: `INVOICE_PAY_BASE_URL`, default `http://localhost:8080` |
| `reference` | `INV_ECO_a1b2c3d4` or UUID | Unique invoice reference |

**Example**:

```text
https://pay.relay.internal/pay/INV_ECO_a1b2c3d4e5f6
```

## API Response Fields

When creating an invoice (`POST /invoices`):

| Field | Type | Description |
|-------|------|-------------|
| `reference` | string | Stable invoice reference embedded in QR |
| `qr_url` | string | Same URL encoded in QR |
| `qr_code_png_base64` | string | `data:image/png;base64,...` for direct display |
| `expires_at` | ISO8601 | Invoice expiry |

## QR Encoding

- Error correction: **M** (medium)
- Module size: 256×256 PNG
- Charset: URL string (UTF-8)

## Payer Flow (v1)

1. Payer scans QR → opens URL (future: hosted pay page)
2. Internal system or relay `POST /invoices/{id}/collect` initiates pawaPay deposit
3. On deposit success, invoice → `paid`, wallet credited

## Security

- References are unguessable (UUID or 128-bit random suffix)
- No API keys or PII in QR payload
- HTTPS required in production (`INVOICE_PAY_BASE_URL`)

## Future (out of v1)

- EMVCo QR for mobile-money STK push
- Static merchant QR with amount embedded
