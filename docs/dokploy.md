# Dokploy deployment

Compose file: **`docker-compose.dokploy.yml`**

Services: **`postgres`**, **`redis`**, **`relay`**, **`webhook-worker`**

Domain → **`relay`** port **`8080`**

## Environment

Paste from [`.env.dokploy.example`](../.env.dokploy.example). Critical:

```env
POSTGRES_PASSWORD=your-strong-password
DATABASE_URL=postgres://relay:your-strong-password@postgres:5432/payment_relay
REDIS_URL=redis://redis:6379
```

Use hosts **`postgres`** and **`redis`** (compose service names). Do **not** use Dokploy database app hostnames like `tekrem-payments-tynhbu` with this compose file.

Also set: `INVOICE_PAY_BASE_URL`, `PAWAPAY_API_TOKEN`, `WEBHOOK_SIGNING_SECRET`, `ADMIN_PASSWORD`.

## Why not separate Dokploy DB apps?

Compose containers often cannot resolve Dokploy database hostnames. Bundling Postgres/Redis in the same compose stack avoids that.

You can stop/remove the unused Dokploy Postgres/Redis services for this project after this stack is healthy.

## Deploy

1. Compose path: `docker-compose.dokploy.yml`
2. Set Environment
3. Deploy (first Rust build is slow)
4. Open `/`, `/swagger-ui/`, `/admin`
