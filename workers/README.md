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

Use the all-in-one compose stack: [`docs/dokploy.md`](../docs/dokploy.md) (`docker-compose.dokploy.yml`).

That deploys API + this worker together. If you split apps instead:

1. Create **Postgres** → set `DATABASE_URL` on the API
2. Create **Redis** → set `REDIS_URL` on **both** API and this worker
3. Deploy API with root `Dockerfile`
4. Deploy this folder with `workers/Dockerfile` and the same `REDIS_URL` / `WEBHOOK_SIGNING_SECRET`

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
