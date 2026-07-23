# Invoices & Pay Page Integration

Collect inbound payments via QR invoices and a hosted payer checkout page.

## Two collection paths

| Path | Who initiates | Endpoint |
|------|---------------|----------|
| **Pay page** | Customer scans QR / opens link | Public `/pay/{reference}` + `/api/pay/{reference}` |
| **Merchant collect** | Your backend | `POST /invoices/{id}/collect` |

Both credit the system wallet and fire an `invoice.paid` webhook.

## Create an invoice

```bash
curl -s -X POST http://localhost:8080/invoices \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $RELAY_API_KEY" \
  -d '{
    "amount": 5000,
    "currency": "ZMW",
    "country": "ZM",
    "description": "Order #42",
    "expires_in_hours": 24
  }' | jq .
```

Response fields:

| Field | Use |
|-------|-----|
| `reference` | Human-readable ID (`INV_SHOP_a1b2c3d4`) |
| `qr_url` | Link for customer checkout |
| `qr_code_png_base64` | Embed in receipts, PDFs, POS screens |
| `status` | `open` until paid, expired, or cancelled |

**TypeScript**

```typescript
const invoice = await relay.invoices.create({
  amount: 5000,
  currency: "ZMW",
  country: "ZM",
  description: "Order #42",
});

// Show QR in your UI
const imgSrc = `data:image/png;base64,${invoice.qr_code_png_base64}`;

// Or redirect customer
window.location.href = invoice.qr_url;
```

## Hosted pay page

Relay serves a React checkout at:

```
https://your-relay.example.com/pay/{reference}
```

Configure the QR link base:

```env
INVOICE_PAY_BASE_URL=https://your-relay.example.com
```

### Custom UI (optional)

Use the public JSON API instead of the hosted page:

```typescript
// No API key required
const page = await relay.pay.get(invoice.reference);

if (page.payable) {
  await relay.pay.submit(invoice.reference, {
    phone: "260763456789",
    provider: page.providers[0].value,
    idempotency_key: page.idempotency_key!,
    form_token: page.form_token!,
  });
}
```

`form_token` and `idempotency_key` come from the GET response and expire with the invoice.

## Merchant-side collect

For server-initiated collection (e.g. USSD flow):

```typescript
await relay.invoices.collect(invoice.id, {
  payment_method: {
    type: "mmo",
    details: { phone: "260763456789", provider: "MTN_MOMO_ZMB" },
  },
  idempotency_key: crypto.randomUUID(),
});
```

## Invoice lifecycle

```
open ──pay──▶ paid
  │
  ├──expire──▶ expired
  │
  └──cancel──▶ cancelled
```

```typescript
// Poll status
const inv = await relay.invoices.get(reference);
if (inv.status === "paid") {
  fulfillOrder(inv.transaction_id);
}

// Cancel before payment
await relay.invoices.cancel(invoiceId);
```

## List open invoices

```typescript
const open = await relay.invoices.list({ status: "open", limit: 50 });
```

## Frontend build

`cargo build` / `cargo run` builds the React pay page via `build.rs`. Requires Node.js/`npm`.

```bash
cargo run
# or skip: SKIP_FRONTEND_BUILD=1 cargo run
```

Dev mode with hot reload:

```bash
SKIP_FRONTEND_BUILD=1 cargo run                # :8080
cd frontend && npm run dev                     # :5173/pay/{reference}
```

UI design tokens: [`frontend/DESIGN.md`](../../frontend/DESIGN.md)

## Webhooks

When an invoice is paid (via pay page or collect), Relay sends `invoice.paid`. See [Webhooks guide](webhooks.md).
