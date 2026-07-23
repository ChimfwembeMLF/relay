# Dokploy deployment

Compose file: **`docker-compose.dokploy.yml`** → services **`relay`** + **`webhook-worker`**.

Postgres and Redis come from **Dokploy database services** (set `DATABASE_URL` / `REDIS_URL`).

## 1. Create databases

In the same Dokploy project:

1. Create a **PostgreSQL** database
2. Create a **Redis** instance
3. Copy their internal connection URLs

## 2. Create the Compose app

1. **Create** → **Docker Compose**
2. Connect this Git repo
3. Compose file: `docker-compose.dokploy.yml`
4. **Environment**: paste from [`.env.dokploy.example`](../.env.dokploy.example) and fill real URLs/secrets

Use `postgres://…` for `DATABASE_URL` (convert from `postgresql://` if needed).

## 3. Domain

- Service: **`relay`**
- Port: **`8080`**
- HTTPS on
- Set `INVOICE_PAY_BASE_URL` to that exact origin (no trailing slash)

## 4. Networking

Compose must reach Dokploy DB hostnames (e.g. `tekrem-payments-…`). Keep the Compose app and the Postgres/Redis services in the **same Dokploy project**. If DNS fails, attach the compose stack to the database network in Dokploy advanced settings.

## 5. Deploy + smoke test

Deploy, then check `/`, `/swagger-ui/`, `/admin`, and a pay link.

Still fill before go-live: `PAWAPAY_API_TOKEN`, `WEBHOOK_SIGNING_SECRET`, `ADMIN_PASSWORD`.
