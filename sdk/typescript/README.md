# @relay/sdk

TypeScript/JavaScript client for the Payment Relay merchant API.

## Install

```bash
cd sdk/typescript
npm install
npm run build
```

Link locally in another project:

```bash
npm link
# in your app
npm link @relay/sdk
```

## Usage

```typescript
import { RelayClient, RelayError } from "@relay/sdk";

const relay = new RelayClient({
  baseUrl: "http://localhost:8080",
  apiKey: process.env.RELAY_API_KEY!,
});

// Create an invoice
const invoice = await relay.invoices.create({
  amount: 5000,
  currency: "ZMW",
  country: "ZM",
  description: "Order #42",
});

console.log(invoice.qr_url);

// Process a payout
try {
  const payment = await relay.payments.process({
    system_id: invoice.system_id,
    external_id: "SHOP_20260719_ABC12345",
    amount: 2500,
    currency: "ZMW",
    country: "ZM",
    payment_method: {
      type: "mmo",
      details: { phone: "260763456789", provider: "MTN_MOMO_ZMB" },
    },
    idempotency_key: crypto.randomUUID(),
  });
  console.log(payment.status);
} catch (err) {
  if (err instanceof RelayError && err.code === "insufficient_balance") {
    console.error("Wallet too low");
  }
}
```

## Webhooks

```typescript
import { parseWebhookEvent, verifyWebhookSignature } from "@relay/sdk";

export async function handleWebhook(req: Request) {
  const rawBody = await req.text();
  const signature = req.headers.get("X-Relay-Signature");

  if (!verifyWebhookSignature(rawBody, signature, process.env.WEBHOOK_SIGNING_SECRET!)) {
    return new Response("Invalid signature", { status: 401 });
  }

  const event = parseWebhookEvent(rawBody);
  // handle payment.status_changed or invoice.paid
  return new Response("OK");
}
```

## Resources

| Property | Methods |
|----------|---------|
| `relay.systems` | `create`, `get` |
| `relay.payments` | `process`, `get` |
| `relay.wallets` | `list` |
| `relay.transactions` | `list` |
| `relay.invoices` | `create`, `list`, `get`, `collect`, `cancel` |
| `relay.reports` | `transactions`, `wallets`, `invoices` |
| `relay.pay` | `get`, `submit` (public pay page API) |

See [integration guides](../../docs/integration/README.md) for full workflows.
