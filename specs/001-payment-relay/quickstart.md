# Quickstart: Internal Payment Relay

**Feature**: 001-payment-relay | **Date**: 2026-07-18

Validation guide for end-to-end feature verification. See [data-model.md](./data-model.md) and
[contracts/openapi.yaml](./contracts/openapi.yaml) for details.

## Prerequisites

- Rust 1.75+ and cargo
- Docker + docker-compose (PostgreSQL)
- pawaPay sandbox API token (or mock gateway in test mode)
- `curl` and `jq`

## 1. Environment Setup

```bash
cp .env.example .env
# Set at minimum:
# DATABASE_URL=postgres://relay:relay@localhost:5432/payment_relay
# PORT=8080
# PAWAPAY_API_TOKEN=...
# PAWAPAY_BASE_URL=https://api.sandbox.pawapay.io
# WEBHOOK_SIGNING_SECRET=dev-secret-change-me
```

```bash
docker compose up -d postgres
cargo sqlx migrate run   # after migrations exist
cargo run
```

**Expected**: Server listens on `:8080`; health/log line confirms DB connection.

## 2. Register a System (User Story 1)

```bash
curl -s -X POST http://localhost:8080/systems \
  -H "Content-Type: application/json" \
  -d '{
    "name": "E-Commerce",
    "prefix": "ECO",
    "enabled_countries": ["ZM", "US"],
    "webhook_url": "https://webhook.site/your-id"
  }' | jq .
```

**Expected**:
- HTTP 201
- Response includes `id`, `prefix`, and one-time `api_key`
- Save `id` as `SYSTEM_ID` and `api_key` as `API_KEY`

**Negative check**: Repeat with same `prefix` → HTTP 409.

## 3. Seed Wallet Balance (precondition for payouts)

Until a funding API exists, insert test balance via SQL or admin script:

```sql
UPDATE wallets SET balance = 100000
WHERE system_id = '<SYSTEM_ID>' AND country = 'ZM' AND currency = 'ZMW';
```

(Wallet row auto-created on first payment attempt if seed step is deferred.)

## 4. Process a Payment (User Story 2)

```bash
curl -s -X POST http://localhost:8080/payments \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $API_KEY" \
  -d "{
    \"system_id\": \"$SYSTEM_ID\",
    \"external_id\": \"ECO_550e_$(date +%Y%m%d)_TEST01\",
    \"amount\": 1500,
    \"currency\": \"ZMW\",
    \"country\": \"ZM\",
    \"idempotency_key\": \"test-$(uuidgen)\",
    \"payment_method\": {
      \"type\": \"mmo\",
      \"details\": {
        \"provider\": \"MTN_MOMO_ZMB\",
        \"phoneNumber\": \"260763456789\"
      }
    }
  }" | jq .
```

**Expected**:
- HTTP 200
- `status` is `completed` or `pending` (sandbox-dependent)
- `gateway_reference` populated when gateway accepts
- Wallet balance decreased by `amount` on `completed`

## 5. Idempotency Check (FR-006)

Re-run the **exact same** curl from step 4 (same `idempotency_key`).

**Expected**:
- HTTP 200 with identical `id` and `status`
- No second gateway charge (verify pawaPay dashboard or mock call count)
- Wallet balance unchanged from first successful run

Change `amount` but keep same `idempotency_key` → HTTP 409.

## 6. Query Wallets (User Story 4)

```bash
curl -s "http://localhost:8080/wallets/$SYSTEM_ID" \
  -H "X-API-Key: $API_KEY" | jq .
```

**Expected**: Array of wallets scoped to `SYSTEM_ID` only; balances match ledger.

## 7. Trace by External ID (User Story 5)

```bash
curl -s "http://localhost:8080/transactions/$SYSTEM_ID?external_id=ECO_550e_20260718_TEST01" \
  -H "X-API-Key: $API_KEY" | jq .
```

**Expected**: Single matching transaction for owning system.

## 8. Webhook Delivery (User Story 3)

After a terminal payment, inspect webhook receiver (webhook.site or local listener).

**Expected**:
- POST to configured `webhook_url`
- Body matches [webhook-payload.json](./contracts/webhook-payload.json)
- Header `X-Relay-Signature` present (HMAC-SHA256 of raw body)
- Delivery within 30s under normal conditions (SC-003)

Verify signature:

```bash
echo -n '<raw-body>' | openssl dgst -sha256 -hmac "$WEBHOOK_SIGNING_SECRET"
```

## 9. Automated Test Suite

```bash
cargo test
```

**Expected coverage** (constitution):
- System registration + auth rejection
- Payment happy path with mocked gateway
- Idempotency replay and conflict
- Wallet auto-create and balance debit
- Webhook sender retries (mock HTTP server)

## 10. Success Criteria Mapping

| Criterion | Validated by |
|-----------|--------------|
| SC-001 | Steps 1–4 complete in < 10 min |
| SC-002 | Step 5 idempotency check |
| SC-003 | Step 8 webhook timing |
| SC-004 | Step 7 external_id lookup |
| SC-005 | Step 6 balance vs transaction history |

## Troubleshooting

| Symptom | Likely cause |
|---------|--------------|
| 401 Unauthorized | Wrong or missing `X-API-Key` |
| 403 Country not enabled | `country` not in system's `enabled_countries` |
| 402 Insufficient balance | Wallet not seeded |
| pending stuck | Gateway timeout — check reconcile job / pawaPay status API |
| Webhook missing | Invalid URL or all 3 delivery attempts failed — check `webhook_delivery_attempts` table |

## Next Step

Run **`/speckit-tasks`** to generate `tasks.md` from this plan.
