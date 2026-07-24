# Dokploy deployment

Compose: **`docker-compose.dokploy.yml`** → **`relay`** + **`webhook-worker`** only.

Domain → **`relay`** : **8080**

## Environment

Use your Dokploy Postgres + Redis **internal URLs**:

```env
DATABASE_URL=postgres://USER:PASS@PG_HOST:5432/DB
REDIS_URL=redis://default:PASS@REDIS_HOST:6379
```

Prefer `postgres://` (not `postgresql://`). Compose joins **`dokploy-network`** so those hosts resolve.

Also set: `INVOICE_PAY_BASE_URL`, `PAWAPAY_API_TOKEN`, `WEBHOOK_SIGNING_SECRET`, `ADMIN_PASSWORD`.

**Remove** `POSTGRES_PASSWORD` from the Environment tab if it is present (empty values can break older deploys).

## If DNS still fails

In Dokploy → Docker, copy the **exact container name** of Postgres/Redis and use that as the hostname in the URL (not the app display name).
