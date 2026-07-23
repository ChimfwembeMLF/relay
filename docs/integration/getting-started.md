# Getting Started

Register a system, save your API key, and make your first API call.

## Prerequisites

- Payment Relay running (`cargo run` on port 8080)
- PostgreSQL with migrations applied
- For local payments: `FALLBACK_GATEWAY=mock` in `.env`

## 1. Register a system

```bash
curl -s -X POST http://localhost:8080/systems \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Shop",
    "prefix": "SHOP",
    "enabled_countries": ["ZM"],
    "webhook_url": "https://your-app.example.com/webhooks/relay"
  }' | jq .
```

Response includes:

- `id` — system UUID (use in payment requests)
- `api_key` — **shown once**; store in your secrets manager
- `wallets_seeded` — wallets created from defaults

### Prefix rules

- 2–8 uppercase alphanumeric characters
- Used in external IDs: `SHOP_20260719_ABC12345`
- Must be unique across the relay instance

## 2. Configure your app

```bash
export RELAY_BASE_URL=http://localhost:8080
export RELAY_API_KEY=sk_live_...
export WEBHOOK_SIGNING_SECRET=your-shared-secret
```

The webhook secret must match `WEBHOOK_SIGNING_SECRET` on the Relay server.

## 3. First API call — list wallets

**curl**

```bash
curl -s http://localhost:8080/wallets/$SYSTEM_ID \
  -H "X-API-Key: $RELAY_API_KEY" | jq .
```

**TypeScript**

```typescript
import { RelayClient } from "@relay/sdk";

const relay = new RelayClient({
  baseUrl: process.env.RELAY_BASE_URL!,
  apiKey: process.env.RELAY_API_KEY!,
});

const wallets = await relay.wallets.list(systemId);
console.log(wallets);
```

**Python**

```python
from relay_sdk import RelayClient

relay = RelayClient(
    base_url=os.environ["RELAY_BASE_URL"],
    api_key=os.environ["RELAY_API_KEY"],
)

wallets = relay.wallets.list(system_id)
print(wallets)
```

## 4. Next steps

| Goal | Guide |
|------|-------|
| Send money out (payouts) | [Payments](payments.md) |
| Collect money in (invoices) | [Invoices & pay page](invoices-and-pay-page.md) |
| Receive async notifications | [Webhooks](webhooks.md) |
| Export data for accounting | [Reports](reports.md) |

## Local development checklist

```bash
cp .env.example .env          # set DATABASE_URL, WEBHOOK_SIGNING_SECRET
docker compose up -d postgres # or local Postgres
cargo run                     # builds React pay UI via build.rs, API on :8080

cd frontend && npm run dev    # optional hot reload on :5173
```

See feature quickstarts for end-to-end validation:

- [Feature 001 quickstart](../specs/001-payment-relay/quickstart.md)
- [Feature 002 quickstart](../specs/002-wallet-invoices-reports/quickstart.md)
- [Feature 003 quickstart](../specs/003-invoice-pay-page/quickstart.md)
