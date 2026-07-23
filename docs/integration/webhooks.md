# Webhooks Integration

Receive async notifications when payments complete or invoices are paid.

## Setup

Register with an optional `webhook_url`, or add endpoints later in the merchant UI (**Webhooks**) / API:

```http
POST /webhook-endpoints
X-API-Key: sk_live_...
{"url":"https://your-app.example.com/webhooks/relay","label":"Production"}
```

```json
{
  "name": "My Shop",
  "prefix": "SHOP",
  "enabled_countries": ["ZM"],
  "webhook_url": "https://your-app.example.com/webhooks/relay"
}
```

Each tenant can register **multiple** HTTPS endpoints (`GET`/`POST /webhook-endpoints`, `PATCH`/`DELETE /webhook-endpoints/{id}`). Relay fans out each event to **every enabled endpoint across all tenants**. Use `system_id` in the payload to filter events that belong to your system.

Configure the same secret on Relay and your app:

```env
# Relay server .env
WEBHOOK_SIGNING_SECRET=whsec_your_shared_secret

# Your app
WEBHOOK_SIGNING_SECRET=whsec_your_shared_secret
```

## Delivery

| Property | Value |
|----------|-------|
| Method | `POST` |
| Content-Type | `application/json` |
| Signature header | `X-Relay-Signature: sha256=<hex>` |
| Retries | 3 attempts with backoff |
| Success | Your endpoint returns 2xx |

## Signature verification

The signature is **HMAC-SHA256 of the raw request body** (not parsed JSON).

**TypeScript (Express)**

```typescript
import express from "express";
import { parseWebhookEvent, verifyWebhookSignature } from "@relay/sdk";

const app = express();

app.post(
  "/webhooks/relay",
  express.raw({ type: "application/json" }),
  (req, res) => {
    const rawBody = req.body as Buffer;
    const signature = req.headers["x-relay-signature"] as string;

    if (!verifyWebhookSignature(rawBody, signature, process.env.WEBHOOK_SIGNING_SECRET!)) {
      return res.status(401).json({ error: "invalid signature" });
    }

    const event = parseWebhookEvent(rawBody.toString());
    // process event...
    res.json({ ok: true });
  },
);
```

**Python (Flask)**

```python
from flask import Flask, request
from relay_sdk import parse_webhook_event, verify_webhook_signature

@app.post("/webhooks/relay")
def relay_webhook():
    raw = request.get_data()
    signature = request.headers.get("X-Relay-Signature")

    if not verify_webhook_signature(raw, signature, os.environ["WEBHOOK_SIGNING_SECRET"]):
        return {"error": "invalid signature"}, 401

    event = parse_webhook_event(raw)
    # process event...
    return {"ok": True}
```

**Manual verification (openssl)**

```bash
echo -n '{"event":"payment.status_changed",...}' \
  | openssl dgst -sha256 -hmac "$WEBHOOK_SIGNING_SECRET"
# Compare hex output to X-Relay-Signature after "sha256="
```

## Event types

### `payment.status_changed`

Sent when a payout reaches terminal status.

```json
{
  "event": "payment.status_changed",
  "payment_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
  "system_id": "550e8400-e29b-41d4-a716-446655440000",
  "external_id": "SHOP_20260719_ABC12345",
  "status": "completed",
  "amount": 2500,
  "currency": "ZMW",
  "country": "ZM",
  "gateway_reference": "f4401bd2-1568-4140-bf2d-eb77d2b2b639",
  "error": null,
  "timestamp": "2026-07-19T10:00:00Z"
}
```

**TypeScript**

```typescript
import { isPaymentStatusChanged } from "@relay/sdk";

if (isPaymentStatusChanged(event)) {
  if (event.status === "completed") {
    markOrderPaid(event.external_id);
  } else {
    notifyPaymentFailed(event.external_id, event.error);
  }
}
```

### `invoice.paid`

Sent when an invoice is paid via pay page or merchant collect.

```json
{
  "event": "invoice.paid",
  "invoice_id": "858d6a5c-4ef9-4b8f-8b13-05a7b3b70802",
  "system_id": "550e8400-e29b-41d4-a716-446655440000",
  "reference": "INV_SHOP_a1b2c3d4",
  "amount": 5000,
  "currency": "ZMW",
  "country": "ZM",
  "status": "paid",
  "transaction_id": "8bbbc2fb-28f4-49ed-a4f3-9c3e2e0572d8",
  "timestamp": "2026-07-19T10:05:00Z"
}
```

Filter by `system_id` if you only care about your own invoices (events are broadcast to all connected merchants).

**Python**

```python
from relay_sdk import is_invoice_paid

if is_invoice_paid(event):
    fulfill_order(event["reference"], event["transaction_id"])
```

## Best practices

1. **Verify signature before parsing** — reject forgeries early
2. **Respond 2xx quickly** — offload heavy work to a queue
3. **Make handlers idempotent** — Relay may retry; use `payment_id` or `reference` as dedup key
4. **Use HTTPS** — required for production webhook URLs
5. **Log failures** — Relay records delivery attempts in `webhook_delivery_attempts`

## Schema reference

Full JSON schema: [`webhook-payload.json`](../../specs/001-payment-relay/contracts/webhook-payload.json)

Note: `invoice.paid` is an additional event type not in the v0.1 schema file — both events are documented here and in the SDK types.
