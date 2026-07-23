# Payment Relay Integration Guides

Connect your internal systems to Payment Relay using the official SDKs or direct HTTP calls.

## SDKs

| Language | Package | Docs |
|----------|---------|------|
| TypeScript / JavaScript | [`@relay/sdk`](../sdk/typescript/) | [SDK README](../sdk/typescript/README.md) |
| Python | `relay-sdk` | [SDK README](../sdk/python/README.md) |

## Guides

| Guide | Topics |
|-------|--------|
| [Getting started](getting-started.md) | Register a system, store API key, first API call |
| [Payments](payments.md) | Payouts, idempotency, external IDs, wallet balances |
| [Invoices & pay page](invoices-and-pay-page.md) | QR invoices, hosted pay page, collect API |
| [Webhooks](webhooks.md) | Signature verification, event types, retry behavior |
| [Reports](reports.md) | Transaction, wallet, and invoice reports (JSON + CSV) |

## API contracts

OpenAPI specs (incremental by feature):

- [Merged spec](../openapi/openapi.yaml) — served at `/api-docs/openapi.yaml`
- [Swagger UI](http://localhost:8080/swagger-ui/) — interactive docs (also `/docs`)
- [v0.1 — payments](../specs/001-payment-relay/contracts/openapi.yaml)
- [v0.2 — invoices & reports](../specs/002-wallet-invoices-reports/contracts/openapi.yaml)
- [v0.3 — pay page](../specs/003-invoice-pay-page/contracts/openapi.yaml)

Webhook JSON schema: [`webhook-payload.json`](../specs/001-payment-relay/contracts/webhook-payload.json)

## Authentication

Protected routes require:

```http
X-API-Key: sk_live_<64 hex characters>
```

The API key is returned **once** from `POST /systems`. Store it in a secrets manager — it cannot be retrieved later.

Each API key is scoped to one system. Path/body `system_id` must match the authenticated system or requests return `403`.

## Amounts

All monetary amounts are in **minor units** (cents):

| Display | API value |
|---------|-----------|
| ZMW 50.00 | `5000` |
| USD 19.99 | `1999` |

## Environment variables (Relay server)

| Variable | Purpose |
|----------|---------|
| `DATABASE_URL` | PostgreSQL connection |
| `WEBHOOK_SIGNING_SECRET` | HMAC secret for webhook signatures |
| `INVOICE_PAY_BASE_URL` | Base URL embedded in invoice QR links |
| `FALLBACK_GATEWAY=mock` | Local dev without pawaPay credentials |

Your integrating app typically needs:

| Variable | Purpose |
|----------|---------|
| `RELAY_BASE_URL` | e.g. `http://localhost:8080` |
| `RELAY_API_KEY` | From system registration |
| `WEBHOOK_SIGNING_SECRET` | Same value as Relay server (for verification) |

## Error handling

Errors return JSON:

```json
{ "error": "insufficient_balance", "message": "Insufficient wallet balance" }
```

Common codes:

| HTTP | `error` | Meaning |
|------|---------|---------|
| 400 | `validation_error` | Invalid request body |
| 401 | `unauthorized` | Missing or invalid API key |
| 402 | `insufficient_balance` | Wallet too low for payout |
| 402 | `invoice_invalid` | Invoice expired or cancelled |
| 403 | `forbidden` | system_id mismatch |
| 404 | `not_found` | Resource not found |
| 409 | `conflict` | Idempotency key reused with different body |

Both SDKs raise typed errors (`RelayError`) with `status`, `code`, and `message`.
