# Quickstart: Wallet Seeding, Invoices & Reports

**Feature**: 002-wallet-invoices-reports | **Date**: 2026-07-18

Prerequisites: Feature `001-payment-relay` running (`cargo run`), Postgres migrated through `002_*`.

## 1. Configure wallet seed defaults

Create `config/wallet_seed_defaults.json`:

```json
{
  "ZM": { "currency": "ZMW", "amount": 100000 },
  "US": { "currency": "USD", "amount": 10000 }
}
```

Add to `.env`:

```env
INVOICE_PAY_BASE_URL=http://localhost:8080
WALLET_SEED_DEFAULTS_PATH=config/wallet_seed_defaults.json
```

Restart relay after config changes.

## 2. Register system with auto-seeded wallets

```bash
curl -s -X POST http://localhost:8080/systems \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Retail POS",
    "prefix": "RTL",
    "enabled_countries": ["ZM", "US"],
    "wallet_seeds": [
      { "country": "ZM", "currency": "ZMW", "amount": 200000 }
    ]
  }' | jq .
```

**Expected**:
- `wallets_seeded: 2` (ZM override 200000, US default 10000)
- No manual SQL required (SC-006)

Verify wallets immediately:

```bash
curl -s "http://localhost:8080/wallets/$SYSTEM_ID" \
  -H "X-API-Key: $API_KEY" | jq .
```

## 3. Create invoice with QR code

```bash
curl -s -X POST http://localhost:8080/invoices \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $API_KEY" \
  -d '{
    "amount": 5000,
    "currency": "ZMW",
    "country": "ZM",
    "description": "Counter sale #42",
    "expires_in_hours": 48
  }' | jq .
```

**Expected**:
- `status: "open"`
- `qr_url` — scannable payment URL
- `qr_code_png_base64` — embed in receipt/PDF
- `reference` — stable lookup key

Decode QR locally (optional):

```bash
# Save qr_code_png_base64 to file and scan with phone
```

## 4. Collect invoice payment (deposit)

```bash
curl -s -X POST "http://localhost:8080/invoices/$INVOICE_ID/collect" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $API_KEY" \
  -d '{
    "idempotency_key": "collect-inv-001",
    "payment_method": {
      "type": "mmo",
      "details": {
        "provider": "MTN_MOMO_ZMB",
        "phoneNumber": "260763456789"
      }
    }
  }' | jq .
```

**Expected**:
- Invoice `status: "paid"`
- Wallet balance increased by invoice amount
- `transaction_id` linked on invoice

## 5. Run reports

**Transaction report (JSON)**:

```bash
curl -s "http://localhost:8080/reports/transactions?from=2026-07-01T00:00:00Z&to=2026-07-31T23:59:59Z&detail=true" \
  -H "X-API-Key: $API_KEY" | jq .
```

**Invoice report (CSV export)**:

```bash
curl -s "http://localhost:8080/reports/invoices?from=2026-07-01T00:00:00Z&to=2026-07-31T23:59:59Z&format=csv" \
  -H "X-API-Key: $API_KEY" -o invoices-july.csv
```

**Wallet report**:

```bash
curl -s "http://localhost:8080/reports/wallets?from=2026-07-01T00:00:00Z&to=2026-07-31T23:59:59Z" \
  -H "X-API-Key: $API_KEY" | jq .
```

## 6. Validation checklist

| Criterion | How to verify |
|-----------|----------------|
| SC-001 | Wallets exist < 5s after registration (step 2) |
| SC-002 | Invoice + QR returned in step 3 |
| SC-003 | `GET /invoices/{reference}` returns in < 1s |
| SC-004 | Transaction report with 30-day window |
| SC-005 | CSV columns match JSON detail fields |
| SC-006 | No SQL seeding in step 2 |

## 7. Automated tests

```bash
DATABASE_URL=postgres://thecodefather@localhost:5432/payment_relay cargo test
```

## Troubleshooting

| Issue | Fix |
|-------|-----|
| `wallets_seeded: 0` | Check `wallet_seed_defaults.json` has entries for enabled countries |
| Invoice collect fails | Verify pawaPay deposit credentials; use mock gateway in tests |
| Expired invoice | Create new invoice or increase `expires_in_hours` |
| Empty report | Widen date range; confirm `from`/`to` are ISO8601 UTC |

## Next Step

Run **`/speckit-tasks`** to generate implementation tasks for this feature.
