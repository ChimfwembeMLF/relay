# Relay workers (BullMQ)

Node workers that drain Redis inboxes from the Rust API into [BullMQ](https://docs.bullmq.io/) queues.

## Queues

| Queue | Inbox key (Redis list) | Purpose |
|-------|------------------------|---------|
| `webhooks` | `relay:inbox:webhooks` | Deliver signed merchant webhooks with retries |

## Local

```bash
# terminal 1 — Redis + Postgres
docker compose up -d postgres redis

# terminal 2 — API
export DATABASE_URL=postgres://relay:relay@localhost:5432/payment_relay
export REDIS_URL=redis://localhost:6379
cargo run

# terminal 3 — worker
cd workers && npm install && npm start
```

Or:

```bash
docker compose --profile workers up webhook-worker
```

## Dokploy

1. Create **Postgres** → set `DATABASE_URL`
2. Create **Redis** → set `REDIS_URL=redis://...` on **both** the Relay API and this worker
3. Deploy **Relay API** (Rust) with:
   ```env
   DATABASE_URL=postgres://...
   REDIS_URL=redis://...
   INVOICE_PAY_BASE_URL=https://payments.tekreminnovations.com
   WEBHOOK_SIGNING_SECRET=...
   ```
4. Deploy **webhook-worker** (`workers/`) as a second app with the same `REDIS_URL`

Same Redis instance for API + worker. No separate Redis DB index required unless you isolate envs (`redis://host:6379/0`).

## Job payload

```json
{
  "job_id": "uuid",
  "url": "https://merchant.example/webhooks/relay",
  "body": "{...}",
  "signature": "sha256=...",
  "transaction_id": "uuid",
  "event_type": "invoice.paid"
}
```

Without `REDIS_URL`, the API delivers webhooks in-process (fine for local tests; not for multi-replica prod).
