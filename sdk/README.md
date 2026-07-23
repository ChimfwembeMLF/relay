# Payment Relay SDKs

Official client libraries for the Payment Relay merchant API.

| Language | Path | Install |
|----------|------|---------|
| TypeScript / JavaScript | [`typescript/`](typescript/) | `npm install` in `sdk/typescript` |
| Python | [`python/`](python/) | `pip install -e sdk/python` |

## Integration guides

Step-by-step guides for connecting your systems:

- [Overview](../docs/integration/README.md)
- [Getting started](../docs/integration/getting-started.md)
- [Payments](../docs/integration/payments.md)
- [Invoices & pay page](../docs/integration/invoices-and-pay-page.md)
- [Webhooks](../docs/integration/webhooks.md)
- [Reports](../docs/integration/reports.md)

## Quick example

**TypeScript**

```typescript
import { RelayClient } from "@relay/sdk";

const relay = new RelayClient({
  baseUrl: "http://localhost:8080",
  apiKey: process.env.RELAY_API_KEY!,
});

const invoice = await relay.invoices.create({
  amount: 5000,
  currency: "ZMW",
  country: "ZM",
  description: "Order #42",
});

console.log(invoice.qr_url);
```

**Python**

```python
from relay_sdk import RelayClient

relay = RelayClient(
    base_url="http://localhost:8080",
    api_key=os.environ["RELAY_API_KEY"],
)

invoice = relay.invoices.create(
    amount=5000,
    currency="ZMW",
    country="ZM",
    description="Order #42",
)

print(invoice["qr_url"])
```

## API reference

OpenAPI contracts:

- [v0.1 — payments](../specs/001-payment-relay/contracts/openapi.yaml)
- [v0.2 — invoices & reports](../specs/002-wallet-invoices-reports/contracts/openapi.yaml)
- [v0.3 — pay page](../specs/003-invoice-pay-page/contracts/openapi.yaml)
