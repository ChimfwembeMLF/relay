# Deploy Relay to production

**Dokploy (recommended):** follow **[`docs/dokploy.md`](dokploy.md)** using `docker-compose.dokploy.yml`.

## What runs

| Service | Role |
|---------|------|
| **Postgres** | Primary store (migrations on API startup) |
| **Redis** | Webhook queue |
| **relay** | Rust API + `frontend/dist` |
| **webhook-worker** | BullMQ consumer (`workers/`) |

## Environment

| Variable | Production value |
|----------|------------------|
| `INVOICE_PAY_BASE_URL` | HTTPS origin (no trailing slash) |
| `PAWAPAY_BASE_URL` | `https://api.pawapay.io` |
| `PAWAPAY_API_TOKEN` | Live token |
| `FALLBACK_GATEWAY` | `pawapay` |
| `WEBHOOK_SIGNING_SECRET` | `openssl rand -hex 32` |
| `ADMIN_PASSWORD` | Strong unique password |

Templates: [`.env.dokploy.example`](../.env.dokploy.example), [`.env.production.example`](../.env.production.example).

Do **not** set `ALLOW_INSECURE_WEBHOOKS` or `FALLBACK_GATEWAY=mock` in prod.

## VPS without Dokploy

```bash
cp .env.production.example .env.prod
docker compose -f docker-compose.prod.yml --env-file .env.prod up -d --build
```

TLS via Caddy/nginx → `127.0.0.1:8080`.

## After deploy

1. Open `/` and `/admin`; create a merchant system (API key shown once).
2. Merchant webhooks must verify `WEBHOOK_SIGNING_SECRET`.
3. Keep Postgres volume backups before major upgrades.
